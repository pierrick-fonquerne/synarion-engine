//! Stack allocator for hierarchical allocations.

/// A stack allocator that supports nested allocation scopes.
pub struct StackAllocator {
    buffer: Vec<u8>,
    markers: Vec<usize>,
}

impl StackAllocator {
    /// Creates a new stack allocator.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            markers: Vec::new(),
        }
    }

    /// Pushes a new scope marker.
    pub fn push_scope(&mut self) {
        self.markers.push(self.buffer.len());
    }

    /// Pops the current scope, freeing all allocations since the last push.
    pub fn pop_scope(&mut self) {
        if let Some(marker) = self.markers.pop() {
            self.buffer.truncate(marker);
        }
    }
}
