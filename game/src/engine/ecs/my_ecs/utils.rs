use std::alloc::{alloc, dealloc, Layout};
use std::any::TypeId;
use std::cmp::max;
use std::ptr::{self, NonNull};

pub trait ID {
    fn id(&self) -> usize;
    fn version(&self) -> u32;
}

/// ============================
/// Byte Buffer ----------------
/// ============================
pub struct ByteBuffer {
    pub data: NonNull<u8>,
    pub len: usize,
    pub capacity: usize,

    type_info: ByteBufferTypeInfo,
}

pub struct ByteBufferTypeInfo {
    pub type_id: TypeId,
    pub bytes_len: usize,
}

impl ByteBuffer {
    pub fn with_capacity<T>(capacity: usize) -> Option<Self>
    where
        T: 'static,
    {
        let layout: Layout = Layout::array::<T>(capacity).unwrap();

        let n = unsafe { alloc(layout) };
        let nn: NonNull<u8> = NonNull::new(n)?;

        Some(Self {
            data: nn,
            len: 0,
            capacity,
            type_info: ByteBufferTypeInfo {
                type_id: TypeId::of::<T>(),
                bytes_len: size_of::<T>(),
            },
        })
    }

    #[must_use]
    pub fn allocate<T>(&mut self, data: T) -> Option<usize> {
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
    pub fn replace<T>(&mut self, index: usize, data: T) -> bool {
        if index >= self.len {
            return false;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(index * size_of::<T>()) as *mut T;
            ptr.write(data);
        }

        true
    }

    pub fn swap<T>(&mut self, index_a: usize, index_b: usize) -> bool {
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

    pub fn swap_untyped(&mut self, index_a: usize, index_b: usize) -> bool {
        if index_a >= self.len || index_b >= self.len {
            return false;
        }

        unsafe {
            // NOTE(JuH): add() works here becase the pointer is u8 type, if not use bytes_add() instead
            let ptr_a = self.data.as_ptr().add(index_a * self.type_info.bytes_len);
            let ptr_b = self.data.as_ptr().add(index_b * self.type_info.bytes_len);

            // NOTE(JuH): this move the memory region, swap() in the other hand (without the type known) will move only on u8 aka byte
            ptr::swap_nonoverlapping(ptr_a, ptr_b, self.type_info.bytes_len);
        }

        true
    }

    pub fn get_ref<'a, T>(&self, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }

        unsafe {
            let ptr_t = self.data.cast::<T>().add(index);
            Some(ptr_t.as_ref())
        }
    }

    pub fn get_mut_ref<T>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        unsafe {
            let mut ptr_t = self.data.cast::<T>().add(index);
            Some(ptr_t.as_mut())
        }
    }

    pub fn clear<T>(&mut self) -> bool {
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

/// ============================
/// Sparse Vec -----------------
/// ============================
pub struct SparseVec {
    sparse: Vec<Option<SparseView>>,
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
            sparse: vec![None; capacity],
        }
    }

    pub fn len(&self) -> usize {
        self.sparse.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn has(&self, index: usize, version: u32) -> bool {
        self.sparse.get(index).map_or(false, |view| {
            if let Some(view) = view {
                return view.version == version;
            }

            false
        })
    }

    pub fn get(&self, index: usize, version: u32) -> Option<usize> {
        self.sparse
            .get(index)?
            .filter(|e| e.version == version)
            .map(|e| e.index)
    }

    pub fn get_mut(&mut self, index: usize, version: u32) -> Option<usize> {
        self.sparse
            .get_mut(index)?
            .take_if(|e| e.version == version)
            .map(|e| e.index)
    }

    pub fn get_unchecked_mut(&mut self, index: usize) -> &mut Option<SparseView> {
        if index >= self.sparse.len() {
            self.sparse.resize_with(
                max(self.sparse.len() * 2, index.next_power_of_two()),
                Default::default,
            );
        }

        unsafe { self.sparse.get_unchecked_mut(index) }
    }

    pub fn get_unchecked(&self, index: usize) -> &Option<SparseView> {
        if index >= self.sparse.len() {
            return &None;
        }

        unsafe { self.sparse.get_unchecked(index) }
    }

    pub fn remove(&mut self, index: usize, version: u32) -> Option<usize> {
        self.sparse
            .get_mut(index)?
            .take_if(|e| e.version == version)
            .map(|s| s.index)
    }

    pub fn clear(&mut self, reset_buffer_size: bool) {
        self.sparse.clear();

        if reset_buffer_size {
            self.sparse = vec![None; self.default_capacity];
        }
    }
}

/// ============================
/// Sparse Set -----------------
/// ============================
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
    pub fn new<E>(capacity: usize) -> Self {
        SparseSet {
            dense_set: Vec::new(),
            sparse_views: SparseVec::new(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.dense_set.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.dense_set.iter()
    }

    pub fn insert(&mut self, entity: T) {
        let view = self.sparse_views.get_unchecked_mut(entity.id());

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
        let slot = self.sparse_views.get_unchecked_mut(entity.id());
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

/// ============================
/// Masks ----------------------
/// ============================
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupMask {
    mask: u64,
}

impl GroupMask {
    pub const fn max_bit_shift() -> usize {
        63
    }

    pub fn new(base: Option<u64>) -> Self {
        Self {
            mask: base.unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mask == 0
    }

    pub fn get_raw(&self) -> u64 {
        self.mask
    }

    pub fn clear(&mut self) -> u64 {
        self.mask = 0;
        self.mask
    }

    pub fn or(&mut self, value: u64) {
        self.mask |= value;
    }

    pub fn and(&mut self, value: u64) {
        self.mask &= value;
    }

    pub fn set(&mut self, index: u8) {
        self.mask |= 1 << index;
    }

    pub fn unset(&mut self, index: u8) {
        self.mask ^= 1 << index;
    }

    pub fn is_set(&self, index: u8) -> bool {
        self.mask & (1 << index) != 0
    }

    pub fn is_match(&self, other: &GroupMask) -> bool {
        self.mask == other.mask
    }

    pub fn intersects(&self, other: &GroupMask) -> bool {
        self.mask & other.mask != 0
    }

    pub fn is_superset_of(&self, other: &GroupMask) -> bool {
        (self.mask & other.mask) == other.mask
    }

    #[inline(always)]
    pub fn least_one(&self) -> u8 {
        #[cfg(TODO_PANIC_ON_NUMERIC_INVALIDS)]
        {
            if self.is_empty() {
                panic!("cttz will be poisoned by 0 values, it can't find any 1s bit set");
            }
        }

        // NOTE(JuH): this call cttz behind the hood, it resolves in 1 CPU cycle most of the time (x86-64) and 2 on Arch
        self.mask.trailing_zeros() as u8
    }

    pub fn excludes(&mut self, other: &GroupMask) {
        let other_inverse = !other.get_raw();
        self.mask &= other_inverse;
    }
}
