use crate::engine::ecs::my_ecs::entities::Entity;
use crate::engine::ecs::my_ecs::utils::{ByteBuffer, SparseVec, SparseView, ID};
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
    pub fn insert<T>(&mut self, ett: Entity, vel: T) -> bool {
        let view = self.entities.get_unchecked(ett.id());

        match view {
            Some(active_view) => {
                active_view.version = ett.version();
                self.entity_to_component[active_view.index] = ett;
                return self.component_buffer.replace(active_view.index, vel);
            }
            None => {
                let Some(dense_index) = self.component_buffer.allocate(vel) else {
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

        let last_view = self.entities.get_unchecked(last_entity.id());
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

    pub fn get<T>(&mut self, ett: Entity) -> Option<&T> {
        let Some(view) = self.entities.get_unchecked(ett.id()) else {
            return None;
        };

        self.component_buffer.get_ref(view.index)
    }

    pub fn get_mut<T>(&mut self, ett: Entity) -> Option<&mut T> {
        let Some(view) = self.entities.get_unchecked(ett.id()) else {
            return None;
        };

        self.component_buffer.get_mut_ref(view.index)
    }
}

/// ============================
/// Component Storage ----------
/// ============================
pub struct ComponentStorage {
    pub(crate) store_view_by_component: HashMap<TypeId, usize>,
    pub(crate) component_storages: Vec<ComponentBufferSparseSet>,
}
impl ComponentStorage {
    pub fn new(capacity: usize) -> Self {
        ComponentStorage {
            store_view_by_component: HashMap::with_capacity(capacity),
            component_storages: Vec::with_capacity(capacity),
        }
    }

    pub fn allocate<T>(&mut self) -> bool
    where
        T: 'static,
    {
        let component_type_id = TypeId::of::<T>();
        let Entry::Vacant(entry) = self.store_view_by_component.entry(component_type_id) else {
            return false;
        };

        self.component_storages.push(ComponentBufferSparseSet {
            entities: SparseVec::new(100),
            component_buffer: ByteBuffer::with_capacity::<T>(100).unwrap(),
            entity_to_component: vec![],
        });
        let index = self.component_storages.len() - 1;
        entry.insert(index);

        true
    }

    pub fn add_component<T>(&mut self, entity: Entity) -> bool
    where
        T: 'static,
    {
        let search_type = TypeId::of::<T>();
        if !self.store_view_by_component.contains_key(&search_type) {
            return false;
        }

        true
    }
}
