use crate::engine::ecs::my_ecs::archetypes::RuntimeGroup;
use crate::engine::ecs::my_ecs::components::{ComponentBufferSparseSet, ComponentMetaData};
use crate::engine::ecs::my_ecs::ecs::ECS;
use crate::engine::ecs::my_ecs::entities::Entity;
use crate::engine::ecs::my_ecs::utils::GroupMask;
use atomic_refcell::AtomicRefMut;
use std::marker::PhantomData;
use std::slice::Iter;

// Queries =============================
//
pub trait TQuery: 'static {
    type ViewMut<'a>;
    type View<'a>: 'a;
    type Item<'a>: 'a;
    type ItemMut<'a>: 'a;
    type Iter<'a>: 'a;

    fn borrow_storage<'a>(world: &'a ECS) -> Self::View<'a>;
    fn borrow_storage_mut(world: &ECS) -> Self::View<'_>;
    fn entities<'a>(world: &'a ECS, views: &Self::View<'_>)
        -> (Option<RuntimeGroup>, &'a [Entity]);
    fn iter_predicate(entity: Entity, world: &ECS, view: &Self::View<'_>) -> bool;
    fn get_dense<'a>(entity: Entity, world: &'a ECS, view: &Self::View<'_>) -> Self::Item<'a>;
    fn get_dense_mut<'a>(
        entity: Entity,
        world: &'a ECS,
        view: &Self::View<'_>,
    ) -> Self::ItemMut<'a>;
}

pub struct StorageView<'a, T> {
    store: ComponentMetaData,
    _phantom: PhantomData<&'a [T]>,
}

pub struct StorageViewMut<'a, T> {
    store: AtomicRefMut<'a, ComponentBufferSparseSet>,
    _phantom: PhantomData<&'a mut [T]>,
}

// impl<A, B> TQuery for (A, B)
// where
//     A: 'static,
//     B: 'static,
// {
//     type ViewMut<'a> = (StorageView<'a, A>, StorageView<'a, B>);
//     type View<'a> = (StorageView<'a, A>, StorageView<'a, B>);
//     type Item<'a> = (&'a A, &'a B);
//     type ItemMut<'a> = (&'a mut A, &'a mut B);
//     type Iter<'a> = Iter<'a, Self::Item<'a>>;
//
//     fn borrow_storage(world: &ECS) -> Self::View<'_> {
//         // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
//         // @todo! using const compilation strategy like Attributes or something ?
//         let a_storage = world.component_storage.get_statage_metadata::<A>();
//         let b_storage = world.component_storage.get_statage_metadata::<B>();
//
//         (
//             StorageView {
//                 store: a_storage,
//                 _phantom: PhantomData,
//             },
//             StorageView {
//                 store: b_storage,
//                 _phantom: PhantomData,
//             },
//         )
//     }
//
//     fn borrow_storage_mut(world: &ECS) -> Self::View<'_> {
//         let a = world.component_storage.get_statage_metadata::<A>();
//         let b = world.component_storage.get_statage_metadata::<B>();
//
//         (
//             StorageView {
//                 store: a,
//                 _phantom: PhantomData,
//             },
//             StorageView {
//                 store: b,
//                 _phantom: PhantomData,
//             },
//         )
//     }
//
//     fn entities<'a>(
//         world: &'a ECS,
//         views: &Self::View<'_>,
//     ) -> (Option<RuntimeGroup>, &'a [Entity]) {
//         let view_a = &views.0;
//         let view_b = &views.1;
//
//         let a_len = world
//             .component_storage
//             .get_storage_by_id(view_a.store.index)
//             .entity_to_component
//             .len();
//         let b_len = world
//             .component_storage
//             .get_storage_by_id(view_b.store.index)
//             .entity_to_component
//             .len();
//
//         let id = if a_len < b_len {
//             view_a.store.index
//         } else {
//             view_b.store.index
//         };
//
//         let store = world.component_storage.get_storage_by_id(id);
//         let entities = store.entity_to_component.as_slice().as_ptr();
//
//         let group = {
//             let arch_manager = world.archetypes.borrow();
//
//             assert!(view_a.store.index <= GroupMask::max_bit_shift());
//             assert!(view_b.store.index <= GroupMask::max_bit_shift());
//
//             let mut query_mask = GroupMask::new(None);
//             query_mask.set(view_a.store.index as u8);
//             query_mask.set(view_b.store.index as u8);
//
//             let group = arch_manager.find_group(&query_mask);
//             if let Some(matching_group) = group {
//                 println!(
//                     "[system.rs] Group found for query (gm: {:?}, len: {}) [{:?}]",
//                     matching_group.mask, matching_group.len, query_mask
//                 );
//             }
//
//             group
//         };
//
//         unsafe {
//             let etts: &'a [Entity] = std::slice::from_raw_parts(entities, a_len.min(b_len));
//             (group, etts)
//         }
//     }
//
//     fn iter_predicate(entity: Entity, world: &ECS, view: &Self::View<'_>) -> bool {
//         let store = world
//             .component_storage
//             .get_storage_by_id(view.0.store.index);
//         let store2 = world
//             .component_storage
//             .get_storage_by_id(view.1.store.index);
//
//         store.has(entity) && store2.has(entity)
//     }
//
//     fn get_dense<'a>(entity: Entity, world: &'a ECS, view: &Self::View<'_>) -> Self::Item<'a> {
//         let store = world
//             .component_storage
//             .get_storage_by_id(view.0.store.index);
//         let store2 = world
//             .component_storage
//             .get_storage_by_id(view.1.store.index);
//
//         let component_a: *const A = store.get::<A>(entity).unwrap();
//         let component_b: *const B = store2.get::<B>(entity).unwrap();
//
//         unsafe {
//             let upa: &'a A = &*component_a;
//             let upb: &'a B = &*component_b;
//
//             (upa, upb)
//         }
//     }
//
//     fn get_dense_mut<'a>(
//         entity: Entity,
//         world: &'a ECS,
//         view: &Self::View<'_>,
//     ) -> Self::ItemMut<'a> {
//         let mut store = world
//             .component_storage
//             .get_storage_mut_by_id(view.0.store.index);
//         let mut store2 = world
//             .component_storage
//             .get_storage_mut_by_id(view.1.store.index);
//
//         let component_a = store.get_mut::<A>(entity).unwrap() as *mut A;
//         let component_b = store2.get_mut::<B>(entity).unwrap() as *mut B;
//
//         unsafe { (&mut *component_a, &mut *component_b) }
//     }
// }

macro_rules! generate_query {
    ( $(($components:tt, $index:tt)),*) => {
      impl<$($components),*> TQuery for ($($components),*)
      where $($components: 'static),* {

          type ViewMut<'a> = ($(StorageView<'a, $components>),*);
          type View<'a> = ($(StorageView<'a, $components>),*);
          type Item<'a> = ($(&'a $components),*);
          type ItemMut<'a> = ($(&'a mut $components),*);
          type Iter<'a> = Iter<'a, Self::Item<'a>>;

        fn borrow_storage(world: &ECS) -> Self::View<'_> {
            // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
            // @todo! using const compilation strategy like Attributes or something ?
            (
                $(
                    StorageView {
                        store: world.component_storage.get_statage_metadata::<$components>(),
                        _phantom: PhantomData,
                    }

                ),*
            )
        }

        fn borrow_storage_mut(world: &ECS) -> Self::View<'_> {
            // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
            // @todo! using const compilation strategy like Attributes or something ?
            (
                $(
                    StorageView {
                        store: world.component_storage.get_statage_metadata::<$components>(),
                        _phantom: PhantomData,
                    }
                ),*
            )
        }

        fn entities<'a>(
            world: &'a ECS,
            views: &Self::View<'_>,
        ) -> (Option<RuntimeGroup>, &'a [Entity]) {
            let stores_length = ($(world.component_storage.get_storage_by_id(views.$index.store.index).entity_to_component.len()),*);

            let min_store_id = {
                let mut min_length = usize::MAX;
                let mut min_id = 0;

                $(
                  if (stores_length.$index < min_length) {
                      min_length = stores_length.$index;
                      min_id = views.$index.store.index;
                  }
                )*

                min_id
            };

            let store = world.component_storage.get_storage_by_id(min_store_id);
            let entities = store.entity_to_component.as_slice().as_ptr();


            let group: Option<RuntimeGroup> = {
                let arch_manager = world.archetypes.borrow();

                $(
                    assert!(views.$index.store.index <= GroupMask::max_bit_shift())
                );*;

                let mut query_mask = GroupMask::new(None);
                $(
                  query_mask.set(views.$index.store.index as u8)
                );*;

                let group = arch_manager.find_group(&query_mask);
                if let Some(matching_group) = group {
                    println!(
                        "[system.rs] Group found for query (gm: {:?}, len: {}) [{:?}]",
                        matching_group.mask, matching_group.len, query_mask
                    );
                }

                group
            };

            unsafe {
                let etts: &'a [Entity] = std::slice::from_raw_parts(entities, store.entity_to_component.len());
                (group, etts)
            }
        }

        fn iter_predicate(entity: Entity, world: &ECS, view: &Self::View<'_>) -> bool {
            $(
               if !world.component_storage.get_storage_by_id(view.$index.store.index).has(entity) {
                   return false;
               }
            )*

            true
        }

        fn get_dense<'a>(entity: Entity, world: &'a ECS, view: &Self::View<'_>) -> Self::Item<'a> {
            let comps = ($(
                world.component_storage
                    .get_storage_by_id(view.$index.store.index)
                    .get::<$components>(entity).unwrap() as *const $components
            ),*);

            unsafe {
                ($(&*comps.$index),*)
            }
        }

        fn get_dense_mut<'a>(
            entity: Entity,
            world: &'a ECS,
            view: &Self::View<'_>,
        ) -> Self::ItemMut<'a> {
            let components = ($(
                world.component_storage
                    .get_storage_mut_by_id(view.$index.store.index)
                    .get_mut::<$components>(entity).unwrap() as *mut $components
            ),*);

            unsafe {
                ($(&mut *components.$index),*)
            }
        }
      }
    };
}

generate_query!((A, 0), (B, 1));
generate_query!((A, 0), (B, 1), (C, 2));
generate_query!((A, 0), (B, 1), (C, 2), (D, 3));
generate_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
