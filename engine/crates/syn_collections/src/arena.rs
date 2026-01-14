//! Arena allocator for bulk allocations.
//!
//! [`Arena<T>`] is a bump allocator that allocates objects in contiguous chunks.
//! It's ideal for allocating many objects with the same lifetime.

/// A simple arena allocator that allocates objects in a contiguous buffer.
///
/// Unlike [`SlotMap`], `Arena` does not support individual removal. It's optimized
/// for bulk allocation and clearing.
///
/// # Example
///
/// ```
/// use syn_collections::Arena;
///
/// let mut arena = Arena::new();
///
/// // Allocate values
/// *arena.alloc(1) = 10;
/// arena.alloc(2);
/// arena.alloc(3);
///
/// assert_eq!(arena.len(), 3);
///
/// arena.clear(); // Deallocate all at once
/// assert!(arena.is_empty());
/// ```
///
/// [`SlotMap`]: crate::SlotMap
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    chunk_size: usize,
}

impl<T> Arena<T> {
    /// Creates a new arena with the default chunk size (1024).
    #[inline]
    pub fn new() -> Self {
        Self::with_chunk_size(1024)
    }

    /// Creates a new arena with the specified chunk size.
    #[inline]
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            chunk_size,
        }
    }

    /// Allocates a new value in the arena, returning a mutable reference.
    ///
    /// The returned reference is valid until [`clear`](Self::clear) is called.
    ///
    /// # Panics
    ///
    /// This function will not panic under normal circumstances.
    #[inline]
    pub fn alloc(&mut self, value: T) -> &mut T {
        if self.needs_new_chunk() {
            self.chunks.push(Vec::with_capacity(self.chunk_size));
        }

        let chunk = self.chunks.last_mut().expect("chunk was just added");
        chunk.push(value);
        chunk.last_mut().expect("value was just pushed")
    }

    /// Returns true if a new chunk needs to be allocated.
    #[inline]
    fn needs_new_chunk(&self) -> bool {
        self.chunks.is_empty()
            || self
                .chunks
                .last()
                .is_none_or(|c| c.len() >= self.chunk_size)
    }

    /// Clears all allocations, invalidating all references.
    #[inline]
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Returns the total number of allocated objects.
    #[inline]
    pub fn len(&self) -> usize {
        self.chunks.iter().map(Vec::len).sum()
    }

    /// Returns true if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty() || self.chunks.iter().all(Vec::is_empty)
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_access() {
        let mut arena = Arena::new();
        let a = arena.alloc(42);
        assert_eq!(*a, 42);
    }

    #[test]
    fn multiple_allocs() {
        let mut arena = Arena::new();
        for i in 0..100 {
            let val = arena.alloc(i);
            assert_eq!(*val, i);
        }
        assert_eq!(arena.len(), 100);
    }

    #[test]
    fn clear_works() {
        let mut arena = Arena::new();
        arena.alloc(1);
        arena.alloc(2);
        arena.clear();
        assert!(arena.is_empty());
    }
}
