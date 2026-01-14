//! `syn_collections` - Specialized collections for Synarion Engine.
//!
//! This crate provides efficient data structures optimized for game engine use cases:
//! - [`SlotMap<T>`] - Generational storage with O(1) operations
//! - [`Arena<T>`] - Bump allocator for bulk allocations
//! - [`SparseSet<T>`] - Cache-friendly sparse storage for ECS

#![deny(warnings)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod arena;
mod slot_map;
mod sparse_set;

pub use arena::Arena;
pub use slot_map::SlotMap;
pub use sparse_set::SparseSet;

// Re-export Handle from syn_core for convenience
pub use syn_core::Handle;
