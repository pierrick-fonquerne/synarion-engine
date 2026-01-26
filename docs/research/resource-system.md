# Resource System - Research Document

**Version**: 0.1.0
**Date**: 2026-01-23
**Status**: Active Research
**Crates**: `syn_terrain`, `syn_resources`
**Game**: Groundbreak (primary use case)

---

## Table of Contents

1. [Overview](#overview)
2. [Geological Distribution](#geological-distribution)
3. [Vein Generation](#vein-generation)
4. [Resource Types](#resource-types)
5. [Terrain Integration](#terrain-integration)
6. [Extraction Mechanics](#extraction-mechanics)
7. [Finite Economy](#finite-economy)
8. [Scanning & Discovery](#scanning--discovery)
9. [Implementation Notes](#implementation-notes)
10. [References](#references)
11. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### Design Goals

For Groundbreak and similar games:

1. **Geologically Plausible**: Resources appear where they would naturally form
2. **Finite & Consequential**: Extraction depletes the world permanently
3. **Discoverable**: Players must explore and scan to find resources
4. **Integrated with Terrain**: Resources are part of the SDF, not just data

### Resource Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       RESOURCE SYSTEM PIPELINE                               │
│                                                                              │
│  GENERATION (World Creation)                                                 │
│  ─────────────────────────────                                               │
│                                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │  TECTONIC    │──▶│  GEOLOGICAL  │──▶│    VEIN      │──▶│   RESOURCE   │ │
│  │   CONTEXT    │   │   LAYERS     │   │  GENERATION  │   │     MAP      │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
│  RUNTIME                                                                     │
│  ───────                                                                     │
│                                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐ │
│  │   SCANNING   │──▶│  DISCOVERY   │──▶│  EXTRACTION  │──▶│  DEPLETION   │ │
│  │   (Player)   │   │   (Reveal)   │   │   (Mining)   │   │   (Delta)    │ │
│  └──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Geological Distribution

### Geological Context

Resources form in specific geological conditions:

```rust
/// Geological formation types
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeologicalContext {
    // Igneous (volcanic/magmatic)
    VolcanicIntrusion,      // Magma chambers, dikes
    VolcanicExtrusion,      // Lava flows, volcanic vents
    Plutonic,               // Deep crystallized magma

    // Sedimentary
    MarineSediment,         // Ancient ocean floor
    FluvialDeposit,         // River deposits
    Evaporite,              // Dried lake/sea (salt, gypsum)
    Coal Basin,             // Ancient swamps

    // Metamorphic
    ContactMetamorphism,    // Near magma intrusions
    RegionalMetamorphism,   // Mountain building pressure

    // Hydrothermal
    HydrothermalVent,       // Hot water deposits
    Geothermal,             // Hot springs, geysers

    // Surface
    Alluvial,               // River placer deposits
    Weathering,             // Surface erosion concentration
    Impact,                 // Meteor crater
}

/// Resource formation rules
pub struct ResourceFormation {
    pub resource: ResourceType,
    pub contexts: Vec<GeologicalContext>,
    pub depth_range: (f32, f32),      // Min/max depth (meters)
    pub temperature_range: (f32, f32), // Required temp for formation
    pub rarity: f32,                   // Base rarity (0-1)
}

impl ResourceFormation {
    pub fn iron() -> Self {
        Self {
            resource: ResourceType::Iron,
            contexts: vec![
                GeologicalContext::MarineSediment,    // Banded iron formations
                GeologicalContext::VolcanicIntrusion, // Magmatic segregation
                GeologicalContext::ContactMetamorphism,
            ],
            depth_range: (10.0, 500.0),
            temperature_range: (0.0, 800.0),
            rarity: 0.3, // Common
        }
    }

    pub fn gold() -> Self {
        Self {
            resource: ResourceType::Gold,
            contexts: vec![
                GeologicalContext::HydrothermalVent,  // Primary deposits
                GeologicalContext::Alluvial,          // Placer gold
                GeologicalContext::ContactMetamorphism,
            ],
            depth_range: (50.0, 2000.0),
            temperature_range: (200.0, 600.0),
            rarity: 0.05, // Rare
        }
    }

    pub fn uranium() -> Self {
        Self {
            resource: ResourceType::Uranium,
            contexts: vec![
                GeologicalContext::MarineSediment,    // Roll-front deposits
                GeologicalContext::Plutonic,          // Granite intrusions
            ],
            depth_range: (100.0, 1000.0),
            temperature_range: (50.0, 300.0),
            rarity: 0.02, // Very rare
        }
    }
}
```

### Depth-Based Stratification

```rust
/// Geological layers from surface to core
pub struct GeologicalProfile {
    layers: Vec<GeologicalLayer>,
}

pub struct GeologicalLayer {
    pub name: String,
    pub depth_start: f32,
    pub depth_end: f32,
    pub rock_type: RockType,
    pub context: GeologicalContext,
    pub porosity: f32,           // For fluid resources
    pub hardness: f32,           // Mining difficulty
}

impl GeologicalProfile {
    /// Generate profile for a location based on tectonic history
    pub fn generate(
        location: GeoCoord,
        tectonic_data: &TectonicData,
        seed: u64,
    ) -> Self {
        let mut layers = Vec::new();
        let mut current_depth = 0.0;

        // Surface layer (weathered)
        layers.push(GeologicalLayer {
            name: "Regolith".into(),
            depth_start: 0.0,
            depth_end: 5.0,
            rock_type: RockType::Soil,
            context: GeologicalContext::Weathering,
            porosity: 0.4,
            hardness: 0.1,
        });
        current_depth = 5.0;

        // Determine base geology from tectonics
        let base_context = tectonic_data.context_at(location);

        match base_context {
            GeologicalContext::VolcanicIntrusion => {
                // Volcanic area: basalt, intrusions
                layers.push(GeologicalLayer {
                    name: "Basalt".into(),
                    depth_start: current_depth,
                    depth_end: current_depth + 50.0,
                    rock_type: RockType::Basalt,
                    context: GeologicalContext::VolcanicExtrusion,
                    porosity: 0.1,
                    hardness: 0.7,
                });
                // Add more layers...
            }
            GeologicalContext::MarineSediment => {
                // Sedimentary basin
                layers.push(GeologicalLayer {
                    name: "Limestone".into(),
                    depth_start: current_depth,
                    depth_end: current_depth + 100.0,
                    rock_type: RockType::Limestone,
                    context: GeologicalContext::MarineSediment,
                    porosity: 0.15,
                    hardness: 0.4,
                });
                // Shale, sandstone layers...
            }
            // ... other contexts
            _ => {}
        }

        Self { layers }
    }

    pub fn layer_at_depth(&self, depth: f32) -> Option<&GeologicalLayer> {
        self.layers.iter().find(|l| depth >= l.depth_start && depth < l.depth_end)
    }
}
```

---

## Vein Generation

### Vein Types

```rust
#[derive(Clone)]
pub enum VeinShape {
    /// Spherical ore body (porphyry copper, kimberlite)
    Blob {
        center: Vec3,
        radius: f32,
        noise_distortion: f32,
    },

    /// Linear vein (hydrothermal gold, quartz veins)
    Vein {
        start: Vec3,
        end: Vec3,
        thickness: f32,
        branching_factor: f32,
    },

    /// Horizontal layer (coal seams, banded iron)
    Layer {
        center: Vec3,
        extent: Vec2,      // Horizontal size
        thickness: f32,
        undulation: f32,   // Waviness
    },

    /// Scattered nodules (manganese, phosphate)
    Nodules {
        region_center: Vec3,
        region_radius: f32,
        nodule_size: f32,
        density: f32,
    },

    /// Placer deposit (alluvial gold, diamonds)
    Placer {
        river_path: Vec<Vec3>,
        width: f32,
        concentration_curve: f32,
    },
}

impl VeinShape {
    /// Sample resource density at a point
    pub fn density_at(&self, point: Vec3, seed: u64) -> f32 {
        match self {
            Self::Blob { center, radius, noise_distortion } => {
                let dist = (point - *center).length();
                if dist > *radius {
                    return 0.0;
                }

                // Add noise distortion
                let noise = fbm_3d(point * 0.1, seed, 4) * noise_distortion;
                let effective_radius = radius * (1.0 + noise);

                // Smooth falloff
                let t = dist / effective_radius;
                (1.0 - t * t).max(0.0)
            }

            Self::Vein { start, end, thickness, branching_factor } => {
                // Distance to line segment
                let line = *end - *start;
                let t = ((point - *start).dot(line) / line.length_squared()).clamp(0.0, 1.0);
                let closest = *start + line * t;
                let dist = (point - closest).length();

                if dist > *thickness {
                    return 0.0;
                }

                // Add variation along vein
                let variation = fbm_3d(point * 0.05, seed, 3) * 0.5 + 0.5;
                let base_density = 1.0 - (dist / thickness);

                base_density * variation
            }

            Self::Layer { center, extent, thickness, undulation } => {
                // Check horizontal bounds
                let dx = (point.x - center.x).abs();
                let dz = (point.z - center.z).abs();
                if dx > extent.x || dz > extent.y {
                    return 0.0;
                }

                // Undulating surface
                let wave = (point.x * 0.01 + point.z * 0.013).sin() * undulation;
                let layer_y = center.y + wave;

                let dy = (point.y - layer_y).abs();
                if dy > *thickness {
                    return 0.0;
                }

                1.0 - (dy / thickness)
            }

            Self::Nodules { region_center, region_radius, nodule_size, density } => {
                let dist_to_region = (point - *region_center).length();
                if dist_to_region > *region_radius {
                    return 0.0;
                }

                // Check if inside any nodule
                let cell = (point / (*nodule_size * 3.0)).floor();
                let cell_seed = hash_vec3(cell, seed);

                if random_from_seed(cell_seed) > *density {
                    return 0.0;
                }

                // Nodule center within cell
                let nodule_offset = random_vec3(cell_seed) * nodule_size * 2.0;
                let nodule_center = cell * nodule_size * 3.0 + nodule_offset;

                let dist_to_nodule = (point - nodule_center).length();
                if dist_to_nodule > *nodule_size {
                    return 0.0;
                }

                1.0 - (dist_to_nodule / nodule_size)
            }

            Self::Placer { river_path, width, concentration_curve } => {
                // Find closest point on river path
                let mut min_dist = f32::MAX;
                for segment in river_path.windows(2) {
                    let dist = distance_to_segment(point, segment[0], segment[1]);
                    min_dist = min_dist.min(dist);
                }

                if min_dist > *width {
                    return 0.0;
                }

                // Concentration higher in center of river bed
                let t = min_dist / width;
                (1.0 - t).powf(*concentration_curve)
            }
        }
    }
}
```

### Procedural Vein Generation

```rust
pub struct VeinGenerator {
    formations: Vec<ResourceFormation>,
    seed: u64,
}

impl VeinGenerator {
    /// Generate all veins for a chunk
    pub fn generate_veins(
        &self,
        chunk_bounds: Aabb,
        geological_profile: &GeologicalProfile,
    ) -> Vec<ResourceVein> {
        let mut veins = Vec::new();
        let chunk_seed = self.seed ^ hash_aabb(&chunk_bounds);

        for formation in &self.formations {
            // Check if this chunk's geology supports this resource
            let layers_with_context: Vec<_> = geological_profile.layers
                .iter()
                .filter(|l| formation.contexts.contains(&l.context))
                .collect();

            if layers_with_context.is_empty() {
                continue;
            }

            // Determine number of veins based on rarity
            let avg_veins = formation.rarity * chunk_volume(&chunk_bounds) / 100_000.0;
            let vein_count = poisson_sample(avg_veins, chunk_seed ^ formation.resource as u64);

            for i in 0..vein_count {
                let vein_seed = chunk_seed ^ (i as u64 * 12345);

                // Pick a suitable layer
                let layer_idx = (random_from_seed(vein_seed) * layers_with_context.len() as f32) as usize;
                let layer = layers_with_context[layer_idx];

                // Generate vein within layer
                let vein = self.generate_single_vein(
                    formation,
                    layer,
                    &chunk_bounds,
                    vein_seed,
                );

                if let Some(v) = vein {
                    veins.push(v);
                }
            }
        }

        veins
    }

    fn generate_single_vein(
        &self,
        formation: &ResourceFormation,
        layer: &GeologicalLayer,
        chunk_bounds: &Aabb,
        seed: u64,
    ) -> Option<ResourceVein> {
        // Random position within layer depth range
        let depth = lerp(
            layer.depth_start.max(formation.depth_range.0),
            layer.depth_end.min(formation.depth_range.1),
            random_from_seed(seed),
        );

        let center = Vec3::new(
            lerp(chunk_bounds.min.x, chunk_bounds.max.x, random_from_seed(seed ^ 1)),
            -depth, // Depth is positive downward
            lerp(chunk_bounds.min.z, chunk_bounds.max.z, random_from_seed(seed ^ 2)),
        );

        // Choose vein shape based on resource type and geology
        let shape = self.choose_vein_shape(formation, layer, center, seed);

        // Calculate vein richness (ore grade)
        let richness = self.calculate_richness(formation, layer, depth, seed);

        Some(ResourceVein {
            resource: formation.resource,
            shape,
            richness,
            discovered: false,
            extracted_amount: 0.0,
        })
    }

    fn choose_vein_shape(
        &self,
        formation: &ResourceFormation,
        layer: &GeologicalLayer,
        center: Vec3,
        seed: u64,
    ) -> VeinShape {
        match formation.resource {
            ResourceType::Coal => {
                // Coal forms in horizontal seams
                VeinShape::Layer {
                    center,
                    extent: Vec2::new(50.0 + random_from_seed(seed) * 100.0,
                                     50.0 + random_from_seed(seed ^ 1) * 100.0),
                    thickness: 1.0 + random_from_seed(seed ^ 2) * 5.0,
                    undulation: 2.0,
                }
            }
            ResourceType::Gold => {
                // Gold in hydrothermal veins
                let length = 20.0 + random_from_seed(seed) * 80.0;
                let direction = random_unit_vec3(seed ^ 1);
                VeinShape::Vein {
                    start: center - direction * length * 0.5,
                    end: center + direction * length * 0.5,
                    thickness: 0.5 + random_from_seed(seed ^ 2) * 2.0,
                    branching_factor: 0.3,
                }
            }
            ResourceType::Iron => {
                // Iron in large blob deposits
                VeinShape::Blob {
                    center,
                    radius: 10.0 + random_from_seed(seed) * 30.0,
                    noise_distortion: 0.3,
                }
            }
            ResourceType::Copper => {
                // Porphyry copper: large diffuse blob
                VeinShape::Blob {
                    center,
                    radius: 30.0 + random_from_seed(seed) * 50.0,
                    noise_distortion: 0.5,
                }
            }
            _ => {
                // Default: blob
                VeinShape::Blob {
                    center,
                    radius: 5.0 + random_from_seed(seed) * 15.0,
                    noise_distortion: 0.2,
                }
            }
        }
    }

    fn calculate_richness(
        &self,
        formation: &ResourceFormation,
        layer: &GeologicalLayer,
        depth: f32,
        seed: u64,
    ) -> f32 {
        // Base richness from formation
        let base = 0.5 + random_from_seed(seed) * 0.5;

        // Depth bonus (deeper often = richer for some resources)
        let depth_factor = match formation.resource {
            ResourceType::Gold | ResourceType::Diamond => {
                // Richer at depth
                (depth / 500.0).min(1.5)
            }
            ResourceType::Coal => {
                // Best at medium depth
                1.0 - ((depth - 100.0) / 200.0).abs().min(0.5)
            }
            _ => 1.0,
        };

        // Context bonus
        let context_factor = if formation.contexts[0] == layer.context {
            1.2 // Primary context
        } else {
            0.8 // Secondary context
        };

        (base * depth_factor * context_factor).clamp(0.1, 2.0)
    }
}
```

---

## Resource Types

### Resource Categories

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceCategory {
    Metal,
    Crystal,
    Fuel,
    Fluid,
    Rare,
    Organic,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceType {
    // Metals (common to rare)
    Iron,
    Copper,
    Aluminum,
    Zinc,
    Lead,
    Nickel,
    Titanium,
    Tungsten,
    Gold,
    Platinum,

    // Crystals
    Quartz,
    Silicon,
    Ruby,
    Sapphire,
    Diamond,

    // Fuels
    Coal,
    Oil,
    NaturalGas,
    Uranium,

    // Fluids
    Water,
    Brine,
    Sulfur,

    // Rare/Exotic
    RareEarth,
    Lithium,
    Helium3,      // If sci-fi

    // Organic (if applicable)
    Peat,
    Guano,
}

impl ResourceType {
    pub fn category(&self) -> ResourceCategory {
        match self {
            Self::Iron | Self::Copper | Self::Aluminum | Self::Zinc |
            Self::Lead | Self::Nickel | Self::Titanium | Self::Tungsten |
            Self::Gold | Self::Platinum => ResourceCategory::Metal,

            Self::Quartz | Self::Silicon | Self::Ruby |
            Self::Sapphire | Self::Diamond => ResourceCategory::Crystal,

            Self::Coal | Self::Oil | Self::NaturalGas |
            Self::Uranium => ResourceCategory::Fuel,

            Self::Water | Self::Brine | Self::Sulfur => ResourceCategory::Fluid,

            Self::RareEarth | Self::Lithium | Self::Helium3 => ResourceCategory::Rare,

            Self::Peat | Self::Guano => ResourceCategory::Organic,
        }
    }

    pub fn base_value(&self) -> f32 {
        match self {
            // Common
            Self::Iron => 1.0,
            Self::Copper => 2.0,
            Self::Coal => 0.5,
            Self::Water => 0.1,

            // Medium
            Self::Aluminum => 3.0,
            Self::Nickel => 4.0,
            Self::Silicon => 3.0,
            Self::Oil => 2.0,

            // Valuable
            Self::Titanium => 10.0,
            Self::Tungsten => 15.0,
            Self::Lithium => 20.0,
            Self::Uranium => 50.0,

            // Precious
            Self::Gold => 100.0,
            Self::Platinum => 150.0,
            Self::Diamond => 200.0,
            Self::RareEarth => 80.0,

            _ => 1.0,
        }
    }

    pub fn hardness(&self) -> f32 {
        // Mining difficulty multiplier
        match self {
            Self::Coal | Self::Peat => 0.3,
            Self::Iron | Self::Copper => 0.5,
            Self::Gold | Self::Platinum => 0.6, // Soft metals
            Self::Quartz | Self::Silicon => 0.8,
            Self::Titanium | Self::Tungsten => 1.2,
            Self::Diamond => 2.0,
            _ => 0.7,
        }
    }

    pub fn visual_color(&self) -> [f32; 3] {
        match self {
            Self::Iron => [0.5, 0.3, 0.2],      // Rusty brown
            Self::Copper => [0.8, 0.5, 0.2],    // Orange-brown
            Self::Gold => [1.0, 0.8, 0.2],      // Gold
            Self::Coal => [0.1, 0.1, 0.1],      // Black
            Self::Diamond => [0.9, 0.95, 1.0],  // Clear/white
            Self::Uranium => [0.3, 0.8, 0.3],   // Green glow
            Self::Ruby => [0.8, 0.1, 0.2],      // Red
            Self::Sapphire => [0.2, 0.3, 0.9],  // Blue
            Self::Quartz => [0.9, 0.9, 0.95],   // White
            Self::Oil => [0.05, 0.05, 0.05],    // Black
            _ => [0.5, 0.5, 0.5],               // Gray default
        }
    }
}
```

### Resource Properties

```rust
pub struct ResourceProperties {
    pub resource_type: ResourceType,

    // Physical properties
    pub density: f32,           // kg/m³
    pub melting_point: f32,     // °C (for processing)

    // Gameplay properties
    pub stack_size: u32,
    pub requires_processing: bool,
    pub processing_outputs: Vec<(ResourceType, f32)>,

    // Environmental
    pub radioactive: bool,
    pub toxic: bool,
    pub flammable: bool,
}

impl ResourceProperties {
    pub fn iron() -> Self {
        Self {
            resource_type: ResourceType::Iron,
            density: 7874.0,
            melting_point: 1538.0,
            stack_size: 100,
            requires_processing: true,
            processing_outputs: vec![
                (ResourceType::Iron, 0.7), // 70% yield after smelting
            ],
            radioactive: false,
            toxic: false,
            flammable: false,
        }
    }

    pub fn uranium() -> Self {
        Self {
            resource_type: ResourceType::Uranium,
            density: 19100.0,
            melting_point: 1132.0,
            stack_size: 20,
            requires_processing: true,
            processing_outputs: vec![
                (ResourceType::Uranium, 0.5), // Heavy processing
            ],
            radioactive: true,
            toxic: true,
            flammable: false,
        }
    }
}
```

---

## Terrain Integration

### Material ID Encoding

```rust
/// Encode resource information in terrain voxel
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VoxelMaterial {
    /// Base rock/soil type (0-255)
    pub base_material: u8,

    /// Resource type (0 = none, 1-255 = resource ID)
    pub resource: u8,

    /// Resource concentration (0-255 mapped to 0.0-1.0)
    pub concentration: u8,

    /// Flags (discovered, being mined, etc.)
    pub flags: u8,
}

impl VoxelMaterial {
    pub const FLAG_DISCOVERED: u8 = 0b0000_0001;
    pub const FLAG_BEING_MINED: u8 = 0b0000_0010;
    pub const FLAG_DEPLETED: u8 = 0b0000_0100;

    pub fn has_resource(&self) -> bool {
        self.resource != 0 && self.concentration > 0
    }

    pub fn resource_amount(&self) -> f32 {
        self.concentration as f32 / 255.0
    }

    pub fn set_concentration(&mut self, amount: f32) {
        self.concentration = (amount.clamp(0.0, 1.0) * 255.0) as u8;
    }

    pub fn is_discovered(&self) -> bool {
        self.flags & Self::FLAG_DISCOVERED != 0
    }

    pub fn mark_discovered(&mut self) {
        self.flags |= Self::FLAG_DISCOVERED;
    }
}
```

### Resource in SDF System

```wgsl
// In density generation shader
struct VoxelData {
    density: f32,
    material: u32,  // Packed VoxelMaterial
}

fn pack_material(base: u32, resource: u32, concentration: f32, flags: u32) -> u32 {
    let conc_u8 = u32(concentration * 255.0);
    return base | (resource << 8u) | (conc_u8 << 16u) | (flags << 24u);
}

fn unpack_resource(material: u32) -> u32 {
    return (material >> 8u) & 0xFFu;
}

fn unpack_concentration(material: u32) -> f32 {
    return f32((material >> 16u) & 0xFFu) / 255.0;
}

// During terrain generation, sample resource veins
fn generate_voxel(world_pos: vec3<f32>) -> VoxelData {
    // Base terrain density
    let terrain_density = terrain_sdf(world_pos);

    // Determine base material from geology
    let base_material = geological_material(world_pos);

    // Sample all resource veins
    var resource_type = 0u;
    var resource_concentration = 0.0;

    for (var i = 0u; i < vein_count; i++) {
        let vein = veins[i];
        let density = sample_vein(world_pos, vein);

        if (density > resource_concentration) {
            resource_type = vein.resource_type;
            resource_concentration = density * vein.richness;
        }
    }

    // Pack material data
    let material = pack_material(base_material, resource_type, resource_concentration, 0u);

    return VoxelData(terrain_density, material);
}
```

### Visual Representation

```wgsl
// Fragment shader: show resource veins in terrain
@fragment
fn fs_terrain(
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) material_packed: u32,
) -> @location(0) vec4<f32> {
    let base_material = material_packed & 0xFFu;
    let resource = unpack_resource(material_packed);
    let concentration = unpack_concentration(material_packed);

    // Base color from rock type
    var color = get_rock_color(base_material);

    // Blend in resource color if present and discovered
    if (resource > 0u && is_discovered(material_packed)) {
        let resource_color = get_resource_color(resource);
        let blend = concentration * 0.8;  // Max 80% resource color
        color = mix(color, resource_color, blend);

        // Add sparkle/glow for valuable resources
        if (is_precious(resource)) {
            let sparkle = sin(world_pos.x * 10.0 + time) * sin(world_pos.z * 10.0);
            color += vec3<f32>(0.2) * max(0.0, sparkle) * concentration;
        }
    }

    // Standard lighting
    let lighting = calculate_lighting(world_pos, normal);
    color *= lighting;

    return vec4<f32>(color, 1.0);
}
```

---

## Extraction Mechanics

### Mining System

```rust
pub struct MiningOperation {
    pub target_voxel: IVec3,
    pub tool: MiningTool,
    pub progress: f32,        // 0.0 - 1.0
    pub resources_pending: Vec<(ResourceType, f32)>,
}

pub struct MiningTool {
    pub tool_type: ToolType,
    pub mining_power: f32,    // Damage per second
    pub efficiency: f32,      // % of resource extracted (vs lost)
    pub radius: f32,          // Area of effect
    pub energy_cost: f32,     // Power consumption
}

#[derive(Clone, Copy)]
pub enum ToolType {
    Pickaxe,        // Manual, precise
    Drill,          // Powered, medium
    Excavator,      // Large vehicle
    LaserCutter,    // High tech, efficient
    Blasting,       // Explosive, fast but lossy
}

impl ToolType {
    pub fn base_stats(&self) -> MiningTool {
        match self {
            Self::Pickaxe => MiningTool {
                tool_type: *self,
                mining_power: 1.0,
                efficiency: 0.9,
                radius: 0.5,
                energy_cost: 0.0,
            },
            Self::Drill => MiningTool {
                tool_type: *self,
                mining_power: 5.0,
                efficiency: 0.85,
                radius: 1.0,
                energy_cost: 10.0,
            },
            Self::Excavator => MiningTool {
                tool_type: *self,
                mining_power: 20.0,
                efficiency: 0.7,
                radius: 3.0,
                energy_cost: 100.0,
            },
            Self::LaserCutter => MiningTool {
                tool_type: *self,
                mining_power: 10.0,
                efficiency: 0.95,
                radius: 0.3,
                energy_cost: 50.0,
            },
            Self::Blasting => MiningTool {
                tool_type: *self,
                mining_power: 100.0,
                efficiency: 0.5,
                radius: 5.0,
                energy_cost: 0.0, // Uses explosive items
            },
        }
    }
}

impl MiningOperation {
    pub fn update(&mut self, dt: f32, terrain: &mut Terrain) -> Vec<(ResourceType, f32)> {
        let voxel = terrain.get_voxel(self.target_voxel);
        let material = VoxelMaterial::from_packed(voxel.material);

        // Calculate mining progress
        let hardness = material.base_material_hardness()
                     * ResourceType::from_id(material.resource).hardness();

        let progress_delta = (self.tool.mining_power * dt) / (hardness * 100.0);
        self.progress += progress_delta;

        // Extract resources as we mine
        if material.has_resource() {
            let extracted = material.resource_amount() * progress_delta * self.tool.efficiency;
            self.resources_pending.push((
                ResourceType::from_id(material.resource),
                extracted,
            ));
        }

        // Complete mining
        if self.progress >= 1.0 {
            // Remove voxel from terrain (apply delta)
            terrain.remove_voxel(self.target_voxel);

            // Return all pending resources
            std::mem::take(&mut self.resources_pending)
        } else {
            Vec::new()
        }
    }
}
```

### Terrain Modification

```rust
impl Terrain {
    /// Remove voxel and update SDF
    pub fn remove_voxel(&mut self, pos: IVec3) {
        // Create removal delta
        let delta = TerrainDelta {
            position: pos,
            delta_type: DeltaType::Remove(1.0),
            material: VoxelMaterial::default().pack(),
        };

        self.delta_storage.add(delta);

        // Mark chunk for regeneration
        let chunk_id = self.position_to_chunk(pos);
        self.dirty_chunks.insert(chunk_id);
    }

    /// Get resource at position
    pub fn resource_at(&self, pos: IVec3) -> Option<(ResourceType, f32)> {
        let voxel = self.get_voxel(pos);
        let material = VoxelMaterial::from_packed(voxel.material);

        if material.has_resource() {
            Some((
                ResourceType::from_id(material.resource),
                material.resource_amount(),
            ))
        } else {
            None
        }
    }

    /// Calculate total resources in a region
    pub fn resources_in_region(&self, bounds: Aabb) -> HashMap<ResourceType, f32> {
        let mut totals = HashMap::new();

        for z in bounds.min.z as i32..bounds.max.z as i32 {
            for y in bounds.min.y as i32..bounds.max.y as i32 {
                for x in bounds.min.x as i32..bounds.max.x as i32 {
                    if let Some((resource, amount)) = self.resource_at(IVec3::new(x, y, z)) {
                        *totals.entry(resource).or_insert(0.0) += amount;
                    }
                }
            }
        }

        totals
    }
}
```

---

## Finite Economy

### Resource Tracking

```rust
/// Global resource tracking for the world
pub struct WorldResources {
    /// Total resources that existed at world generation
    initial_totals: HashMap<ResourceType, f64>,

    /// Resources currently remaining (not extracted)
    remaining: HashMap<ResourceType, f64>,

    /// Resources extracted by player
    extracted: HashMap<ResourceType, f64>,

    /// Resources lost (inefficient mining, destruction)
    lost: HashMap<ResourceType, f64>,

    /// Resources in player inventory/storage
    stored: HashMap<ResourceType, f64>,

    /// Resources used/consumed
    consumed: HashMap<ResourceType, f64>,
}

impl WorldResources {
    pub fn on_extraction(&mut self, resource: ResourceType, amount: f64, efficiency: f32) {
        let actually_extracted = amount * efficiency as f64;
        let lost_amount = amount * (1.0 - efficiency) as f64;

        *self.remaining.entry(resource).or_insert(0.0) -= amount;
        *self.extracted.entry(resource).or_insert(0.0) += actually_extracted;
        *self.lost.entry(resource).or_insert(0.0) += lost_amount;
        *self.stored.entry(resource).or_insert(0.0) += actually_extracted;
    }

    pub fn on_consumption(&mut self, resource: ResourceType, amount: f64) {
        *self.stored.entry(resource).or_insert(0.0) -= amount;
        *self.consumed.entry(resource).or_insert(0.0) += amount;
    }

    pub fn depletion_percentage(&self, resource: ResourceType) -> f32 {
        let initial = *self.initial_totals.get(&resource).unwrap_or(&1.0);
        let remaining = *self.remaining.get(&resource).unwrap_or(&0.0);
        ((initial - remaining) / initial * 100.0) as f32
    }

    pub fn estimated_remaining(&self, resource: ResourceType) -> f64 {
        *self.remaining.get(&resource).unwrap_or(&0.0)
    }

    /// Generate report for player
    pub fn generate_report(&self) -> ResourceReport {
        ResourceReport {
            by_resource: self.remaining.keys().map(|&r| {
                ResourceStatus {
                    resource: r,
                    initial: *self.initial_totals.get(&r).unwrap_or(&0.0),
                    remaining: *self.remaining.get(&r).unwrap_or(&0.0),
                    extracted: *self.extracted.get(&r).unwrap_or(&0.0),
                    stored: *self.stored.get(&r).unwrap_or(&0.0),
                    consumed: *self.consumed.get(&r).unwrap_or(&0.0),
                    lost: *self.lost.get(&r).unwrap_or(&0.0),
                }
            }).collect(),
        }
    }
}
```

### Scarcity Events

```rust
/// Track resource scarcity for gameplay events
pub struct ScarcityTracker {
    thresholds: HashMap<ResourceType, Vec<ScarcityThreshold>>,
    triggered_events: HashSet<(ResourceType, u32)>,
}

pub struct ScarcityThreshold {
    pub depletion_percent: f32,
    pub event_id: u32,
    pub message: String,
    pub gameplay_effect: ScarcityEffect,
}

pub enum ScarcityEffect {
    /// Just a warning
    Warning,
    /// Price increase in trading
    PriceMultiplier(f32),
    /// Unlock alternative technology
    UnlockTech(TechId),
    /// Trigger environmental event
    EnvironmentalEvent(EventId),
    /// Game ending condition
    CriticalShortage,
}

impl ScarcityTracker {
    pub fn check_thresholds(&mut self, resources: &WorldResources) -> Vec<ScarcityEvent> {
        let mut events = Vec::new();

        for (&resource, thresholds) in &self.thresholds {
            let depletion = resources.depletion_percentage(resource);

            for threshold in thresholds {
                let event_key = (resource, threshold.event_id);

                if depletion >= threshold.depletion_percent
                   && !self.triggered_events.contains(&event_key)
                {
                    self.triggered_events.insert(event_key);
                    events.push(ScarcityEvent {
                        resource,
                        threshold: threshold.clone(),
                        current_depletion: depletion,
                    });
                }
            }
        }

        events
    }
}
```

---

## Scanning & Discovery

### Scanner System

```rust
pub struct ResourceScanner {
    pub scanner_type: ScannerType,
    pub range: f32,
    pub resolution: f32,      // Minimum detectable vein size
    pub scan_speed: f32,      // Area per second
    pub energy_cost: f32,
}

#[derive(Clone, Copy)]
pub enum ScannerType {
    Handheld,       // Short range, low resolution
    Vehicle,        // Medium range, good resolution
    Stationary,     // Long range, high resolution
    Satellite,      // Orbital scan, surface only
    DeepSonar,      // Very deep, low resolution
}

pub struct ScanResult {
    pub position: Vec3,
    pub resource: ResourceType,
    pub estimated_size: VeinSizeEstimate,
    pub estimated_richness: RichnessEstimate,
    pub confidence: f32,
}

#[derive(Clone, Copy)]
pub enum VeinSizeEstimate {
    Trace,          // < 100 units
    Small,          // 100 - 1000 units
    Medium,         // 1000 - 10000 units
    Large,          // 10000 - 100000 units
    Massive,        // > 100000 units
}

impl ResourceScanner {
    pub fn scan_area(
        &self,
        center: Vec3,
        terrain: &Terrain,
        duration: f32,
    ) -> Vec<ScanResult> {
        let scan_radius = (self.scan_speed * duration).sqrt();
        let effective_range = scan_radius.min(self.range);

        let mut results = Vec::new();

        // Sample terrain at resolution intervals
        let samples_per_axis = (effective_range * 2.0 / self.resolution) as i32;

        for x in 0..samples_per_axis {
            for z in 0..samples_per_axis {
                let offset = Vec3::new(
                    (x as f32 - samples_per_axis as f32 / 2.0) * self.resolution,
                    0.0,
                    (z as f32 - samples_per_axis as f32 / 2.0) * self.resolution,
                );

                let sample_pos = center + offset;

                // Scan down through depth
                for depth in (0..self.range as i32).step_by(self.resolution as usize) {
                    let pos = sample_pos - Vec3::Y * depth as f32;

                    if let Some((resource, concentration)) = terrain.resource_at(pos.as_ivec3()) {
                        if concentration > 0.1 { // Minimum detectable
                            // Estimate vein properties
                            let vein_info = self.estimate_vein(pos, terrain, resource);

                            // Add noise based on scanner quality
                            let noise = 1.0 - self.confidence_at_distance(pos.distance(center));

                            results.push(ScanResult {
                                position: pos + random_vec3() * noise * 10.0,
                                resource,
                                estimated_size: vein_info.0,
                                estimated_richness: vein_info.1,
                                confidence: (1.0 - noise).clamp(0.0, 1.0),
                            });
                        }
                    }
                }
            }
        }

        // Deduplicate nearby results (merge into single vein detection)
        self.merge_nearby_results(results)
    }

    fn estimate_vein(
        &self,
        pos: Vec3,
        terrain: &Terrain,
        resource: ResourceType,
    ) -> (VeinSizeEstimate, RichnessEstimate) {
        // Sample nearby to estimate size
        let mut total_amount = 0.0;
        let sample_radius = 20.0;

        for _ in 0..50 {
            let offset = random_in_sphere() * sample_radius;
            if let Some((r, amount)) = terrain.resource_at((pos + offset).as_ivec3()) {
                if r == resource {
                    total_amount += amount;
                }
            }
        }

        let size = match total_amount {
            x if x < 5.0 => VeinSizeEstimate::Trace,
            x if x < 20.0 => VeinSizeEstimate::Small,
            x if x < 100.0 => VeinSizeEstimate::Medium,
            x if x < 500.0 => VeinSizeEstimate::Large,
            _ => VeinSizeEstimate::Massive,
        };

        let richness = match total_amount / 50.0 {
            x if x < 0.2 => RichnessEstimate::Poor,
            x if x < 0.5 => RichnessEstimate::Average,
            x if x < 0.8 => RichnessEstimate::Rich,
            _ => RichnessEstimate::VeryRich,
        };

        (size, richness)
    }
}
```

### Discovery UI Integration

```rust
/// Player's resource discovery state
pub struct PlayerDiscoveries {
    /// Scanned resource locations
    pub scan_results: Vec<ScanResult>,

    /// Veins the player has actually seen/mined
    pub confirmed_veins: Vec<ConfirmedVein>,

    /// Map overlay data (for minimap/map screen)
    pub map_data: ResourceMapData,
}

pub struct ConfirmedVein {
    pub position: Vec3,
    pub resource: ResourceType,
    pub estimated_total: f32,
    pub extracted_so_far: f32,
    pub last_visited: f64, // Game time
}

impl PlayerDiscoveries {
    /// Mark terrain as discovered when player sees it
    pub fn on_terrain_visible(&mut self, terrain: &mut Terrain, visible_chunks: &[ChunkId]) {
        for chunk_id in visible_chunks {
            // Mark all resource voxels in chunk as discovered
            terrain.mark_resources_discovered(*chunk_id);
        }
    }

    /// Add scan result to player knowledge
    pub fn add_scan_result(&mut self, result: ScanResult) {
        // Check for duplicates
        let dominated = self.scan_results.iter().any(|r| {
            r.resource == result.resource
            && r.position.distance(result.position) < 20.0
            && r.confidence >= result.confidence
        });

        if !dominated {
            // Remove older, less confident results for same vein
            self.scan_results.retain(|r| {
                r.resource != result.resource
                || r.position.distance(result.position) >= 20.0
                || r.confidence > result.confidence
            });

            self.scan_results.push(result);
            self.update_map_data();
        }
    }
}
```

---

## Implementation Notes

### Suggested Crate Structure

```
engine/crates/syn_resources/
├── src/
│   ├── lib.rs
│   ├── types.rs          # ResourceType, properties
│   ├── geology/
│   │   ├── mod.rs
│   │   ├── context.rs    # GeologicalContext
│   │   ├── profile.rs    # GeologicalProfile
│   │   └── formation.rs  # ResourceFormation
│   ├── generation/
│   │   ├── mod.rs
│   │   ├── vein.rs       # VeinShape, VeinGenerator
│   │   └── distribution.rs
│   ├── extraction/
│   │   ├── mod.rs
│   │   ├── mining.rs     # MiningOperation
│   │   └── tools.rs      # MiningTool
│   ├── economy/
│   │   ├── mod.rs
│   │   ├── tracking.rs   # WorldResources
│   │   └── scarcity.rs   # ScarcityTracker
│   ├── scanning/
│   │   ├── mod.rs
│   │   └── scanner.rs    # ResourceScanner
│   └── integration/
│       └── terrain.rs    # Terrain integration
```

### Performance Budget

```
Resource operations:
├── Vein generation (world gen): 1-5ms per chunk
├── Resource sampling (runtime): 0.01ms per voxel
├── Mining update: 0.1ms per active operation
├── Scanning: 1-10ms depending on area
└── Economy update: 0.1ms per frame
```

---

## References

### Geology

1. **Economic Geology** (textbook) - Ore deposit formation
2. **USGS Resources** - Mineral deposit models
3. **Minecraft Wiki** - Game design reference for ore distribution

### Game Design

- **Factorio** - Resource patches and depletion
- **Satisfactory** - Node-based resource extraction
- **Dwarf Fortress** - Geological layers and minerals

---

## Ideas & Future Work

### To Research

- [ ] **Procedural geology**: Full tectonic history → realistic ore placement
- [ ] **Fluid dynamics**: Oil/gas reservoir simulation
- [ ] **Renewable resources**: Regrowth of organic materials
- [ ] **Secondary deposits**: Resources from player waste/pollution
- [ ] **Asteroid mining**: Extension for space resources

### Gameplay Ideas

- [ ] **Surveying profession**: Detailed scanning gameplay
- [ ] **Black market**: Trade in rare resources
- [ ] **Environmental regulations**: Penalties for over-extraction
- [ ] **Resource wars**: Multiplayer competition for deposits

### Notes

```
2026-01-23: Initial document creation
- Focus on Groundbreak requirements
- Geological realism balanced with gameplay
- Need to integrate with pollution system later
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
