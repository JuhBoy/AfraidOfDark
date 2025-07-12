use crate::engine::ecs::my_ecs::components::{ComponentBufferSparseSet, ComponentStorage};
use crate::engine::ecs::my_ecs::ecs::ECS;
use crate::engine::ecs::my_ecs::entities::{Entity, EntityAllocator, EntityStorage};
use crate::engine::ecs::my_ecs::systems::{System, SystemUpdate};
use crate::engine::ecs::my_ecs::utils::{ByteBuffer, SparseSet, SparseVec, ID};
use std::any::TypeId;

pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[test]
pub fn test_entity_storage_implementation() {
    let mut storage = EntityStorage {
        entities: SparseSet {
            dense_set: Vec::new(),
            sparse_views: SparseVec::new(10),
        },
        allocator: EntityAllocator::new(),
    };

    for _i in 0..10 {
        let _entity = storage.create();
    }
    assert_eq!(storage.entities.len(), 10);

    storage.entities.insert(Entity { id: 3, version: 2 });
    assert_eq!(storage.entities.has(Entity { id: 3, version: 2 }), true);
    assert_eq!(storage.entities.has(Entity { id: 3, version: 1 }), false);

    let removed = storage.remove(Entity { id: 0, version: 1 });
    let _re_create_0_entity = storage.create();
    assert_eq!(storage.entities.has(Entity { id: 0, version: 2 }), true);
    assert_eq!(true, removed);

    let mut i = 0;
    for ett in storage.entities.iter() {
        assert_eq!(i, ett.id);
        i += 1;
    }
    assert_eq!(i, 10);

    storage.reset();
}

#[test]
pub fn test_byte_buffer_implementation() {
    let Some(mut buffer) = ByteBuffer::with_capacity::<Velocity>(100) else {
        return;
    };

    for i in 0..100 {
        let alloc_idx = buffer.allocate(Velocity {
            x: i as f32 * 1.0,
            y: 0.0,
        });
        assert!(alloc_idx.is_some());
        assert_eq!(alloc_idx.unwrap(), i);
    }
    assert!(true);

    let velocity_mut = buffer.get_mut_ref::<Velocity>(50);
    if let Some(velvel) = velocity_mut {
        velvel.x = 2455.123;
    }

    for i in 0..100 {
        let velocity = buffer.get_ref::<Velocity>(i);
        assert!(velocity.is_some());

        if i == 50 {
            assert_eq!(velocity.unwrap().x, 2455.123f32);
            continue;
        }

        let vel = velocity.unwrap();
        assert_eq!(vel.x, i as f32 * 1.0);
    }

    buffer.clear::<Velocity>();
}

#[test]
pub fn test_component_storage() {
    let mut com_storage = ComponentBufferSparseSet {
        entities: SparseVec::new(100),
        component_buffer: ByteBuffer::with_capacity::<Velocity>(100).unwrap(),
        entity_to_component: Vec::with_capacity(100),
    };

    let entity_0 = Entity { id: 0, version: 1 };
    let entity_10 = Entity { id: 10, version: 1 };

    let success = com_storage.insert(entity_0, Velocity { x: 0.0, y: 0.0 });
    assert!(success);

    let removed = com_storage.remove::<Velocity>(entity_0);
    assert!(removed.is_some());
    assert!(!com_storage.has(entity_0));

    let inserted = com_storage.insert(entity_10, Velocity { x: 50.0, y: 0.0 });
    let Some(pos) = com_storage.entities.get_unchecked(entity_10.id()) else {
        panic!("couldn't get the entity view");
    };
    assert!(inserted);
    assert_eq!(pos.index, 0);

    let insert = com_storage.insert(entity_0, Velocity { x: 100.0, y: 0.0 });
    let Some(pos_0) = com_storage.entities.get_unchecked(entity_0.id) else {
        panic!("couldn't get the entity view");
    };
    assert!(insert);
    assert_eq!(pos_0.index, 1);

    let removed_2 = com_storage.remove::<Velocity>(entity_0);
    let removed_3 = com_storage.remove::<Velocity>(entity_10);
    assert!(removed_2.is_some());
    assert!(removed_3.is_some());
    assert_eq!(com_storage.component_buffer.len, 0);

    for i in 0..100 {
        let ett = Entity { id: i, version: 1 };
        let loop_inserted = com_storage.insert(
            ett,
            Velocity {
                x: i as f32,
                y: 0.0,
            },
        );
        assert!(loop_inserted);
    }
    assert_eq!(com_storage.component_buffer.len, 100);

    for i in 0..50 {
        let ett = Entity { id: i, version: 1 };
        let loop_removed = com_storage.remove::<Velocity>(ett);
        assert!(loop_removed.is_some());
    }
    assert_eq!(com_storage.component_buffer.len, 50);

    for i in 50..100 {
        let ett = Entity { id: i, version: 1 };
        let is_there = com_storage.has(ett);
        assert!(is_there);

        {
            let get = com_storage.get::<Velocity>(ett).unwrap();
            assert_eq!(i as f32, get.x);
        }

        let sparse = com_storage.entities.get_unchecked(ett.id());
        assert!(sparse.is_some());
        assert_eq!(99 - i, sparse.unwrap().index);

        let velocity = com_storage.get::<Velocity>(ett);
        assert!(velocity.is_some());
        assert!(velocity.unwrap().x >= 50f32);
    }

    for i in 50..100 {
        let ett = Entity { id: i, version: 1 };
        com_storage.remove::<Velocity>(ett);
    }
    assert_eq!(com_storage.component_buffer.len, 0);

    for i in 0..10 {
        let ett = Entity { id: i, version: 1 };
        let i_inserted = com_storage.insert(
            ett,
            Velocity {
                x: i as f32,
                y: 0.0,
            },
        );
        assert!(i_inserted);
    }

    let r_5 = com_storage.insert(Entity { id: 5, version: 1 }, Velocity { x: 500f32, y: 0.0 });
    assert!(r_5);

    let r_5_vel = com_storage.get::<Velocity>(Entity { id: 5, version: 1 }).unwrap();
    assert_eq!(500f32, r_5_vel.x);
}

#[test]
pub fn test_ecs_implementation() {
    let mut ecs = ECS {
        entity_storage: EntityStorage::new(1000),
        component_storage: ComponentStorage::new(100),
        update_systems: vec![],
    };

    // create an entity
    let entity = ecs.entity_storage.create();

    // register one system WIP
    let system = System {
        name: "the one system",
        update_type: SystemUpdate::Update,
        action: ecs_system,
        query: vec![TypeId::of::<Velocity>(), TypeId::of::<Position>()],
    };
    ecs.register_system(system);

    // registries
    ecs.component_storage.allocate::<Velocity>();
    ecs.allocate_storage::<Position>();

    // add a new component to an entity
    let _ = ecs.component_storage.add_component::<Velocity>(entity);
    let _ = ecs.add_component::<Position>(entity);
}

pub fn ecs_system() {}
