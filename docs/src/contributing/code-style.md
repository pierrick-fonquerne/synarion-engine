# Code Style

This document defines the coding standards for Synarion Engine.

## Quick Reference

| Aspect | Rule |
|--------|------|
| Line length | 100 characters max |
| Formatting | `cargo fmt` (rustfmt) |
| Linting | `clippy::pedantic` |
| Documentation | Required on public API |
| Unsafe | Forbidden by default |

## Compiler Settings

### Warnings

All warnings are treated as errors:

```rust
#![deny(warnings)]
```

### Documentation

Public API must be documented:

```rust
#![deny(missing_docs)]
```

### Unsafe Code

Unsafe is forbidden by default:

```rust
#![forbid(unsafe_code)]
```

For crates that require unsafe (e.g., `syn_vulkan`), this can be overridden:

```rust
#![allow(unsafe_code)]
```

## Formatting

We use `rustfmt` with custom settings. Run before committing:

```bash
cargo fmt --all
# or
make fmt
```

### Import Order

Imports are automatically organized by rustfmt into groups:

```rust
// 1. Standard library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates
use ash::vk;
use thiserror::Error;

// 3. Current crate
use crate::handle::Handle;

// 4. Parent/sibling modules
use super::Arena;
```

## Linting

We use Clippy with pedantic lints:

```bash
cargo clippy --workspace --all-targets -- \
    -D warnings \
    -D clippy::pedantic \
    -A clippy::module_name_repetitions \
    -A clippy::must_use_candidate
```

Or simply:

```bash
make lint
```

### Allowed Exceptions

- `module_name_repetitions`: Allows `syn_core::CoreError` instead of requiring `syn_core::Error`
- `must_use_candidate`: Not all functions need `#[must_use]`

## Naming Conventions

### Crates

All crates use the `syn_` prefix:

- `syn_core` - Foundation types
- `syn_math` - Mathematics
- `syn_vulkan` - Vulkan backend

### Types

Follow Rust conventions:

- **Structs**: `PascalCase` - `Handle<T>`, `Arena<T>`
- **Traits**: `PascalCase` - `Renderable`, `Serializable`
- **Functions**: `snake_case` - `get_handle()`, `create_instance()`
- **Constants**: `SCREAMING_SNAKE_CASE` - `MAX_FRAMES_IN_FLIGHT`
- **Modules**: `snake_case` - `handle.rs`, `arena.rs`

## Documentation

### Module Documentation

Every module should have a top-level doc comment:

```rust
//! Handle management for safe resource references.
//!
//! This module provides [`Handle<T>`] for type-safe resource identification
//! and [`Arena<T>`] for efficient storage.
```

### Item Documentation

Public items require documentation with examples:

```rust
/// A type-safe handle to a resource.
///
/// Handles use generational indices to detect stale references.
///
/// # Example
///
/// ```
/// use syn_core::{Arena, Handle};
///
/// let mut arena = Arena::new();
/// let handle = arena.insert(42);
/// assert_eq!(arena.get(handle), Some(&42));
/// ```
pub struct Handle<T> { ... }
```

### Safety Documentation

All `unsafe` blocks must have a `// SAFETY:` comment:

```rust
// SAFETY: The pointer is valid because...
unsafe {
    ptr.read()
}
```

## Error Handling

### Error Types

Use `thiserror` for all error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid handle")]
    InvalidHandle,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Error Propagation

Use `?` operator for error propagation:

```rust
fn load_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}
```

## Tests

### Unit Tests

Place unit tests in the same file, at the bottom:

```rust
// In src/handle.rs

pub struct Handle<T> { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_equality() {
        let h1 = Handle::new(1, 1);
        let h2 = Handle::new(1, 1);
        assert_eq!(h1, h2);
    }
}
```

### Integration Tests

Place integration tests in the `tests/` directory:

```
syn_core/
├── src/
│   └── lib.rs
└── tests/
    └── arena_integration.rs
```

### Running Tests

```bash
cargo test --workspace
# or
make test
```

## Dependencies

### Workspace Dependencies

Common dependencies are defined in the root `Cargo.toml`:

```toml
# Root Cargo.toml
[workspace.dependencies]
thiserror = "2.0"
ash = "0.38"
```

Use them in crates:

```toml
# syn_core/Cargo.toml
[dependencies]
thiserror.workspace = true
```

### Feature Flags

Use feature flags for optional functionality:

```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde"]
```

## Git Workflow

### Commit Messages

Use Conventional Commits:

```
feat(syn_core): add Handle<T> generational handles
fix(syn_vulkan): correct swapchain recreation
docs: update code style guide
refactor(syn_math): simplify quaternion operations
test(syn_core): add Arena edge case tests
```

### Before Committing

Always run the CI pipeline locally:

```bash
make ci
```

This runs: format check, clippy, tests, and documentation build.
