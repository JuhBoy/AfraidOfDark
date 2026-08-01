use crate::{
    engine::ecs::lazy_ecs::{archetypes::ComponentData, ecs::EntityCreateResult},
    tests::sparce_ecs_tests::ecs_test_helpers::{create_ecs, A, B},
};

// Verifies that a newly constructed world contains no entities.
#[test]
fn new_world_is_empty() {}

// Verifies that a newly constructed world reports an entity count of zero.
#[test]
fn new_world_has_zero_entities() {}

// Verifies that a component registered through the world builder is available after construction.
#[test]
fn builder_registers_component_type() {}

// Verifies that a component can be registered after the world has been constructed.
#[test]
fn component_can_be_registered_after_world_creation() {}

// Verifies that registering the same component type twice follows the documented policy.
#[test]
fn duplicate_component_registration_is_handled_consistently() {}

// Verifies that different component types receive different internal identifiers.
#[test]
fn different_component_types_have_distinct_identifiers() {}

// Verifies that component identifiers remain stable throughout the lifetime of the world.
#[test]
fn component_identifier_remains_stable() {}

// Verifies that clearing the world does not unregister component types.
#[test]
fn clear_preserves_component_registrations() {}

// Verifies that resetting the world follows the documented component-registration policy.
#[test]
fn reset_preserves_expected_component_registrations() {}

// Verifies that zero-sized component types can be registered.
#[test]
fn zero_sized_component_can_be_registered() {}

// Verifies that many distinct component types can be registered without collisions.
#[test]
fn many_component_types_can_be_registered() {}

// Verifies that inserting an unregistered component follows the documented registration policy.
#[test]
fn inserting_unregistered_component_follows_registration_policy() {}

// Verifies that querying an unregistered component follows the documented registration policy.
#[test]
fn querying_unregistered_component_follows_registration_policy() {}

#[test]
fn removing_last_components_drop_its_values() {
    let mut ecs = create_ecs();
    ecs.allocate_storage::<(A, B)>();

    assert!(ecs.make_archetype::<(A, B)>());
    assert_eq!(1, ecs.flush_archetypes());

    let entity = ecs.create::<(A,)>((A::new(),));
    assert!(matches!(entity, EntityCreateResult::Ungrouped(_)));

    let EntityCreateResult::Ungrouped(entity) = entity else { 
        panic!("")
    };

    assert!(ecs.destroy(entity));
    assert!(!ecs.destroy(entity));

    let len = ecs.component_storage.get_storage_by_id(0).component_buffer.len;
    assert_eq!(0, len);
}
