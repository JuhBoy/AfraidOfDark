use bevy_ecs::error::panic;
use glfw::Key::W;

use crate::engine::ecs::my_ecs::archetypes::{ComponentData, ComponentSet};
use crate::engine::ecs::my_ecs::ecs::{EntityCreateResult, EntityUpdateResult};
use crate::engine::ecs::my_ecs::entities::Entity;
use crate::engine::ecs::my_ecs::systems::{Query, QueryMut};
use crate::engine::ecs::my_ecs::{self, ecs};
use crate::tests::sparce_ecs_tests::ecs_test_helpers::{create_ecs, NonCopyA, B, C};
use crate::tests::sparce_ecs_tests::ecs_test_helpers::{AData, A};

// Verifies that an entity can be created with one component.
#[test]
fn entity_can_be_created_with_one_component() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<A>();

    let entity_res_a = ecs.create::<(A,)>((A {},));
    let mut entity_a: Entity = Entity::null();

    match entity_res_a {
        EntityCreateResult::Failed(_) => assert!(false),
        EntityCreateResult::Grouped(_grouped_entity) => assert!(false),
        EntityCreateResult::Ungrouped(entity) => {
            entity_a = entity;
        }
    }

    let has_a = ecs.has_component::<A>(entity_a);
    assert!(has_a);
}

// Verifies that an entity can be created with multiple components.
#[test]
fn entity_can_be_created_with_component_bundle() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<A>();
    ecs.allocate_storage::<B>();

    let entity_res_ab = ecs.create::<(A, B)>((A {}, B {}));

    let EntityCreateResult::Ungrouped(entity_ab) = entity_res_ab else {
        panic!("failed to create entity with A and B component");
    };

    assert!(ecs.has_component::<A>(entity_ab));
    assert!(ecs.has_component::<B>(entity_ab));
}

// Verifies that an inserted component can be read through a shared reference.
#[test]
fn inserted_component_can_be_read() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<AData>();

    let entity = ecs.create::<(AData,)>((AData(42),));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    let binding = ecs.component_storage.get_storage::<AData>();
    let comp = binding.get::<AData>(entity);
    assert!(comp.is_some());

    let comp: &AData = comp.unwrap();
    assert_eq!(42, comp.0);
}

// Verifies that an inserted component can be obtained through a mutable reference.
#[test]
fn inserted_component_can_be_mutably_accessed() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<AData>();

    let entity = ecs.create::<(AData,)>((AData(42),));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    let mut binding = ecs.component_storage.get_storage_mut::<AData>();
    let comp = binding.get_mut::<AData>(entity);
    assert!(comp.is_some());

    let comp: &mut AData = comp.unwrap();
    assert_eq!(42, comp.0);
}

// Verifies that mutations made through a mutable component reference persist.
#[test]
fn component_mutation_persists() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<AData>();

    let entity = ecs.create::<(AData,)>((AData(42),));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    {
        let mut binding = ecs.component_storage.get_storage_mut::<AData>();
        let comp = binding.get_mut::<AData>(entity);
        assert!(comp.is_some());

        let comp: &mut AData = comp.unwrap();
        assert_eq!(42, comp.0);
        comp.0 = 84;
    }

    let binding = ecs.component_storage.get_storage::<AData>();
    assert_eq!(84, binding.get::<AData>(entity).unwrap().0);
}

// Verifies that checking for an existing component returns true.
#[test]
fn contains_component_returns_true_for_present_component() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A, B, C)>((A {}, B {}, C {}));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.component_storage.get_storage::<A>().has(entity));
    assert!(ecs.component_storage.get_storage::<B>().has(entity));
    assert!(ecs.component_storage.get_storage::<C>().has(entity));
}

// Verifies that checking for a missing component returns false.
#[test]
fn contains_component_returns_false_for_missing_component() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A, C)>((A {}, C {}));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.component_storage.get_storage::<A>().has(entity));
    assert!(!ecs.component_storage.get_storage::<B>().has(entity));
    assert!(ecs.component_storage.get_storage::<C>().has(entity));
}

// Verifies that inserting one component does not modify unrelated components.
#[test]
fn inserting_component_preserves_existing_components() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A,)>((A {},));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    let comps_set_b = (B {},);
    let comps_set_c = (C {},);
    let EntityUpdateResult::Ungrouped(entity) = ecs.add_component::<(B,)>(entity, comps_set_b)
    else {
        panic!("entity were not updated with the B component");
    };
    let EntityUpdateResult::Ungrouped(entity) = ecs.add_component::<(C,)>(entity, comps_set_c)
    else {
        panic!("entity were not updated with the C component");
    };

    assert!(ecs.component_storage.get_storage::<A>().has(entity));
    assert!(ecs.component_storage.get_storage::<B>().has(entity));
    assert!(ecs.component_storage.get_storage::<C>().has(entity));
}

// Verifies that removing one component does not modify unrelated components.
#[test]
fn removing_component_preserves_other_components() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A, B, C)>((A {}, B {}, C {}));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.remove_component::<(B,)>(entity));
    assert!(!ecs.has_component::<B>(entity));
    assert!(ecs.component_storage.get_storage::<A>().has(entity));
    assert!(ecs.component_storage.get_storage::<C>().has(entity));

    let comps = ecs.entity_storage.get_group(entity).unwrap();
    assert_eq!(5, comps.get_raw());
}

// Verifies that removing a missing component fails safely.
#[test]
fn removing_missing_component_fails_safely() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A,)>((A {},));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.component_storage.get_storage::<A>().has(entity));
    assert!(!ecs.remove_component::<(B,)>(entity));
    assert!(!ecs.remove_component::<(C,)>(entity));

    let comps = ecs.entity_storage.get_group(entity).unwrap();

    const MASK_A: u64 = 1;
    assert_eq!(MASK_A, comps.get_raw());
}

// Verifies that inserting a component of an already present type follows the replacement policy.
#[test]
fn inserting_existing_component_follows_replacement_policy() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(AData, B, C)>();

    let entity = ecs.create::<(AData,)>((AData(32),));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.has_component::<AData>(entity));

    let entity = ecs.add_component::<(AData,)>(entity, (AData(64),));
    let EntityUpdateResult::Ungrouped(entity) = entity else {
        panic!("add component failed");
    };

    let comp_ref = ecs.component_storage.get_storage::<AData>();
    let comp_ref = comp_ref.get::<AData>(entity);

    assert!(comp_ref.is_some(), "component not found");
    assert_eq!(64, comp_ref.unwrap().0);
}

// Verifies that inserting a component into a destroyed entity fails safely.
#[test]
fn component_cannot_be_inserted_into_destroyed_entity() {
    let mut ecs = create_ecs();
    ecs.allocate_storages::<(AData, B, C)>();

    let entity = ecs.create::<(AData,)>((AData(32),));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.destroy(entity));

    let update_res = ecs.add_component::<(A,)>(entity, (A {},));
    let EntityUpdateResult::Failed(_failed_res) = update_res else {
        panic!("the add component should not be a success");
    };
}

// Verifies that reading a component from a destroyed entity returns no value.
#[test]
fn component_cannot_be_read_from_destroyed_entity() {
    let mut ecs = create_ecs();

    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A,)>((A {},));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.destroy(entity));
    assert!(!ecs.has_component::<A>(entity));

    let query: Query<(A,)> = Query::new(&mut ecs);
    assert_eq!(0, query.iter().count());
}

// Verifies that mutating a component on a destroyed entity fails safely.
#[test]
fn component_cannot_be_mutated_on_destroyed_entity() {
    let mut ecs = create_ecs();

    ecs.allocate_storages::<(A, B, C)>();

    let entity = ecs.create::<(A,)>((A {},));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.destroy(entity));
    assert!(!ecs.has_component::<A>(entity));

    let mut query: Query<(A,)> = Query::new(&mut ecs);
    assert_eq!(0, query.iter_mut().count());
}

// Verifies that non-Copy and heap-owning component values are stored correctly.
#[test]
fn non_copy_component_supports_full_lifecycle() {
    let mut ecs = create_ecs();

    ecs.allocate_storages::<(NonCopyA,)>();

    let entity = ecs.create::<(NonCopyA,)>((NonCopyA { a: 32 },));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    assert!(ecs.has_component::<NonCopyA>(entity));

    let mut query: Query<(NonCopyA,)> = Query::new(&mut ecs);
    assert_eq!(1, query.iter_mut().count());

    query.iter_mut().for_each(|a| {
        a.1.a = 64;
    });

    let store = ecs.component_storage.get_storage::<NonCopyA>();
    let val = store.get::<NonCopyA>(entity).map_or(0, |c| c.a);
    assert_eq!(64, val);
}

// TODO(JuH): wait bulk implementation
// ===================================

// Verifies that creating many entities with the same component preserves every value.
#[test]
fn bulk_component_creation_preserves_values() {}

// Verifies that component storage can be reused after all of its components are removed.
#[test]
fn component_storage_can_be_reused_after_becoming_empty() {
    let mut ecs = create_ecs();

    ecs.allocate_storages::<(A, B)>();
    {
        let mut archetypes = ecs.archetypes.borrow_mut();
        const AB_GRP: &[ComponentData] = &[ComponentData::new::<A>(), ComponentData::new::<B>()];

        assert!(archetypes.register(AB_GRP));
        assert_eq!(1, archetypes.flush_archetypes(&mut ecs.component_storage));
    }

    let entity = ecs.create::<(A,)>((A {},));

    let EntityCreateResult::Ungrouped(entity) = entity else {
        panic!("failed to created entity with component AData");
    };

    ecs.reset();

    assert!(!ecs.entity_storage.entities.has(entity));
    assert!(!ecs.has_component::<A>(entity));

    let archertype_count = ecs.archetypes.borrow().archetypes.len();
    assert_eq!(0, archertype_count);
}
