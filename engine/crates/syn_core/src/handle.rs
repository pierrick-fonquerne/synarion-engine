//! Generational handles for safe resource references.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A type-safe handle to a resource of type `T`.
///
/// Handles consist of an index and a generation counter. When a resource
/// is removed, the generation increments, invalidating old handles.
///
/// # Example
///
/// ```
/// use syn_core::Handle;
///
/// // Handles are typically created by collection types like SlotMap
/// let handle: Handle<String> = Handle::new(0, 1);
///
/// assert_eq!(handle.index(), 0);
/// assert_eq!(handle.generation(), 1);
/// ```
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<T>,
}

impl<T> Handle<T> {
    /// Creates a new handle from raw parts.
    ///
    /// This is primarily used by collection types like `SlotMap`.
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index component of this handle.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component of this handle.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }
}

impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = std::any::type_name::<T>();
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);

        if cfg!(debug_assertions) {
            f.debug_struct(&format!("Handle<{short_name}>"))
                .field("index", &self.index)
                .field("generation", &self.generation)
                .finish()
        } else {
            write!(f, "Handle<{short_name}>#{}", self.index)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_is_copy() {
        let h1: Handle<u32> = Handle::new(1, 1);
        let h2 = h1;
        assert_eq!(h1, h2);
    }

    #[test]
    fn handle_equality() {
        let h1: Handle<u32> = Handle::new(1, 1);
        let h2: Handle<u32> = Handle::new(1, 1);
        let h3: Handle<u32> = Handle::new(1, 2);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn handle_hash_works() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let h1: Handle<u32> = Handle::new(1, 1);
        let h2: Handle<u32> = Handle::new(1, 2);

        set.insert(h1);
        assert!(set.contains(&h1));
        assert!(!set.contains(&h2));
    }
}
