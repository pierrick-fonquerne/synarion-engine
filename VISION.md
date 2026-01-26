# Synarion Engine - Vision Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Draft - Research & Development
**Author**: Pierrick Fonquerne - Synarion Entertainments SAS

---

## Executive Summary

Synarion Engine is a **high-performance 3D game engine** built in Rust with Vulkan, designed specifically for **vast procedurally-generated worlds with seamless transitions**.

The engine introduces a unique architectural concept:

> **Multi-Layered Mix-Seeded-Constructed Fluid Transitional Worlds**

This means the ability to create interconnected worlds that can be:
- **Multi-Layered**: Galaxies, systems, planets, dimensions, caves - all coexisting
- **Mix-Seeded-Constructed**: Procedural generation mixed with handcrafted content
- **Fluid Transitional**: Seamless transitions between any layer without loading screens

---

## Core Philosophy

### The Problem We Solve

| Existing Engine | Limitation |
|-----------------|------------|
| Unity/Unreal | Discrete scenes, transitions = loading screens |
| No Man's Sky (custom) | 100% procedural, limited handcrafted content |
| Star Citizen (custom) | Hybrid but very game-specific, not reusable |
| Typical voxel engines | Either small scale or poor visual quality |

**Synarion Engine** provides a **generic, reusable architecture** that allows any combination:
- A No Man's Sky-like universe with multiple galaxies
- A WoW-like fantasy world with dimension portals
- A single detailed planet with caves and underground
- Or any combination of the above

### Design Principles

1. **Simulation over Randomness**: Climate, biomes, ecosystems are *calculated* from physics, not randomly assigned
2. **GPU-Driven Generation**: The CPU orchestrates, the GPU generates and renders
3. **Deterministic Procedural**: Same seed = same result, always reconstructible
4. **Edit Anywhere**: Procedural base + handcrafted overlays + player modifications
5. **Tools-First**: Powerful editor for world creation, not just runtime

---

## Architecture Overview

### The Three Layers

```
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 3: CREATION TOOLS                          │
│                                                                      │
│  World Editor │ Terrain Sculptor │ Ecosystem Placer │ Debug Views  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                    LAYER 2: SIMULATION                              │
│                                                                      │
│  Orbital Mechanics │ Climate Sim │ Ecosystem Sim │ NPC/AI Systems  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                    LAYER 1: GENERATION & RUNTIME                    │
│                                                                      │
│  World Graph │ Terrain SDF │ GPU Meshing │ Streaming │ Rendering   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Layer 1: Generation & Runtime

### 1.1 World Graph System

The world is represented as a **graph of interconnected nodes**, not a fixed hierarchy.

```
                    ┌──────────┐
                    │ GALAXY A │ (Seeded, Infinite)
                    └────┬─────┘
                         │ seamless
              ┌──────────┼──────────┐
              ▼          ▼          ▼
         ┌────────┐ ┌────────┐ ┌────────┐
         │SYSTEM 1│ │SYSTEM 2│ │SYSTEM 3│ (Seeded)
         └───┬────┘ └────────┘ └────────┘
             │ seamless
    ┌────────┼────────┐
    ▼        ▼        ▼
┌────────┐┌────────┐┌────────┐
│PLANET A││PLANET B││ MOON   │ (Hybrid: Seeded + Overlays)
└───┬────┘└────────┘└────────┘
    │ seamless            ▲
    ▼                     │ portal
┌────────┐          ┌─────┴────┐
│ CAVES  │          │DIMENSION │ (Seeded, different rules)
└────────┘          └──────────┘
```

#### World Node Types

```rust
pub enum GenerationStrategy {
    /// 100% procedural from seed
    Seeded {
        seed: u64,
        generator: GeneratorId,
    },

    /// 100% handcrafted (traditional level)
    Constructed {
        scene_asset: AssetId,
    },

    /// Procedural base + handcrafted overlays + player modifications
    Hybrid {
        seed: u64,
        generator: GeneratorId,
        overlays: Vec<OverlayLayer>,
        deltas: DeltaStorage,
    },
}
```

#### Transition Types

```rust
pub enum TransitionType {
    /// No visible transition (space → planet approach)
    Seamless { blend_distance: f32 },

    /// Explicit passage point (WoW-style portal)
    Portal { visual_effect: EffectId },

    /// Visual fade (cinematic transition)
    Fade { duration: f32, fade_type: FadeType },

    /// Triggered by condition (altitude, area, event)
    Conditional { condition: TransitionCondition },
}
```

### 1.2 Terrain System (SDF + Dual Contouring)

#### Signed Distance Field (SDF)

Terrain is stored as a **density field**, not heightmaps:
- Enables caves, overhangs, arches
- Natural CSG operations (dig, fill)
- Smooth LOD transitions

#### GPU-Driven Pipeline

```
CPU                          GPU
────                         ───
1. Determine visible         2. Generate density (compute)
   chunks (frustum+LOD)         ↓
        ↓                    3. Polygonize - Marching Cubes (compute)
   Request list                 ↓
        ↓                    4. Scatter instances (compute)
   Priority queue               ↓
                             5. Render (DrawIndirect)
```

#### Planetary Projection (Cube-Sphere)

For spherical planets:
- 6 faces (cube projected onto sphere)
- Quadtree LOD per face
- No polar singularities
- Seamless face transitions

### 1.3 Multi-Scale LOD

The LOD system understands **scale and context**:

| Distance | Representation |
|----------|----------------|
| 10,000 km | Planet as impostor/billboard |
| 1,000 km | Low-res sphere with atmosphere |
| 100 km | Quadtree tiles, coarse detail |
| 10 km | Medium detail, major features |
| 1 km | High detail, vegetation visible |
| 100 m | Full detail, individual objects |
| 1 m | Maximum detail, ground textures |

### 1.4 Streaming System

Predictive streaming based on:
- Current position and velocity
- Look direction
- Nearby transition points
- Historical player behavior

---

## Layer 2: Simulation

The simulation layer is the heart of Synarion Engine's "physics-based world" philosophy.
Instead of randomly placing biomes and features, we **calculate** them from first principles.

### Simulation Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         SIMULATION PIPELINE                                      │
│                                                                                  │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────┐        │
│  │ ORBITAL  │───▶│  TECTONICS   │───▶│  HYDROLOGY   │───▶│ ATMOSPHERE  │        │
│  │ PARAMS   │    │  (optional)  │    │              │    │             │        │
│  └──────────┘    └──────────────┘    └──────────────┘    └─────────────┘        │
│       │                │                    │                   │               │
│       │                ▼                    ▼                   ▼               │
│       │         ┌──────────────┐    ┌──────────────┐    ┌─────────────┐        │
│       └────────▶│   CLIMATE    │◀───│   TERRAIN    │◀───│   BIOMES    │        │
│                 │  SIMULATION  │    │  ELEVATION   │    │ RESOLUTION  │        │
│                 └──────────────┘    └──────────────┘    └─────────────┘        │
│                        │                                       │               │
│                        └───────────────────────────────────────┘               │
│                                         │                                       │
│                                         ▼                                       │
│                               ┌─────────────────┐                              │
│                               │   ECOSYSTEM     │                              │
│                               │  Flora + Fauna  │                              │
│                               └─────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Orbital Mechanics

Planets have **real orbital parameters** that drive all downstream simulation:

```rust
pub struct StarParameters {
    pub luminosity: f64,         // Solar luminosities (L☉)
    pub temperature: f64,        // Surface temperature (K)
    pub spectral_class: SpectralClass,
}

pub struct OrbitalParameters {
    pub semi_major_axis: f64,    // Distance to star (AU)
    pub eccentricity: f64,       // Orbit shape (0=circle, 1=parabola)
    pub inclination: f64,        // Orbital plane tilt (rad)
    pub axial_tilt: f64,         // Planet rotation axis tilt (rad)
    pub rotation_period: f64,    // Sidereal day (seconds)
    pub orbital_period: f64,     // Year length (seconds)
}

pub struct PlanetParameters {
    pub radius: f64,             // Planet radius (m)
    pub mass: f64,               // Planet mass (kg)
    pub orbital: OrbitalParameters,
    pub atmosphere: Option<AtmosphereComposition>,
}
```

#### Calculated Effects

| Parameter | Effect |
|-----------|--------|
| `semi_major_axis` | Base temperature (inverse square law) |
| `eccentricity` | Seasonal temperature variation |
| `axial_tilt` | Latitude-based seasons, polar regions |
| `rotation_period` | Day/night cycle, Coriolis effect strength |
| `luminosity` | Habitable zone boundaries |

### 2.2 Tectonic Simulation (Optional)

For geologically active worlds, simulate plate tectonics:

```rust
pub struct TectonicPlate {
    pub id: PlateId,
    pub boundary: Polygon,       // Plate outline
    pub velocity: Vec2,          // Movement direction & speed
    pub density: f32,            // Oceanic vs Continental
    pub age: f64,                // Geological age
}

pub enum PlateBoundary {
    Divergent,    // Plates moving apart → rifts, mid-ocean ridges
    Convergent,   // Plates colliding → mountains, trenches
    Transform,    // Plates sliding → fault lines, earthquakes
}
```

#### Geological Features Generated

- **Convergent boundaries** → Mountain ranges (Himalayas, Andes)
- **Divergent boundaries** → Rift valleys, volcanic islands
- **Hotspots** → Volcanic chains (Hawaii)
- **Subduction zones** → Ocean trenches, volcanic arcs

### 2.3 Hydrology System

Water flow is simulated, not painted:

```rust
pub struct HydrologySimulation {
    pub sea_level: f32,
    pub drainage_basins: DrainageGraph,
    pub aquifers: AquiferMap,
    pub glaciers: Vec<Glacier>,
}

pub struct DrainageGraph {
    pub nodes: Vec<DrainageNode>,  // Pour points
    pub edges: Vec<RiverSegment>,  // Water flow
}

pub struct RiverSegment {
    pub source: NodeId,
    pub target: NodeId,
    pub discharge: f32,    // m³/s - cumulative upstream flow
    pub width: f32,        // Calculated from discharge
    pub depth: f32,
    pub sediment_load: f32,
}
```

#### Water Cycle

```
                    PRECIPITATION
                         │
           ┌─────────────┼─────────────┐
           ▼             ▼             ▼
       Snow/Ice      Surface       Infiltration
           │          Runoff           │
           ▼             │             ▼
       Glaciers          │         Aquifers
           │             │             │
           └──────┬──────┴──────┬──────┘
                  ▼             ▼
               Rivers        Springs
                  │             │
                  └──────┬──────┘
                         ▼
                    OCEAN/LAKES
                         │
                         ▼
                   EVAPORATION
                         │
                         ▼
                 CLOUD FORMATION
                         │
                         └──────────▶ PRECIPITATION
```

### 2.4 Atmosphere & Wind Simulation

```rust
pub struct AtmosphereSimulation {
    pub composition: AtmosphericComposition,
    pub surface_pressure: f32,    // Pa
    pub scale_height: f32,        // m (atmosphere thickness)
    pub cells: Vec<AtmosphericCell>,
}

pub struct AtmosphericComposition {
    pub n2: f32,     // Nitrogen %
    pub o2: f32,     // Oxygen %
    pub co2: f32,    // Carbon dioxide ppm
    pub h2o: f32,    // Water vapor (variable)
    pub greenhouse_factor: f32,  // Calculated warming
}
```

#### Atmospheric Circulation Cells

Based on real planetary physics (Hadley-Ferrel-Polar model):

```
                POLE (90°)
                    │
            ┌───────┴───────┐
            │  POLAR CELL   │  Cold, dry, sinking air
            │   60°-90°     │  → Polar easterlies
            └───────┬───────┘
                    │
            ┌───────┴───────┐
            │ FERREL CELL   │  Mid-latitude dynamics
            │   30°-60°     │  → Westerlies
            └───────┬───────┘
                    │
            ┌───────┴───────┐
            │ HADLEY CELL   │  Warm, moist rising air
            │   0°-30°      │  → Trade winds (Easterlies)
            └───────┬───────┘
                    │
               EQUATOR (0°)
```

#### Coriolis Effect

Wind deflection based on rotation:
- **Northern hemisphere**: Deflected right
- **Southern hemisphere**: Deflected left
- **Strength**: Proportional to `rotation_period` and latitude

### 2.5 Climate Simulation

Climate is **derived** from all previous systems:

```rust
pub struct ClimateData {
    pub temperature_annual_mean: f32,     // °C
    pub temperature_amplitude: f32,        // Seasonal variation
    pub precipitation_annual: f32,         // mm/year
    pub precipitation_seasonality: f32,    // Wet/dry season ratio
    pub humidity: f32,
    pub wind_dominant: Vec2,
    pub sunshine_hours: f32,
}
```

#### Temperature Calculation

```
T_surface = T_equilibrium × (1 + greenhouse) - altitude_lapse - latitude_modifier

Where:
- T_equilibrium = (L_star / (16π σ d²))^0.25  (Stefan-Boltzmann)
- greenhouse = f(CO2, H2O, CH4)
- altitude_lapse ≈ 6.5°C per 1000m
- latitude_modifier = f(axial_tilt, season, latitude)
```

#### Precipitation Patterns

Calculated from:
1. **Moisture source**: Ocean proximity, prevailing winds
2. **Orographic lift**: Mountains force air up → cooling → rain
3. **Rain shadow**: Dry areas on leeward side of mountains
4. **Convergence zones**: ITCZ (Inter-Tropical Convergence Zone)

### 2.6 Biome Resolution (Köppen Classification)

Biomes are **derived** from climate data using the Köppen-Geiger system:

```rust
pub enum KoppenClimate {
    // Group A: Tropical (T_coldest ≥ 18°C)
    Af,  // Tropical Rainforest
    Am,  // Tropical Monsoon
    Aw,  // Tropical Savanna

    // Group B: Arid (P < threshold)
    BWh, // Hot Desert
    BWk, // Cold Desert
    BSh, // Hot Steppe
    BSk, // Cold Steppe

    // Group C: Temperate (T_coldest > 0°C, T_warmest > 10°C)
    Cfa, // Humid Subtropical
    Cfb, // Oceanic
    Csa, // Mediterranean (dry summer)

    // Group D: Continental (T_coldest ≤ 0°C, T_warmest > 10°C)
    Dfa, // Hot-summer Continental
    Dfb, // Warm-summer Continental
    Dfc, // Subarctic

    // Group E: Polar (T_warmest < 10°C)
    ET,  // Tundra
    EF,  // Ice Cap
}

fn classify_koppen(climate: &ClimateData) -> KoppenClimate {
    let t_warmest = climate.temperature_annual_mean + climate.temperature_amplitude / 2.0;
    let t_coldest = climate.temperature_annual_mean - climate.temperature_amplitude / 2.0;
    let p_annual = climate.precipitation_annual;

    // Group E: Polar
    if t_warmest < 10.0 {
        return if t_warmest < 0.0 { KoppenClimate::EF } else { KoppenClimate::ET };
    }

    // Group B: Arid (complex threshold)
    let p_threshold = calculate_aridity_threshold(climate);
    if p_annual < p_threshold {
        return classify_arid(climate);
    }

    // Group A: Tropical
    if t_coldest >= 18.0 {
        return classify_tropical(climate);
    }

    // Group C vs D: Temperate vs Continental
    if t_coldest > 0.0 {
        classify_temperate(climate)
    } else {
        classify_continental(climate)
    }
}
```

### 2.7 Ecosystem Simulation

#### Flora Generation

Plants are generated based on biome and local conditions:

```rust
pub struct FloraParameters {
    pub biome: KoppenClimate,
    pub moisture: f32,
    pub soil_type: SoilType,
    pub altitude: f32,
    pub slope: f32,
}

pub struct VegetationLayer {
    pub canopy: Option<TreeSpecies>,      // Tallest layer
    pub understory: Option<TreeSpecies>,   // Mid-height trees
    pub shrub_layer: Vec<ShrubSpecies>,
    pub ground_cover: GroundCover,
    pub density: f32,
}
```

#### Fauna Generation

Procedural creatures adapted to their environment:

```rust
pub struct CreatureArchetype {
    pub role: EcologicalRole,       // Predator, Herbivore, Decomposer
    pub size_class: SizeClass,      // Tiny, Small, Medium, Large, Massive
    pub locomotion: Locomotion,     // Walk, Run, Fly, Swim, Burrow
    pub diet: Diet,
    pub activity_pattern: Activity, // Diurnal, Nocturnal, Crepuscular
}

pub enum EcologicalRole {
    ApexPredator,      // Top of food chain
    Mesopredator,      // Mid-level predator
    Herbivore,         // Primary consumers
    Omnivore,
    Decomposer,        // Fungi, insects
    Producer,          // Autotrophs (if non-plant)
}
```

#### Ecological Placement Rules

```rust
pub struct EcosystemRules {
    pub predator_prey_ratio: f32,      // ~0.1 (10 prey per predator)
    pub carrying_capacity: f32,         // Max biomass per km²
    pub species_diversity: f32,         // Higher in tropics
    pub endemism_factor: f32,           // Unique species probability
}
```

### 2.8 NPC System

Three tiers of NPCs for different needs:

| Type | Definition | Generation | Use Case |
|------|------------|------------|----------|
| **Scripted** | Unique, handcrafted | Manual design | Main quest NPCs, bosses |
| **Generic** | Archetype-based | Template + variation | Town guards, merchants |
| **Procedural** | Fully generated | AI-driven | Random encounters, wildlife |

```rust
pub enum NpcType {
    Scripted {
        asset_id: AssetId,
        dialogue_tree: DialogueId,
        behavior: BehaviorTreeId,
    },
    Generic {
        archetype: ArchetypeId,
        variation_seed: u64,
        role: NpcRole,
    },
    Procedural {
        seed: u64,
        constraints: NpcConstraints,
        ai_profile: AiProfileId,
    },
}
```

---

## Layer 3: Creation Tools

### 3.1 Developer Navigator

Fast world exploration for developers:

```rust
pub enum DevSpeed {
    Walk,           // 5 m/s (player speed)
    Fast,           // 100 m/s
    Supersonic,     // 10 km/s
    Orbital,        // 1,000 km/s
    Interplanetary, // 100,000 km/s
    Galactic,       // Instant teleport
}
```

### 3.2 Debug Overlays

Visual debugging tools:

- **Biome Map**: Color by biome type
- **Temperature Map**: Heatmap visualization
- **Wind Vectors**: Directional arrows
- **Precipitation Map**: Moisture visualization
- **Chunk Boundaries**: LOD/streaming debug
- **SDF Visualization**: Density field debug

### 3.3 Terrain Sculptor

Large-scale terrain editing:

```rust
pub enum TerrainBrush {
    // Basic operations
    Raise { radius: f32, strength: f32 },
    Lower { radius: f32, strength: f32 },
    Smooth { radius: f32, iterations: u32 },

    // Geological features
    MountainRange { length: f32, height: f32, peaks: u32 },
    Volcano { crater_size: f32, height: f32 },
    Canyon { length: f32, depth: f32, width: f32 },
    River { source: GeoCoord, mouth: GeoCoord },
    Lake { center: GeoCoord, size: f32 },
    Archipelago { center: GeoCoord, islands: u32 },

    // Overrides
    BiomePaint { biome: Biome, blend: f32 },
}
```

### 3.4 Creature Editor

Tools for designing creatures:
- Morphology editor (body parts, proportions)
- Behavior editor (AI configuration)
- Appearance editor (textures, colors)
- Animation preview

---

## Crate Organization

```
synarion-engine/
├── engine/crates/
│   ├── syn_core/           # Foundation types, handles, pools
│   ├── syn_math/           # Vectors, matrices, transforms, geo
│   ├── syn_world/          # World Graph system
│   │   ├── graph.rs        # WorldGraph, WorldNode
│   │   ├── transition.rs   # Transitions, blending
│   │   └── generation.rs   # Generation strategies
│   ├── syn_terrain/        # Terrain system
│   │   ├── sdf/            # Signed Distance Fields
│   │   ├── meshing/        # Dual Contouring GPU
│   │   ├── planetary/      # Cube-sphere projection
│   │   └── volumetric/     # Caves, space volumes
│   ├── syn_planet_sim/     # Planetary simulation
│   │   ├── orbital.rs      # Orbital mechanics
│   │   ├── climate.rs      # Climate simulation
│   │   ├── biome.rs        # Biome resolution
│   │   └── hydrology.rs    # Rivers, oceans
│   ├── syn_ecosystem/      # Ecosystem simulation
│   │   ├── creature_gen.rs # Creature generation
│   │   ├── flora_gen.rs    # Plant generation
│   │   └── placer.rs       # Ecological placement
│   ├── syn_procgen/        # Procedural generation
│   │   ├── seed.rs         # Seed hierarchy
│   │   ├── noise.rs        # Noise functions
│   │   └── generators/     # Specific generators
│   ├── syn_streaming/      # Streaming & LOD
│   ├── syn_ai/             # AI systems
│   ├── syn_vulkan/         # Vulkan backend
│   ├── syn_renderer/       # Rendering
│   └── ...
│
├── editor/crates/
│   ├── syn_world_editor/   # World editing tools
│   │   ├── navigator.rs    # Dev navigation
│   │   ├── sculptor.rs     # Terrain sculpting
│   │   └── overlays.rs     # Debug visualizations
│   ├── syn_creature_editor/# Creature tools
│   └── ...
│
└── tools/crates/
    └── ...
```

---

## Use Cases

### Use Case 1: Space Exploration Game (No Man's Sky-like)

```
World Graph:
- Multiple galaxies (Seeded, Infinite)
- Billions of star systems (Seeded)
- Planets with caves (Hybrid: Seeded + player modifications)
- All transitions seamless

Features Used:
- Full orbital mechanics
- Climate simulation
- Creature generation
- Galaxy-to-ground LOD
```

### Use Case 2: Fantasy MMORPG (WoW-like)

```
World Graph:
- Main continent (Hybrid: Seeded terrain + constructed cities)
- Alternate dimensions (Seeded with different rules)
- Dungeons (Constructed instances)
- Portal transitions between dimensions

Features Used:
- Biome-based terrain
- Handcrafted city overlays
- NPC placement (scripted + generic)
- Portal transition system
```

### Use Case 3: Industrial Survival (Groundbreak)

```
World Graph:
- Single planet (Hybrid: Seeded + heavy player modifications)
- Underground caves (Seeded, seamless transition)

Features Used:
- Destructible terrain (SDF modifications)
- Resource distribution (geological rules)
- Climate/biome for gameplay variety
- Persistent delta storage
```

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Frame rate | 60 FPS | At 1080p, medium settings |
| Terrain generation | < 16ms/chunk | GPU compute |
| Transition latency | 0ms | Seamless, no stutter |
| Memory (terrain) | < 2 GB | With streaming |
| Planet detail | 1cm resolution | At player position |
| View distance | 100+ km | With LOD |

---

## Roadmap

### Phase 1: Foundation (Current)
- Core engine systems
- Basic Vulkan rendering
- ECS implementation

### Phase 2: Terrain
- SDF terrain system
- GPU meshing (Marching Cubes)
- Cube-sphere planetary projection
- Basic LOD and streaming

### Phase 3: World Graph
- Multi-world support
- Transition system
- Seamless loading

### Phase 4: Simulation
- Orbital mechanics
- Climate simulation
- Biome resolution

### Phase 5: Ecosystem
- Creature generation
- Flora generation
- Ecological placement

### Phase 6: Tools
- World editor
- Terrain sculptor
- Debug overlays

---

## Modular Architecture

Synarion Engine is not a monolithic engine but a **collection of independent crates** that can be assembled according to project needs.

### Design Philosophy

> **"Use only what you need"**

Every feature is optional. A simple platformer shouldn't compile MMO networking code. A RTS shouldn't include planetary terrain systems.

### Tier System

```
┌─────────────────────────────────────────────────────────────┐
│  TIER 0: FOUNDATION                                          │
│  Standalone libraries, useful even outside the engine        │
│  syn_core, syn_math, syn_collections                         │
├─────────────────────────────────────────────────────────────┤
│  TIER 1: PLATFORM                                            │
│  OS abstraction layer                                        │
│  syn_platform, syn_input, syn_filesystem, syn_tasks          │
├─────────────────────────────────────────────────────────────┤
│  TIER 2: GRAPHICS                                            │
│  Rendering with swappable backends                           │
│  syn_gpu (traits), syn_vulkan, syn_shaders, syn_renderer     │
├─────────────────────────────────────────────────────────────┤
│  TIER 3: GAME FRAMEWORK                                      │
│  Minimum viable game engine                                  │
│  syn_app, syn_ecs, syn_scene, syn_assets                     │
├─────────────────────────────────────────────────────────────┤
│  TIER 4: GAME SYSTEMS                                        │
│  Each system is completely independent                       │
│  Physics, Audio, UI, Animation, AI, Networking               │
├─────────────────────────────────────────────────────────────┤
│  TIER 5: SPECIALIZED                                         │
│  Advanced systems for specific use cases                     │
│  Terrain, Procedural, Simulation, Persistence, Delta         │
└─────────────────────────────────────────────────────────────┘
```

### Example Configurations

| Game Type | Crates Used |
|-----------|-------------|
| **Simple 3D Platformer** | Tier 0-3 + Physics (~10 crates) |
| **RTS with Terrain** | Tier 0-3 + Physics + AI + Heightmap (~15 crates) |
| **Exploration Game** | Tier 0-4 + SDF Terrain + Procgen (~25 crates) |
| **Groundbreak (Full)** | All Tiers including distributed delta (~40 crates) |

### Backend Abstraction

Systems that can have multiple implementations use a trait-based API:

```rust
// Graphics: syn_gpu defines traits, syn_vulkan implements them
// Physics: syn_physics_api defines traits, syn_physics_rapier implements them
// Audio: syn_audio_api defines traits, syn_audio_rodio implements them
```

This allows swapping backends without changing game code.

### Feature Presets

For convenience, meta-crates provide common configurations:

```toml
# Minimal game
synarion_engine = { version = "0.1", features = ["minimal"] }

# Full engine
synarion_engine = { version = "0.1", features = ["full"] }

# Groundbreak preset
synarion_engine = { version = "0.1", features = ["groundbreak"] }
```

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed crate documentation.

---

## Conclusion

Synarion Engine aims to be the **first general-purpose engine** capable of:
- Seamless galaxy-to-ground exploration
- Mixed procedural and handcrafted content
- Physically-based world simulation
- Professional creation tools

This is not just a game engine. It's a **world creation platform**.

---

*"From the center of a galaxy to a single grain of sand, without a loading screen."*

— Synarion Engine Vision
