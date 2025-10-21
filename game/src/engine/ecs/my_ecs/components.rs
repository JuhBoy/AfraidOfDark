use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};

use crate::engine::ecs::my_ecs::entities::Entity;
use crate::engine::ecs::my_ecs::utils::{ByteBuffer, GroupMask, SparseVec, SparseView, ID};
use std::any::TypeId;
use std::cell::{RefCell, RefMut};
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

    pub fn remove<T>(&mut self, ett: Entity) -> Option<usize> {
        let removed_index: usize = { self.entities.remove(ett.id, ett.version) }?;

        let last_dense_index = self.component_buffer.len - 1;

        let last_entity = self.entity_to_component[last_dense_index];
        if last_entity.eq(&ett) {
            self.component_buffer.len -= 1;
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
            .swap::<T>(removed_index, last_dense_index);
        self.component_buffer.len -= 1;

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

    pub fn get_entity(&self, position: usize) -> Option<Entity> {
        let entity = self.entity_to_component.get(position);
        entity.copied()
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

pub struct ComponentStorage {
    pub(crate) storages_index_by_type_id: HashMap<TypeId, ComponentMetaData>,
    pub(crate) storages: Vec<AtomicRefCell<ComponentBufferSparseSet>>,
}
impl ComponentStorage {
    pub fn new(capacity: usize) -> Self {
        ComponentStorage {
            storages_index_by_type_id: HashMap::with_capacity(capacity),
            storages: Vec::with_capacity(capacity),
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
            entities: SparseVec::new(2000),
            component_buffer: ByteBuffer::with_capacity::<T>(2000).unwrap(),
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

    pub fn get_storage<T>(&self) -> AtomicRef<ComponentBufferSparseSet>
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages[self.storages_index_by_type_id[&component_type_id].index].borrow()
    }

    pub fn get_storage_mut<T>(&self) -> AtomicRefMut<ComponentBufferSparseSet>
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages[self.storages_index_by_type_id[&component_type_id].index].borrow_mut()
    }

    pub fn get_storage_mut_by_id(&self, index: usize) -> AtomicRefMut<ComponentBufferSparseSet> {
        self.storages[index].borrow_mut()
    }

    pub fn get_storage_by_id(&self, index: usize) -> AtomicRef<ComponentBufferSparseSet> {
        self.storages[index].borrow()
    }

    pub fn get_storage_metadata<T>(&self) -> ComponentMetaData
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        self.storages_index_by_type_id[&component_type_id]
    }
}
