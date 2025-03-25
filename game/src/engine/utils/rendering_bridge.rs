use crate::engine::ecs::components::{Camera, Transform};
use crate::engine::ecs::resources::CameraCullingState;
use crate::engine::ecs::{components::SpriteRenderer2D, resources::RenderingFrameData};
use crate::engine::rendering::components::{
    ARGB8Color, MeshInfo, RenderRequest, RenderUpdate, RenderingCamera,
};
use crate::engine::rendering::renderer::{RenderCmdHd, Renderer};
use crate::engine::rendering::renderer_helpers::prepare_material;
use crate::engine::rendering::shaders::Material;
use crate::engine::utils::maths::Rect;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::{EntityRef, World};
use bit_set::BitSet;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct RenderingBridge {
    ecs_world: Rc<RefCell<World>>,
    entity_handle_pairs: RefCell<Vec<(RenderCmdHd, Entity)>>, // Cache all entities and their associated render handle
    handle_index_by_entity: RefCell<HashMap<Entity, usize>>, // map entity to the index of their entity<->render_handle pair
    culled_check: RefCell<BitSet>, // Used by culling function to check if culling has been done on handles
}

impl RenderingBridge {
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self {
            ecs_world: world,
            entity_handle_pairs: RefCell::new(Vec::new()),
            handle_index_by_entity: RefCell::new(HashMap::new()),
            culled_check: RefCell::new(BitSet::with_capacity(2048)),
        }
    }

    pub fn get_world(&self) -> Ref<'_, World> {
        self.ecs_world.borrow()
    }

    pub fn get_world_mut(&self) -> RefMut<'_, World> {
        self.ecs_world.borrow_mut()
    }

    pub fn inject_new_rendering_entities(&self, renderer: &mut Renderer) {
        let world: Ref<World> = self.get_world();
        let container = world.resource::<RenderingFrameData>();

        for entity in container.new_2d_render.iter() {
            let entity_ref: EntityRef<'_> = world.entity(entity.clone());

            if let Some(comp) = entity_ref.get::<SpriteRenderer2D>() {
                let transform = entity_ref
                    .get::<Transform>()
                    .expect("Entity have no transform");
                let handle: RenderCmdHd = renderer.create_render_command(RenderRequest {
                    mesh_info: MeshInfo {
                        // Todo: Will be default until mesh is implemented
                        file_path: None,
                        count: 0,
                        vertices_set: None,
                    },
                    material: prepare_material(comp, comp.material.as_ref()),
                    transform: transform.clone(),
                });

                let links_len: usize = self.entity_handle_pairs.borrow().len();

                self.entity_handle_pairs
                    .borrow_mut()
                    .push((handle, entity.clone()));
                self.handle_index_by_entity
                    .borrow_mut()
                    .insert(entity.clone(), links_len);

                println!("[ECS Rendering] New command created with link (entity: {} <=> rendering_handle: {})", entity.index(), handle);
            }
        }
    }

    pub fn flush_rendering_command_handles(&mut self, renderer: &mut Renderer) {
        // Process updates & deletions for sprites entities
        {
            let world = self.ecs_world.borrow_mut();
            let container = world
                .get_resource::<RenderingFrameData>()
                .expect("[ECS] Failed to fetch Rendering Resources");
            RenderingBridge::process_deleted_2d_sprites(self, renderer, container);
            RenderingBridge::process_updated_2d_sprites(self, &world, renderer, container);
        }

        // Flush all remaining sprite entities to the rendering layer
        for (handle, _entity) in self.entity_handle_pairs.borrow().iter() {
            renderer.enqueue_cmd_for_current_frame(*handle);
        }

        let mut world = self.get_world_mut();
        // clear all rendering container states
        let mut mut_container = world.get_resource_mut::<RenderingFrameData>().unwrap();
        mut_container.new_2d_render.clear();
        mut_container.deleted_2d_render.clear();
        mut_container.updated_2d_render.clear();
        drop(world);

        // Process frustum culling for 2D sprite entities
        RenderingBridge::process_culled_2d_sprites(self, renderer);
    }

    fn process_culled_2d_sprites(&mut self, renderer: &mut Renderer) {
        let mut culled_count = 0;
        let force_full_pass: bool;

        // flush culling state for entities with transformation changes
        {
            let mut world = self.get_world_mut();
            let mut culling_state = world.get_resource_mut::<CameraCullingState>().unwrap();
            let index_by_entity = self.handle_index_by_entity.borrow();
            let handles = self.entity_handle_pairs.borrow();

            for (entity, state) in culling_state.entities.iter_mut() {
                if let None = self.handle_index_by_entity.borrow().get(entity) {
                    println!("[ECS] Culling entity does not exist {}", entity.index());
                    continue;
                }

                let entity_index = index_by_entity[entity];
                let (rendering_hdl, _) = handles[entity_index];

                if !state.is_visible() {
                    culled_count += 1;
                }

                renderer.cull(rendering_hdl, !state.is_visible());
                self.culled_check.borrow_mut().insert(rendering_hdl);
            }

            force_full_pass = culling_state.force_full_pass;
            culling_state.reset();
        }

        // if the camera moved or a special event occurs, a check of all entities is required
        if force_full_pass {
            let world = self.get_world();
            let culling_state = world.get_resource::<CameraCullingState>().unwrap();

            for (rendering_hdl, sprite_entity) in self.entity_handle_pairs.borrow().iter() {
                if self.culled_check.borrow().contains(*rendering_hdl) {
                    continue;
                }

                let camera_transform = world
                    .get::<Transform>(culling_state.camera_entity.unwrap())
                    .unwrap();
                let entity_transform = world.get::<Transform>(*sprite_entity).unwrap();
                let camera_rect: Rect<f32> = Rect {
                    x: camera_transform.position.x,
                    y: camera_transform.position.y,
                    ..culling_state.camera_world_viewport
                };
                let culled_state = culling_state.compute_visibility(camera_rect, &entity_transform);

                renderer.cull(*rendering_hdl, !culled_state.is_visible());
            }
        }

        if culled_count > 0 {
            println!("[ECS Rendering Bridge] Culled entities count {}", culled_count);
        }

        self.culled_check.borrow_mut().clear();
    }

    fn process_updated_2d_sprites(
        &self,
        world: &World,
        renderer: &mut Renderer,
        container: &RenderingFrameData,
    ) {
        for updated_entity in container.updated_2d_render.iter() {
            let link_index: usize = self
                .handle_index_by_entity
                .borrow()
                .get(updated_entity)
                .unwrap()
                .clone();
            let cmd_handle: RenderCmdHd = self
                .entity_handle_pairs
                .borrow()
                .get(link_index.clone())
                .unwrap()
                .0
                .clone();

            let component: &SpriteRenderer2D =
                world.get::<SpriteRenderer2D>(*updated_entity).unwrap();
            let transform: &Transform = world.get::<Transform>(*updated_entity).unwrap();

            // Blit the texture from the sprite component to the rendering material sprite
            let new_material: Option<Material> =
                component.material.as_ref().map(|map: &Material| {
                    let mut new_material: Material = map.clone();
                    new_material.main_texture = component.texture.clone();
                    return new_material;
                });

            renderer.update_render_command(RenderUpdate {
                render_cmd: cmd_handle,
                mesh_info: None,
                material: new_material,
                transform: Option::from(transform.clone()),
            });
        }
    }

    fn process_deleted_2d_sprites(&self, renderer: &mut Renderer, container: &RenderingFrameData) {
        for entity in container.deleted_2d_render.iter() {
            if self.handle_index_by_entity.borrow().contains_key(entity) {
                let link_index: usize = self
                    .handle_index_by_entity
                    .borrow()
                    .get(entity)
                    .unwrap()
                    .clone();
                let handle: RenderCmdHd = self
                    .entity_handle_pairs
                    .borrow()
                    .get(link_index)
                    .unwrap()
                    .0
                    .clone();

                renderer.remove_render_command(handle);
                self.entity_handle_pairs.borrow_mut().remove(link_index);
                self.handle_index_by_entity.borrow_mut().remove(entity);
            }
        }
    }

    pub fn flush_camera_changes(&mut self, renderer: &mut Renderer) {
        let mut world: RefMut<World> = self.get_world_mut();

        let culling_state = world.get_resource::<CameraCullingState>().unwrap();
        let camera = world.entity(culling_state.camera_entity.unwrap());

        // Flush viewport in case of any changes
        {
            let camera_component = camera.get::<Camera>().unwrap();
            let (x, y, w, h) = camera_component.viewport.clone();
            renderer.update_normalized_viewport(x, y, w, h);
        }

        let resources = world.get_resource::<RenderingFrameData>().unwrap();

        for entity in resources.updated_camera_settings.iter() {
            let camera_comp: &Camera = world.get::<Camera>(*entity).unwrap();
            let transform_comp: &Transform = world.get::<Transform>(*entity).unwrap();
            let background_color = camera_comp.background_color.unwrap_or(ARGB8Color::black());

            renderer.update_camera_settings(RenderingCamera {
                near: camera_comp.near,
                far: camera_comp.far,
                ppu: camera_comp.ppu,
                clear_color: background_color,
                transform: transform_comp.clone(),
            });
        }

        for entity in resources.updated_camera_transform.iter() {
            let camera_comp: &Transform = world.get::<Transform>(*entity).unwrap();
            renderer.update_camera_transform(camera_comp.clone());
        }

        let mut mut_container = world.get_resource_mut::<RenderingFrameData>().unwrap();
        mut_container.updated_camera_settings.clear();
        mut_container.updated_camera_transform.clear();
    }

    pub fn build_camera(world: &mut World) -> Entity {
        let camera = world
            .spawn((Camera::default(), Camera::default_transform()))
            .id();
        camera
    }
}
