use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bevy_ecs::entity::Entity;

use crate::engine::ecs::components::{Camera, Transform};
use crate::engine::ecs::components::Projection;
use crate::engine::ecs::{components::SpriteRenderer2D, time::RenderingResourcesContainer};
use crate::engine::rendering::components::{
    MeshInfo, RenderRequest, RenderUpdate, RenderingCamera,
};
use crate::engine::rendering::renderer::{RenderCmdHd, Renderer};
use crate::engine::rendering::renderer_helpers::prepare_material;
use crate::engine::rendering::shaders::Material;

pub struct World {
    m_world: bevy_ecs::world::World,
    rhandle_links: RefCell<Vec<(RenderCmdHd, Entity)>>, // rendering handle that map a gfx handle to an Entity
    rhandle_to_link_index: RefCell<HashMap<Entity, usize>>, // map entity to the index of the rhandle_links

    main_camera: Entity,
}

impl World {
    pub fn new() -> Self {
        let mut world = bevy_ecs::world::World::new();

        Self {
            main_camera: world
                .spawn((
                    Camera {
                        fov: 80.0,
                        near: 0.1,
                        far: 100.0,
                        viewport: (0.0, 0.0, 1.0, 1.0),
                        mode: Projection::Orthographic,
                        output_target: Option::None,
                        background_color: Option::None,
                    },
                    Transform {
                        position: Default::default(),
                        rotation: Default::default(),
                        scale: Default::default(),
                    },
                ))
                .id(),

            m_world: world,
            rhandle_links: RefCell::new(Vec::new()),
            rhandle_to_link_index: RefCell::new(HashMap::new()),
        }
    }

    pub fn inject_new_rendering_entities(&self, renderer: &mut Renderer) {
        let container = self.m_world.resource::<RenderingResourcesContainer>();

        for entity in container.new_2d_render.iter() {
            let entity_ref: bevy_ecs::world::EntityRef<'_> = self.m_world.entity(entity.clone());

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

                let links_len: usize = self.rhandle_links.borrow().len();

                self.rhandle_links
                    .borrow_mut()
                    .push((handle, entity.clone()));
                self.rhandle_to_link_index
                    .borrow_mut()
                    .insert(entity.clone(), links_len);

                println!("[ECS Rendering] New command created with link (entity: {} <=> rendering_handle: {})", entity.index(), handle);
            }
        }
    }

    pub fn flush_rendering_command_handles(&mut self, renderer: &mut Renderer) {
        let world: &mut bevy_ecs::world::World = &mut self.m_world;
        let container = world
            .get_resource::<RenderingResourcesContainer>()
            .expect("[ECS] Failed to fetch Rendering Ressources");

        for entity in container.deleted_2d_render.iter() {
            if self.rhandle_to_link_index.borrow().contains_key(entity) {
                let link_index: usize = self
                    .rhandle_to_link_index
                    .borrow()
                    .get(entity)
                    .unwrap()
                    .clone();
                let handle: RenderCmdHd = self
                    .rhandle_links
                    .borrow()
                    .get(link_index)
                    .unwrap()
                    .0
                    .clone();

                renderer.remove_render_command(handle);
                self.rhandle_links.borrow_mut().remove(link_index);
                self.rhandle_to_link_index.borrow_mut().remove(entity);
            }
        }

        for updated_entity in container.updated_2d_render.iter() {
            let link_index: usize = self
                .rhandle_to_link_index
                .borrow()
                .get(updated_entity)
                .unwrap()
                .clone();
            let cmd_handle: RenderCmdHd = self
                .rhandle_links
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

        for (handle, _entity) in self.rhandle_links.borrow().iter() {
            renderer.enqueue_cmd_for_current_frame(handle.clone());
        }

        let mut mut_container = world
            .get_resource_mut::<RenderingResourcesContainer>()
            .unwrap();

        mut_container.new_2d_render.clear();
        mut_container.deleted_2d_render.clear();
        mut_container.updated_2d_render.clear();
    }

    pub fn flush_camera_changes(&mut self, renderer: &mut Renderer) {
        let world: &mut bevy_ecs::world::World = &mut self.m_world;
        let camera = world.entity(self.main_camera);

        // Flush viewport in case of any changes
        {
            let camera_component = camera.get::<Camera>().unwrap();
            let (x, y, w, h) = camera_component.viewport.clone();
            renderer.update_viewport(x, y, w, h);
        }

        let resources = world.get_resource::<RenderingResourcesContainer>().unwrap();

        for entity in resources.updated_camera_settings.iter() {
            const DEFAULT_BACKGROUND_COLOR: [f32; 3] = [1.0, 1.0, 1.0];

            let camera_comp: &Camera = world.get::<Camera>(*entity).unwrap();
            let transform_comp: &Transform = world.get::<Transform>(*entity).unwrap();
            let background_color = camera_comp
                .background_color
                .unwrap_or(DEFAULT_BACKGROUND_COLOR);

            renderer.update_camera_settings(RenderingCamera {
                near: camera_comp.near,
                far: camera_comp.far,
                background_color,
                transform: transform_comp.clone(),
            });
        }

        for entity in resources.updated_camera_transform.iter() {
            let camera_comp: &Transform = world.get::<Transform>(*entity).unwrap();
            renderer.update_camera_transform(camera_comp.clone());
        }

        let mut mut_container = world
            .get_resource_mut::<RenderingResourcesContainer>()
            .unwrap();
        mut_container.updated_camera_settings.clear();
        mut_container.updated_camera_transform.clear();
    }

    pub fn get_main_camera(&self) -> Entity {
        self.main_camera
    }
}

impl Deref for World {
    type Target = bevy_ecs::world::World;

    fn deref(&self) -> &Self::Target {
        &self.m_world
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.m_world
    }
}
