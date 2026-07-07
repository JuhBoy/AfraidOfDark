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

pub struct ECSStats {
    pub archetypes_broken: i32,
}
impl ECSStats {
    pub fn new() -> RefCell<ECSStats> {
        RefCell::new(ECSStats {
            archetypes_broken: 0,
        })
    }
}

#[derive(PartialEq)]
pub enum EntityCreateResult {
    Failed(String),
    Grouped(EntityWithGroup),
    Ungrouped(Entity),
}
#[derive(PartialEq)]
pub enum EntityUpdateResult {
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
    pub stats: RefCell<ECSStats>,
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

    pub fn has_component<T>(&self, entity: Entity) -> bool
    where
        T: 'static,
    {
        return self.component_storage.has_component::<T>(entity);
    }

    pub fn add_component<TCompSet>(
        &mut self,
        entity: Entity,
        components: TCompSet,
    ) -> EntityUpdateResult
    where
        TCompSet: ComponentSet + 'static,
    {
        let storage = &mut self.component_storage;
        let mut arch_manager = self.archetypes.borrow_mut();

        let inserted = TCompSet::insert(&entity, storage, components);
        if !inserted {
            let error = format!(
                "failed to insert entity, ensure components have storages (comps: {:?})",
                TypeId::of::<TCompSet>(),
            );
            return EntityUpdateResult::Failed(error);
        }

        let _group_result = TCompSet::group(&entity, &mut arch_manager, storage);

        // @todo: see later how this should be used to debug easily
        #[cfg(debug_ecs)]
        {
            println!("entity grouped on add component ? {}", _group_result);
        }

        let group_mask = TCompSet::group_mask(&storage);

        return match _group_result {
            true => EntityUpdateResult::Grouped(EntityWithGroup {
                group: group_mask,
                entity: entity,
            }),
            _ => EntityUpdateResult::Ungrouped(entity),
        };
    }

    pub fn remove_component<T>(&mut self, entity: Entity) -> bool
    where
        T: ComponentSet + 'static,
    {
        let storage = &mut self.component_storage;
        let mut archetype = self.archetypes.borrow_mut();

        let ungrouped = T::ungroup(entity, &mut archetype, storage);

        ungrouped
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

        let inserted: bool = A::insert(&entity, &mut self.component_storage, comps);
        if !inserted {
            let error = format!(
                "failed to insert entity, ensure components have storages (comps: {:?})",
                TypeId::of::<A>(),
            );

            self.entity_storage.remove(entity);
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
