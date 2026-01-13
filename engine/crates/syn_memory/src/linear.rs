//! Linear allocator for temporary allocations.

/// A linear allocator that allocates memory sequentially.
pub struct LinearAllocator {
    buffer: Vec<u8>,
    offset: usize,
}

impl LinearAllocator {
    /// Creates a new linear allocator with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            offset: 0,
        }
    }

    /// Resets the allocator, freeing all allocations.
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// Returns the remaining capacity.
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.offset
    }
}

impl Default for LinearAllocator {
    fn default() -> Self {
        Self::new(1024 * 1024) // 1MB default
    }
}
