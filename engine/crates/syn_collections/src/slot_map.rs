//! A slot map implementation with generational indices.

use syn_core::Handle;

/// A slot map that stores values and returns handles to them.
pub struct SlotMap<T> {
    values: Vec<Option<T>>,
    generations: Vec<u32>,
    free_list: Vec<u32>,
}

impl<T> SlotMap<T> {
    /// Creates a new empty slot map.
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Inserts a value and returns a handle.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let gen = self.generations[index as usize];
            self.values[index as usize] = Some(value);
            Handle::new(index, gen)
        } else {
            let index = self.values.len() as u32;
            self.values.push(Some(value));
            self.generations.push(0);
            Handle::new(index, 0)
        }
    }

    /// Removes a value by handle.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let idx = handle.index() as usize;
        if idx < self.values.len() && self.generations[idx] == handle.generation() {
            self.generations[idx] = self.generations[idx].wrapping_add(1);
            self.free_list.push(handle.index());
            self.values[idx].take()
        } else {
            None
        }
    }

    /// Gets a reference by handle.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let idx = handle.index() as usize;
        if idx < self.values.len() && self.generations[idx] == handle.generation() {
            self.values[idx].as_ref()
        } else {
            None
        }
    }

    /// Gets a mutable reference by handle.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let idx = handle.index() as usize;
        if idx < self.values.len() && self.generations[idx] == handle.generation() {
            self.values[idx].as_mut()
        } else {
            None
        }
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}
