use std::any::TypeId;
use std::cell::RefCell;

use crate::engine::ecs::my_ecs::archetypes::{
    ArchetypeDefinition, ArchetypesManager, ComponentData,
};
use crate::engine::ecs::my_ecs::components::{ComponentMetaData, ComponentStorage};
use crate::engine::ecs::my_ecs::entities::{Entity, EntityStorage};
use crate::engine::ecs::my_ecs::systems::{System, SystemParams, TSystem};

use super::archetypes::ComponentSet;
use super::utils::GroupMask;

#[derive(PartialEq)]
pub enum EntityCreateResult {
    Failed(String),
    Grouped(EntityWithGroup),
    Ungrouped(Entity),
}
#[derive(PartialEq)]
pub struct EntityWithGroup {
    pub group: GroupMask,
    pub entity: Entity,
}
pub struct ECS {
    pub entity_storage: EntityStorage,
    pub component_storage: ComponentStorage,
    pub update_systems: Vec<System>,
    pub archetypes: RefCell<ArchetypesManager>,
}
impl ECS {
    pub fn update(&mut self) {
        let ptr = self as *mut ECS;

        for system in self.update_systems.iter() {
            unsafe {
                let params = SystemParams {
                    world: &mut *ptr,
                    system_name: system.name,
                };
                system.run(params);
            }
        }
    }

    pub fn fixed_update(&mut self) {}

    pub fn late_update(&mut self) {}

    pub fn register_system(&mut self, system: System) {
        self.update_systems.push(system);
    }

    pub fn allocate_storage<T>(&mut self) -> Option<ComponentMetaData>
    where
        T: 'static,
    {
        self.component_storage.allocate::<T>()
    }

    pub fn add_component<T>(&mut self, entity: Entity, comp: T) -> bool
    where
        T: 'static,
    {
        self.component_storage.add_component::<T>(entity, comp)
    }

    pub fn make_archetype<A>(&self) -> bool
    where
        A: ArchetypeDefinition,
    {
        let components: &[ComponentData] = A::COMPONENTS;
        let mut manager = self.archetypes.borrow_mut();
        let success: bool = manager.register(components);

        success
    }

    pub fn flush_archetypes(&mut self) -> usize {
        let flushed = self
            .archetypes
            .borrow_mut()
            .flush_archetypes(&mut self.component_storage);

        flushed
    }

    pub fn create<A>(&mut self, comps: A) -> EntityCreateResult
    where
        A: ComponentSet + 'static,
    {
        let entity = self.entity_storage.create();

        let inserted = A::insert(&entity, &mut self.component_storage, comps);
        if !inserted {
            let error = format!(
                "failed to insert entity, ensure components have storages (comps: {:?})",
                TypeId::of::<A>(),
            );
            return EntityCreateResult::Failed(error);
        }

        let entity_group_mask: GroupMask = A::group_mask(&self.component_storage);
        let grouped: bool = A::group(
            &entity,
            &mut self.archetypes.borrow_mut(),
            &mut self.component_storage,
        );

        match grouped {
            true => EntityCreateResult::Grouped(EntityWithGroup {
                group: entity_group_mask,
                entity,
            }),
            _ => EntityCreateResult::Ungrouped(entity),
        }
    }
}
