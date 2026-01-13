//! A sparse set implementation for efficient component storage.

/// A sparse set providing O(1) insert, remove, and lookup.
pub struct SparseSet<T> {
    sparse: Vec<Option<usize>>,
    dense: Vec<(u32, T)>,
}

impl<T> SparseSet<T> {
    /// Creates a new empty sparse set.
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
        }
    }

    /// Inserts a value at the given index.
    pub fn insert(&mut self, index: u32, value: T) {
        let idx = index as usize;
        if idx >= self.sparse.len() {
            self.sparse.resize(idx + 1, None);
        }
        if let Some(dense_idx) = self.sparse[idx] {
            self.dense[dense_idx].1 = value;
        } else {
            self.sparse[idx] = Some(self.dense.len());
            self.dense.push((index, value));
        }
    }

    /// Removes the value at the given index.
    pub fn remove(&mut self, index: u32) -> Option<T> {
        let idx = index as usize;
        if idx >= self.sparse.len() {
            return None;
        }
        if let Some(dense_idx) = self.sparse[idx].take() {
            let removed = self.dense.swap_remove(dense_idx);
            if dense_idx < self.dense.len() {
                self.sparse[self.dense[dense_idx].0 as usize] = Some(dense_idx);
            }
            Some(removed.1)
        } else {
            None
        }
    }

    /// Gets a reference to the value at the given index.
    pub fn get(&self, index: u32) -> Option<&T> {
        let idx = index as usize;
        self.sparse.get(idx)?.map(|dense_idx| &self.dense[dense_idx].1)
    }

    /// Gets a mutable reference to the value at the given index.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        let idx = index as usize;
        let dense_idx = (*self.sparse.get(idx)?)?;
        Some(&mut self.dense[dense_idx].1)
    }

    /// Returns true if the set contains the given index.
    pub fn contains(&self, index: u32) -> bool {
        let idx = index as usize;
        idx < self.sparse.len() && self.sparse[idx].is_some()
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Iterates over all values.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.dense.iter().map(|(idx, val)| (*idx, val))
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
