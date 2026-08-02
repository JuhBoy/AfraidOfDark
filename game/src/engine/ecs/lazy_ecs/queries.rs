use crate::engine::ecs::lazy_ecs::archetypes::{ArchetypesManager, RuntimeGroup};
use crate::engine::ecs::lazy_ecs::components::{
    ComponentBufferSparseSet, ComponentMetaData, ComponentStorage,
};
use crate::engine::ecs::lazy_ecs::ecs::ECS;
use crate::engine::ecs::lazy_ecs::entities::Entity;
use crate::engine::ecs::lazy_ecs::utils::GroupMask;
use atomic_refcell::AtomicRefMut;
use std::cell::Ref;
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

    fn borrow_storage(storage: &ComponentStorage) -> Self::View<'_>;
    fn borrow_storage_mut(storage: &ComponentStorage) -> Self::View<'_>;
    fn entities<'a>(
        storage: &'a ComponentStorage,
        archetypes: &'a Ref<ArchetypesManager>,
        views: &Self::View<'_>,
    ) -> (Option<RuntimeGroup>, &'a [Entity]);
    fn iter_predicate(entity: Entity, storage: &ComponentStorage, view: &Self::View<'_>) -> bool;
    fn get_dense<'a>(
        entity: Entity,
        storage: &'a ComponentStorage,
        view: &Self::View<'_>,
    ) -> Self::Item<'a>;
    fn get_dense_mut<'a>(
        entity: Entity,
        storage: &'a ComponentStorage,
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

macro_rules! generate_query {
    ( $( ($components:tt, $index:tt) ),+) => {
      impl<$($components),*> TQuery for ($($components,)*)
      where $($components: 'static),* {

          type ViewMut<'a> = ($(StorageView<'a, $components>,)*);
          type View<'a> = ($(StorageView<'a, $components>,)*);
          type Item<'a> = (Entity, $(&'a $components,)*);
          type ItemMut<'a> = (Entity, $(&'a mut $components,)*);
          type Iter<'a> = Iter<'a, Self::Item<'a>>;

        fn borrow_storage(storage: &ComponentStorage) -> Self::View<'_> {
            // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
            // @todo! using const compilation strategy like Attributes or something ?
            (
                $(
                    StorageView {
                        store: storage.get_storage_metadata::<$components>(),
                        _phantom: PhantomData,
                    },
                )*
            )
        }

        fn borrow_storage_mut(storage: &ComponentStorage) -> Self::View<'_> {
            // @todo! maybe find a way to reduce the TypeId to a unique i64/32 identifier
            // @todo! using const compilation strategy like Attributes or something ?
            (
                $(
                    StorageView {
                        store: storage.get_storage_metadata::<$components>(),
                        _phantom: PhantomData,
                    },
                )*
            )
        }

        fn entities<'a>(
            storage: &'a ComponentStorage,
            archetypes: &Ref<ArchetypesManager>,
            views: &Self::View<'_>,
        ) -> (Option<RuntimeGroup>, &'a [Entity]) {
            let stores_length = ($(storage.get_storage_by_id(views.$index.store.index).entity_to_component.len(),)*);

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

            let store = storage.get_storage_by_id(min_store_id);
            let entities = store.entity_to_component.as_slice().as_ptr();

            let group: Option<RuntimeGroup> = {
                let arch_manager = archetypes;

                $(
                    assert!(views.$index.store.index <= GroupMask::max_bit_shift())
                );*;

                let mut query_mask = GroupMask::new(None);
                $(
                  query_mask.set(views.$index.store.index as u8)
                );*;

                let group = arch_manager.find_group(&query_mask);

                #[cfg(feature = "ecs_queries_logs")]
                {
                    if let Some(matching_group) = group {
                        println!(
                            "[system.rs] Group found for query (gm: {:?}, len: {}) [{:?}]",
                            matching_group.mask, matching_group.len, query_mask
                        );
                    }
                }

                group
            };

            // NOTE(JuH): either take the buffer len or the group len if any
            let entities_len = group.map_or(store.component_buffer.len, |g| g.len as usize);
            unsafe {
                let etts: &'a [Entity] = std::slice::from_raw_parts(entities, entities_len);
                (group, etts)
            }
        }

        fn iter_predicate(entity: Entity, storage: &ComponentStorage, view: &Self::View<'_>) -> bool {
            $(
               if !storage.get_storage_by_id(view.$index.store.index).has(entity) {
                   return false;
               }
            )*

            true
        }

        fn get_dense<'a>(entity: Entity, storage: &'a ComponentStorage, view: &Self::View<'_>) -> Self::Item<'a> {
            unsafe {
                let comps = ($(
                    storage
                        .get_storage_unchecked(view.$index.store.index)
                        .get::<$components>(entity).unwrap() as *const $components,
                )*);
                (entity, $(&*comps.$index,)*)
            }
        }

        fn get_dense_mut<'a>(
            entity: Entity,
            storage: &'a ComponentStorage,
            view: &Self::View<'_>,
        ) -> Self::ItemMut<'a> {
            unsafe {
                let components = ($(
                    storage.get_storage_unchecked_mut(view.$index.store.index)
                        .get_mut::<$components>(entity).unwrap() as *mut $components,
                )*);

                (entity, $(&mut *components.$index,)*)
            }
        }
      }
    };
}

generate_query!((A, 0));
generate_query!((A, 0), (B, 1));
generate_query!((A, 0), (B, 1), (C, 2));
generate_query!((A, 0), (B, 1), (C, 2), (D, 3));
generate_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
generate_query!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
