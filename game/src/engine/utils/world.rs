use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use bevy_ecs::entity::Entity;

use crate::engine::rendering::renderer::{MeshInfo, RenderCmdHd, RenderRequest, Renderer };
use crate::engine::rendering::shaders::Material;
use crate::engine::ecs::{ time::RenderingResourcesContainer, components::SpriteRenderer2D };

pub struct World {
    m_world: bevy_ecs::world::World,
    rhandle_links: RefCell<Vec<(RenderCmdHd, Entity)>>, // rendering handle that map a gfx handle to an Entity
    rhandle_to_link_index: RefCell<HashMap<Entity, usize>> // map entity to the index of the rhandle_links
}

impl World {
    pub fn new() -> Self {
        Self {
            m_world: bevy_ecs::world::World::new(),
            rhandle_links: RefCell::new(Vec::new()),
            rhandle_to_link_index: RefCell::new(HashMap::new())
        }
    }

    pub fn inject_new_rendering_entities(&self, renderer: &mut Renderer) {
        let container = self.m_world.resource::<RenderingResourcesContainer>();

        for entity in container.new_2d_render.iter() {
            let entity_ref: bevy_ecs::world::EntityRef<'_> = self.m_world.entity(entity.clone());

            if let Some(comp) = entity_ref.get::<SpriteRenderer2D>() {
                let handle: RenderCmdHd = renderer.create_render_command(RenderRequest {
                    mesh_info: MeshInfo { // Todo: Will be default until mesh is implemented
                        file_path: None,
                        count: 0,
                        vertices_set: None,
                    },
                    material: comp.material.clone().unwrap_or(Arc::new(Material::default())),
                });

                let links_len: usize = self.rhandle_links.borrow().len();

                self.rhandle_links.borrow_mut().push((handle, entity.clone()));
                self.rhandle_to_link_index.borrow_mut().insert(entity.clone(), links_len);

                println!("new request command will fetch with tex {}", comp.texture.as_ref().unwrap_or(&String::from("none")));
            }
        }
    }

    pub fn flush_rendering_command_handles(&mut self, renderer: &mut Renderer) {
        let world: &mut bevy_ecs::world::World = &mut self.m_world;
        let mut container = world.get_resource_mut::<RenderingResourcesContainer>().expect("RenderingResourcesContainer not found");

        for entity in container.deleted_2d_render.iter() {
            if self.rhandle_to_link_index.borrow().contains_key(entity) {
                let link_index: usize = self.rhandle_to_link_index.borrow().get(entity).unwrap().clone();
                let handle: RenderCmdHd = self.rhandle_links.borrow().get(link_index).unwrap().0.clone();

                renderer.remove_render_command(handle);
                self.rhandle_links.borrow_mut().remove(link_index);
                self.rhandle_to_link_index.borrow_mut().remove(entity);
            }
        }

        for updated_entity in container.updated_2d_render.iter() { 
            println!("[TODO]: updated entity {:?}", updated_entity); // todo: implement changes there.
        }

        for (handle, _entity) in self.rhandle_links.borrow().iter() {
            renderer.enqueue_cmd_for_current_frame(handle.clone());
            println!("[Rendering Bridge]: enqueue entity {:?}", _entity);
        }

        container.new_2d_render.clear();
        container.deleted_2d_render.clear();
        container.updated_2d_render.clear();
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