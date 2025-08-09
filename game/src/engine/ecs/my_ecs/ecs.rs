use std::cell::RefCell;

use crate::engine::ecs::my_ecs::archetypes::{
    ArchetypeDefinition, ArchetypesManager, ComponentData,
};
use crate::engine::ecs::my_ecs::components::ComponentStorage;
use crate::engine::ecs::my_ecs::entities::{Entity, EntityStorage};
use crate::engine::ecs::my_ecs::systems::{System, SystemParams, TSystem};

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

    pub fn allocate_storage<T>(&mut self) -> bool
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
}
