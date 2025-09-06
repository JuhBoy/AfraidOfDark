use std::any::TypeId;
use std::cell::RefCell;
use std::string;

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
    WithGroup(EntityWithGroup),
    WithoutGroup(Entity),
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
        // let components: &[ComponentData] = A::COMPONENTS;

        // ----
        // @todo: working here probably needs to rework everything after this point /!\
        let inserted = A::insert(&entity, &mut self.component_storage, comps);
        if !inserted {
            let error = format!(
                "failed to insert entity, ensure components have storages (comps: {:?})",
                TypeId::of::<A>(),
            );
            return EntityCreateResult::Failed(error);
        }

        // now try to push this entity in a group if possible !
        let entity_group: GroupMask = A::group_mask(&self.component_storage);
        let am = &mut self.archetypes.borrow_mut();
        let maybe_runtime_group = am.find_archetype_with_group(&entity_group);

        if let Some((arch_id, runtime_group)) = maybe_runtime_group {
            let supersets = am.get_supersets_slice(arch_id, &runtime_group.mask);

            for superset in 0..supersets.len() {


            }
        }

        match maybe_runtime_group {
            Some(_) => EntityCreateResult::WithGroup(EntityWithGroup {
                group: entity_group,
                entity,
            }),
            None => EntityCreateResult::WithoutGroup(entity),
        }
    }
}
