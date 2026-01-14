//! `syn_core` - Foundation types for Synarion Engine.
//!
//! This crate provides the fundamental building blocks used throughout the engine:
//! - [`Handle<T>`] - Type-safe generational handles for resource references
//!
//! For collections that use handles, see `syn_collections`.

#![deny(warnings)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod handle;

pub use handle::Handle;
