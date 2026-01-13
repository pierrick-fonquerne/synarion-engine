//! Pool allocator for fixed-size allocations.

/// A pool allocator for fixed-size objects.
pub struct PoolAllocator {
    block_size: usize,
    blocks: Vec<Vec<u8>>,
}

impl PoolAllocator {
    /// Creates a new pool allocator.
    pub fn new(block_size: usize) -> Self {
        Self {
            block_size,
            blocks: Vec::new(),
        }
    }

    /// Returns the block size.
    pub fn block_size(&self) -> usize {
        self.block_size
    }
}
