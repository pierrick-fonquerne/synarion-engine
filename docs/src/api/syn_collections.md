# syn_collections

Specialized collections for Synarion Engine.

## Overview

`syn_collections` provides efficient data structures optimized for game engine use cases:
- [`SlotMap<T>`](#slotmapt) - Generational storage with O(1) operations
- [`Arena<T>`](#arenat) - Bump allocator for bulk allocations
- [`SparseSet<T>`](#sparsesett) - Cache-friendly sparse storage for ECS

## Types

### SlotMap\<T\>

A slot map that stores values and returns handles to them. Unlike a simple `Vec`, `SlotMap` allows O(1) removal and reuses slots efficiently.

```rust
use syn_collections::SlotMap;

let mut map = SlotMap::new();

let h1 = map.insert("first");
let h2 = map.insert("second");

assert_eq!(map.get(h1), Some(&"first"));

map.remove(h1);
assert_eq!(map.get(h1), None); // Handle invalidated

// Slot is reused
let h3 = map.insert("third");
assert_eq!(h3.index(), h1.index());
```

#### API

```rust
impl<T> SlotMap<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, value: T) -> Handle<T>;
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T>;
    pub fn get(&self, handle: Handle<T>) -> Option<&T>;
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T>;
    pub fn contains(&self, handle: Handle<T>) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

#### Performance

| Operation | Complexity |
|-----------|------------|
| `insert`  | O(1) amortized |
| `remove`  | O(1) |
| `get`     | O(1) |
| `contains`| O(1) |

---

### Arena\<T\>

A bump allocator that allocates objects in contiguous chunks. Ideal for allocating many objects with the same lifetime.

```rust
use syn_collections::Arena;

let mut arena = Arena::new();

// Allocate values
*arena.alloc(1) = 10;
arena.alloc(2);
arena.alloc(3);

assert_eq!(arena.len(), 3);

arena.clear(); // Deallocate all at once
assert!(arena.is_empty());
```

#### When to use Arena vs SlotMap

| Use Case | Best Choice |
|----------|-------------|
| Need to remove individual items | `SlotMap` |
| Allocate many items, clear all at once | `Arena` |
| Need handles for later access | `SlotMap` |
| Temporary allocations during frame | `Arena` |

#### API

```rust
impl<T> Arena<T> {
    pub fn new() -> Self;
    pub fn with_chunk_size(chunk_size: usize) -> Self;
    pub fn alloc(&mut self, value: T) -> &mut T;
    pub fn clear(&mut self);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

---

### SparseSet\<T\>

A sparse set providing O(1) insert, remove, and lookup with cache-friendly iteration. Commonly used in ECS for component storage.

```rust
use syn_collections::SparseSet;

let mut set = SparseSet::new();

set.insert(10, "entity 10");
set.insert(42, "entity 42");

assert_eq!(set.get(10), Some(&"entity 10"));
assert!(set.contains(42));

// Efficient iteration over dense storage
for (index, value) in set.iter() {
    println!("Entity {}: {}", index, value);
}
```

#### API

```rust
impl<T> SparseSet<T> {
    pub fn new() -> Self;
    pub fn insert(&mut self, index: u32, value: T);
    pub fn remove(&mut self, index: u32) -> Option<T>;
    pub fn get(&self, index: u32) -> Option<&T>;
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T>;
    pub fn contains(&self, index: u32) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)>;
}
```

## Re-exports

For convenience, `syn_collections` re-exports `Handle<T>` from `syn_core`:

```rust
use syn_collections::{SlotMap, Handle};
// Instead of:
// use syn_core::Handle;
// use syn_collections::SlotMap;
```

## Related Crates

- [`syn_core`](./syn_core.md) - Handle<T> type definition
- `syn_ecs` - Uses SparseSet for component storage
