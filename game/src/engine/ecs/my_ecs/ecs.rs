use crate::engine::ecs::my_ecs::components::ComponentStorage;
use crate::engine::ecs::my_ecs::entities::{Entity, EntityStorage};
use crate::engine::ecs::my_ecs::systems::System;

pub struct ECS {
    pub entity_storage: EntityStorage,
    pub component_storage: ComponentStorage,
    pub update_systems: Vec<System>,
}
impl ECS {
    pub fn update(&mut self) {
        self.update_systems
            .iter_mut()
            .for_each(|system| ((system).action)())
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

    pub fn add_component<T>(&mut self, entity: Entity) -> bool
    where
        T: 'static,
    {
        self.component_storage.add_component::<T>(entity)
    }
}
