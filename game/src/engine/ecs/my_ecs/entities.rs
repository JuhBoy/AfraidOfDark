use crate::engine::ecs::my_ecs::utils::{GroupMask, SparseSet, ID};
use std::collections::VecDeque;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Entity {
    pub id: usize,
    pub version: u32,
}
impl Entity {
    pub fn null() -> Entity {
        Entity { id: 0, version: 0 }
    }
}
impl ID for Entity {
    fn id(&self) -> usize {
        self.id
    }
    fn version(&self) -> u32 {
        self.version
    }
}

pub struct EntityMetadata {
    pub group_masks: Vec<GroupMask>,
}
impl EntityMetadata {
    pub fn new(capacity: usize) -> Self {
        let mut meta = Self {
            group_masks: Vec::with_capacity(capacity),
        };
        meta.group_masks
            .resize_with(capacity, || GroupMask::new(None));

        meta
    }

    pub fn group(&self, pos: usize) -> Option<&GroupMask> {
        self.group_masks.get(pos)
    }

    pub fn group_mut(&mut self, pos: usize) -> Option<&mut GroupMask> {
        self.group_masks.get_mut(pos)
    }
}

pub struct EntityStorage {
    pub entities: SparseSet<Entity>,
    pub allocator: EntityAllocator,
    pub metadata: EntityMetadata,
}
impl EntityStorage {
    pub fn new(capacity: usize) -> Self {
        EntityStorage {
            entities: SparseSet::new::<Entity>(capacity),
            allocator: EntityAllocator::new(),
            metadata: EntityMetadata::new(capacity),
        }
    }

    pub fn create(&mut self) -> Entity {
        let entity = self.allocator.get();
        self.entities.insert(entity);

        entity
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        let recycled = self.allocator.recycle(entity);
        if !recycled {
            return false;
        }

        self.entities.remove(entity)
    }

    pub fn reset(&mut self) {
        self.entities.clear();
        self.allocator.reset();
    }

    pub fn dense_slice(&self) -> &[Entity] {
        &self.entities.dense_set
    }

    /// metadata ===========

    pub fn set_group(&mut self, entity: Entity, mask: GroupMask) -> bool {
        if !self.entities.has(entity) {
            return false;
        }

        let mb_group = self.metadata.group_mut(entity.id);

        if let Some(group_mask) = mb_group {
            *group_mask = mask;
            return true;
        } else {
            println!("[ERROR] entity overflow metadata capacity")
        }

        false
    }

    pub fn get_group(&mut self, entity: Entity) -> Option<GroupMask> {
        if !self.entities.has(entity) {
            return None;
        }

        let mb_group = self.metadata.group(entity.id);

        if let Some(group_mask) = mb_group {
            return Some(*group_mask);
        }

        None
    }
}

pub struct EntityAllocator {
    pub next_allocated_index: usize,
    pub next_recycled_index: usize,
    pub recycled_indexes: VecDeque<Entity>,
}
impl EntityAllocator {
    pub fn new() -> Self {
        EntityAllocator {
            next_allocated_index: 0,
            next_recycled_index: 0,
            recycled_indexes: VecDeque::new(),
        }
    }

    pub fn get(&mut self) -> Entity {
        if self.next_recycled_index < self.recycled_indexes.len() {
            let entity =
                self.recycled_indexes[self.recycled_indexes.len() - 1 - self.next_recycled_index];
            self.next_recycled_index += 1;
            return entity;
        }

        let index = self.next_allocated_index;
        self.next_allocated_index += 1;

        Entity {
            id: index,
            version: 1,
        }
    }

    pub fn recycle(&mut self, entity: Entity) -> bool {
        if (entity.id >= self.next_allocated_index) {
            return false;
        }

        self.recycled_indexes.push_front(Entity {
            id: entity.id,
            version: entity.version + 1,
        });

        true
    }

    pub fn reset(&mut self) {
        self.next_allocated_index = 0;
        self.next_recycled_index = 0;
        self.recycled_indexes.clear();
    }
}
