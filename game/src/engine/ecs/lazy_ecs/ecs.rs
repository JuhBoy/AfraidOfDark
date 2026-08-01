use std::any::TypeId;
use std::cell::RefCell;

use crate::engine::ecs::lazy_ecs::archetypes::{
    ArchetypeDefinition, ArchetypesManager, ComponentData,
};
use crate::engine::ecs::lazy_ecs::components::{ComponentMetaData, ComponentStorage};
use crate::engine::ecs::lazy_ecs::entities::{Entity, EntityStorage};
use crate::engine::ecs::lazy_ecs::systems::{System, SystemParams, TSystem};

use super::archetypes::ComponentSet;
use super::utils::GroupMask;

// used for macro generation, allow to register multiple storage at once using tuples
pub trait AllocateStorageSet {
    fn allocates(ecs: &mut ECS) -> AllocationResult;
}
pub enum AllocationResult {
    Success,
    Partial,
    Failed,
}

pub struct ECSStats {
    pub archetypes_broken: i32,
}
impl ECSStats {
    pub fn new() -> RefCell<ECSStats> {
        RefCell::new(ECSStats {
            archetypes_broken: 0,
        })
    }

    pub fn reset(&mut self) {
        self.archetypes_broken = 0;
    }
}

#[derive(PartialEq)]
pub enum EntityCreateResult {
    Failed(String),
    Grouped(GroupedEntity),
    Ungrouped(Entity),
}
#[derive(PartialEq)]
pub enum EntityUpdateResult {
    Failed(&'static str),
    Grouped(GroupedEntity),
    Ungrouped(Entity),
}
#[derive(PartialEq)]
pub struct GroupedEntity {
    pub group: GroupMask,
    pub entity: Entity,
}
impl GroupedEntity {
    pub fn default(entity: Entity) -> Self {
        Self {
            entity: entity,
            group: GroupMask::new(None),
        }
    }

    pub fn from(entity: Entity, group: GroupMask) -> Self {
        Self { entity, group }
    }
}
pub struct ECS {
    pub entity_storage: EntityStorage,
    pub component_storage: ComponentStorage,
    pub update_systems: Vec<System>,
    pub archetypes: RefCell<ArchetypesManager>,
    pub stats: RefCell<ECSStats>,
}
impl ECS {
    pub fn default() -> Self {
        ECS {
            entity_storage: EntityStorage::new(2000),
            component_storage: ComponentStorage::new(10, 100),
            update_systems: vec![],
            archetypes: RefCell::new(ArchetypesManager::new()),
            stats: ECSStats::new(),
        }
    }

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

    pub fn allocate_storages<T>(&mut self)
    where
        T: AllocateStorageSet,
    {
        match T::allocates(self) {
            AllocationResult::Partial | AllocationResult::Failed => {
                panic!("failed to allocate all storages")
            }
            _ => (),
        }
    }

    pub fn has_component<T>(&self, entity: Entity) -> bool
    where
        T: 'static,
    {
        if !self.entity_storage.is_valid(entity) {
            return false;
        }
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
        if !self.entity_storage.is_valid(entity) {
            return EntityUpdateResult::Failed("Entity is not valid");
        }

        let storage = &mut self.component_storage;
        let mut arch_manager = self.archetypes.borrow_mut();

        let inserted = TCompSet::insert(&entity, storage, components);
        if !inserted {
            return EntityUpdateResult::Failed(
                "failed to insert entity, ensure components have storages",
            );
        }

        let group = self
            .entity_storage
            .get_group(entity)
            .map_or(GroupMask::new(None), |g| g);
        let group_result = TCompSet::group(
            &GroupedEntity::from(entity, group),
            &mut arch_manager,
            storage,
        );
        let mut requested_group = TCompSet::group_mask(&storage);

        // update group for entity metadata
        {
            let current_group: GroupMask = self
                .entity_storage
                .get_group(entity)
                .map_or(GroupMask::new(None), |f| f);

            requested_group.or(current_group.get_raw());
            self.entity_storage.set_group(entity, requested_group);
        }

        match group_result {
            true => EntityUpdateResult::Grouped(GroupedEntity {
                group: requested_group,
                entity: entity,
            }),
            _ => EntityUpdateResult::Ungrouped(entity),
        }
    }

    pub fn remove_component<TCompSet>(&mut self, entity: Entity) -> bool
    where
        TCompSet: ComponentSet + 'static,
    {
        if !self.entity_storage.is_valid(entity) {
            return false;
        }

        let storage = &mut self.component_storage;
        let mut archetype = self.archetypes.borrow_mut();

        let ungrouped = TCompSet::ungroup(entity, &mut archetype, storage);
        let mb_group = self.entity_storage.get_group(entity);

        let has_removed_comps = TCompSet::remove(entity, storage) > 0;

        if let Some(mut current_mask) = mb_group {
            let exclusive_mask = TCompSet::group_mask(storage);
            current_mask.excludes(&exclusive_mask);

            self.entity_storage.set_group(entity, current_mask);

            if ungrouped {
                println!("entity {:?} ungrouped", entity);
            }
        }

        has_removed_comps
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

    pub fn create<TCompSet>(&mut self, comps: TCompSet) -> EntityCreateResult
    where
        TCompSet: ComponentSet + 'static,
    {
        let entity = self.entity_storage.create();

        let inserted: bool = TCompSet::insert(&entity, &mut self.component_storage, comps);
        if !inserted {
            let error = format!(
                "failed to insert entity, ensure components have storages (comps: {:?})",
                TypeId::of::<TCompSet>(),
            );

            self.entity_storage.remove(entity);
            return EntityCreateResult::Failed(error);
        }

        let group_mask: GroupMask = TCompSet::group_mask(&self.component_storage);
        let grouped: bool = TCompSet::group(
            &GroupedEntity::default(entity),
            &mut self.archetypes.borrow_mut(),
            &mut self.component_storage,
        );
        self.entity_storage.set_group(entity, group_mask);

        match grouped {
            true => {
                return EntityCreateResult::Grouped(GroupedEntity {
                    group: group_mask,
                    entity,
                });
            }
            _ => {
                // do something
                return EntityCreateResult::Ungrouped(entity);
            }
        }
    }

    pub fn destroy(&mut self, entity: Entity) -> bool {
        let entity_group: Option<GroupMask> = self.entity_storage.get_group(entity);
        let removed: bool = self.entity_storage.remove(entity);

        if !removed {
            return false;
        }

        if let Some(entity_group) = entity_group {
            let archetypes = &mut self.archetypes.borrow_mut();
            let _ungrouped = self
                .component_storage
                .ungroup(entity, entity_group, archetypes);
        }

        if let Some(mut entity_group) = entity_group {
            while !entity_group.is_empty() {
                let store_id: usize = entity_group.least_one() as usize;
                entity_group.and(entity_group.get_raw() - 1);

                let mut store = self.component_storage.get_storage_mut_by_id(store_id);
                let removed = store.remove(entity);

                if removed.is_none() {
                    panic!("a component present in the entity metadata is not present in storage component");
                }
            }
        }

        true
    }

    pub fn reset(&mut self) {
        self.entity_storage.reset();
        self.component_storage.reset();
        self.update_systems.clear();
        self.archetypes.borrow_mut().reset();
        self.stats.borrow_mut().reset();
    }
}

macro_rules! allocate_buffers {
    ( ($( ($comps:tt, $index:tt) ),+); $count:tt ) => {
        impl<$($comps),*> AllocateStorageSet for ($($comps,)*) where $($comps: 'static),* {
            fn allocates(ecs: &mut ECS) -> AllocationResult {
                let mut allocated: usize = 0;

                $(
                    let meta = ecs.component_storage.allocate::<$comps>();
                    if meta.is_some() {
                        allocated += 1;
                    } else {
                        println!("[storage] failed to allocate $comps");
                    }
                )*


                match allocated {
                    $count => AllocationResult::Success,
                    1.. => AllocationResult::Partial,
                    _ => AllocationResult::Failed
                }
            }
        }
    };
}

allocate_buffers!(((A, 0)); 1);
allocate_buffers!(((A, 0), (B, 1)); 2);
allocate_buffers!(((A, 0), (B, 1), (C, 2)); 3);
allocate_buffers!(((A, 0), (B, 1), (C, 2), (D, 3)); 4);
allocate_buffers!(((A, 0), (B, 1), (C, 2), (D, 3), (E, 4)); 5);
