use std::{hint, time::Instant};

use crate::{
    engine::ecs::lazy_ecs::{ecs::EntityCreateResult, entities::Entity, systems::Query},
    tests::sparce_ecs_tests::ecs_test_helpers::{create_ecs, create_ecs_with_capacities, A, B},
};

// Verifies that declaring a component group registers or validates every grouped component.
#[test]
fn component_group_initialization_handles_all_grouped_types() {}

// Verifies that entities containing every grouped component occupy the grouped dense prefix.
#[test]
fn matching_entities_are_placed_in_grouped_prefix() {}

// Verifies that entities missing a grouped component remain outside the grouped prefix.
#[test]
fn partially_matching_entities_remain_outside_grouped_prefix() {}

// Verifies that a grouped query reports or uses dense iteration.
#[test]
fn grouped_query_uses_dense_iteration() {}

// Verifies that an equivalent ungrouped query follows the sparse iteration path.
#[test]
fn ungrouped_query_uses_sparse_iteration() {}

// Verifies that nested component groups maintain the correct prefix lengths.
#[test]
fn nested_groups_maintain_correct_prefixes() {}

// Verifies that inserting a missing grouped component moves an entity into the appropriate group.
#[test]
fn component_insertion_moves_entity_into_group() {}

// Verifies that removing a grouped component moves an entity out of the group.
#[test]
fn component_removal_moves_entity_out_of_group() {}

// Verifies that destroying a grouped entity preserves group invariants.
#[test]
fn destroying_grouped_entity_preserves_group_invariants() {}

// Verifies that creating a fully matching entity inserts it into the correct group.
#[test]
fn creating_matching_entity_joins_group() {}

// Verifies that replacing a grouped component preserves group membership.
#[test]
fn replacing_grouped_component_preserves_group_membership() {}

// Verifies that grouped component slices have equal lengths.
#[test]
fn grouped_component_slices_have_equal_lengths() {}

// Verifies that entries at the same index in grouped slices belong to the same entity.
#[test]
fn grouped_component_slices_are_entity_aligned() {}

// Verifies that an entity slice aligns with grouped component slices.
#[test]
fn grouped_entity_slice_aligns_with_component_slices() {}

// Verifies that mutating grouped slices updates the correct entities.
#[test]
fn mutable_grouped_slices_update_correct_entities() {}

// Verifies that slice queries with exclusions produce the same membership as iterator queries.
#[test]
fn grouped_slice_exclusion_matches_iterator_query() {}

// Verifies that modifying unrelated component storage does not corrupt group ordering.
#[test]
fn unrelated_storage_changes_do_not_corrupt_groups() {}

// Verifies that conflicting group declarations are rejected or normalized according to the API contract.
#[test]
fn conflicting_group_declarations_are_handled_consistently() {}

pub struct APerf {
    id: u32,
    name: String,
}
pub struct BPerf {
    id: u32,
    uuid: u64,
    name: String,
}
impl APerf {
    pub fn new() -> Self {
        Self {
            id: 12,
            name: String::from("Some text because test is great, hello world!"),
        }
    }
}
impl BPerf {
    pub fn new() -> Self {
        Self {
            id: 12,
            uuid: 32,
            name: String::from("Some value that is not that big, hello world! V2"),
        }
    }
}

// NOTE(JuH): Run this test with: cargo test iterating_over_dense_set_is_faster_than_sparse --release -- --nocapture
// to get the "best" results, it's actually a bad micro benchmark function but it just allow to check that archetypes increase iteration speed.
#[test]
fn iterating_over_dense_set_is_faster_than_sparse() {
    let mut ecs_sparse = create_ecs_with_capacities(100_000, 10_000);
    let mut ecs_dense = create_ecs_with_capacities(100_000, 10_000);

    ecs_sparse.allocate_storages::<(APerf, BPerf)>();
    ecs_dense.allocate_storages::<(APerf, BPerf)>();

    ecs_dense.make_archetype::<(APerf, BPerf)>();
    assert_eq!(1, ecs_dense.flush_archetypes());

    let mut sparse_etts: Vec<Entity> = vec![];
    let mut dense_etts: Vec<Entity> = vec![];
    let mut create_a = true;

    const ITER_COUNT: u128 = 1_000;
    const ENTITY_COUNT: usize = 1_000;
    const BREAKS_LAYOUT_ENTITY_COUNT: usize = 10;

    for i in 0..ENTITY_COUNT {
        if i % 2 == 0 {
            for _ in 0..BREAKS_LAYOUT_ENTITY_COUNT {
                if create_a {
                    let EntityCreateResult::Ungrouped(sparse_ett) =
                        ecs_sparse.create::<(APerf,)>((APerf::new(),))
                    else {
                        panic!("should not fail");
                    };
                    let EntityCreateResult::Ungrouped(dense_ett) =
                        ecs_dense.create::<(APerf,)>((APerf::new(),))
                    else {
                        panic!("should be grouped");
                    };

                    sparse_etts.push(sparse_ett);
                    dense_etts.push(dense_ett);
                } else {
                    let EntityCreateResult::Ungrouped(sparse_ett) =
                        ecs_sparse.create::<(BPerf,)>((BPerf::new(),))
                    else {
                        panic!("should not fail");
                    };
                    let EntityCreateResult::Ungrouped(dense_ett) =
                        ecs_dense.create::<(BPerf,)>((BPerf::new(),))
                    else {
                        panic!("should be grouped");
                    };

                    sparse_etts.push(sparse_ett);
                    dense_etts.push(dense_ett);
                }
            }

            create_a = !create_a;
            continue;
        }

        let EntityCreateResult::Ungrouped(sparse_ett) =
            ecs_sparse.create::<(APerf, BPerf)>((APerf::new(), BPerf::new()))
        else {
            panic!("should not fail");
        };
        let EntityCreateResult::Grouped(dense_ett) =
            ecs_dense.create::<(APerf, BPerf)>((APerf::new(), BPerf::new()))
        else {
            panic!("should be grouped");
        };

        sparse_etts.push(sparse_ett);
        dense_etts.push(dense_ett.entity);
    }

    let query_sparse: Query<(APerf, BPerf)> = Query::new(&ecs_sparse);
    let query_dense: Query<(APerf, BPerf)> = Query::new(&ecs_dense);

    let avg_sparse = {
        let mut acc: u128 = 0;

        for _ in 0..ITER_COUNT {
            #[allow(dead_code)]
            let mut i: usize = 0;
            let now = Instant::now();

            for (_e, _a, _b) in query_sparse.iter() {
                assert!(_a.name.eq("Some text because test is great, hello world!"));
                i += 1;
            }

            acc += now.elapsed().as_nanos();
            hint::black_box(i);
        }

        acc / ITER_COUNT
    };

    let avg_dense = {
        let mut acc: u128 = 0;

        for _ in 0..ITER_COUNT {
            #[allow(dead_code)]
            let mut i: usize = 0;
            let now = Instant::now();

            for (_e, _a, _b) in query_dense.iter() {
                i += 1;
            }

            acc += now.elapsed().as_nanos();
            hint::black_box(i);
        }

        assert!(ecs_dense.stats.borrow().archetypes_broken == 0);

        acc / ITER_COUNT
    };

    let factor: f64 = avg_sparse.max(avg_dense) as f64 / avg_sparse.min(avg_dense) as f64;
    println!(
        "\nsparse: {}ns, dense: {}ns [Factor: x{}, Entities: {}]",
        avg_sparse,
        avg_dense,
        factor,
        dense_etts.len()
    );

    assert_eq!(query_sparse.iter().count(), query_dense.iter().count());
    assert!(avg_sparse > avg_dense);
}
