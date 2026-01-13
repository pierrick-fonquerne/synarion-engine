//! Object pool implementations.
//!
//! Pools provide efficient allocation and reuse of objects.

use crate::handle::Handle;
use std::marker::PhantomData;

/// A slot in the pool, containing either a value or a link to the next free slot.
enum Slot<T> {
    Occupied { value: T, generation: u32 },
    Vacant { next_free: Option<u32> },
}

/// A generational pool that stores objects and returns handles to them.
///
/// The pool uses generational indices to detect stale handles.
pub struct Pool<T> {
    slots: Vec<Slot<T>>,
    first_free: Option<u32>,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> Pool<T> {
    /// Creates a new empty pool.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            first_free: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Creates a new pool with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            first_free: None,
            len: 0,
            _marker: PhantomData,
        }
    }

    /// Inserts a value into the pool and returns a handle to it.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.first_free {
            let slot = &mut self.slots[index as usize];
            if let Slot::Vacant { next_free } = slot {
                self.first_free = *next_free;
                *slot = Slot::Occupied { value, generation: 0 };
                self.len += 1;
                return Handle::new(index, 0);
            }
        }

        let index = self.slots.len() as u32;
        self.slots.push(Slot::Occupied { value, generation: 0 });
        self.len += 1;
        Handle::new(index, 0)
    }

    /// Removes the value associated with the handle and returns it.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let index = handle.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        let slot = &mut self.slots[index];
        if let Slot::Occupied { generation, .. } = slot {
            if *generation != handle.generation() {
                return None;
            }
        } else {
            return None;
        }

        let old_slot = std::mem::replace(
            slot,
            Slot::Vacant { next_free: self.first_free },
        );

        self.first_free = Some(handle.index());
        self.len -= 1;

        if let Slot::Occupied { value, .. } = old_slot {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a reference to the value associated with the handle.
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let index = handle.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        if let Slot::Occupied { value, generation } = &self.slots[index] {
            if *generation == handle.generation() {
                return Some(value);
            }
        }

        None
    }

    /// Returns a mutable reference to the value associated with the handle.
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let index = handle.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        if let Slot::Occupied { value, generation } = &mut self.slots[index] {
            if *generation == handle.generation() {
                return Some(value);
            }
        }

        None
    }

    /// Returns the number of elements in the pool.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}
