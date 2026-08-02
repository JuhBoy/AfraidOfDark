use std::cell::RefCell;

use lazy_macro::lazy_ecs_component;

use crate::{
    engine::ecs::lazy_ecs::{
        archetypes::{ArchetypesManager, ComponentData, ComponentSet},
        components::ComponentStorage,
        ecs::{ECSStats, EntityCreateResult, GroupedEntity, ECS},
        entities::EntityStorage,
        resources::Resources,
    },
    make_lazy_resources, 
};

#[lazy_ecs_component]
pub struct A {}

#[derive(Clone, Copy)]
pub struct B;

#[derive(Clone, Copy)]
pub struct C;

#[derive(Clone, Copy)]
pub struct D;

#[derive(Clone, Copy)]
pub struct E;

#[derive(Clone, Copy)]
pub struct AData(pub i32);

pub struct NonCopyA {
    pub a: u32,
}

// NOTE(JuH): Exemple of resources, PlayerLife & PlayerMana needs the `make_lazy_resources!` macro to get the trait implemented
// which grant them with indexes for the Resources container
#[allow(dead_code)]
pub struct PlayerLife {
    pub value: i32,
    pub name: String,
}
#[allow(dead_code)]
pub struct PlayerMana(pub u32);
make_lazy_resources!(PlayerLife, PlayerMana);

pub const GROUP_AB: &[ComponentData] = &[ComponentData::new::<A>(), ComponentData::new::<B>()];
pub const GROUP_ABC: &[ComponentData] = &[
    ComponentData::new::<A>(),
    ComponentData::new::<B>(),
    ComponentData::new::<C>(),
];
pub const GROUP_ABCD: &[ComponentData] = &[
    ComponentData::new::<A>(),
    ComponentData::new::<B>(),
    ComponentData::new::<C>(),
    ComponentData::new::<D>(),
];
pub const GROUP_ABCDE: &[ComponentData] = &[
    ComponentData::new::<A>(),
    ComponentData::new::<B>(),
    ComponentData::new::<C>(),
    ComponentData::new::<D>(),
    ComponentData::new::<E>(),
];

pub fn create_ecs() -> ECS {
    let ecs = ECS {
        entity_storage: EntityStorage::new(2000),
        component_storage: ComponentStorage::new(100, 2000),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
        stats: ECSStats::new(),

        resources: RefCell::new(Resources::new()),
    };
    ecs
}

pub fn create_ecs_with_capacities(entities: usize, components: usize) -> ECS {
    let ecs = ECS {
        entity_storage: EntityStorage::new(entities),
        component_storage: ComponentStorage::new(100, components),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
        stats: ECSStats::new(),

        resources: RefCell::new(Resources::new()),
    };
    ecs
}

pub fn create_archetypes(ecs: &mut ECS, components: Vec<&'static [ComponentData]>) {
    let mut archetypes = ecs.archetypes.borrow_mut();

    for comp_data in components.iter() {
        let result = archetypes.register(*comp_data);
        assert!(result, "faield to register group AB/C");
    }

    let flush_len = archetypes.flush_archetypes(&mut ecs.component_storage);
    assert!(flush_len > 0);
}

pub fn create_entities<TComponentSet>(
    ecs: &mut ECS,
    count: usize,
    components: fn(usize) -> TComponentSet,
) -> Vec<GroupedEntity>
where
    TComponentSet: ComponentSet + 'static,
{
    let mut entities_buffer: Vec<GroupedEntity> = Vec::with_capacity(count);

    for i in 0..count {
        let comp_set = (components)(i);
        let create_res = ecs.create(comp_set);

        match create_res {
            EntityCreateResult::Failed(reason) => {
                assert!(false, "failed to create entity {}: {}", i, &reason)
            }
            EntityCreateResult::Grouped(grouped_entity) => entities_buffer.push(grouped_entity),
            EntityCreateResult::Ungrouped(entity) => {
                entities_buffer.push(GroupedEntity::default(entity))
            }
        }
    }

    entities_buffer
}
