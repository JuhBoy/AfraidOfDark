use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;

pub struct Uuid<T> {
    free_id: u32,
    phantom: PhantomData<T>,
}

impl<T> Default for Uuid<T> {
    fn default() -> Self {
        Self {
            free_id: 0u32,
            phantom: PhantomData,
        }
    }
}

impl<T> Uuid<T> {
    pub fn get_next(&mut self) -> u32 {
        debug_assert!(self.free_id < u32::MAX);

        let value = self.free_id;
        self.free_id += 1;

        value
    }

    pub fn new(&self, value: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(value.as_bytes());

        hasher.finish()
    }
}
