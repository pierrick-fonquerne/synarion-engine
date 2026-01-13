# Synarion Engine

A professional open-source 3D game engine written in Rust with Vulkan.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org/)

## Status

**Early Development** - Building foundation crates.

See [CHANGELOG.md](CHANGELOG.md) for recent changes.

## Goals

- **Performance**: Native Rust + Vulkan
- **Modularity**: ~55 independent crates
- **Professional Editor**: Modern UX/DX for game teams
- **Open Source**: MIT licensed

## Architecture

```
synarion-engine/
├── engine/        # 37 runtime crates
│   └── crates/
│       ├── syn_core/        # Foundational types
│       ├── syn_math/        # 3D mathematics
│       ├── syn_vulkan/      # Vulkan backend
│       ├── syn_renderer/    # High-level rendering
│       ├── syn_ecs/         # Entity Component System
│       └── ...
├── editor/        # 20 editor crates
│   └── crates/
│       ├── syn_viewport/    # 3D viewport
│       ├── syn_inspector/   # Property inspector
│       └── ...
├── tools/         # 11 tool crates
│   └── crates/
│       ├── syn_cli/         # Command-line interface
│       └── ...
├── examples/      # 8 demo projects
└── docs/          # Documentation (mdBook)
```

## Requirements

- Rust 1.85+
- Vulkan SDK
- (Windows/Linux/macOS)

## Building

```bash
# Clone
git clone https://github.com/pierrick-fonquerne/synarion-engine.git
cd synarion-engine

# Build
cargo build

# Run example (when implemented)
cargo run --example hello_triangle
```

## Documentation

```bash
# Install mdBook
cargo install mdbook

# Serve documentation locally
cd docs && mdbook serve
```

## Contributing

We welcome contributions! Please read:

- [How to Contribute](docs/src/contributing/how-to-contribute.md)
- [Commit Conventions](docs/src/contributing/commit-conventions.md)

### Commit Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(syn_vulkan): add swapchain creation
fix(syn_math): correct quaternion slerp
docs: update installation guide
```

## Roadmap

| Milestone | Status |
|-----------|--------|
| M0: Bootstrap | Complete |
| M1: Hello Triangle | Not Started |
| M2: Textured Cube | Not Started |
| M3: Lighting & PBR | Not Started |
| M4: ECS & Scene | Not Started |
| M5: Model Loading | Not Started |

See [full roadmap](docs/src/roadmap/milestones.md).

## License

MIT License - See [LICENSE](LICENSE) for details.

Copyright (c) 2025 Pierrick Fonquerne
