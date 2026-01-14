//! A slot map implementation with generational indices.
//!
//! [`SlotMap<T>`] provides O(1) insert, remove, and access operations while
//! safely detecting stale handles through generational indices.

use syn_core::Handle;

/// Entry state in the slot map.
enum Entry<T> {
    /// Slot contains a value.
    Occupied { value: T, generation: u32 },
    /// Slot is empty and links to the next free slot.
    Vacant {
        next_free: Option<u32>,
        generation: u32,
    },
}

/// A slot map that stores values and returns handles to them.
///
/// Unlike a simple `Vec`, `SlotMap` allows O(1) removal and reuses slots
/// efficiently. Each slot has a generation counter that increments on removal,
/// invalidating any existing handles to that slot.
///
/// # Example
///
/// ```
/// use syn_collections::SlotMap;
///
/// let mut map = SlotMap::new();
///
/// let h1 = map.insert("first");
/// let h2 = map.insert("second");
///
/// assert_eq!(map.get(h1), Some(&"first"));
///
/// map.remove(h1);
/// assert_eq!(map.get(h1), None); // Handle invalidated
///
/// // Slot is reused
/// let h3 = map.insert("third");
/// assert_eq!(h3.index(), h1.index());
/// ```
pub struct SlotMap<T> {
    entries: Vec<Entry<T>>,
    free_head: Option<u32>,
    len: usize,
}

impl<T> SlotMap<T> {
    /// Creates a new empty slot map.
    #[inline]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    /// Inserts a value into the slot map, returning a handle to it.
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(free_index) = self.free_head {
            // Reuse a vacant slot
            let entry = &mut self.entries[free_index as usize];

            let generation = match entry {
                Entry::Vacant {
                    next_free,
                    generation,
                } => {
                    self.free_head = *next_free;
                    *generation
                }
                Entry::Occupied { .. } => unreachable!("free_head pointed to occupied slot"),
            };

            *entry = Entry::Occupied { value, generation };
            self.len += 1;

            Handle::new(free_index, generation)
        } else {
            // Allocate a new slot
            // RATIONALE: SlotMap will never have more than u32::MAX entries
            #[allow(clippy::cast_possible_truncation)]
            let index = self.entries.len() as u32;
            let generation = 0;

            self.entries.push(Entry::Occupied { value, generation });
            self.len += 1;

            Handle::new(index, generation)
        }
    }

    /// Removes the value associated with the handle, returning it if valid.
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let index = handle.index() as usize;

        if index >= self.entries.len() {
            return None;
        }

        let entry = &mut self.entries[index];

        match entry {
            Entry::Occupied { generation, .. } if *generation == handle.generation() => {
                // Generation matches, remove the value
                let new_generation = generation.wrapping_add(1);

                let old_entry = std::mem::replace(
                    entry,
                    Entry::Vacant {
                        next_free: self.free_head,
                        generation: new_generation,
                    },
                );

                self.free_head = Some(handle.index());
                self.len -= 1;

                match old_entry {
                    Entry::Occupied { value, .. } => Some(value),
                    Entry::Vacant { .. } => unreachable!(),
                }
            }
            _ => None, // Wrong generation or vacant
        }
    }

    /// Returns a reference to the value if the handle is valid.
    #[inline]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let index = handle.index() as usize;

        self.entries.get(index).and_then(|entry| match entry {
            Entry::Occupied { value, generation } if *generation == handle.generation() => {
                Some(value)
            }
            _ => None,
        })
    }

    /// Returns a mutable reference to the value if the handle is valid.
    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let index = handle.index() as usize;

        self.entries.get_mut(index).and_then(|entry| match entry {
            Entry::Occupied { value, generation } if *generation == handle.generation() => {
                Some(value)
            }
            _ => None,
        })
    }

    /// Returns the number of values in the slot map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the slot map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if the handle is valid.
    #[inline]
    pub fn contains(&self, handle: Handle<T>) -> bool {
        let index = handle.index() as usize;

        self.entries.get(index).is_some_and(|entry| {
            matches!(entry, Entry::Occupied { generation, .. } if *generation == handle.generation())
        })
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut map = SlotMap::new();
        let handle = map.insert(42);

        assert_eq!(map.get(handle), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_invalidates_handle() {
        let mut map = SlotMap::new();
        let handle = map.insert("test");

        assert_eq!(map.remove(handle), Some("test"));
        assert_eq!(map.get(handle), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn slot_reuse_with_new_generation() {
        let mut map = SlotMap::new();

        let h1 = map.insert("first");
        map.remove(h1);

        let h2 = map.insert("second");

        // Same index, different generation
        assert_eq!(h1.index(), h2.index());
        assert_ne!(h1.generation(), h2.generation());

        // Old handle is invalid
        assert_eq!(map.get(h1), None);
        assert_eq!(map.get(h2), Some(&"second"));
    }

    #[test]
    fn get_mut_works() {
        let mut map = SlotMap::new();
        let handle = map.insert(10);

        if let Some(value) = map.get_mut(handle) {
            *value = 20;
        }

        assert_eq!(map.get(handle), Some(&20));
    }

    #[test]
    fn contains_works() {
        let mut map = SlotMap::new();
        let h1 = map.insert(1);

        assert!(map.contains(h1));
        map.remove(h1);
        assert!(!map.contains(h1));
    }

    #[test]
    fn multiple_inserts_and_removes() {
        let mut map = SlotMap::new();

        let h1 = map.insert(1);
        let h2 = map.insert(2);
        let h3 = map.insert(3);

        assert_eq!(map.len(), 3);

        map.remove(h2);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(h2), None);
        assert_eq!(map.get(h1), Some(&1));
        assert_eq!(map.get(h3), Some(&3));

        // Reuse slot
        let h4 = map.insert(4);
        assert_eq!(h4.index(), h2.index());
        assert_eq!(map.len(), 3);
    }
}
