//! Arena allocator for bulk allocations.

/// A simple arena allocator that allocates objects in a contiguous buffer.
pub struct Arena<T> {
    chunks: Vec<Vec<T>>,
    chunk_size: usize,
}

impl<T> Arena<T> {
    /// Creates a new arena with the default chunk size.
    pub fn new() -> Self {
        Self::with_chunk_size(1024)
    }

    /// Creates a new arena with the specified chunk size.
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunks: Vec::new(),
            chunk_size,
        }
    }

    /// Allocates a new value in the arena.
    pub fn alloc(&mut self, value: T) -> &mut T {
        if self.chunks.is_empty() || self.chunks.last().unwrap().len() >= self.chunk_size {
            self.chunks.push(Vec::with_capacity(self.chunk_size));
        }
        let chunk = self.chunks.last_mut().unwrap();
        chunk.push(value);
        chunk.last_mut().unwrap()
    }

    /// Clears all allocations.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Returns the total number of allocated objects.
    pub fn len(&self) -> usize {
        self.chunks.iter().map(|c| c.len()).sum()
    }

    /// Returns true if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty() || self.chunks.iter().all(|c| c.is_empty())
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
