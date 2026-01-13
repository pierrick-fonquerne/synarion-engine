# Commit Conventions

We use **Conventional Commits** for all commits. This enables automatic changelog generation and clear commit history.

## Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

## Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Code style (formatting, no logic change) |
| `refactor` | Refactoring (no feature/fix) |
| `perf` | Performance improvement |
| `test` | Adding/updating tests |
| `build` | Build system or dependencies |
| `ci` | CI configuration |
| `chore` | Other changes (tooling, etc.) |

## Scopes

Use the crate name as scope:

- `syn_core`
- `syn_math`
- `syn_vulkan`
- `editor`
- `tools`
- `docs`

## Examples

```
feat(syn_vulkan): add swapchain creation

Implements VkSwapchain creation with configurable present modes.

Closes #42
```

```
fix(syn_math): correct quaternion slerp edge case

When quaternions are nearly identical, slerp now returns
the first quaternion instead of NaN.
```

```
docs(readme): update installation instructions
```

```
refactor(syn_core): simplify Handle implementation

Reduced code duplication by extracting common logic
into a trait.
```

## Breaking Changes

For breaking changes, add `!` after the type:

```
feat(syn_ecs)!: redesign component storage

BREAKING CHANGE: Component trait now requires Send + Sync.
```

## Pre-commit Checks

Before committing:

1. `cargo fmt` - Format code
2. `cargo clippy` - Lint check
3. `cargo test` - Run tests
4. `cargo check` - Type check
