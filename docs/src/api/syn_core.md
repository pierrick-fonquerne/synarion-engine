# syn_core

Foundation types for Synarion Engine.

## Overview

`syn_core` provides the fundamental building blocks used throughout the engine. It focuses on minimal, dependency-free core types.

## Types

### Handle\<T\>

A type-safe generational handle for referencing resources.

```rust
use syn_core::Handle;

// Handles are typically created by collection types like SlotMap
let handle: Handle<String> = Handle::new(0, 1);

assert_eq!(handle.index(), 0);
assert_eq!(handle.generation(), 1);
```

#### Why Generational Handles?

Traditional approaches to resource management have drawbacks:

| Approach | Problem |
|----------|---------|
| `Rc<T>` / `Arc<T>` | Reference cycles, runtime overhead |
| Raw pointers | Dangling pointers, unsafe |
| Simple indices | ABA problem after removal |

Generational handles solve these issues by combining:
- **Index**: Position in the storage
- **Generation**: Counter incremented on slot reuse

When a resource is removed and the slot is reused, the generation changes. Old handles become invalid and return `None` on access.

#### API

```rust
impl<T> Handle<T> {
    /// Creates a handle from raw parts.
    pub const fn new(index: u32, generation: u32) -> Self;

    /// Returns the index component.
    pub const fn index(&self) -> u32;

    /// Returns the generation component.
    pub const fn generation(&self) -> u32;
}
```

#### Traits

`Handle<T>` implements:
- `Copy`, `Clone`
- `PartialEq`, `Eq`
- `Hash`
- `Debug`

## Design Decisions

### Why u32 for indices?

- 4 billion entries is sufficient for any realistic use case
- Smaller memory footprint than `usize`
- Handle fits in 64 bits (index + generation)

### Why `#![forbid(unsafe_code)]`?

`syn_core` is designed to be 100% safe Rust. The engine's safety guarantees start here.

### Why no collections?

`syn_core` is intentionally minimal. Collections that use `Handle<T>` are in `syn_collections` to avoid circular dependencies and keep the core crate lightweight.

## Related Crates

- [`syn_collections`](./syn_collections.md) - SlotMap, Arena, SparseSet using Handle<T>
