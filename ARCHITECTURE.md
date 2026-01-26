# Synarion Engine - Architecture Document

**Version**: 0.1.0
**Date**: 2026-01-26
**Status**: Draft
**Author**: Pierrick Fonquerne - Synarion Entertainments SAS

---

## Overview

Synarion Engine uses a **modular crate ecosystem** architecture. Instead of a monolithic engine, it provides a collection of independent crates that can be composed according to project needs.

### Core Principles

1. **Use only what you need** - Unused features don't compile
2. **Swappable backends** - Trait-based APIs allow multiple implementations
3. **Independent testing** - Each crate is tested in isolation
4. **Zero-cost abstractions** - Rust's trait system provides abstraction without runtime overhead
5. **Clear dependencies** - Each tier only depends on lower tiers

---

## Tier System

### Tier 0: Foundation

Standalone libraries that are useful even outside the engine.

| Crate | Description | Dependencies |
|-------|-------------|--------------|
| `syn_core` | Handles, IDs, pools, basic types | None |
| `syn_math` | Vectors, matrices, quaternions, transforms | `glam` |
| `syn_collections` | SlotMap, Arena, SparseSet | `syn_core` |

**Key Feature**: These crates can be published independently on crates.io.

```rust
// syn_core - Generational handles
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _marker: PhantomData<T>,
}

// syn_collections - O(1) insert/remove/get
pub struct SlotMap<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<u32>,
}
```

---

### Tier 1: Platform

Operating system abstraction layer.

| Crate | Description | Dependencies |
|-------|-------------|--------------|
| `syn_platform` | Window creation, events | `winit` |
| `syn_input` | Keyboard, mouse, gamepad input | `syn_platform` |
| `syn_filesystem` | Virtual file system | `syn_core` |
| `syn_tasks` | Async job system | `tokio` |

```rust
// syn_platform - Window abstraction
pub trait Window {
    fn size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

// syn_input - Input state
pub struct InputState {
    pub keyboard: KeyboardState,
    pub mouse: MouseState,
    pub gamepads: Vec<GamepadState>,
}
```

---

### Tier 2: Graphics

Rendering system with swappable backends.

| Crate | Description | Dependencies |
|-------|-------------|--------------|
| `syn_gpu` | RHI traits (Render Hardware Interface) | Tier 0-1 |
| `syn_vulkan` | Vulkan backend | `ash`, `syn_gpu` |
| `syn_dx12` | DirectX 12 backend (future) | `syn_gpu` |
| `syn_metal` | Metal backend (future) | `syn_gpu` |
| `syn_webgpu` | WebGPU backend (future) | `syn_gpu` |
| `syn_shaders` | Shader compilation (SPIR-V) | `shaderc` |
| `syn_renderer` | High-level rendering | `syn_gpu` |

```rust
// syn_gpu - Backend-agnostic traits
pub trait Device {
    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle>;
    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle>;
}

pub trait CommandBuffer {
    fn begin(&mut self);
    fn end(&mut self);
    fn bind_pipeline(&mut self, pipeline: PipelineHandle);
    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>);
}

// syn_vulkan - Implements these traits using ash
pub struct VulkanDevice {
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    // ...
}

impl Device for VulkanDevice {
    // Vulkan-specific implementation
}
```

---

### Tier 3: Game Framework

Minimum viable game engine.

| Crate | Description | Dependencies |
|-------|-------------|--------------|
| `syn_app` | Application loop, lifecycle | Tier 0-2 |
| `syn_ecs` | Entity Component System | `syn_core`, `syn_collections` |
| `syn_scene` | Scene graph, transforms | `syn_ecs`, `syn_math` |
| `syn_assets` | Asset loading, hot reload | `syn_filesystem` |

```rust
// syn_ecs - Archetypal ECS
pub struct World {
    entities: EntityStorage,
    archetypes: Vec<Archetype>,
    components: ComponentStorage,
}

pub trait System {
    fn run(&mut self, world: &mut World);
}

// syn_app - Application lifecycle
pub trait App {
    fn init(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context, dt: f32);
    fn render(&mut self, ctx: &mut Context);
    fn shutdown(&mut self, ctx: &mut Context);
}
```

---

### Tier 4: Game Systems

Each system is completely independent and optional.

#### Physics

| Crate | Description |
|-------|-------------|
| `syn_physics_api` | Physics traits and types |
| `syn_physics_rapier` | Rapier3D backend |
| `syn_physics_physx` | PhysX backend (future) |

#### Audio

| Crate | Description |
|-------|-------------|
| `syn_audio_api` | Audio traits and types |
| `syn_audio_rodio` | Rodio backend |
| `syn_audio_fmod` | FMOD backend (future) |

#### UI

| Crate | Description |
|-------|-------------|
| `syn_ui_api` | UI traits and types |
| `syn_ui_egui` | egui integration |
| `syn_ui_custom` | Custom immediate mode UI |

#### Animation

| Crate | Description |
|-------|-------------|
| `syn_animation` | Skeletal animation |
| `syn_animation_ik` | Inverse kinematics |

#### AI

| Crate | Description |
|-------|-------------|
| `syn_ai_bt` | Behavior trees |
| `syn_ai_fsm` | Finite state machines |
| `syn_ai_goap` | Goal-oriented action planning |
| `syn_ai_pathfinding` | NavMesh, A* |

#### Networking

| Crate | Description |
|-------|-------------|
| `syn_net_core` | Serialization, protocol |
| `syn_net_client` | Client-side networking |
| `syn_net_server` | Server-side networking |
| `syn_net_p2p` | Peer-to-peer (optional) |

---

### Tier 5: Specialized Systems

Advanced systems for specific use cases.

#### Terrain

| Crate | Description | Use Case |
|-------|-------------|----------|
| `syn_terrain_heightmap` | Classic heightmap terrain | RTS, open world |
| `syn_terrain_sdf` | SDF terrain (read-only) | Caves, overhangs |
| `syn_terrain_sdf_edit` | Editable SDF terrain | Minecraft-like |
| `syn_terrain_planet` | Planetary cube-sphere | Space games |
| `syn_terrain_voxel` | Minecraft-style voxels | Block games |

#### Procedural Generation

| Crate | Description |
|-------|-------------|
| `syn_procgen_noise` | Noise functions (Perlin, Simplex, Voronoi) |
| `syn_procgen_terrain` | Terrain generation algorithms |
| `syn_procgen_biome` | Biome distribution |
| `syn_procgen_city` | City/dungeon generation |
| `syn_procgen_creature` | Creature generation |

#### Simulation

| Crate | Description |
|-------|-------------|
| `syn_sim_orbital` | Orbital mechanics |
| `syn_sim_climate` | Climate simulation |
| `syn_sim_hydrology` | Rivers, water cycle |
| `syn_sim_ecosystem` | Flora/fauna simulation |
| `syn_sim_economy` | Economic simulation |

#### Persistence

| Crate | Description | Use Case |
|-------|-------------|----------|
| `syn_save_local` | Local save files | Single player |
| `syn_save_cloud` | Cloud saves | Cross-device |
| `syn_persist_server` | Server persistence | Multiplayer |
| `syn_persist_distributed` | Distributed storage | MMO |

#### Delta System (Destructible Terrain Sync)

| Crate | Description | Use Case |
|-------|-------------|----------|
| `syn_delta_local` | Local delta storage | Single player destructible |
| `syn_delta_server` | Server delta | Small multiplayer |
| `syn_delta_distributed` | Distributed delta | MMO with destructible terrain |

---

## Dependency Graph

```
TIER 5: SPECIALIZED
    │
    ├── syn_terrain_* ────────────────────────────────┐
    ├── syn_procgen_* ────────────────────────────────┤
    ├── syn_sim_* ────────────────────────────────────┤
    ├── syn_persist_* ────────────────────────────────┤
    └── syn_delta_* ──────────────────────────────────┤
                                                      │
TIER 4: GAME SYSTEMS                                  │
    │                                                 │
    ├── syn_physics_* ────────────────────────────────┤
    ├── syn_audio_* ──────────────────────────────────┤
    ├── syn_ui_* ─────────────────────────────────────┤
    ├── syn_animation_* ──────────────────────────────┤
    ├── syn_ai_* ─────────────────────────────────────┤
    └── syn_net_* ────────────────────────────────────┤
                                                      │
TIER 3: GAME FRAMEWORK ◄──────────────────────────────┤
    │                                                 │
    ├── syn_app ──────────────────────────────────────┤
    ├── syn_ecs ──────────────────────────────────────┤
    ├── syn_scene ────────────────────────────────────┤
    └── syn_assets ───────────────────────────────────┤
                                                      │
TIER 2: GRAPHICS ◄────────────────────────────────────┤
    │                                                 │
    ├── syn_gpu ──────────────────────────────────────┤
    ├── syn_vulkan ───────────────────────────────────┤
    ├── syn_shaders ──────────────────────────────────┤
    └── syn_renderer ─────────────────────────────────┤
                                                      │
TIER 1: PLATFORM ◄────────────────────────────────────┤
    │                                                 │
    ├── syn_platform ─────────────────────────────────┤
    ├── syn_input ────────────────────────────────────┤
    ├── syn_filesystem ───────────────────────────────┤
    └── syn_tasks ────────────────────────────────────┤
                                                      │
TIER 0: FOUNDATION ◄──────────────────────────────────┘
    │
    ├── syn_core
    ├── syn_math
    └── syn_collections
```

---

## Feature Presets

The meta-crate `synarion_engine` provides convenience presets:

```toml
[features]
default = ["minimal"]

# Minimum viable engine
minimal = [
    "syn_core",
    "syn_math",
    "syn_collections",
    "syn_platform",
    "syn_input",
    "syn_gpu",
    "syn_vulkan",
    "syn_renderer",
    "syn_app",
    "syn_ecs",
    "syn_scene",
    "syn_assets",
]

# Everything
full = [
    "minimal",
    "physics",
    "audio",
    "ui",
    "animation",
    "ai",
    "networking",
    "terrain",
    "procgen",
]

# Physics options
physics = ["syn_physics_rapier"]
physics-physx = ["syn_physics_physx"]

# Audio options
audio = ["syn_audio_rodio"]
audio-fmod = ["syn_audio_fmod"]

# Terrain options
terrain-heightmap = ["syn_terrain_heightmap"]
terrain-sdf = ["syn_terrain_sdf"]
terrain-sdf-edit = ["syn_terrain_sdf_edit"]
terrain-planet = ["syn_terrain_planet"]
terrain-voxel = ["syn_terrain_voxel"]

# Networking options
networking = ["syn_net_client", "syn_net_server"]
networking-p2p = ["syn_net_p2p"]

# Simulation (from VISION.md)
simulation = [
    "syn_sim_orbital",
    "syn_sim_climate",
    "syn_sim_hydrology",
    "syn_sim_ecosystem",
]

# Persistence options
save-local = ["syn_save_local"]
persist-server = ["syn_persist_server"]
persist-distributed = ["syn_persist_distributed"]

# Delta system (requires terrain-sdf-edit)
delta-local = ["terrain-sdf-edit", "syn_delta_local"]
delta-server = ["terrain-sdf-edit", "syn_delta_server"]
delta-distributed = ["terrain-sdf-edit", "syn_delta_distributed"]

# Game-specific presets
groundbreak = [
    "full",
    "terrain-sdf-edit",
    "terrain-planet",
    "simulation",
    "persist-distributed",
    "delta-distributed",
    "syn_sim_economy",
]
```

---

## Delta System Architecture

For games with destructible terrain in multiplayer (like Groundbreak), the delta system provides:

### Event Sourcing

All terrain modifications are stored as immutable events:

```rust
pub enum TerrainEvent {
    Dig { position: Vec3, radius: f32, tool: ToolType },
    Fill { position: Vec3, material: MaterialId, shape: Shape },
    Explosion { position: Vec3, yield_joules: f64 },
}
```

### Snapshot System

Periodic snapshots prevent replaying entire history:

```rust
pub struct ChunkSnapshot {
    pub chunk_coord: IVec3,
    pub timestamp: u64,
    pub last_event_id: EventId,
    pub svo_data: CompressedSVO,
}
```

### Storage Layers

```
┌─────────────────────────────────────────────────────┐
│  L1: GPU VRAM (per-player, ~100 chunks)             │
├─────────────────────────────────────────────────────┤
│  L2: Client RAM (per-player, ~1000 chunks)          │
├─────────────────────────────────────────────────────┤
│  L3: Server RAM / Redis (shared, hot chunks)        │
├─────────────────────────────────────────────────────┤
│  L4: Snapshots (Object Storage, all chunks)         │
└─────────────────────────────────────────────────────┘
```

See [docs/research/delta-storage.md](./docs/research/delta-storage.md) for full technical details.

---

## Coding Standards

All crates follow the same standards:

```rust
#![deny(warnings)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]  // Opt-in per crate for FFI
```

- **Clippy**: Pedantic level
- **Format**: rustfmt with 100 char lines
- **Tests**: Inline unit tests + integration tests
- **Docs**: All public items documented

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

---

## Build System

### Workspace Structure

```
synarion-engine/
├── Cargo.toml              # Workspace root
├── engine/
│   └── crates/
│       ├── syn_core/
│       ├── syn_math/
│       └── ...
├── editor/
│   └── crates/
│       └── ...
├── tools/
│   └── crates/
│       └── ...
└── examples/
    └── ...
```

### Makefile Commands

```bash
make check      # Quick compilation check
make lint       # Clippy pedantic
make test       # Run all tests
make fmt        # Format code
make ci         # Full CI pipeline
make doc-open   # Generate and open documentation
```

---

## Future Considerations

### WebAssembly Support

Tier 0-3 should eventually compile to WASM for web deployment:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
syn_webgpu = "0.1"  # Instead of syn_vulkan
```

### Mobile Support

iOS/Android support via:
- `syn_metal` for iOS
- `syn_vulkan` for Android

### Console Support

PS5/Xbox/Switch would require:
- Platform-specific crates
- Certification compliance
- NDA-protected code (separate repos)

---

*"Simplicity is the ultimate sophistication."* — Leonardo da Vinci
