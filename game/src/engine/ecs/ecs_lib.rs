use std::alloc::{alloc, dealloc, Layout};
use std::cmp::max;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Entity {
    pub id: usize,
    pub version: u32,
}
impl ID for Entity {
    fn id(&self) -> usize {
        self.id
    }
    fn version(&self) -> u32 {
        self.version
    }
}

pub struct EntityStorage {
    pub entities: SparseSet<Entity>,
    pub allocator: EntityAllocator,
}
impl EntityStorage {
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

#[test]
pub fn test_me() {
    let mut storage = EntityStorage {
        entities: SparseSet {
            dense_set: Vec::new(),
            sparse_views: SparseVec::new(10),
        },
        allocator: EntityAllocator::new(),
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
    for ett in storage.entities.iter() {
        assert_eq!(i, ett.id);
        i += 1;
    }
    assert_eq!(i, 10);

    storage.reset();
}

pub trait ID {
    fn id(&self) -> usize;
    fn version(&self) -> u32;
}

pub struct SparseSet<T>
where
    T: Copy + ID,
{
    pub dense_set: Vec<T>,
    pub sparse_views: SparseVec,
}
impl<T> SparseSet<T>
where
    T: Copy + ID,
{
    pub fn len(&self) -> usize {
        self.dense_set.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.dense_set.iter()
    }

    pub fn insert(&mut self, entity: T) {
        let view = self.sparse_views.get_unchecked(entity.id());

        // in case of existing view just make the version match and replace the entity structure
        // maybe using mem::replace would be more efficient ? this need to be profiled as i suspect the compiler
        // to do the exact same thing behind the hood
        if let Some(existing_view) = view {
            existing_view.version = entity.version();
            self.dense_set[existing_view.index] = entity;
        } else {
            *view = Some(SparseView {
                index: self.dense_set.len(),
                version: entity.version(),
            });
            self.dense_set.push(entity);
        }
    }

    pub fn remove(&mut self, entity: T) -> bool {
        let Some(dense_id) = self.sparse_views.remove(entity.id(), entity.version()) else {
            return false;
        };

        // zero for version allow us to reuse slot but ensure that the entity is marked removed
        // so all entities must have a version >= 1 to be considered valid
        let slot = self.sparse_views.get_unchecked(entity.id());
        *slot = Some(SparseView {
            index: dense_id,
            version: 0,
        });

        true
    }

    #[must_use]
    pub fn has(&self, entity: T) -> bool {
        self.sparse_views.has(entity.id(), entity.version())
    }

    pub fn clear(&mut self) {
        self.sparse_views.clear(false);
        self.dense_set.clear();
    }
}

// the underlying sparse vector for index access
pub struct SparseVec {
    sparses: Vec<Option<SparseView>>,
    default_capacity: usize,
}
#[derive(Default, Clone, Copy)]
pub struct SparseView {
    pub index: usize,
    pub version: u32,
}
impl SparseVec {
    pub fn new(capacity: usize) -> Self {
        SparseVec {
            default_capacity: capacity,
            sparses: vec![None; capacity],
        }
    }

    pub fn has(&self, index: usize, version: u32) -> bool {
        self.sparses.get(index).map_or(false, |view| {
            if let Some(view) = view {
                return view.version == version;
            }

            false
        })
    }

    pub fn get_unchecked(&mut self, index: usize) -> &mut Option<SparseView> {
        if index >= self.sparses.len() {
            self.sparses.resize_with(
                max(self.sparses.len() * 2, index.next_power_of_two()),
                Default::default,
            );
        }

        unsafe { self.sparses.get_unchecked_mut(index) }
    }

    pub fn remove(&mut self, index: usize, version: u32) -> Option<usize> {
        self.sparses
            .get_mut(index)?
            .take_if(|e| e.version == version)
            .map(|s| s.index)
    }

    pub fn clear(&mut self, reset_buffer_size: bool) {
        self.sparses.clear();

        if reset_buffer_size {
            self.sparses = vec![None; self.default_capacity];
        }
    }
}

////// COMPONENTS

pub struct ByteBuffer<T> {
    pub data: NonNull<u8>,
    pub len: usize,
    pub capacity: usize,

    phantom: PhantomData<T>,
}

impl<T> ByteBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Option<Self> {
        let layout: Layout = Layout::array::<T>(capacity).unwrap();

        let n = unsafe { alloc(layout) };
        let nn: NonNull<u8> = NonNull::new(n)?;

        Some(Self {
            data: nn,
            len: 0,
            capacity,
            phantom: PhantomData,
        })
    }

    #[must_use]
    pub fn allocate(&mut self, data: T) -> Option<usize> {
        if self.len >= self.capacity {
            return None;
        }

        unsafe {
            let index = self.len;
            let slot = self.data.as_ptr().add(index * size_of::<T>()) as *mut T;
            slot.write(data);
        }
        self.len += 1;

        Some(self.len - 1)
    }

    #[must_use]
    pub fn replace(&mut self, index: usize, data: T) -> bool {
        if index >= self.len {
            return false;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(index * size_of::<T>()) as *mut T;
            ptr.write(data);
        }

        true
    }

    pub fn swap(&mut self, index_a: usize, index_b: usize) -> bool {
        if index_a >= self.len || index_b >= self.len {
            return false;
        }

        unsafe {
            let ptr = self.data.as_ptr().cast::<T>().add(index_a);
            let ptr_b = self.data.as_ptr().cast::<T>().add(index_b);
            ptr.swap(ptr_b);
        }

        true
    }

    pub fn get_ref(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        unsafe {
            let ptr_t = self.data.cast::<T>().add(index);
            Some(ptr_t.as_ref())
        }
    }

    pub fn get_mut_ref(&self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        unsafe {
            let mut ptr_t = self.data.cast::<T>().add(index);
            Some(ptr_t.as_mut())
        }
    }

    pub fn clear(&mut self) -> bool {
        let Ok(layout) = Layout::array::<T>(self.capacity) else {
            return false;
        };

        self.capacity = 0;
        self.len = 0;

        unsafe {
            dealloc(self.data.as_ptr(), layout);
            true
        }
    }
}

pub struct ComponentStorage {
    entity_to_component: Vec<Entity>,

    pub entities: SparseVec,
    pub memory_buffer: ByteBuffer<Velocity>,
}
impl ComponentStorage {
    pub fn insert(&mut self, ett: Entity, vel: Velocity) -> bool {
        let view = self.entities.get_unchecked(ett.id());

        match view {
            Some(active_view) => {
                active_view.version = ett.version();
                self.entity_to_component[active_view.index] = ett;
                return self.memory_buffer.replace(active_view.index, vel);
            }
            None => {
                let Some(dense_index) = self.memory_buffer.allocate(vel) else {
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

        let last_dense_index = self.memory_buffer.len - 1;

        let last_entity = self.entity_to_component[last_dense_index];
        if last_entity.eq(&ett) {
            self.memory_buffer.len -= 1;
            return Some(removed_index);
        }

        let last_view = self.entities.get_unchecked(last_entity.id());
        *last_view = Some(SparseView {
            index: removed_index,
            version: last_entity.version,
        });
        self.entity_to_component
            .swap(removed_index, last_dense_index);
        self.memory_buffer.swap(removed_index, last_dense_index);
        self.memory_buffer.len -= 1;

        Some(removed_index)
    }

    pub fn has(&self, ett: Entity) -> bool {
        self.entities.has(ett.id(), ett.version())
    }

    pub fn get(&mut self, ett: Entity) -> Option<&Velocity> {
        let Some(view) = self.entities.get_unchecked(ett.id()) else {
            return None;
        };

        self.memory_buffer.get_ref(view.index)
    }

    pub fn get_mut(&mut self, ett: Entity) -> Option<&mut Velocity> {
        let Some(view) = self.entities.get_unchecked(ett.id()) else {
            return None;
        };

        self.memory_buffer.get_mut_ref(view.index)
    }
}

#[test]
pub fn test_buffer() {
    let Some(mut buffer) = ByteBuffer::<Velocity>::with_capacity(100) else {
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

    let velocity_mut = buffer.get_mut_ref(50);
    if let Some(velvel) = velocity_mut {
        velvel.x = 2455.123;
    }

    for i in 0..100 {
        let velocity = buffer.get_ref(i);
        assert!(velocity.is_some());

        if i == 50 {
            assert_eq!(velocity.unwrap().x, 2455.123f32);
            continue;
        }

        let vel = velocity.unwrap();
        assert_eq!(vel.x, i as f32 * 1.0);
    }

    buffer.clear();
}

#[test]
pub fn test_component_storage() {
    let mut com_storage = ComponentStorage {
        entities: SparseVec::new(100),
        memory_buffer: ByteBuffer::<Velocity>::with_capacity(100).unwrap(),
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
    let Some(pos) = com_storage.entities.get_unchecked(entity_10.id()) else {
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
    assert_eq!(com_storage.memory_buffer.len, 0);

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
    assert_eq!(com_storage.memory_buffer.len, 100);

    for i in 0..50 {
        let ett = Entity { id: i, version: 1 };
        let loop_removed = com_storage.remove(ett);
        assert!(loop_removed.is_some());
    }
    assert_eq!(com_storage.memory_buffer.len, 50);

    for i in 50..100 {
        let ett = Entity { id: i, version: 1 };
        let is_there = com_storage.has(ett);
        assert!(is_there);

        {
            let get = com_storage.get(ett).unwrap();
            assert_eq!(i as f32, get.x);
        }

        let sparse = com_storage.entities.get_unchecked(ett.id());
        assert!(sparse.is_some());
        assert_eq!(99 - i, sparse.unwrap().index);

        let velocity = com_storage.get(ett);
        assert!(velocity.is_some());
        assert!(velocity.unwrap().x >= 50f32);
    }

    for i in 50..100 {
        let ett = Entity { id: i, version: 1 };
        com_storage.remove(ett);
    }
    assert_eq!(com_storage.memory_buffer.len, 0);

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

    let r_5_vel = com_storage.get(Entity { id: 5, version: 1 }).unwrap();
    assert_eq!(500f32, r_5_vel.x);
}
