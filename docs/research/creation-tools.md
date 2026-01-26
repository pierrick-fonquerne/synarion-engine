# Creation Tools - Research Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Active Research
**Crates**: `syn_world_editor`, `syn_creature_editor`, `syn_debug_overlay`

---

## Table of Contents

1. [Overview](#overview)
2. [Developer Navigator](#developer-navigator)
3. [Terrain Sculptor](#terrain-sculptor)
4. [Debug Overlays](#debug-overlays)
5. [Ecosystem Editor](#ecosystem-editor)
6. [World Editor Architecture](#world-editor-architecture)
7. [Undo/Redo System](#undoredo-system)
8. [UI Framework](#ui-framework)
9. [Implementation Notes](#implementation-notes)
10. [References](#references)
11. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### Goals

Creation tools should enable developers and content creators to:

1. **Explore**: Navigate vast procedural worlds at any speed
2. **Visualize**: See underlying systems (climate, biomes, terrain)
3. **Edit**: Modify any aspect of the world
4. **Test**: Quickly iterate on game design

### Tool Categories

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CREATION TOOLS                                   │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │    NAVIGATOR    │  │     SCULPTOR    │  │     DEBUG       │         │
│  │                 │  │                 │  │    OVERLAYS     │         │
│  │ • Free camera   │  │ • Terrain brush │  │ • Biome map     │         │
│  │ • Speed control │  │ • Geology tools │  │ • Temperature   │         │
│  │ • Bookmarks     │  │ • Water tools   │  │ • Wind vectors  │         │
│  │ • Teleport      │  │ • Vegetation    │  │ • Chunk bounds  │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   ECOSYSTEM     │  │     WORLD       │  │    TIMELINE     │         │
│  │    EDITOR       │  │    EDITOR       │  │    CONTROLS     │         │
│  │                 │  │                 │  │                 │         │
│  │ • Creature gen  │  │ • Node graph    │  │ • Time of day   │         │
│  │ • Flora editor  │  │ • Transitions   │  │ • Season        │         │
│  │ • Placement     │  │ • Overlays      │  │ • Weather       │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Developer Navigator

### Speed Modes

Navigate worlds at various scales:

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum NavigatorSpeed {
    Walk,           // 5 m/s - Player speed
    Run,            // 20 m/s - Fast exploration
    Vehicle,        // 100 m/s - Ground vehicle
    Aircraft,       // 500 m/s - Low altitude flight
    Supersonic,     // 2,000 m/s - High altitude
    Orbital,        // 10,000 m/s - Approaching orbit
    Planetary,      // 100,000 m/s - Planet to planet
    Interplanetary, // 10,000,000 m/s - System scale
    FTL,            // Instant teleport - Galaxy scale
}

impl NavigatorSpeed {
    pub fn meters_per_second(&self) -> f64 {
        match self {
            Self::Walk => 5.0,
            Self::Run => 20.0,
            Self::Vehicle => 100.0,
            Self::Aircraft => 500.0,
            Self::Supersonic => 2_000.0,
            Self::Orbital => 10_000.0,
            Self::Planetary => 100_000.0,
            Self::Interplanetary => 10_000_000.0,
            Self::FTL => f64::INFINITY,
        }
    }

    /// Auto-adjust speed based on altitude
    pub fn for_altitude(altitude: f64, planet_radius: f64) -> Self {
        let normalized_alt = altitude / planet_radius;

        if normalized_alt < 0.001 {        // < 6km (Earth)
            Self::Walk
        } else if normalized_alt < 0.01 {   // < 60km
            Self::Aircraft
        } else if normalized_alt < 0.1 {    // < 600km
            Self::Orbital
        } else if normalized_alt < 10.0 {   // < 60,000km
            Self::Planetary
        } else {
            Self::Interplanetary
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Walk => Self::Run,
            Self::Run => Self::Vehicle,
            Self::Vehicle => Self::Aircraft,
            Self::Aircraft => Self::Supersonic,
            Self::Supersonic => Self::Orbital,
            Self::Orbital => Self::Planetary,
            Self::Planetary => Self::Interplanetary,
            Self::Interplanetary => Self::FTL,
            Self::FTL => Self::FTL,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Walk => Self::Walk,
            Self::Run => Self::Walk,
            Self::Vehicle => Self::Run,
            Self::Aircraft => Self::Vehicle,
            Self::Supersonic => Self::Aircraft,
            Self::Orbital => Self::Supersonic,
            Self::Planetary => Self::Orbital,
            Self::Interplanetary => Self::Planetary,
            Self::FTL => Self::Interplanetary,
        }
    }
}
```

### Navigator Implementation

```rust
pub struct DeveloperNavigator {
    /// Current position (world space, double precision)
    position: DVec3,

    /// Current orientation
    rotation: DQuat,

    /// Current speed mode
    speed_mode: NavigatorSpeed,

    /// Speed multiplier (mouse wheel)
    speed_multiplier: f64,

    /// Input state
    input: NavigatorInput,

    /// Bookmarks
    bookmarks: Vec<NavigatorBookmark>,

    /// History for back/forward navigation
    history: NavigationHistory,

    /// Auto-speed adjustment enabled
    auto_speed: bool,

    /// Collision enabled (for ground-level navigation)
    collision: bool,
}

pub struct NavigatorInput {
    // WASD + Space/Ctrl for movement
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,

    // Mouse for rotation
    pub mouse_delta: Vec2,

    // Speed controls
    pub speed_up: bool,
    pub speed_down: bool,
}

impl DeveloperNavigator {
    pub fn update(&mut self, dt: f64, world: &World) {
        // Auto-adjust speed if enabled
        if self.auto_speed {
            let altitude = self.calculate_altitude(world);
            let planet_radius = world.active_node().planet_radius().unwrap_or(1_000_000.0);
            self.speed_mode = NavigatorSpeed::for_altitude(altitude, planet_radius);
        }

        // Handle speed changes
        if self.input.speed_up {
            self.speed_mode = self.speed_mode.next();
        }
        if self.input.speed_down {
            self.speed_mode = self.speed_mode.prev();
        }

        // Calculate velocity
        let speed = self.speed_mode.meters_per_second() * self.speed_multiplier;
        let mut velocity = DVec3::ZERO;

        let forward = self.rotation * DVec3::NEG_Z;
        let right = self.rotation * DVec3::X;
        let up = self.rotation * DVec3::Y;

        if self.input.forward { velocity += forward; }
        if self.input.backward { velocity -= forward; }
        if self.input.right { velocity += right; }
        if self.input.left { velocity -= right; }
        if self.input.up { velocity += up; }
        if self.input.down { velocity -= up; }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize() * speed;
        }

        // Apply movement
        let new_position = self.position + velocity * dt;

        // Collision check if enabled
        if self.collision && self.speed_mode <= NavigatorSpeed::Vehicle {
            if let Some(collision_pos) = self.check_collision(new_position, world) {
                self.position = collision_pos;
            } else {
                self.position = new_position;
            }
        } else {
            self.position = new_position;
        }

        // Handle rotation from mouse
        let sensitivity = 0.002;
        let yaw = -self.input.mouse_delta.x as f64 * sensitivity;
        let pitch = -self.input.mouse_delta.y as f64 * sensitivity;

        let yaw_quat = DQuat::from_rotation_y(yaw);
        let pitch_quat = DQuat::from_rotation_x(pitch);

        self.rotation = yaw_quat * self.rotation * pitch_quat;
        self.rotation = self.rotation.normalize();
    }

    pub fn teleport_to_geo(&mut self, coord: GeoCoord, world: &World) {
        if let Some(planet) = world.active_planet() {
            self.position = coord.to_cartesian(planet.radius);
            // Orient up relative to planet surface
            let up = self.position.normalize();
            self.rotation = look_at_with_up(self.position, self.position + up, up);
            self.add_to_history();
        }
    }

    pub fn add_bookmark(&mut self, name: String) {
        self.bookmarks.push(NavigatorBookmark {
            name,
            position: self.position,
            rotation: self.rotation,
            world_node: self.current_world_node,
            timestamp: Instant::now(),
        });
    }
}
```

### Bookmark System

```rust
#[derive(Clone)]
pub struct NavigatorBookmark {
    pub name: String,
    pub position: DVec3,
    pub rotation: DQuat,
    pub world_node: WorldNodeId,
    pub timestamp: Instant,
}

pub struct NavigationHistory {
    entries: Vec<HistoryEntry>,
    current_index: usize,
    max_entries: usize,
}

impl NavigationHistory {
    pub fn push(&mut self, entry: HistoryEntry) {
        // Remove forward history when adding new entry
        self.entries.truncate(self.current_index + 1);
        self.entries.push(entry);

        // Limit history size
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        self.current_index = self.entries.len() - 1;
    }

    pub fn back(&mut self) -> Option<&HistoryEntry> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }

    pub fn forward(&mut self) -> Option<&HistoryEntry> {
        if self.current_index < self.entries.len() - 1 {
            self.current_index += 1;
            Some(&self.entries[self.current_index])
        } else {
            None
        }
    }
}
```

---

## Terrain Sculptor

### Brush System

```rust
pub struct TerrainSculptor {
    /// Current brush
    brush: TerrainBrush,

    /// Brush size (radius in meters)
    brush_radius: f32,

    /// Brush strength (0.0 - 1.0)
    brush_strength: f32,

    /// Brush falloff curve
    falloff: FalloffCurve,

    /// Material to apply (for painting)
    current_material: MaterialId,

    /// Preview visualization
    preview_enabled: bool,
}

#[derive(Clone)]
pub enum TerrainBrush {
    // === Basic Operations ===

    /// Raise terrain
    Raise,
    /// Lower terrain
    Lower,
    /// Smooth terrain (average neighbors)
    Smooth,
    /// Flatten to target height
    Flatten { target_height: f32 },
    /// Noise addition
    Noise { noise_type: NoiseType, scale: f32 },

    // === Geological Features ===

    /// Create mountain range
    MountainRange {
        length: f32,
        height: f32,
        peak_count: u32,
        roughness: f32,
    },
    /// Create volcano with crater
    Volcano {
        crater_radius: f32,
        rim_height: f32,
        cone_slope: f32,
    },
    /// Carve canyon/river valley
    Canyon {
        width: f32,
        depth: f32,
        meandering: f32,
    },
    /// Create cliff face
    Cliff {
        height: f32,
        angle: f32,
    },

    // === Water Features ===

    /// Define river source point
    RiverSource,
    /// Create lake at current level
    Lake { water_level: f32 },
    /// Erode terrain (simulate water flow)
    Erode { iterations: u32, intensity: f32 },

    // === Painting ===

    /// Paint material/texture
    PaintMaterial,
    /// Paint biome override
    PaintBiome { biome: BiomeOverride },
    /// Paint vegetation density
    PaintVegetation { density: f32 },

    // === Special ===

    /// Carve cave entrance
    CaveEntrance { depth: f32, width: f32 },
    /// Place stamp (rock formation, etc.)
    Stamp { stamp_id: StampId, rotation: f32, scale: f32 },
}

#[derive(Clone, Copy)]
pub enum FalloffCurve {
    Linear,           // f(d) = 1 - d
    Smooth,           // f(d) = smoothstep(1, 0, d)
    Gaussian,         // f(d) = exp(-d² * 4)
    Constant,         // f(d) = 1 (hard edge)
    Custom(CurveId),
}

impl FalloffCurve {
    pub fn evaluate(&self, normalized_distance: f32) -> f32 {
        let d = normalized_distance.clamp(0.0, 1.0);
        match self {
            Self::Linear => 1.0 - d,
            Self::Smooth => {
                let t = d * d * (3.0 - 2.0 * d); // smoothstep
                1.0 - t
            }
            Self::Gaussian => (-d * d * 4.0).exp(),
            Self::Constant => if d < 1.0 { 1.0 } else { 0.0 },
            Self::Custom(curve_id) => curve_id.evaluate(d),
        }
    }
}
```

### Brush Application

```rust
impl TerrainSculptor {
    pub fn apply_brush(
        &self,
        world_position: DVec3,
        delta_storage: &mut DeltaStorage,
        terrain: &Terrain,
    ) {
        // Get affected voxels
        let affected_voxels = self.get_affected_voxels(world_position, terrain);

        for voxel_pos in affected_voxels {
            let distance = (voxel_pos.as_dvec3() - world_position).length() as f32;
            let normalized_dist = distance / self.brush_radius;

            if normalized_dist > 1.0 {
                continue;
            }

            let falloff = self.falloff.evaluate(normalized_dist);
            let strength = self.brush_strength * falloff;

            let delta = match &self.brush {
                TerrainBrush::Raise => {
                    TerrainDelta {
                        position: voxel_pos,
                        delta_type: DeltaType::Add(strength),
                        material: self.current_material,
                    }
                }
                TerrainBrush::Lower => {
                    TerrainDelta {
                        position: voxel_pos,
                        delta_type: DeltaType::Remove(strength),
                        material: MaterialId::AIR,
                    }
                }
                TerrainBrush::Smooth => {
                    let avg = self.calculate_neighbor_average(voxel_pos, terrain);
                    let current = terrain.sample(voxel_pos);
                    let new_value = current + (avg - current) * strength;
                    TerrainDelta {
                        position: voxel_pos,
                        delta_type: DeltaType::Set(new_value),
                        material: self.current_material,
                    }
                }
                TerrainBrush::Flatten { target_height } => {
                    let current = terrain.sample(voxel_pos);
                    let target = *target_height - voxel_pos.y as f32;
                    let new_value = current + (target - current) * strength;
                    TerrainDelta {
                        position: voxel_pos,
                        delta_type: DeltaType::Set(new_value),
                        material: self.current_material,
                    }
                }
                // ... other brush implementations
                _ => continue,
            };

            delta_storage.add(delta);
        }
    }

    fn get_affected_voxels(&self, center: DVec3, terrain: &Terrain) -> Vec<IVec3> {
        let voxel_size = terrain.voxel_size();
        let radius_voxels = (self.brush_radius / voxel_size).ceil() as i32;

        let center_voxel = terrain.world_to_voxel(center);
        let mut voxels = Vec::new();

        for x in -radius_voxels..=radius_voxels {
            for y in -radius_voxels..=radius_voxels {
                for z in -radius_voxels..=radius_voxels {
                    let voxel = center_voxel + IVec3::new(x, y, z);
                    voxels.push(voxel);
                }
            }
        }

        voxels
    }
}
```

### Geological Feature Brushes

```rust
impl TerrainSculptor {
    /// Create a mountain range along a path
    pub fn apply_mountain_range(
        &self,
        start: DVec3,
        end: DVec3,
        params: &MountainRangeParams,
        delta_storage: &mut DeltaStorage,
        terrain: &Terrain,
    ) {
        let length = (end - start).length();
        let direction = (end - start).normalize();
        let perpendicular = DVec3::new(-direction.z, 0.0, direction.x);

        // Generate peak positions along the range
        let mut peaks = Vec::new();
        for i in 0..params.peak_count {
            let t = i as f64 / (params.peak_count - 1).max(1) as f64;
            let base_pos = start + direction * length * t;

            // Add some randomness to peak position
            let offset = perpendicular * (rand_f64() - 0.5) * params.length as f64 * 0.2;
            let peak_pos = base_pos + offset;
            let peak_height = params.height * (0.7 + rand_f64() as f32 * 0.3);

            peaks.push((peak_pos, peak_height));
        }

        // For each voxel in range, calculate mountain contribution
        let range_width = params.length * 0.3;
        let affected = self.get_voxels_along_path(start, end, range_width as f64, terrain);

        for voxel_pos in affected {
            let world_pos = terrain.voxel_to_world(voxel_pos);

            // Calculate contribution from all peaks
            let mut height_contribution = 0.0f32;
            for (peak_pos, peak_height) in &peaks {
                let dist_to_peak = (DVec3::new(world_pos.x, 0.0, world_pos.z)
                    - DVec3::new(peak_pos.x, 0.0, peak_pos.z)).length() as f32;

                let falloff = (-dist_to_peak.powi(2) / (range_width * range_width)).exp();
                let ridge_noise = (dist_to_peak * params.roughness).sin() * 0.1;

                height_contribution += (*peak_height + ridge_noise) * falloff;
            }

            // Apply to terrain
            let target_height = terrain.sample_height_2d(world_pos.xz()) + height_contribution;
            let current_density = terrain.sample(voxel_pos);
            let target_density = target_height - voxel_pos.y as f32;

            if target_density > current_density {
                delta_storage.add(TerrainDelta {
                    position: voxel_pos,
                    delta_type: DeltaType::Set(target_density),
                    material: MaterialId::ROCK,
                });
            }
        }
    }

    /// Create river by hydraulic erosion simulation
    pub fn apply_river(
        &self,
        source: DVec3,
        terrain: &Terrain,
        delta_storage: &mut DeltaStorage,
    ) {
        // Simulate water droplets flowing downhill
        let mut droplets = vec![WaterDroplet::new(source)];

        for _ in 0..1000 {
            let mut new_droplets = Vec::new();

            for droplet in &mut droplets {
                if droplet.water <= 0.0 {
                    continue;
                }

                // Find flow direction (steepest descent)
                let gradient = terrain.calculate_gradient(droplet.position);
                let flow_dir = -gradient.normalize_or_zero();

                // Move droplet
                let speed = gradient.length().min(1.0);
                droplet.position += DVec3::new(flow_dir.x as f64, 0.0, flow_dir.y as f64) * speed as f64;

                // Erode terrain based on speed
                let erosion = speed * droplet.water * 0.1;
                let voxel = terrain.world_to_voxel(droplet.position);

                delta_storage.add(TerrainDelta {
                    position: voxel,
                    delta_type: DeltaType::Remove(erosion),
                    material: MaterialId::AIR,
                });

                // Pick up sediment
                droplet.sediment += erosion;
                droplet.water -= 0.01; // Evaporation

                // Deposit sediment in slow areas
                if speed < 0.3 {
                    let deposit = droplet.sediment * 0.5;
                    droplet.sediment -= deposit;

                    delta_storage.add(TerrainDelta {
                        position: voxel,
                        delta_type: DeltaType::Add(deposit),
                        material: MaterialId::SEDIMENT,
                    });
                }

                // Branch into multiple streams occasionally
                if droplet.water > 0.5 && rand_f64() < 0.01 {
                    let mut branch = droplet.clone();
                    branch.water *= 0.3;
                    droplet.water *= 0.7;
                    new_droplets.push(branch);
                }
            }

            droplets.append(&mut new_droplets);
            droplets.retain(|d| d.water > 0.0);

            if droplets.is_empty() {
                break;
            }
        }
    }
}
```

---

## Debug Overlays

### Overlay System

```rust
pub struct DebugOverlayManager {
    /// Active overlays
    active_overlays: HashSet<OverlayType>,

    /// Overlay rendering resources
    overlay_pipelines: HashMap<OverlayType, RenderPipeline>,

    /// Data textures for overlays
    data_textures: HashMap<OverlayType, Texture>,

    /// Color ramps for visualization
    color_ramps: HashMap<OverlayType, ColorRamp>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayType {
    // Terrain
    ChunkBoundaries,
    LodLevels,
    SdfValues,
    Wireframe,

    // Climate
    BiomeMap,
    TemperatureMap,
    PrecipitationMap,
    WindVectors,
    SunlightMap,

    // Hydrology
    DrainageBasins,
    RiverFlow,
    WaterTable,

    // Ecosystem
    VegetationDensity,
    CreatureHabitat,
    FoodWeb,

    // Performance
    ChunkGenTime,
    DrawCalls,
    MemoryUsage,
}

pub struct ColorRamp {
    colors: Vec<(f32, Color)>,  // (position 0-1, color)
}

impl ColorRamp {
    pub fn sample(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);

        // Find surrounding colors
        let mut prev = &self.colors[0];
        for entry in &self.colors {
            if entry.0 >= t {
                let local_t = (t - prev.0) / (entry.0 - prev.0);
                return prev.1.lerp(entry.1, local_t);
            }
            prev = entry;
        }

        self.colors.last().unwrap().1
    }

    pub fn temperature() -> Self {
        Self {
            colors: vec![
                (0.0, Color::rgb(0.0, 0.0, 0.5)),   // Dark blue (-40°C)
                (0.25, Color::rgb(0.0, 0.5, 1.0)),  // Light blue (0°C)
                (0.5, Color::rgb(0.0, 1.0, 0.0)),   // Green (20°C)
                (0.75, Color::rgb(1.0, 1.0, 0.0)),  // Yellow (35°C)
                (1.0, Color::rgb(1.0, 0.0, 0.0)),   // Red (50°C)
            ],
        }
    }

    pub fn precipitation() -> Self {
        Self {
            colors: vec![
                (0.0, Color::rgb(0.9, 0.8, 0.6)),   // Tan (desert)
                (0.25, Color::rgb(0.8, 0.7, 0.4)),  // Light brown
                (0.5, Color::rgb(0.5, 0.7, 0.3)),   // Yellow-green
                (0.75, Color::rgb(0.2, 0.6, 0.2)),  // Green
                (1.0, Color::rgb(0.0, 0.3, 0.1)),   // Dark green (rainforest)
            ],
        }
    }

    pub fn biome() -> HashMap<KoppenClimate, Color> {
        use KoppenClimate::*;
        HashMap::from([
            (Af, Color::rgb(0.0, 0.4, 0.0)),   // Tropical Rainforest - Dark green
            (Am, Color::rgb(0.0, 0.6, 0.2)),   // Tropical Monsoon
            (Aw, Color::rgb(0.6, 0.8, 0.4)),   // Tropical Savanna - Light green
            (BWh, Color::rgb(0.9, 0.8, 0.5)),  // Hot Desert - Sand
            (BWk, Color::rgb(0.8, 0.7, 0.6)),  // Cold Desert
            (BSh, Color::rgb(0.8, 0.7, 0.4)),  // Hot Steppe
            (BSk, Color::rgb(0.7, 0.6, 0.4)),  // Cold Steppe
            (Cfa, Color::rgb(0.4, 0.7, 0.3)),  // Humid Subtropical
            (Cfb, Color::rgb(0.3, 0.6, 0.3)),  // Oceanic
            (Csa, Color::rgb(0.7, 0.7, 0.2)),  // Mediterranean
            (Dfa, Color::rgb(0.2, 0.5, 0.4)),  // Continental
            (Dfb, Color::rgb(0.2, 0.4, 0.3)),  // Warm Continental
            (Dfc, Color::rgb(0.3, 0.4, 0.5)),  // Subarctic
            (ET, Color::rgb(0.7, 0.8, 0.8)),   // Tundra - Light gray
            (EF, Color::rgb(1.0, 1.0, 1.0)),   // Ice Cap - White
        ])
    }
}
```

### Overlay Rendering

```rust
impl DebugOverlayManager {
    pub fn render_overlay(
        &self,
        overlay: OverlayType,
        render_pass: &mut RenderPass,
        terrain: &Terrain,
        camera: &Camera,
    ) {
        match overlay {
            OverlayType::BiomeMap => {
                self.render_biome_overlay(render_pass, terrain, camera);
            }
            OverlayType::TemperatureMap => {
                self.render_scalar_overlay(
                    render_pass,
                    terrain,
                    camera,
                    |pos| terrain.climate_at(pos).temperature,
                    &ColorRamp::temperature(),
                    -40.0,
                    50.0,
                );
            }
            OverlayType::WindVectors => {
                self.render_vector_overlay(render_pass, terrain, camera);
            }
            OverlayType::ChunkBoundaries => {
                self.render_chunk_grid(render_pass, terrain, camera);
            }
            _ => {}
        }
    }

    fn render_scalar_overlay(
        &self,
        render_pass: &mut RenderPass,
        terrain: &Terrain,
        camera: &Camera,
        value_fn: impl Fn(DVec3) -> f32,
        ramp: &ColorRamp,
        min_value: f32,
        max_value: f32,
    ) {
        // Generate overlay texture if needed
        let resolution = 256; // Per visible chunk
        let mut colors = Vec::with_capacity(resolution * resolution);

        let visible_bounds = self.calculate_visible_bounds(camera);

        for y in 0..resolution {
            for x in 0..resolution {
                let u = x as f64 / resolution as f64;
                let v = y as f64 / resolution as f64;

                let world_pos = DVec3::new(
                    visible_bounds.min.x + u * (visible_bounds.max.x - visible_bounds.min.x),
                    0.0,
                    visible_bounds.min.z + v * (visible_bounds.max.z - visible_bounds.min.z),
                );

                let value = value_fn(world_pos);
                let t = (value - min_value) / (max_value - min_value);
                let color = ramp.sample(t);

                colors.push(color);
            }
        }

        // Upload to GPU and render as ground overlay
        self.upload_and_render_overlay(render_pass, &colors, resolution, camera);
    }

    fn render_vector_overlay(
        &self,
        render_pass: &mut RenderPass,
        terrain: &Terrain,
        camera: &Camera,
    ) {
        // Render wind vectors as arrows
        let grid_spacing = 1000.0; // meters between arrows
        let visible_bounds = self.calculate_visible_bounds(camera);

        let mut arrow_instances = Vec::new();

        let mut x = visible_bounds.min.x;
        while x < visible_bounds.max.x {
            let mut z = visible_bounds.min.z;
            while z < visible_bounds.max.z {
                let pos = DVec3::new(x, terrain.height_at(DVec3::new(x, 0.0, z)) + 10.0, z);
                let wind = terrain.wind_at(pos);

                arrow_instances.push(ArrowInstance {
                    position: pos.as_vec3(),
                    direction: wind.normalize().as_vec3(),
                    length: wind.length() as f32 * 10.0,
                    color: self.wind_speed_to_color(wind.length() as f32),
                });

                z += grid_spacing;
            }
            x += grid_spacing;
        }

        self.render_arrows(render_pass, &arrow_instances, camera);
    }
}
```

---

## Ecosystem Editor

### Creature Editor

```rust
pub struct CreatureEditor {
    /// Currently editing creature
    current_creature: Option<CreatureDefinition>,

    /// Body part library
    body_parts: HashMap<BodyPartType, Vec<BodyPartAsset>>,

    /// Preview renderer
    preview: CreaturePreviewRenderer,

    /// Animation player
    animation_player: AnimationPlayer,
}

pub struct CreatureDefinition {
    pub name: String,
    pub body_plan: BodyPlan,
    pub appearance: AppearanceParams,
    pub behavior: BehaviorProfile,
    pub stats: CreatureStats,
    pub ecological_niche: EcologicalNiche,
}

pub struct BodyPlan {
    pub base_skeleton: SkeletonType,
    pub parts: Vec<AttachedBodyPart>,
    pub symmetry: Symmetry,
    pub scale: Vec3,
}

pub struct AttachedBodyPart {
    pub part_type: BodyPartType,
    pub asset: BodyPartAsset,
    pub attachment_point: String,
    pub scale: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Copy)]
pub enum BodyPartType {
    Head,
    Torso,
    Limb,
    Tail,
    Wing,
    Fin,
    Horn,
    Eye,
    Mouth,
    Appendage,
}

impl CreatureEditor {
    pub fn generate_random(&mut self, constraints: &CreatureConstraints) {
        let body_plan = self.generate_body_plan(constraints);
        let appearance = self.generate_appearance(constraints);
        let behavior = self.generate_behavior(constraints);

        self.current_creature = Some(CreatureDefinition {
            name: self.generate_name(constraints),
            body_plan,
            appearance,
            behavior,
            stats: self.calculate_stats(&body_plan, constraints),
            ecological_niche: constraints.niche.clone(),
        });
    }

    fn generate_body_plan(&self, constraints: &CreatureConstraints) -> BodyPlan {
        let skeleton = match constraints.locomotion {
            Locomotion::Bipedal => SkeletonType::Biped,
            Locomotion::Quadruped => SkeletonType::Quadruped,
            Locomotion::Hexapod => SkeletonType::Hexapod,
            Locomotion::Flying => SkeletonType::Winged,
            Locomotion::Swimming => SkeletonType::Aquatic,
            Locomotion::Serpentine => SkeletonType::Serpent,
        };

        let mut parts = Vec::new();

        // Add required parts based on skeleton
        parts.push(self.random_head_for_diet(constraints.diet));
        parts.push(self.random_torso_for_size(constraints.size_class));

        // Add limbs
        let limb_count = skeleton.default_limb_count();
        for i in 0..limb_count {
            parts.push(self.random_limb(i, constraints));
        }

        // Optional parts based on constraints
        if rand_f32() < 0.3 {
            parts.push(self.random_tail(constraints));
        }

        BodyPlan {
            base_skeleton: skeleton,
            parts,
            symmetry: Symmetry::Bilateral,
            scale: constraints.size_class.base_scale(),
        }
    }
}
```

### Flora Editor

```rust
pub struct FloraEditor {
    current_plant: Option<PlantDefinition>,
    growth_preview: GrowthPreview,
}

pub struct PlantDefinition {
    pub name: String,
    pub plant_type: PlantType,
    pub growth_params: GrowthParams,
    pub appearance: PlantAppearance,
    pub biome_preferences: BiomePreferences,
}

#[derive(Clone, Copy)]
pub enum PlantType {
    Tree,
    Shrub,
    Grass,
    Flower,
    Fern,
    Cactus,
    Vine,
    Aquatic,
    Mushroom,
}

pub struct GrowthParams {
    pub max_height: f32,
    pub growth_rate: f32,
    pub branch_angle: f32,
    pub branch_probability: f32,
    pub leaf_density: f32,
    pub seasonal_behavior: SeasonalBehavior,
}

#[derive(Clone, Copy)]
pub enum SeasonalBehavior {
    Evergreen,
    Deciduous,
    FloweringSeasonal { bloom_season: Season },
    Dormant { active_season: Season },
}
```

---

## World Editor Architecture

### Main Editor Window

```rust
pub struct WorldEditor {
    /// The world being edited
    world: World,

    /// Active edit mode
    mode: EditMode,

    /// Selection state
    selection: Selection,

    /// Tool panels
    panels: EditorPanels,

    /// Undo/redo stack
    history: EditHistory,

    /// Viewport state
    viewport: EditorViewport,
}

pub enum EditMode {
    Navigate,
    TerrainSculpt,
    TerrainPaint,
    ObjectPlace,
    ObjectSelect,
    PathDraw,
    ZonePaint,
    OverlayEdit,
}

pub struct EditorPanels {
    pub hierarchy: HierarchyPanel,
    pub properties: PropertiesPanel,
    pub tools: ToolsPanel,
    pub world_settings: WorldSettingsPanel,
    pub debug: DebugPanel,
}

pub struct Selection {
    pub selected_objects: Vec<ObjectId>,
    pub selected_terrain: Option<TerrainSelection>,
    pub selected_zone: Option<ZoneId>,
}
```

### Panel Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  File  Edit  View  Tools  World  Help                               │ │ │ X │
├────────────────┬──────────────────────────────────────┬─────────────────────┤
│                │                                      │                     │
│   HIERARCHY    │                                      │    PROPERTIES      │
│                │                                      │                     │
│  ▼ World       │                                      │  Transform          │
│    ▼ Planet    │                                      │   Position: ...     │
│      Surface   │         3D VIEWPORT                  │   Rotation: ...     │
│      Caves     │                                      │   Scale: ...        │
│    ▼ Overlays  │                                      │                     │
│      City A    │                                      │  Terrain Brush      │
│      Road 1    │                                      │   Type: Raise       │
│                │                                      │   Size: 50m         │
│                │                                      │   Strength: 0.5     │
│                │                                      │                     │
├────────────────┼──────────────────────────────────────┼─────────────────────┤
│                │                                      │                     │
│     TOOLS      │           MINI MAP                   │      OVERLAYS       │
│                │                                      │                     │
│  [Raise]       │        [====]                        │  ☑ Biome Map        │
│  [Lower]       │                                      │  ☐ Temperature      │
│  [Smooth]      │                                      │  ☐ Wind Vectors     │
│  [Paint]       │                                      │  ☐ Chunk Bounds     │
│                │                                      │                     │
└────────────────┴──────────────────────────────────────┴─────────────────────┘
```

---

## Undo/Redo System

### Command Pattern

```rust
pub trait EditCommand: Send + Sync {
    /// Execute the command
    fn execute(&mut self, world: &mut World) -> Result<()>;

    /// Undo the command
    fn undo(&mut self, world: &mut World) -> Result<()>;

    /// Human-readable description
    fn description(&self) -> String;

    /// Can this command be merged with another?
    fn can_merge(&self, other: &dyn EditCommand) -> bool { false }

    /// Merge with another command
    fn merge(&mut self, other: Box<dyn EditCommand>) -> Result<()> {
        Err(anyhow!("Cannot merge"))
    }
}

pub struct TerrainEditCommand {
    /// Deltas to apply
    deltas: Vec<TerrainDelta>,
    /// Previous values (for undo)
    previous_values: Vec<(IVec3, f32)>,
    /// Chunk that was affected
    affected_chunks: HashSet<PlanetTileId>,
}

impl EditCommand for TerrainEditCommand {
    fn execute(&mut self, world: &mut World) -> Result<()> {
        // Store previous values
        self.previous_values.clear();
        for delta in &self.deltas {
            let prev = world.terrain().sample(delta.position);
            self.previous_values.push((delta.position, prev));
        }

        // Apply deltas
        world.terrain_mut().apply_deltas(&self.deltas);

        // Mark chunks dirty
        for chunk in &self.affected_chunks {
            world.chunk_manager_mut().mark_dirty(*chunk);
        }

        Ok(())
    }

    fn undo(&mut self, world: &mut World) -> Result<()> {
        // Restore previous values
        for (pos, value) in &self.previous_values {
            world.terrain_mut().set_value(*pos, *value);
        }

        // Mark chunks dirty
        for chunk in &self.affected_chunks {
            world.chunk_manager_mut().mark_dirty(*chunk);
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!("Terrain edit ({} voxels)", self.deltas.len())
    }

    fn can_merge(&self, other: &dyn EditCommand) -> bool {
        // Can merge continuous brush strokes
        other.as_any().downcast_ref::<Self>().is_some()
    }

    fn merge(&mut self, other: Box<dyn EditCommand>) -> Result<()> {
        let other = other.as_any().downcast::<Self>().unwrap();
        self.deltas.extend(other.deltas);
        self.affected_chunks.extend(other.affected_chunks);
        Ok(())
    }
}
```

### Edit History

```rust
pub struct EditHistory {
    /// Executed commands
    undo_stack: Vec<Box<dyn EditCommand>>,

    /// Undone commands (for redo)
    redo_stack: Vec<Box<dyn EditCommand>>,

    /// Maximum history size
    max_size: usize,

    /// Merge window (commands within this time can be merged)
    merge_window: Duration,

    /// Last command timestamp
    last_command_time: Instant,
}

impl EditHistory {
    pub fn execute(&mut self, mut command: Box<dyn EditCommand>, world: &mut World) -> Result<()> {
        // Execute the command
        command.execute(world)?;

        // Try to merge with previous command
        let now = Instant::now();
        let should_merge = now.duration_since(self.last_command_time) < self.merge_window
            && self.undo_stack.last().map(|c| c.can_merge(&*command)).unwrap_or(false);

        if should_merge {
            let mut prev = self.undo_stack.pop().unwrap();
            prev.merge(command)?;
            self.undo_stack.push(prev);
        } else {
            self.undo_stack.push(command);
        }

        // Clear redo stack (new action invalidates redo)
        self.redo_stack.clear();

        // Limit history size
        while self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }

        self.last_command_time = now;
        Ok(())
    }

    pub fn undo(&mut self, world: &mut World) -> Result<bool> {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(world)?;
            self.redo_stack.push(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self, world: &mut World) -> Result<bool> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(world)?;
            self.undo_stack.push(command);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
```

---

## UI Framework

### Immediate Mode GUI

For editor UI, use an immediate-mode approach:

```rust
pub fn draw_terrain_panel(ui: &mut Ui, sculptor: &mut TerrainSculptor) {
    ui.heading("Terrain Sculptor");

    // Brush selection
    ui.label("Brush Type");
    ui.horizontal(|ui| {
        if ui.selectable_label(matches!(sculptor.brush, TerrainBrush::Raise), "Raise").clicked() {
            sculptor.brush = TerrainBrush::Raise;
        }
        if ui.selectable_label(matches!(sculptor.brush, TerrainBrush::Lower), "Lower").clicked() {
            sculptor.brush = TerrainBrush::Lower;
        }
        if ui.selectable_label(matches!(sculptor.brush, TerrainBrush::Smooth), "Smooth").clicked() {
            sculptor.brush = TerrainBrush::Smooth;
        }
    });

    ui.separator();

    // Brush parameters
    ui.label("Brush Size (m)");
    ui.add(Slider::new(&mut sculptor.brush_radius, 1.0..=500.0).logarithmic(true));

    ui.label("Brush Strength");
    ui.add(Slider::new(&mut sculptor.brush_strength, 0.0..=1.0));

    ui.label("Falloff");
    ui.horizontal(|ui| {
        for falloff in [FalloffCurve::Linear, FalloffCurve::Smooth, FalloffCurve::Gaussian] {
            if ui.selectable_label(sculptor.falloff == falloff, format!("{:?}", falloff)).clicked() {
                sculptor.falloff = falloff;
            }
        }
    });

    // Preview toggle
    ui.checkbox(&mut sculptor.preview_enabled, "Show Preview");
}
```

---

## Implementation Notes

### Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Brush preview | < 1ms | Real-time feedback |
| Brush apply | < 16ms | Single frame |
| Undo/Redo | < 5ms | Instant feel |
| Overlay update | < 5ms | Background update OK |
| Navigator | 60 FPS | Smooth at all speeds |

### Suggested Crate Structure

```
editor/crates/
├── syn_world_editor/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── editor.rs         # Main editor
│   │   ├── navigator.rs      # Developer navigator
│   │   ├── sculptor/
│   │   │   ├── mod.rs
│   │   │   ├── brush.rs      # Brush system
│   │   │   └── geological.rs # Geological features
│   │   ├── overlay/
│   │   │   ├── mod.rs
│   │   │   └── types.rs      # Overlay types
│   │   ├── history/
│   │   │   ├── mod.rs
│   │   │   ├── command.rs    # Command pattern
│   │   │   └── stack.rs      # History stack
│   │   └── ui/
│   │       ├── mod.rs
│   │       └── panels.rs     # UI panels
│   └── tests/
├── syn_creature_editor/
│   └── ...
└── syn_debug_overlay/
    └── ...
```

---

## References

### Editor Design

1. **Unity Editor** - General workflow inspiration
2. **Unreal Editor** - Terrain tools reference
3. **World Machine** - Geological terrain generation
4. **Gaea** - Modern terrain workflow

### UI Frameworks

- egui (Rust immediate mode GUI)
- Dear ImGui (reference implementation)

---

## Ideas & Future Work

### To Research

- [ ] **Node-based editing**: Visual scripting for world generation
- [ ] **Collaborative editing**: Multiple users editing simultaneously
- [ ] **Version control**: Git-like versioning for world data
- [ ] **Procedural brushes**: Brushes that use noise/rules
- [ ] **AI-assisted tools**: ML-based terrain generation

### Tool Ideas

- [ ] **Road/path tool**: Auto-conforms to terrain
- [ ] **City generator**: Procedural city placement
- [ ] **Biome painter**: Override procedural biomes
- [ ] **Time scrubber**: Preview day/night and seasons
- [ ] **Weather controls**: Test weather conditions

### Notes

```
2026-01-22: Initial document creation
- Focused on terrain tools and overlays
- Need to prototype UI framework
- Navigator speeds need testing at scale
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
