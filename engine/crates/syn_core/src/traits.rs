//! Common traits used throughout the engine.

use crate::id::Id;

/// A trait for objects that have a unique identifier.
pub trait Identifiable {
    /// Returns the unique identifier of this object.
    fn id(&self) -> Id;
}

/// A trait for objects that have a name.
pub trait Named {
    /// Returns the name of this object.
    fn name(&self) -> &str;
}
