# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **syn_core**: `Handle<T>` - Type-safe generational handles for resource references
- **syn_collections**: `SlotMap<T>` - Generational storage with O(1) insert/remove/get operations
- **syn_collections**: `Arena<T>` - Bump allocator for bulk allocations
- **syn_collections**: Re-exports `Handle<T>` from syn_core for convenience
- Coding standards: rustfmt.toml, clippy pedantic, CI pipeline
- Makefile with development commands (`make ci`, `make lint`, `make test`)
- GitHub Actions CI (format, clippy, tests, docs)
- Code style documentation (`docs/src/contributing/code-style.md`)
- API documentation for `syn_core` and `syn_collections`

### Changed

- `rustfmt.toml` simplified to use only stable options
- `Cargo.lock` now versioned for build reproducibility

### Infrastructure

- Initial project bootstrap with 55 crates
- Engine crates: core, math, platform, vulkan, renderer, ecs, physics, audio, etc.
- Editor crates: viewport, inspector, asset browser, material editor, etc.
- Tools crates: asset compiler, importers, CLI
- 8 example projects
- Documentation structure with mdBook
- Conventional commits guidelines

## [0.0.1] - 2025-01-13

### Added

- Project structure and Cargo workspace
- Empty crate scaffolding
- Basic README and LICENSE (MIT)

[Unreleased]: https://github.com/pierrick-fonquerne/synarion-engine/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/pierrick-fonquerne/synarion-engine/releases/tag/v0.0.1
