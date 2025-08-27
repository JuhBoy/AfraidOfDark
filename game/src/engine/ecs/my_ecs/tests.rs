use bevy_ecs::world;

use super::{
    archetypes::ComponentData,
    components::ComponentStorage,
    ecs::ECS,
    systems::{System, SystemUpdate},
};
use crate::engine::ecs::my_ecs::{
    archetypes::ArchetypesManager, ecs::EntityCreateResult, systems::{make_system, Query, QueryMut, SystemParams}
};
use crate::engine::ecs::my_ecs::{
    components::ComponentBufferSparseSet,
    entities::{Entity, EntityAllocator, EntityStorage},
    utils::{ByteBuffer, SparseSet},
};
use crate::engine::ecs::my_ecs::{
    ecs::EntityWithGroup,
    utils::{GroupMask, SparseVec},
};
use std::{any::TypeId, cell::RefCell, time::SystemTime};

pub struct Position {
    pub x: f32,
    pub y: f32,
}

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

pub struct Transform(i32);
pub struct Rigidbody2D(i32);
pub struct BoxCollider(i32);

pub struct A;
pub struct B;
pub struct C;
pub struct D;
pub struct E;

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
    let Some(pos) = com_storage.entities.get_unchecked(entity_10.id) else {
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

        let sparse = com_storage.entities.get_unchecked(ett.id);
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

    let r_5_vel = com_storage
        .get::<Velocity>(Entity { id: 5, version: 1 })
        .unwrap();
    assert_eq!(500f32, r_5_vel.x);
}

#[test]
pub fn test_ecs_implementation() {
    let mut ecs = ECS {
        entity_storage: EntityStorage::new(2000),
        component_storage: ComponentStorage::new(100),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
    };

    // create an entity
    let _entity = ecs.entity_storage.create();

    // register one system
    let system = make_system("the one system", SystemUpdate::Update, iter_test_system);
    ecs.register_system(system);

    // registries
    ecs.component_storage.allocate::<Velocity>();
    ecs.component_storage.allocate::<Position>();
    ecs.allocate_storage::<Position>();

    // creates archetype (WIP)
    let _success = ecs.make_archetype::<(Position, Velocity)>();
    let _success_2 = ecs.make_archetype::<(Position, Velocity, Rigidbody2D)>();
    let _success_3 = ecs.make_archetype::<(Position, Velocity, Rigidbody2D, BoxCollider)>();

    for i in 0..1000 {
        let ett = ecs.entity_storage.create();
        let add_pos = ecs.add_component::<Position>(
            ett,
            Position {
                x: i as f32,
                y: 0f32,
            },
        );
        let add_vel = ecs.add_component::<Velocity>(
            ett,
            Velocity {
                x: i as f32,
                y: 0f32,
            },
        );

        if !add_pos || !add_vel {
            panic!("[Tests] Failled to add components");
        }
    }

    for i in 0..60 {
        println!("Update frame {}", i);
        let time = SystemTime::now();
        ecs.update();
        match time.elapsed() {
            Ok(time) => println!(
                "Duration: {}ns | {}ms ==================",
                time.as_nanos(),
                (time.as_nanos() as f64) / 1e+6f64
            ),
            Err(_) => panic!(),
        }
    }
}

#[test]
pub fn test_archetypes_registers() {
    let mut manager = ArchetypesManager::new();

    const FIRST_GROUP: &[ComponentData] = &[ComponentData::new::<A>(), ComponentData::new::<B>()];
    const SECOND_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
    ];
    const NON_INSERTED_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<D>(),
    ];
    const THIRD_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
        ComponentData::new::<E>(),
    ];
    const FOURTH_GROUP: &[ComponentData] = &[
        ComponentData::new::<Position>(),
        ComponentData::new::<Rigidbody2D>(),
    ];
    const FIFTH_GROUP: &[ComponentData] = &[
        ComponentData::new::<Position>(),
        ComponentData::new::<Rigidbody2D>(),
        ComponentData::new::<Velocity>(),
    ];

    let second_group_inserted = manager.register(SECOND_GROUP);
    let third_group_inserted = manager.register(THIRD_GROUP);
    let first_group_inserted = manager.register(FIRST_GROUP);
    let fourth_group_inserted = manager.register(FOURTH_GROUP);
    let fifth_group_inserted = manager.register(FIFTH_GROUP);
    let fail_group_inserted = manager.register(NON_INSERTED_GROUP);

    // inserted groups
    assert!(first_group_inserted);
    assert!(second_group_inserted);
    assert!(third_group_inserted);
    assert!(fourth_group_inserted);
    assert!(fifth_group_inserted);

    // rejected groups
    assert!(!fail_group_inserted);

    assert_eq!(FIRST_GROUP.len(), manager.layouts[0].components.len());
    assert_eq!(SECOND_GROUP.len(), manager.layouts[1].components.len());

    // parenting
    assert_eq!(3, manager.layouts[0].set_len);
    assert_eq!(1, manager.layouts[1].set_len);
    assert_eq!(1, manager.layouts[2].set_len);
    assert_eq!(2, manager.layouts[3].set_len);
    assert_eq!(1, manager.layouts[4].set_len);

    // assert group order
    assert!(FIRST_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[0].components[i] == *y }));
    assert!(SECOND_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[1].components[i] == *y }));
    assert!(THIRD_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[2].components[i] == *y }));
    assert!(FOURTH_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[3].components[i] == *y }));
}

#[test]
pub fn should_flush_archetypes() {
    let mut manager = ArchetypesManager::new();
    let mut component_storage = ComponentStorage::new(1000);

    const FIRST_GROUP: &[ComponentData] = &[ComponentData::new::<A>(), ComponentData::new::<B>()];
    const SECOND_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
    ];
    const THIRD_GROUP: &[ComponentData] =
        &[ComponentData::new::<D>(), ComponentData::new::<Position>()];
    const FOUTH_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
        ComponentData::new::<E>(),
    ];

    let first_group_inserted = manager.register(FIRST_GROUP);
    let second_group_inserted = manager.register(SECOND_GROUP);
    let third_group_inserted = manager.register(THIRD_GROUP);
    let fourth_group_inserted = manager.register(FOUTH_GROUP);

    assert!(first_group_inserted);
    assert!(second_group_inserted);
    assert!(third_group_inserted);
    assert!(fourth_group_inserted);

    let inserted_len = manager.flush_archetypes(&mut component_storage);

    assert_eq!(2, inserted_len);
    assert_eq!(3, manager.archetypes[0].groups_len());
    assert_eq!(1, manager.archetypes[1].groups_len());

    // Check if first archetype contains storage A, B, C, E (store index are 0, 1, 2, 4)
    // E is storage_id 4 because layouts are re-ordered by archetypes
    let val = GroupMask::new(Some(1 | 2 | (1 << 2) | (1 << 3)));
    assert_eq!(val, manager.archetypes[0].groups_mask);

    // Check if 2nd archetype contains storage D, E (store index are 5, 6)
    let val = GroupMask::new(Some((1 << 4) | (1 << 5)));
    assert_eq!(val, manager.archetypes[1].groups_mask);
}

#[test]
pub fn should_find_group_for_queries() {
    let mut ecs = ECS {
        entity_storage: EntityStorage::new(2000),
        component_storage: ComponentStorage::new(100),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
    };

    const FIRST_GROUP: &[ComponentData] = &[ComponentData::new::<A>(), ComponentData::new::<B>()];
    const SECOND_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
    ];
    const THIRD_GROUP: &[ComponentData] =
        &[ComponentData::new::<D>(), ComponentData::new::<Position>()];
    const FOUTH_GROUP: &[ComponentData] = &[
        ComponentData::new::<A>(),
        ComponentData::new::<B>(),
        ComponentData::new::<C>(),
        ComponentData::new::<E>(),
    ];

    // inserts all archetypes & groups then flush to ecs world 
    {
        let mut manager = ecs.archetypes.borrow_mut();
        let _ = manager.register(FIRST_GROUP);
        let _ = manager.register(SECOND_GROUP);
        let _ = manager.register(THIRD_GROUP);
        let _ = manager.register(FOUTH_GROUP);

        let flushed = manager.flush_archetypes(&mut ecs.component_storage);
        assert_eq!(2, flushed);
    }

    // insert 10 entities to ecs world
    for _i in 0..10 {
        let result: EntityCreateResult = ecs.create::<(A, B)>();
        assert!(matches!(result, EntityCreateResult::WithGroup(_)));
    }

    // assert a group has been found for this query
    let query: Query<(A, B)> = Query::new(&ecs);
    let iter = query.iter();
    assert!(iter.use_group);

    // assert the entity count is ok
    // @todo:
    // ⚠️⚠️⚠️Warning ⚠️⚠️⚠️ ==========
    // this is not working yet, the group must intersects the current group layout and build
    // contiguous topology for those entities
    // ⚠️⚠️⚠️Warning ⚠️⚠️⚠️ 
    //
    let iterated_entities = iter.fold(0, |acc, (_a, _b)| acc + 1);
    assert_eq!(10, iterated_entities);
}

pub fn iter_test_system(params: &mut SystemParams) {
    let mut query: Query<(Position, Velocity)> = Query::new(params.world);

    for (pos, vel) in query.iter() {
        // println!("{}", pos.x);
        // println!("{}", vel.x);
    }

    for (pos, vel) in query.iter_mut() {
        pos.x = 125f32;
        vel.x = 125f32;
        // println!("{}", pos.x);
        // println!("{}", vel.x);
    }
}
