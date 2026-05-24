# World Graph & Transitions - Research Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Active Research
**Crate**: `syn_world`

---

## Table of Contents

1. [Overview](#overview)
2. [World Graph Architecture](#world-graph-architecture)
3. [World Node Types](#world-node-types)
4. [Transition System](#transition-system)
5. [Coordinate Systems](#coordinate-systems)
6. [Streaming & Loading](#streaming--loading)
7. [Persistence](#persistence)
8. [Implementation Notes](#implementation-notes)
9. [References](#references)
10. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### The Problem

Traditional game engines use discrete scenes/levels with loading screens.
We want:

- **Seamless transitions**: Galaxy → System → Planet → Surface → Caves
- **Mixed content**: Procedural + handcrafted in the same world
- **Arbitrary connections**: Portals linking distant locations

### Solution: World Graph

Represent the entire game universe as a **directed graph** of world nodes:

```
┌────────────────────────────────────────────────────────────────┐
│                      WORLD GRAPH                               │
│                                                                │
│   ┌──────────┐      seamless      ┌──────────┐               │
│   │  GALAXY  │ ──────────────────▶│  SYSTEM  │               │
│   │ (Seeded) │                    │ (Seeded) │               │
│   └──────────┘                    └────┬─────┘               │
│                                        │ seamless            │
│                              ┌─────────┼─────────┐           │
│                              ▼         ▼         ▼           │
│                         ┌────────┐┌────────┐┌────────┐       │
│                         │PLANET A││PLANET B││  MOON  │       │
│                         │(Hybrid)││(Seeded)││(Seeded)│       │
│                         └───┬────┘└────────┘└───┬────┘       │
│                             │ seamless          │ portal     │
│                             ▼                   ▼            │
│                         ┌────────┐         ┌────────┐        │
│                         │ CAVES  │         │DUNGEON │        │
│                         │(Seeded)│         │(Constr)│        │
│                         └────────┘         └────────┘        │
│                                                              │
└────────────────────────────────────────────────────────────────┘
```

---

## World Graph Architecture

### Core Types

```rust
/// Unique identifier for a world node
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldNodeId(pub u64);

/// The complete world graph
pub struct WorldGraph {
    /// All nodes in the graph
    nodes: HashMap<WorldNodeId, WorldNode>,

    /// All transitions between nodes
    transitions: Vec<TransitionEdge>,

    /// Quick lookup: which transitions involve this node?
    transitions_by_node: HashMap<WorldNodeId, Vec<TransitionId>>,

    /// Currently active node (where the player/camera is)
    active_node: WorldNodeId,

    /// Nodes being streamed (adjacent to active)
    streaming_nodes: HashSet<WorldNodeId>,
}

/// A single world (planet, dungeon, dimension, etc.)
pub struct WorldNode {
    /// Unique identifier
    pub id: WorldNodeId,

    /// Human-readable name for debugging
    pub name: String,

    /// How this world is generated
    pub generation: GenerationStrategy,

    /// Spatial topology (sphere, flat, volume)
    pub topology: WorldTopology,

    /// LOD management policy
    pub lod_policy: LodPolicy,

    /// Physical properties
    pub physics: PhysicsParams,

    /// Optional parent (for hierarchical relationships)
    pub parent: Option<WorldNodeId>,

    /// Metadata for game logic
    pub metadata: WorldMetadata,
}
```

### Generation Strategies

```rust
pub enum GenerationStrategy {
    /// 100% procedural from seed
    Seeded {
        seed: u64,
        generator: GeneratorId,
        params: GeneratorParams,
    },

    /// 100% handcrafted (traditional level)
    Constructed {
        scene_asset: AssetId,
    },

    /// Procedural base + handcrafted overlays + runtime modifications
    Hybrid {
        seed: u64,
        generator: GeneratorId,
        params: GeneratorParams,
        overlays: Vec<OverlayLayer>,
        deltas: DeltaStorageId,
    },
}

/// Overlay layer for hybrid worlds
pub struct OverlayLayer {
    /// What kind of content
    pub layer_type: OverlayType,
    /// Region this overlay affects
    pub bounds: OverlayBounds,
    /// The content to apply
    pub content: OverlayContent,
    /// Priority (higher = applied later)
    pub priority: i32,
}

pub enum OverlayType {
    /// Replace terrain in region with handcrafted content
    TerrainReplace,
    /// Blend terrain with handcrafted heightmap
    TerrainBlend { blend_distance: f32 },
    /// Place static objects (buildings, landmarks)
    Objects,
    /// Override biome in region
    BiomeOverride,
    /// Spawn points for entities
    EntitySpawns,
}

pub enum OverlayContent {
    /// Asset reference (scene, prefab)
    Asset(AssetId),
    /// Inline data
    Inline(Box<dyn OverlayData>),
    /// Streaming asset (load on demand)
    Streaming { url: String, checksum: u64 },
}
```

### World Topology

```rust
pub enum WorldTopology {
    /// Spherical planet
    Sphere {
        radius: f64,  // meters
    },

    /// Flat infinite plane (traditional game world)
    FlatPlane {
        origin: DVec3,
        up: DVec3,
    },

    /// Bounded flat region
    FlatBounded {
        bounds: DAabb,
    },

    /// Unbounded 3D volume (space, caves)
    Volume {
        bounds: Option<DAabb>,  // None = infinite
    },

    /// Ring world (like Halo)
    Ring {
        major_radius: f64,
        minor_radius: f64,
    },

    /// Torus (donut world)
    Torus {
        major_radius: f64,
        minor_radius: f64,
    },

    /// Custom topology defined by code
    Custom {
        handler: TopologyHandlerId,
    },
}

impl WorldTopology {
    /// Calculate the "up" direction at a given position
    pub fn up_direction(&self, position: DVec3) -> DVec3 {
        match self {
            Self::Sphere { .. } => position.normalize(),
            Self::FlatPlane { up, .. } => *up,
            Self::FlatBounded { .. } => DVec3::Y,
            Self::Volume { .. } => DVec3::Y, // Or none?
            Self::Ring { .. } => todo!("Ring up calculation"),
            Self::Torus { .. } => todo!("Torus up calculation"),
            Self::Custom { handler } => handler.up_direction(position),
        }
    }

    /// Calculate gravity direction at a position
    pub fn gravity_direction(&self, position: DVec3) -> DVec3 {
        -self.up_direction(position)
    }

    /// Wrap position for worlds that wrap (torus, ring)
    pub fn wrap_position(&self, position: DVec3) -> DVec3 {
        match self {
            Self::Torus { major_radius, .. } => {
                // Wrap around the major circumference
                todo!("Torus position wrapping")
            }
            _ => position, // Most topologies don't wrap
        }
    }
}
```

---

## World Node Types

### Example: Galaxy Node

```rust
pub fn create_galaxy_node(seed: u64) -> WorldNode {
    WorldNode {
        id: WorldNodeId(hash("galaxy", seed)),
        name: format!("Galaxy_{:X}", seed),
        generation: GenerationStrategy::Seeded {
            seed,
            generator: GeneratorId::GALAXY,
            params: GeneratorParams::Galaxy {
                arm_count: 4,
                star_density: 0.1,
                diameter_ly: 100_000.0,
            },
        },
        topology: WorldTopology::Volume {
            bounds: None,  // Infinite
        },
        lod_policy: LodPolicy::Galactic {
            star_lod_distances: vec![1000.0, 100.0, 10.0, 1.0], // light years
        },
        physics: PhysicsParams {
            gravity_type: GravityType::None,
            time_scale: 1.0,
        },
        parent: None,
        metadata: WorldMetadata::default(),
    }
}
```

### Example: Planet Node (Hybrid)

```rust
pub fn create_groundbreak_planet(seed: u64) -> WorldNode {
    WorldNode {
        id: WorldNodeId(hash("planet_vr7742b", seed)),
        name: "VR-7742-b".to_string(),
        generation: GenerationStrategy::Hybrid {
            seed,
            generator: GeneratorId::TERRESTRIAL_PLANET,
            params: GeneratorParams::Planet {
                radius: 6_371_000.0,  // Earth-like
                gravity: 9.81,
                atmosphere: AtmosphereParams::earthlike(),
                biome_seed: seed ^ 0xBIOME,
            },
            overlays: vec![
                // Landing zone with tutorial structures
                OverlayLayer {
                    layer_type: OverlayType::Objects,
                    bounds: OverlayBounds::Sphere {
                        center: GeoCoord::new(0.0, 0.0),
                        radius: 1000.0,
                    },
                    content: OverlayContent::Asset(AssetId::from("tutorial_landing_zone")),
                    priority: 100,
                },
            ],
            deltas: DeltaStorageId::new("player_modifications"),
        },
        topology: WorldTopology::Sphere {
            radius: 6_371_000.0,
        },
        lod_policy: LodPolicy::PlanetarySurface {
            face_count: 6,
            max_lod: 15,  // Down to ~1m resolution
        },
        physics: PhysicsParams {
            gravity_type: GravityType::Spherical { g: 9.81 },
            time_scale: 1.0,
        },
        parent: None,  // Or system node if part of solar system
        metadata: WorldMetadata {
            tags: vec!["habitable", "resource_rich", "groundbreak_main"].into(),
            ..Default::default()
        },
    }
}
```

### Example: Dungeon Node (Constructed)

```rust
pub fn create_dungeon_node(asset_id: AssetId) -> WorldNode {
    WorldNode {
        id: WorldNodeId(hash("dungeon", asset_id.0)),
        name: format!("Dungeon_{}", asset_id),
        generation: GenerationStrategy::Constructed {
            scene_asset: asset_id,
        },
        topology: WorldTopology::FlatBounded {
            bounds: DAabb::new(DVec3::ZERO, DVec3::new(500.0, 100.0, 500.0)),
        },
        lod_policy: LodPolicy::Fixed {
            lod_level: 0,  // Always max detail
        },
        physics: PhysicsParams {
            gravity_type: GravityType::Uniform { direction: DVec3::NEG_Y, g: 9.81 },
            time_scale: 1.0,
        },
        parent: None,
        metadata: WorldMetadata {
            tags: vec!["instanced", "dungeon"].into(),
            ..Default::default()
        },
    }
}
```

---

## Transition System

### Transition Types

```rust
#[derive(Clone, Copy)]
pub struct TransitionId(pub u32);

pub struct TransitionEdge {
    pub id: TransitionId,

    /// Source world
    pub from_node: WorldNodeId,
    /// Destination world
    pub to_node: WorldNodeId,

    /// How the transition works
    pub transition_type: TransitionType,

    /// Where in source world this transition can be triggered
    pub trigger_region: TriggerRegion,

    /// Where in destination world the player appears
    pub destination_transform: DestinationTransform,

    /// Is this transition bidirectional?
    pub bidirectional: bool,
}

pub enum TransitionType {
    /// Seamless blend (e.g., space → planet surface)
    Seamless {
        /// Distance over which to blend
        blend_distance: f32,
        /// How to blend content (fade, crossfade, etc.)
        blend_mode: BlendMode,
    },

    /// Explicit portal (e.g., WoW-style portal)
    Portal {
        /// Visual effect on the portal
        visual_effect: Option<EffectId>,
        /// Sound effect when entering
        sound_effect: Option<SoundId>,
        /// Whether portal is visible from both sides
        two_sided: bool,
    },

    /// Screen fade transition
    Fade {
        /// Fade out duration
        fade_out: f32,
        /// Fade in duration
        fade_in: f32,
        /// Color to fade to
        fade_color: Color,
        /// Optional loading screen
        loading_screen: Option<AssetId>,
    },

    /// Triggered by game event, not position
    Event {
        /// Event that triggers this transition
        trigger_event: EventId,
    },

    /// Cutscene transition
    Cinematic {
        /// Cutscene to play
        cutscene: AssetId,
    },
}

pub enum BlendMode {
    /// Simple alpha blend between worlds
    Alpha,
    /// Crossfade based on altitude/distance
    DistanceBased { center: f64, range: f64 },
    /// Use a noise-based dissolve
    Dissolve { noise_scale: f32 },
    /// Custom blend shader
    Custom { shader: ShaderId },
}
```

### Trigger Regions

```rust
pub enum TriggerRegion {
    /// Spherical region in world space
    Sphere {
        center: DVec3,
        radius: f64,
    },

    /// Box region
    Box {
        bounds: DAabb,
    },

    /// Surface region on a planet (lat/lon/altitude)
    PlanetarySurface {
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
        min_altitude: f64,
        max_altitude: f64,
    },

    /// Altitude-based (for planet → space transitions)
    Altitude {
        min: f64,
        max: f64,
    },

    /// Portal mesh (player walks through it)
    PortalMesh {
        mesh: MeshId,
        transform: Transform,
    },

    /// Any point in the world (for event-based transitions)
    Anywhere,
}

impl TriggerRegion {
    pub fn contains(&self, position: DVec3, world: &WorldNode) -> bool {
        match self {
            Self::Sphere { center, radius } => {
                (position - *center).length() < *radius
            }
            Self::Altitude { min, max } => {
                let altitude = match &world.topology {
                    WorldTopology::Sphere { radius } => position.length() - radius,
                    _ => position.y,
                };
                altitude >= *min && altitude <= *max
            }
            // ... other implementations
            _ => todo!()
        }
    }
}
```

### Destination Transform

```rust
pub enum DestinationTransform {
    /// Fixed position in destination world
    Fixed {
        position: DVec3,
        rotation: DQuat,
    },

    /// Position relative to transition point
    Relative {
        offset: DVec3,
        maintain_orientation: bool,
    },

    /// Mirror across portal plane
    Mirror {
        plane_normal: DVec3,
    },

    /// Calculated based on entry velocity (for seamless)
    Continuous {
        velocity_scale: f64,
    },

    /// Spawn at designated spawn point
    SpawnPoint {
        spawn_id: SpawnPointId,
    },

    /// Custom calculation
    Custom {
        handler: TransformHandlerId,
    },
}
```

### Transition Manager

```rust
pub struct TransitionManager {
    graph: WorldGraph,
    current_node: WorldNodeId,

    /// Transitions currently in progress
    active_transitions: Vec<ActiveTransition>,

    /// Pre-loaded content for nearby transitions
    preloaded: HashMap<TransitionId, PreloadedContent>,
}

pub struct ActiveTransition {
    pub edge: TransitionId,
    pub progress: f32,  // 0.0 = start, 1.0 = complete
    pub start_time: f64,
    pub player_position: DVec3,
}

impl TransitionManager {
    pub fn update(&mut self, dt: f64, player: &Player) {
        // Check if player is in any trigger region
        for edge_id in self.graph.transitions_from(self.current_node) {
            let edge = &self.graph.transitions[edge_id];

            if edge.trigger_region.contains(player.position, self.current_world()) {
                self.begin_transition(edge_id, player);
            }
        }

        // Update active transitions
        for transition in &mut self.active_transitions {
            transition.progress += dt as f32 / self.transition_duration(transition.edge);

            if transition.progress >= 1.0 {
                self.complete_transition(transition);
            }
        }

        // Preload content for nearby transitions
        self.update_preloading(player);
    }

    fn begin_transition(&mut self, edge_id: TransitionId, player: &Player) {
        let edge = &self.graph.transitions[&edge_id];

        match &edge.transition_type {
            TransitionType::Seamless { .. } => {
                // Start blending immediately
                self.active_transitions.push(ActiveTransition {
                    edge: edge_id,
                    progress: 0.0,
                    start_time: self.current_time,
                    player_position: player.position,
                });
            }
            TransitionType::Portal { visual_effect, sound_effect, .. } => {
                // Play effects, teleport when player enters
                if let Some(effect) = visual_effect {
                    self.spawn_effect(*effect, player.position);
                }
                if let Some(sound) = sound_effect {
                    self.play_sound(*sound);
                }
                self.instant_transition(edge_id, player);
            }
            TransitionType::Fade { fade_out, fade_color, loading_screen, .. } => {
                // Start fade out
                self.start_fade_out(*fade_out, *fade_color, *loading_screen);
                self.active_transitions.push(ActiveTransition {
                    edge: edge_id,
                    progress: 0.0,
                    start_time: self.current_time,
                    player_position: player.position,
                });
            }
            _ => {}
        }
    }

    fn complete_transition(&mut self, transition: &ActiveTransition) {
        let edge = &self.graph.transitions[&transition.edge];

        // Calculate destination position
        let dest_position = self.calculate_destination(edge, transition.player_position);

        // Unload old world (if not seamless)
        if !matches!(edge.transition_type, TransitionType::Seamless { .. }) {
            self.unload_world(self.current_node);
        }

        // Set new active world
        self.current_node = edge.to_node;

        // Move player
        self.teleport_player(dest_position);
    }
}
```

---

## Coordinate Systems

### The Floating Origin Problem

At planetary scales, `f32` precision breaks down:

```
Earth radius: 6,371,000 meters
f32 precision at 6M: ~0.5 meters
Result: Jittering, rendering artifacts
```

### Solution: Local Coordinate Spaces

```rust
/// Double-precision world position
pub type WorldPosition = DVec3;

/// Single-precision local position (relative to origin)
pub type LocalPosition = Vec3;

/// The current rendering origin
pub struct FloatingOrigin {
    /// World position of the local origin
    pub world_position: WorldPosition,

    /// Threshold for recentering (typically 1-10 km)
    pub recenter_threshold: f64,
}

impl FloatingOrigin {
    pub fn world_to_local(&self, world: WorldPosition) -> LocalPosition {
        (world - self.world_position).as_vec3()
    }

    pub fn local_to_world(&self, local: LocalPosition) -> WorldPosition {
        self.world_position + local.as_dvec3()
    }

    pub fn should_recenter(&self, camera_local: LocalPosition) -> bool {
        camera_local.length() as f64 > self.recenter_threshold
    }

    pub fn recenter(&mut self, new_world_position: WorldPosition) {
        self.world_position = new_world_position;
        // All local positions need to be updated!
    }
}
```

### Geographic Coordinates

For planets, often easier to work in lat/lon/altitude:

```rust
#[derive(Clone, Copy)]
pub struct GeoCoord {
    /// Latitude in radians (-π/2 to π/2)
    pub latitude: f64,
    /// Longitude in radians (-π to π)
    pub longitude: f64,
    /// Altitude above sea level in meters
    pub altitude: f64,
}

impl GeoCoord {
    pub fn to_cartesian(&self, planet_radius: f64) -> DVec3 {
        let r = planet_radius + self.altitude;
        DVec3::new(
            r * self.latitude.cos() * self.longitude.cos(),
            r * self.latitude.sin(),
            r * self.latitude.cos() * self.longitude.sin(),
        )
    }

    pub fn from_cartesian(position: DVec3, planet_radius: f64) -> Self {
        let r = position.length();
        Self {
            latitude: (position.y / r).asin(),
            longitude: position.z.atan2(position.x),
            altitude: r - planet_radius,
        }
    }

    /// Great circle distance to another point (in radians)
    pub fn angular_distance(&self, other: &GeoCoord) -> f64 {
        let d_lat = other.latitude - self.latitude;
        let d_lon = other.longitude - self.longitude;

        let a = (d_lat / 2.0).sin().powi(2)
            + self.latitude.cos() * other.latitude.cos() * (d_lon / 2.0).sin().powi(2);

        2.0 * a.sqrt().asin()
    }

    /// Distance in meters (ignoring altitude)
    pub fn surface_distance(&self, other: &GeoCoord, planet_radius: f64) -> f64 {
        self.angular_distance(other) * planet_radius
    }
}
```

---

## Streaming & Loading

### Content Priority

```rust
pub struct StreamingManager {
    /// Current player position (world space)
    player_position: WorldPosition,

    /// Current player velocity
    player_velocity: DVec3,

    /// Loaded content
    loaded: HashMap<ContentId, LoadedContent>,

    /// Loading queue (priority queue)
    loading_queue: PriorityQueue<ContentRequest>,

    /// Budget per frame
    load_budget_bytes: usize,
    unload_budget_items: usize,
}

pub struct ContentRequest {
    pub content_id: ContentId,
    pub priority: f32,
    pub estimated_size: usize,
    pub callback: Option<LoadCallback>,
}

impl StreamingManager {
    pub fn calculate_priority(&self, content: &ContentId) -> f32 {
        let position = self.content_position(content);
        let distance = (position - self.player_position).length();

        // Base priority from distance
        let distance_priority = 1.0 / (distance as f32 + 1.0);

        // Boost for content in front of player
        let to_content = (position - self.player_position).normalize();
        let player_forward = self.player_velocity.normalize_or_zero();
        let facing_boost = to_content.dot(player_forward).max(0.0) as f32 * 0.5;

        // Boost for transition regions
        let transition_boost = if self.is_near_transition(position) { 0.5 } else { 0.0 };

        distance_priority + facing_boost + transition_boost
    }

    pub fn update(&mut self) {
        // Calculate priorities for all potential content
        self.update_priorities();

        // Load high-priority content
        let mut loaded_bytes = 0;
        while loaded_bytes < self.load_budget_bytes {
            if let Some(request) = self.loading_queue.pop() {
                if request.priority < self.min_load_priority {
                    break; // Nothing important enough to load
                }

                self.start_loading(request);
                loaded_bytes += request.estimated_size;
            } else {
                break;
            }
        }

        // Unload low-priority content
        let mut unloaded = 0;
        while unloaded < self.unload_budget_items {
            if let Some(content_id) = self.find_lowest_priority_loaded() {
                if self.calculate_priority(&content_id) > self.max_unload_priority {
                    break; // Everything loaded is important
                }

                self.unload(content_id);
                unloaded += 1;
            } else {
                break;
            }
        }
    }
}
```

### Async Loading

```rust
pub struct AsyncLoader {
    /// Thread pool for loading
    pool: ThreadPool,

    /// Active loading tasks
    tasks: HashMap<ContentId, LoadingTask>,

    /// Completed but not yet integrated
    completed: ConcurrentQueue<LoadedContent>,
}

impl AsyncLoader {
    pub fn start_load(&mut self, request: ContentRequest) {
        let content_id = request.content_id;

        // Spawn async task
        let task = self.pool.spawn(async move {
            // Load from disk/network
            let data = load_content_data(content_id).await?;

            // Parse/decompress (still on worker thread)
            let content = parse_content(data)?;

            Ok(content)
        });

        self.tasks.insert(content_id, LoadingTask {
            task,
            started: Instant::now(),
            request,
        });
    }

    pub fn poll_completed(&mut self) -> Vec<LoadedContent> {
        let mut completed = Vec::new();

        for (id, task) in self.tasks.iter_mut() {
            if let Some(result) = task.task.try_poll() {
                completed.push(LoadedContent {
                    id: *id,
                    data: result,
                    load_time: task.started.elapsed(),
                });
            }
        }

        // Remove completed tasks
        for content in &completed {
            self.tasks.remove(&content.id);
        }

        completed
    }
}
```

---

## Persistence

### Save System

```rust
pub struct WorldSaveData {
    /// World graph configuration
    pub graph_config: WorldGraphConfig,

    /// Per-node save data
    pub node_data: HashMap<WorldNodeId, NodeSaveData>,

    /// Player state
    pub player: PlayerSaveData,

    /// Global game state
    pub global: GlobalSaveData,

    /// Save metadata
    pub metadata: SaveMetadata,
}

pub struct NodeSaveData {
    /// Terrain modifications (deltas)
    pub terrain_deltas: Option<DeltaStorage>,

    /// Entity states
    pub entities: Vec<EntitySaveData>,

    /// Custom node data (game-specific)
    pub custom: HashMap<String, serde_json::Value>,
}

impl WorldSaveData {
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Serialize
        let data = bincode::serialize(self)?;

        // Compress
        let compressed = zstd::encode_all(&data[..], 3)?;

        // Write
        std::fs::write(path, compressed)?;

        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        // Read
        let compressed = std::fs::read(path)?;

        // Decompress
        let data = zstd::decode_all(&compressed[..])?;

        // Deserialize
        let save: Self = bincode::deserialize(&data)?;

        Ok(save)
    }
}
```

### Delta Storage Persistence

```rust
impl DeltaStorage {
    /// Serialize to compact binary format
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Header
        buffer.extend_from_slice(&(self.deltas.len() as u32).to_le_bytes());

        // Deltas
        for delta in &self.deltas {
            buffer.extend_from_slice(&delta.position.x.to_le_bytes());
            buffer.extend_from_slice(&delta.position.y.to_le_bytes());
            buffer.extend_from_slice(&delta.position.z.to_le_bytes());
            buffer.push(delta.delta_type as u8);
            buffer.extend_from_slice(&delta.value.to_le_bytes());
            buffer.extend_from_slice(&delta.material.0.to_le_bytes());
        }

        buffer
    }

    /// Incremental save (only new deltas since last save)
    pub fn serialize_incremental(&self, since: u64) -> Vec<u8> {
        let new_deltas: Vec<_> = self.deltas.iter()
            .filter(|d| d.timestamp > since)
            .collect();

        // Same format as full serialize, just fewer deltas
        self.serialize_deltas(&new_deltas)
    }
}
```

---

## Implementation Notes

### Performance Considerations

| Operation | Target | Notes |
|-----------|--------|-------|
| Transition check | < 0.1ms/frame | Per-player |
| World switch | < 16ms | For portals |
| Seamless blend | 0ms stutter | Fully streamed |
| Save game | < 1s | Background thread |
| Load game | < 5s | With streaming |

### Memory Budget

```
Active world: 2 GB
Adjacent worlds (streaming): 1 GB each (up to 4)
Preloaded transitions: 500 MB
Save data in memory: 100 MB
Total: ~6 GB max
```

### Suggested Crate Structure

```
syn_world/
├── src/
│   ├── lib.rs
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── node.rs           # WorldNode
│   │   ├── edge.rs           # TransitionEdge
│   │   └── query.rs          # Graph queries
│   ├── generation/
│   │   ├── mod.rs
│   │   ├── strategy.rs       # GenerationStrategy
│   │   ├── overlay.rs        # Overlay system
│   │   └── delta.rs          # Delta storage
│   ├── transition/
│   │   ├── mod.rs
│   │   ├── types.rs          # TransitionType
│   │   ├── trigger.rs        # TriggerRegion
│   │   └── manager.rs        # TransitionManager
│   ├── coordinates/
│   │   ├── mod.rs
│   │   ├── floating_origin.rs
│   │   └── geo.rs            # Geographic coords
│   ├── streaming/
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   └── loader.rs
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── save.rs
│   │   └── load.rs
│   └── prelude.rs
└── tests/
```

---

## References

### Games with Seamless Transitions

1. **No Man's Sky** - Galaxy → Planet → Surface
2. **Star Citizen** - Multi-scale seamless
3. **Outer Wilds** - Small solar system, seamless
4. **Kerbal Space Program** - SOI-based transitions

### Papers

1. **Floating Point Issues in Games**
   - "Beating the World of Warcraft Precision Bug" (GDC)

2. **Streaming Systems**
   - "Streaming Open Worlds" - GDC presentations

### Online Resources

- Sebastian Lague: "Solar System" video series
- Chris Roberts on Star Citizen streaming

---

## Ideas & Future Work

### To Research

- [ ] **Multiple players**: Different players in different worlds
- [ ] **World instancing**: Same world, multiple instances (dungeons)
- [ ] **Procedural transitions**: Generate connecting content on-the-fly
- [ ] **Time dilation**: Different time scales in different worlds
- [ ] **Physics boundaries**: How to handle physics at transitions

### Optimization Ideas

- [ ] **Predictive loading**: ML-based prediction of player movement
- [ ] **LOD-aware streaming**: Lower LOD for distant transition targets
- [ ] **Compression**: Specialized compression for different content types
- [ ] **Deduplication**: Shared content across worlds

### Gameplay Ideas

- [ ] **Portal physics**: Portals with non-Euclidean geometry
- [ ] **World bleeding**: Effects that cross world boundaries
- [ ] **Persistent connections**: Bridges between worlds
- [ ] **Dimension rifts**: Random/dynamic transitions

### Notes

```
2026-01-22: Initial document creation
- Focused on graph structure and transitions
- Need to implement floating origin in practice
- Coordinate systems need testing at scale
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
