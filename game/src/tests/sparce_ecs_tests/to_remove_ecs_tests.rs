use glfw::Key::W;

use crate::engine::ecs::lazy_ecs::resources::Resources;
use crate::engine::ecs::lazy_ecs::utils::{GroupMask, SparseVec};
use crate::engine::ecs::lazy_ecs::{
    archetypes::{ComponentData, ComponentSet},
    components::ComponentStorage,
    ecs::{EntityUpdateResult, ECS},
    entities::EntityMetadata,
    systems::SystemUpdateType,
};
use crate::engine::ecs::lazy_ecs::{
    components::ComponentBufferSparseSet,
    entities::{Entity, EntityAllocator, EntityStorage},
    utils::{ByteBuffer, SparseSet},
};
use crate::{
    engine::ecs::lazy_ecs::{
        archetypes::{ArchetypesManager, MatchType},
        ecs::{ECSStats, EntityCreateResult},
        systems::{make_system, Query, SystemParams},
    },
    tests::sparce_ecs_tests::ecs_test_helpers::{
        create_archetypes, create_ecs, create_entities, A, B, C, D, E, GROUP_AB, GROUP_ABC,
        GROUP_ABCD, GROUP_ABCDE,
    },
};
use std::{cell::RefCell, collections::HashSet, time::SystemTime};

#[allow(dead_code)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
pub struct Transform(i32);
#[allow(dead_code)]
pub struct Rigidbody2D(i32);
#[allow(dead_code)]
pub struct BoxCollider(i32);

#[test]
pub fn test_entity_storage_implementation() {
    let mut storage = EntityStorage {
        entities: SparseSet {
            dense_set: Vec::new(),
            sparse_views: SparseVec::new(10),
        },
        allocator: EntityAllocator::new(),
        metadata: EntityMetadata::new(10),
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
    let mut set: HashSet<usize> = HashSet::new();
    for ett in storage.entities.iter() {
        set.insert(ett.id);
        i += 1;
    }
    assert_eq!(i, 10);

    for i in 0..10 {
        assert_ne!(0, storage.entities.dense_set[i].version);
        assert!(set.contains(&(i as usize)));
        set.remove(&(i as usize));
    }
    assert!(set.is_empty());

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

    // testing swap component and none
    {
        for i in 0..100 {
            let velocity = buffer.get_mut_ref::<Velocity>(i);
            velocity.unwrap().x = i as f32;
        }

        // swap using compile time generic type
        //
        let swaped = buffer.swap::<Velocity>(0, 99);
        assert!(swaped);

        let zero = buffer.get_ref::<Velocity>(0);
        let last = buffer.get_ref::<Velocity>(99);

        assert_eq!(99, zero.unwrap().x as usize);
        assert_eq!(0, last.unwrap().x as usize);

        // swap using type_info data
        //
        let swaped: bool = buffer.swap_untyped(0, 99);
        assert!(swaped);

        let zero = buffer.get_ref::<Velocity>(0);
        let last = buffer.get_ref::<Velocity>(99);

        assert_eq!(
            0,
            zero.unwrap().x as usize,
            "invalid value: {}",
            zero.unwrap().x
        );
        assert_eq!(
            99,
            last.unwrap().x as usize,
            "invalid value: {}",
            last.unwrap().x
        );
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

    let removed = com_storage.remove(entity_0);
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

    let removed_2 = com_storage.remove(entity_0);
    let removed_3 = com_storage.remove(entity_10);
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
        let loop_removed = com_storage.remove(ett);
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
        com_storage.remove(ett);
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

    let r_5_version_update = Entity { id: 5, version: 2 };
    let r_5_comp_up = com_storage.get::<Velocity>(r_5_version_update);
    assert!(r_5_comp_up.is_none());
}

#[test]
pub fn test_ecs_implementation() {
    let mut ecs = ECS {
        entity_storage: EntityStorage::new(2000),
        component_storage: ComponentStorage::new(100, 2000),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
        stats: ECSStats::new(),

        resources: RefCell::new(Resources::new()) 
    };

    // create an entity
    let _entity = ecs.entity_storage.create();

    // register one system
    let system = make_system("the one system", SystemUpdateType::Update, iter_test_system);
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
        let add_pos = ecs.add_component::<(Position,)>(
            ett,
            (Position {
                x: i as f32,
                y: 0f32,
            },),
        );
        let add_vel = ecs.add_component::<(Velocity,)>(
            ett,
            (Velocity {
                x: i as f32,
                y: 0f32,
            },),
        );

        if let EntityUpdateResult::Failed(_) = add_pos {
            panic!("[Tests] Failled to add components");
        }
        if let EntityUpdateResult::Failed(_) = add_vel {
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

    assert_eq!(THIRD_GROUP.len(), manager.layouts[0].components.len());
    assert_eq!(SECOND_GROUP.len(), manager.layouts[1].components.len());

    // parenting
    assert_eq!(3, manager.layouts[0].set_len);
    assert_eq!(1, manager.layouts[1].set_len);
    assert_eq!(1, manager.layouts[2].set_len);
    assert_eq!(2, manager.layouts[3].set_len);
    assert_eq!(1, manager.layouts[4].set_len);

    // print all groups
    for (i, layout) in manager.layouts.iter().enumerate() {
        let mut group_str: String = String::from("[");
        for component in layout.components.iter() {
            group_str += component.metadata.get_name();
            group_str += " ";
        }
        group_str += "]";

        println!("[GROUP][{i}]: {group_str}");
    }

    // assert group order
    assert!(FIRST_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[2].components[i] == *y }));
    assert!(SECOND_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[1].components[i] == *y }));
    assert!(THIRD_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[0].components[i] == *y }));
    assert!(FOURTH_GROUP
        .iter()
        .enumerate()
        .all(|(i, y)| { manager.layouts[3].components[i] == *y }));
}

#[test]
pub fn test_should_flush_archetypes() {
    let mut manager = ArchetypesManager::new();
    let mut component_storage = ComponentStorage::new(1000, 1000);

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

    // check that the first group is A B C E
    let val = GroupMask::new(Some(1 | 2 | (1 << 2) | (1 << 3)));
    assert_eq!(val, manager.archetypes[0].groups[0].mask);

    // check that the second group is A B C
    let val = GroupMask::new(Some(1 | 2 | (1 << 2)));
    assert_eq!(val, manager.archetypes[0].groups[1].mask);

    // check that the third group is A B
    let val = GroupMask::new(Some(1 | 2));
    assert_eq!(val, manager.archetypes[0].groups[2].mask);
}

#[test]
pub fn test_should_find_group_for_queries() {
    let mut ecs = ECS {
        entity_storage: EntityStorage::new(2000),
        component_storage: ComponentStorage::new(100, 2000),
        update_systems: vec![],
        archetypes: RefCell::new(ArchetypesManager::new()),
        stats: ECSStats::new(),

        resources: RefCell::new(Resources::new()) 
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

    let mut grouped_entity_id: [Entity; 7] = [Entity { id: 0, version: 0 }; 7];
    let mut gei_i = 0;

    // insert 10 entities to ecs world
    for i in 0..10 {
        let result: EntityCreateResult;

        if i == 4 || i == 8 {
            result = ecs.create((A::new(), B {}, C {}, E {}));
            assert!(matches!(result, EntityCreateResult::Grouped(_)));
        } else if i % 2 == 0 {
            result = ecs.create((A::new(), C {}));
        } else {
            result = ecs.create((A::new(), B {}));
            assert!(matches!(result, EntityCreateResult::Grouped(_)));
        }

        if let EntityCreateResult::Grouped(grouped) = result {
            let ett_group_msk = grouped.group;
            grouped_entity_id[gei_i] = grouped.entity;
            gei_i += 1;

            // mask of the target group
            let meta_a = ecs.component_storage.get_storage_metadata::<A>();
            let meta_b = ecs.component_storage.get_storage_metadata::<B>();
            let meta_c = ecs.component_storage.get_storage_metadata::<C>();
            let meta_e = ecs.component_storage.get_storage_metadata::<E>();

            let group_msk_ab = GroupMask::new(Some(
                (1 << meta_a.index as u64) | (1 << meta_b.index as u64),
            ));
            let group_msk_abce = GroupMask::new(Some(
                (1 << meta_a.index as u64)
                    | (1 << meta_b.index as u64)
                    | (1 << meta_c.index as u64)
                    | (1 << meta_e.index as u64),
            ));

            assert!(
                group_msk_ab.is_match(&ett_group_msk) || group_msk_abce.is_match(&ett_group_msk)
            );
        }
    }

    // testing group length
    {
        let am = ecs.archetypes.borrow();
        let ai = ecs.component_storage.get_storage_metadata::<A>();
        let bi = ecs.component_storage.get_storage_metadata::<B>();
        let ci = ecs.component_storage.get_storage_metadata::<C>();
        let ei = ecs.component_storage.get_storage_metadata::<E>();

        let mut group_mask = GroupMask::new(None);
        group_mask.set(ai.index as u8);
        group_mask.set(bi.index as u8);

        let (arch_id, runtime_group) = am.find_group_exact_match(&group_mask).unwrap();

        assert_eq!(0, arch_id, "the archetype id is invalid");
        assert_eq!(
            7, runtime_group.len,
            "AB group has not enough elements, they should be added by ABCE superset"
        );

        group_mask.set(ci.index as u8);
        group_mask.set(ei.index as u8);

        let (_, runtime_group) = am.find_group_exact_match(&group_mask).unwrap();
        assert_eq!(2, runtime_group.len);
    }

    _ = ecs.create((A::new(), E {}));
    _ = ecs.create((A::new(), E {}));
    _ = ecs.create((A::new(), E {}));

    // assert a group has been found for this query
    let query: Query<(A, B)> = Query::new(&ecs);
    let iter = query.iter();
    assert!(iter.group.is_some());

    let iterated_entities = iter.fold(0, |acc, (_ett, _a, _b)| acc + 1);
    assert_eq!(7, iterated_entities);

    let query: Query<(A, B, C, E)> = Query::new(&ecs);
    assert!(!query.iter().is_empty());
    assert_eq!(2, query.iter().len());
}

#[test]
fn test_ungrouping() -> () {
    let mut ecs = create_ecs();

    // create archetype ==========
    create_archetypes(&mut ecs, vec![GROUP_ABC, GROUP_AB]);

    // prepare masks for abc and ab groups  ======
    let amask = ecs.component_storage.get_storage_metadata::<A>().index as u8;
    let bmask = ecs.component_storage.get_storage_metadata::<B>().index as u8;
    let cmask = ecs.component_storage.get_storage_metadata::<C>().index as u8;
    let mut mask_abc = GroupMask::new(None);
    mask_abc.or((1 << amask) | (1 << bmask) | (1 << cmask));
    let mut mask_ab = GroupMask::new(None);
    mask_ab.or((1 << amask) | (1 << bmask));

    // create entities ==========
    let entity_abc = ecs.create((A::new(), B {}, C {}));
    let _entity_ab_1 = ecs.create((A::new(), B {}));
    let _entity_ab_2 = ecs.create((A::new(), B {}));
    let entity_ab_3 = ecs.create((A::new(), B {}));
    let mut entity: Entity = Entity::null();
    let mut grouped = false;
    match entity_abc {
        EntityCreateResult::Grouped(ett_wg) => {
            grouped = true;
            entity = ett_wg.entity;
        }
        EntityCreateResult::Ungrouped(ett) => {
            entity = ett;
        }
        _ => (),
    };
    assert!(grouped, "failed to group entity");

    // test entities ==========
    {
        let query: Query<(A, B)> = Query::new(&ecs);
        let ab_count: usize = query.iter().count();
        assert!(ab_count == 4);
        assert!(ecs.stats.borrow().archetypes_broken == 0);

        let has_a = ecs.has_component::<A>(entity);
        let has_b = ecs.has_component::<B>(entity);
        assert!(has_a, "has no A component");
        assert!(has_b, "has no B component");
    }

    // remove B ======================
    {
        let removed = ecs.remove_component::<(B,)>(entity);
        assert!(
            removed,
            "failed to remove component B for entity {:?}",
            entity
        );
        let still_has_b: bool = ecs.has_component::<B>(entity);
        assert_eq!(false, still_has_b);
        assert_eq!(5, ecs.entity_storage.get_group(entity).unwrap().get_raw()); // 5 => 0101 or A and C comps

        let a_stre = ecs.component_storage.get_storage::<A>();
        let ett_index = a_stre.get_entity_index(entity);
        assert_eq!(3, ett_index.unwrap());

        if let EntityCreateResult::Grouped(ab_3) = entity_ab_3 {
            assert_eq!(0, a_stre.get_entity_index(ab_3.entity).unwrap());
        } else {
            assert!(false, "entity ab_3 was not grouped")
        }
    }

    // test every individual queries after remove
    {
        let query_a: Query<(A,)> = Query::new(&ecs);
        let query_b: Query<(B,)> = Query::new(&ecs);
        let query_c: Query<(C,)> = Query::new(&ecs);

        let c_a = query_a.iter().count();
        let c_b = query_b.iter().count();
        let c_c = query_c.iter().count();

        assert_eq!(4, c_a);
        assert_eq!(3, c_b);
        assert_eq!(1, c_c);

        assert_eq!(ecs.stats.borrow().archetypes_broken, 0, "");
    }

    // test AB group =============
    {
        {
            let mut archetypes = ecs.archetypes.borrow_mut();
            let groups = archetypes.get_supersets_with_archetype(0, &mask_abc, MatchType::Exact).unwrap();

            assert!(mask_ab.is_match(&groups[1].mask));
            assert_eq!(3, groups[1].len);
        }

        ecs.stats.borrow_mut().archetypes_broken = 0;

        let query: Query<(A, B)> = Query::new(&ecs);
        let ab_count: usize = query.iter().count();
        assert_eq!(3, ab_count, "");
        assert_eq!(ecs.stats.borrow().archetypes_broken, 0, "");
    }

    // test superset to subset order + check len of groups
    // [0] A B C
    // [1] A B
    {
        let mut archetypes = ecs.archetypes.borrow_mut();
        let groups = archetypes.get_supersets_with_archetype(0, &mask_abc, MatchType::Exact).unwrap();

        assert!(mask_abc.is_match(&groups[0].mask));
        assert_eq!(0, groups[0].len);

        assert!(mask_ab.is_match(&groups[1].mask));
        assert_eq!(3, groups[1].len);

        let grouped_etts_len = groups.iter().fold(0, |acc, g| acc + g.len);
        assert_eq!(grouped_etts_len, 3);

        let ett_has_b = ecs.component_storage.has_component::<B>(entity);
        assert_eq!(ett_has_b, false);
    }

    // test entities re-add
    {
        let query: Query<(A, C)> = Query::new(&ecs);
        let count = query.iter().count();
        assert_eq!(1, count);
    }

    // test storage access by group mask
    {
        let mut store_found_abc = 0;
        let mut store_found_ab = 0;
        ecs.component_storage
            .it_storages_by_mask_mut(mask_abc, |_store| {
                store_found_abc += 1;
            });
        ecs.component_storage
            .it_storages_by_mask_mut(mask_ab, |_store| {
                store_found_ab += 1;
            });

        assert_eq!(3, store_found_abc);
        assert_eq!(2, store_found_ab);
    }

    // test add B again
    {
        {
            let mut archetypes = ecs.archetypes.borrow_mut();
            let groups = archetypes.get_supersets_with_archetype(0, &mask_abc, MatchType::Exact).unwrap();

            assert_eq!(0, groups[0].len);
            assert_eq!(3, groups[1].len);
        }

        let b_added_res = ecs.add_component::<(B,)>(entity, (B {},));
        assert_eq!(mask_abc, ecs.entity_storage.get_group(entity).unwrap());

        if let EntityUpdateResult::Failed(_) = b_added_res {
            assert!(false, "adding B component failed!");
        }
        if let EntityUpdateResult::Ungrouped(_) = b_added_res {
            assert!(false, "adding B component didn't archetype regroup!");
        }

        {
            let mut archetypes = ecs.archetypes.borrow_mut();
            let groups = archetypes.get_supersets_with_archetype(0, &mask_abc, MatchType::Exact).unwrap();

            assert_eq!(1, groups[0].len);
            assert_eq!(4, groups[1].len);
        }

        {
            let query_abc: Query<(A, B, C)> = Query::new(&ecs);
            let query_ab: Query<(A, B)> = Query::new(&ecs);

            let abc = query_abc.iter().count();
            let ab = query_ab.iter().count();

            assert_eq!(1, abc);
            assert_eq!(4, ab);

            assert_eq!(ecs.stats.borrow().archetypes_broken, 0, "");
        }
    }
}

#[test]
pub fn test_entity_overflow_grouping() {
    let mut ecs = create_ecs();
    create_archetypes(&mut ecs, vec![GROUP_ABCDE, GROUP_ABCD, GROUP_ABC, GROUP_AB]);

    // NOTE(JuH): 15A, 12B, 2C
    let a_etts = create_entities(&mut ecs, 3, |i| (A::new(),));
    let ab_etts = create_entities(&mut ecs, 10, |i| (A::new(), B {}));
    let abc_etts = create_entities(&mut ecs, 2, |i| (A::new(), B {}, C {}));
    let abcd_etts = create_entities(&mut ecs, 5, |i| (A::new(), B {}, C {}, D {}));

    // assert entity are very far
    {
        let store = ecs.component_storage.get_storage::<A>();
        let index = store.get_entity_index(a_etts[2].entity).unwrap();
        assert!(
            index
                > a_etts
                    .len()
                    .max(ab_etts.len())
                    .max(abc_etts.len())
                    .max(abcd_etts.len())
        );
    }

    // add a new ABC entity to the group
    {
        let new_abc_ett: Entity = a_etts.last().unwrap().entity;
        let result = ecs.add_component::<(B, C)>(new_abc_ett, (B {}, C {}));

        match result {
            EntityUpdateResult::Failed(fail_msg) => assert!(false, "{}", fail_msg),
            EntityUpdateResult::Grouped(grouped_entity) => {
                assert_eq!(7, grouped_entity.group.get_raw())
            }
            EntityUpdateResult::Ungrouped(entity) => {
                assert!(false, "failed to group entity in ABC {:?}", entity)
            }
        }

        let query_abc: Query<(A, B, C)> = Query::new(&ecs);
        let query_ab: Query<(A, B)> = Query::new(&ecs);

        assert_eq!(8, query_abc.iter().count());
        assert_eq!(18, query_ab.iter().count());

        let brokens = ecs.stats.borrow().archetypes_broken;
        assert_eq!(0, brokens);
    }

    // remove AB entities
    {
        for grouped_entity in ab_etts.iter() {
            let r = ecs.remove_component::<(A, B)>(grouped_entity.entity);
            assert!(
                r,
                "could not remove AB components to entity {:?}",
                grouped_entity.entity
            );
        }

        let query_abcd: Query<(A, B, C, D)> = Query::new(&ecs);
        let query_abc: Query<(A, B, C)> = Query::new(&ecs);
        let query_ab: Query<(A, B)> = Query::new(&ecs);

        assert_eq!(8, query_ab.iter().count());
        assert_eq!(8, query_abc.iter().count());
        assert_eq!(5, query_abcd.iter().count());

        let brokens = ecs.stats.borrow().archetypes_broken;
        assert_eq!(0, brokens);
    }

    // check layout
    {
        let store_a = ecs.component_storage.get_storage::<A>();
        let store_b = ecs.component_storage.get_storage::<B>();
        let store_c = ecs.component_storage.get_storage::<C>();
        let store_d = ecs.component_storage.get_storage::<D>();

        for ett in abcd_etts.iter() {
            let i_a = store_a.get_entity_index(ett.entity).unwrap();
            let i_b = store_b.get_entity_index(ett.entity).unwrap();
            let i_c = store_c.get_entity_index(ett.entity).unwrap();
            let i_d = store_d.get_entity_index(ett.entity).unwrap();

            assert!(
                (i_a == i_b && i_b == i_c && i_c == i_d),
                "abcd layout is invalid"
            );
        }
        for ett in abc_etts.iter() {
            let i_a = store_a.get_entity_index(ett.entity).unwrap();
            let i_b = store_b.get_entity_index(ett.entity).unwrap();
            let i_c = store_c.get_entity_index(ett.entity).unwrap();

            assert!((i_a == i_b && i_b == i_c), "abcd layout is invalid");
        }
        for ett in ab_etts.iter() {
            let a = store_a.get_entity_index(ett.entity).map_or(30000, |e| e);
            let b = store_b.get_entity_index(ett.entity).map_or(30000, |e| e);

            assert!(a == b, "abcd layout is invalid");
        }
    }
}

#[test]
pub fn test_layout_sync_when_group_overlaps() {
    let mut ecs = create_ecs();
    create_archetypes(&mut ecs, vec![GROUP_ABCDE, GROUP_ABCD, GROUP_ABC, GROUP_AB]);

    let ab_etts = create_entities(&mut ecs, 10, |i| (A::new(), B));
    let _abc_etts = create_entities(&mut ecs, 2, |i| (A::new(), B, C));

    for ett in ab_etts.iter() {
        let result = ecs.add_component::<(C,)>(ett.entity, (C {},));
        match result {
            EntityUpdateResult::Failed(_) => assert!(false),
            EntityUpdateResult::Grouped(grouped_entity) => {
                assert_eq!(7, grouped_entity.group.get_raw())
            }
            EntityUpdateResult::Ungrouped(_entity) => assert!(false),
        }
    }

    let query_abc: Query<(A, B, C)> = Query::new(&ecs);
    let query_ab: Query<(A, B)> = Query::new(&ecs);

    assert_eq!(12, query_ab.iter().count());
    assert_eq!(12, query_abc.iter().count());
    assert_eq!(0, ecs.stats.borrow().archetypes_broken);
}

#[allow(dead_code)]
pub fn iter_test_system(params: &mut SystemParams) {
    let mut query: Query<(Position, Velocity)> = Query::new(params.world);

    for (_ett, pos, vel) in query.iter() {
        println!("{}", pos.x);
        println!("{}", vel.x);
    }

    for (_ett, pos, vel) in query.iter_mut() {
        pos.x = 125f32;
        vel.x = 125f32;
        println!("{}", pos.x);
        println!("{}", vel.x);
    }
}
