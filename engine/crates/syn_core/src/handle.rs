//! Type-safe handles for resource management.
//!
//! Handles provide a safe way to reference resources without direct pointers.
//! They use generational indices to detect use-after-free bugs at runtime.

use std::marker::PhantomData;

/// A type-safe handle to a resource of type `T`.
///
/// Handles are lightweight identifiers that can be used to look up resources
/// in a pool or registry. They contain a generational index to detect stale references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Creates a new handle with the given index and generation.
    pub fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index part of this handle.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation part of this handle.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Creates an invalid handle that doesn't point to any resource.
    pub fn invalid() -> Self {
        Self {
            index: u32::MAX,
            generation: u32::MAX,
            _marker: PhantomData,
        }
    }

    /// Returns true if this handle is invalid.
    pub fn is_invalid(&self) -> bool {
        self.index == u32::MAX && self.generation == u32::MAX
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::invalid()
    }
}
