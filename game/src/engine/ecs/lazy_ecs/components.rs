use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use image::buffer;

use crate::engine::ecs::lazy_ecs::archetypes::{ArchetypesManager, MatchType};
use crate::engine::ecs::lazy_ecs::entities::Entity;
use crate::engine::ecs::lazy_ecs::utils::{ByteBuffer, GroupMask, SparseVec, SparseView, ID};
use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

/// ============================
/// Component Buffer Sparse Set
/// ============================
pub struct ComponentBufferSparseSet {
    pub entities: SparseVec,
    pub component_buffer: ByteBuffer,
    pub entity_to_component: Vec<Entity>,
}

impl ComponentBufferSparseSet {
    pub fn clear(&mut self) {
        self.component_buffer.clear_untyped();
        self.entities.clear(true);
        self.entity_to_component.clear();
    }

    pub fn insert<T>(&mut self, ett: Entity, component: T) -> bool {
        let view = self.entities.get_unchecked_mut(ett.id());

        match view {
            Some(active_view) => {
                active_view.version = ett.version();
                self.entity_to_component[active_view.index] = ett;
                return self.component_buffer.replace(active_view.index, component);
            }
            None => {
                let Some(dense_index) = self.component_buffer.allocate(component) else {
                    return false;
                };
                if dense_index >= self.entity_to_component.len() {
                    self.entity_to_component.push(ett);
                } else {
                    self.entity_to_component[dense_index] = ett;
                }

                *view = Some(SparseView {
                    index: dense_index,
                    version: ett.version(),
                });
            }
        }

        true
    }

    pub fn remove(&mut self, ett: Entity) -> Option<usize> {
        let removed_index: usize = { self.entities.remove(ett.id, ett.version) }?;
        let last_dense_index = self.component_buffer.len - 1;
        let last_entity = self.entity_to_component[last_dense_index];

        if last_entity.eq(&ett) {
            self.component_buffer.remove_last();
            self.entity_to_component.pop();
            return Some(removed_index);
        }

        let last_view = self.entities.get_unchecked_mut(last_entity.id());
        *last_view = Some(SparseView {
            index: removed_index,
            version: last_entity.version,
        });
        self.entity_to_component
            .swap(removed_index, last_dense_index);
        self.component_buffer
            .swap_untyped(removed_index, last_dense_index);
        self.entity_to_component.pop();
        self.component_buffer.remove_last();

        Some(removed_index)
    }

    pub fn has(&self, ett: Entity) -> bool {
        self.entities.has(ett.id(), ett.version())
    }

    pub fn get<T>(&self, ett: Entity) -> Option<&T> {
        let view_index = self.entities.get(ett.id(), ett.version())?;

        self.component_buffer.get_ref(view_index)
    }

    pub fn get_mut<T>(&mut self, ett: Entity) -> Option<&mut T> {
        let view_index = self.entities.get_mut(ett.id(), ett.version())?;

        self.component_buffer.get_mut_ref(view_index)
    }

    pub fn swap<T>(&mut self, ett_source: Entity, ett_dest: Entity) -> bool {
        let Some(view_source) = self.entities.get(ett_source.id(), ett_source.version()) else {
            return false;
        };
        let Some(view_dest) = self.entities.get(ett_dest.id(), ett_dest.version()) else {
            return false;
        };

        // swap components stored in memory buffer
        if !self.component_buffer.swap::<T>(view_source, view_dest) {
            return false;
        }

        // swap entity_to_component for archetypes, they are stored in order
        self.entity_to_component.swap(view_source, view_dest);

        let source_index = view_source;
        let dest_index = view_dest;

        let source = self
            .entities
            .get_unchecked_mut(ett_source.id())
            .as_mut()
            .unwrap();
        source.index = dest_index;

        let dest = self
            .entities
            .get_unchecked_mut(ett_dest.id())
            .as_mut()
            .unwrap();
        dest.index = source_index;

        true
    }

    pub fn swap_untyped(&mut self, ett_source: Entity, ett_dest: Entity) -> bool {
        let Some(view_source) = self.entities.get(ett_source.id(), ett_source.version()) else {
            return false;
        };
        let Some(view_dest) = self.entities.get(ett_dest.id(), ett_dest.version()) else {
            return false;
        };

        // swap components stored in memory buffer
        if !self.component_buffer.swap_untyped(view_source, view_dest) {
            return false;
        }

        // swap entity_to_component for archetypes, they are stored in order
        self.entity_to_component.swap(view_source, view_dest);

        let source_index = view_source;
        let dest_index = view_dest;

        let source = self
            .entities
            .get_unchecked_mut(ett_source.id())
            .as_mut()
            .unwrap();
        source.index = dest_index;

        let dest = self
            .entities
            .get_unchecked_mut(ett_dest.id())
            .as_mut()
            .unwrap();
        dest.index = source_index;

        true
    }

    pub fn get_entity(&self, position: usize) -> Option<Entity> {
        let entity = self.entity_to_component.get(position);
        entity.copied()
    }

    pub fn get_entity_index(&self, entity: Entity) -> Option<usize> {
        let entity = self.entities.get(entity.id(), entity.version())?;
        Some(entity)
    }

    pub fn get_entity_count(&self) -> usize {
        self.entity_to_component.len()
    }
}

/// ============================
/// Component Storage ----------
/// ============================

#[derive(Clone, Copy)]
pub struct ComponentMetaData {
    pub index: usize,
    pub mask: GroupMask, // all the group containing this component
}

pub struct StoreIteratorContainer {
    pub last_position: usize,
    pub container: Vec<usize>, // sotre indexes !
}
impl StoreIteratorContainer {
    pub fn new(capacity: usize) -> Self {
        Self {
            last_position: 0,
            container: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, val: usize) {
        self.container.push(val)
    }

    pub fn clear(&mut self) {
        self.container.clear()
    }

    pub fn slice_ref(&self) -> &[usize] {
        &self.container[0..self.last_position]
    }
}

pub struct ComponentStorage {
    pub(crate) storages_index_by_type_id: HashMap<TypeId, ComponentMetaData>,
    pub(crate) storages: Vec<AtomicRefCell<ComponentBufferSparseSet>>,

    iterator_container: StoreIteratorContainer,
    dense_buffer_capacity: usize,
}
impl ComponentStorage {
    pub fn new(capacity: usize, buffer_capacity: usize) -> Self {
        ComponentStorage {
            storages_index_by_type_id: HashMap::with_capacity(capacity),
            storages: Vec::with_capacity(capacity),
            iterator_container: StoreIteratorContainer::new(capacity),
            dense_buffer_capacity: buffer_capacity,
        }
    }

    pub fn allocate<T>(&mut self) -> Option<ComponentMetaData>
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        let Entry::Vacant(entry) = self.storages_index_by_type_id.entry(component_type_id) else {
            return Some(self.storages_index_by_type_id[&component_type_id]);
        };

        let storage = ComponentBufferSparseSet {
            entities: SparseVec::new(self.dense_buffer_capacity),
            component_buffer: ByteBuffer::with_capacity::<T>(self.dense_buffer_capacity).unwrap(),
            entity_to_component: vec![],
        };
        self.storages.push(AtomicRefCell::new(storage));

        let metadata = ComponentMetaData {
            index: self.storages.len() - 1,
            mask: GroupMask::new(None),
        };
        entry.insert(metadata);

        Some(metadata)
    }

    pub fn add_component<T>(&mut self, entity: Entity, comp: T) -> bool
    where
        T: 'static,
    {
        let search_type = TypeId::of::<T>();
        if !self.storages_index_by_type_id.contains_key(&search_type) {
            return false;
        }

        let metadata = self.storages_index_by_type_id[&search_type];
        let store = &mut self.storages[metadata.index].borrow_mut();

        store.insert::<T>(entity, comp)
    }

    pub fn get_storage<T>(&'_ self) -> AtomicRef<'_, ComponentBufferSparseSet>
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages[self.storages_index_by_type_id[&component_type_id].index].borrow()
    }

    pub fn get_storage_mut<T>(&'_ self) -> AtomicRefMut<'_, ComponentBufferSparseSet>
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages[self.storages_index_by_type_id[&component_type_id].index].borrow_mut()
    }

    pub fn has_component<T>(&self, entity: Entity) -> bool
    where
        T: 'static,
    {
        let search_type = TypeId::of::<T>();

        if let Some(metadata) = self.storages_index_by_type_id.get(&search_type) {
            let store = self.storages[metadata.index].borrow();
            return store.has(entity);
        }

        false
    }

    pub fn get_storage_mut_by_id(&self, index: usize) -> AtomicRefMut<ComponentBufferSparseSet> {
        self.storages[index].borrow_mut()
    }

    pub fn get_storage_by_id(&self, index: usize) -> AtomicRef<ComponentBufferSparseSet> {
        self.storages[index].borrow()
    }

    pub fn it_storages_by_mask_mut<TCallback>(
        &self,
        mut mask: GroupMask,
        mut it_callback: TCallback,
    ) where
        TCallback: FnMut(AtomicRefMut<ComponentBufferSparseSet>),
    {
        while !mask.is_empty() {
            let next_index = mask.least_one() as usize;

            let storage_ref: AtomicRefMut<ComponentBufferSparseSet> =
                self.storages[next_index].borrow_mut();
            it_callback(storage_ref);

            mask.and(mask.get_raw() - 1);
        }
    }

    pub fn storage_ids_by_mask(&mut self, mut mask: GroupMask) -> &[usize] {
        self.iterator_container.clear();

        while !mask.is_empty() {
            let index = mask.least_one() as usize;
            mask.and(mask.get_raw() - 1);
            self.iterator_container.push(index);
        }

        // returns from [0, last_pos] slice
        self.iterator_container.slice_ref()
    }

    pub fn get_storage_metadata<T>(&self) -> ComponentMetaData
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages_index_by_type_id[&component_type_id]
    }

    pub fn ungroup(&mut self, entity: Entity, input_group: GroupMask, archetypes: &mut ArchetypesManager) -> bool {
        let groups_option = archetypes.get_supersets(&input_group, MatchType::Partial);
        let mut ungrouped: bool = false;

        for group in groups_option.unwrap_or(&mut []) {
            if group.len == 0 {
                continue;
            }
            if !group.mask.intersects(&input_group) {
                break;
            }
            let mut has_swaped: bool = false;
            let mut all_masks = group.mask;

            while !all_masks.is_empty() {
                let store_id = all_masks.least_one() as usize;
                all_masks.and(all_masks.get_raw() - 1);

                let mut store = self.get_storage_mut_by_id(store_id);
                let entity_index = store.get_entity_index(entity);

                if let Some(entity_index) = entity_index {
                    // NOTE(JuH): the entity is not part of this superset
                    if entity_index >= group.len as usize {
                        continue;
                    }

                    let swap_index: usize = group.len as usize - 1;

                    if swap_index != entity_index {
                        let last_entity = store.get_entity(swap_index).unwrap();
                        has_swaped |= store.swap_untyped(entity, last_entity);

                        println!(
                            "swaped entity: {:?}[{}] <-> {:?}[{}] (store: {}, gid: {})",
                            entity,
                            entity_index,
                            last_entity,
                            swap_index,
                            store_id,
                            group.mask.get_raw()
                        );
                    } else {
                        has_swaped = true;
                    }
                }
            }

            if has_swaped {
                println!("reduce group len by 1 for gid: {}", group.mask.get_raw());
                group.len -= 1;
                ungrouped = true;
            }
        }

        ungrouped
    }

    pub fn reset(&mut self) {
        for store in self.storages.iter() {
            store.borrow_mut().clear();
        }

        self.storages.clear();
        self.storages_index_by_type_id.clear();
        self.iterator_container.clear();
    }
}
