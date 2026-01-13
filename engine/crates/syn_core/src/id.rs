//! Unique identifier types.
//!
//! This module provides various identifier types used throughout the engine.

use std::sync::atomic::{AtomicU64, Ordering};

/// A simple unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    /// Creates a new unique ID.
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Creates an ID from a raw value.
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw value of this ID.
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Returns an invalid ID (0).
    pub fn invalid() -> Self {
        Self(0)
    }

    /// Returns true if this ID is invalid.
    pub fn is_invalid(&self) -> bool {
        self.0 == 0
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

/// A generational identifier that includes a generation counter.
///
/// This is useful for detecting stale references in slot-based data structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenerationalId {
    index: u32,
    generation: u32,
}

impl GenerationalId {
    /// Creates a new generational ID.
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Returns the index part.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation part.
    pub fn generation(&self) -> u32 {
        self.generation
    }
}
