# Terrain Generation - Research Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Active Research
**Crates**: `syn_terrain`, `syn_procgen`

---

## Table of Contents

1. [Overview](#overview)
2. [Signed Distance Fields (SDF)](#signed-distance-fields-sdf)
3. [Mesh Generation Algorithms](#mesh-generation-algorithms)
4. [Cube-Sphere Planetary Projection](#cube-sphere-planetary-projection)
5. [GPU-Driven Pipeline](#gpu-driven-pipeline)
6. [Level of Detail (LOD)](#level-of-detail-lod)
7. [Terrain Modification](#terrain-modification)
8. [Noise Functions](#noise-functions)
9. [Implementation Notes](#implementation-notes)
10. [References](#references)
11. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### Goals

1. **Volumetric terrain**: Caves, overhangs, arches - not just heightmaps
2. **Planetary scale**: From space (10,000 km) to ground (1 cm)
3. **Real-time modification**: Dig, fill, sculpt at runtime
4. **GPU-driven**: CPU orchestrates, GPU generates

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TERRAIN PIPELINE                                 │
│                                                                          │
│   CPU Side                           │   GPU Side                        │
│   ─────────                          │   ────────                        │
│                                      │                                   │
│   ┌──────────────┐                   │   ┌──────────────────────┐       │
│   │ Chunk        │  Request list     │   │ Density Generation   │       │
│   │ Management   │ ─────────────────▶│   │ (Compute Shader)     │       │
│   └──────────────┘                   │   └──────────┬───────────┘       │
│         │                            │              │                    │
│   ┌─────▼────────┐                   │   ┌──────────▼───────────┐       │
│   │ Frustum      │                   │   │ Polygonization       │       │
│   │ Culling      │                   │   │ (Marching Cubes)     │       │
│   └──────────────┘                   │   └──────────┬───────────┘       │
│         │                            │              │                    │
│   ┌─────▼────────┐                   │   ┌──────────▼───────────┐       │
│   │ LOD          │                   │   │ Instance Scattering  │       │
│   │ Selection    │                   │   │ (Vegetation, etc.)   │       │
│   └──────────────┘                   │   └──────────┬───────────┘       │
│         │                            │              │                    │
│   ┌─────▼────────┐                   │   ┌──────────▼───────────┐       │
│   │ Priority     │                   │   │ DrawIndirect         │       │
│   │ Queue        │                   │   │ Rendering            │       │
│   └──────────────┘                   │   └──────────────────────┘       │
│                                      │                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Signed Distance Fields (SDF)

### Concept

A Signed Distance Field stores, for each point in space, the **distance to the nearest surface**:

- **Positive**: Outside the surface (air)
- **Negative**: Inside the surface (solid)
- **Zero**: On the surface

```
     Air (+)
        │
        ▼
  ──────────────── Surface (SDF = 0)
        │
        ▼
    Solid (-)
```

### Advantages over Heightmaps

| Feature | Heightmap | SDF |
|---------|-----------|-----|
| Caves | No | Yes |
| Overhangs | No | Yes |
| Floating islands | No | Yes |
| CSG operations | Complex | Trivial |
| Memory | 2D (efficient) | 3D (more memory) |

### SDF Primitives

```rust
/// Distance to a sphere centered at origin
pub fn sdf_sphere(p: Vec3, radius: f32) -> f32 {
    p.length() - radius
}

/// Distance to an infinite plane
pub fn sdf_plane(p: Vec3, normal: Vec3, offset: f32) -> f32 {
    p.dot(normal) + offset
}

/// Distance to a box centered at origin
pub fn sdf_box(p: Vec3, half_extents: Vec3) -> f32 {
    let q = p.abs() - half_extents;
    q.max(Vec3::ZERO).length() + q.max_element().min(0.0)
}

/// Distance to a capsule (cylinder with rounded ends)
pub fn sdf_capsule(p: Vec3, a: Vec3, b: Vec3, radius: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba * h).length() - radius
}

/// Distance to a torus (donut shape)
pub fn sdf_torus(p: Vec3, major_radius: f32, minor_radius: f32) -> f32 {
    let q = Vec2::new(Vec2::new(p.x, p.z).length() - major_radius, p.y);
    q.length() - minor_radius
}
```

### SDF Operations (CSG)

```rust
/// Boolean union (OR): combine two shapes
pub fn op_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}

/// Boolean intersection (AND): only where both shapes overlap
pub fn op_intersection(d1: f32, d2: f32) -> f32 {
    d1.max(d2)
}

/// Boolean subtraction: remove d2 from d1
pub fn op_subtraction(d1: f32, d2: f32) -> f32 {
    d1.max(-d2)
}

/// Smooth union: blend shapes together
pub fn op_smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d2.lerp(d1, h) - k * h * (1.0 - h)
}

/// Smooth subtraction: smooth carving
pub fn op_smooth_subtraction(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 - 0.5 * (d2 + d1) / k).clamp(0.0, 1.0);
    d1.lerp(-d2, h) + k * h * (1.0 - h)
}
```

### Terrain as SDF

```rust
/// Terrain SDF: base height + 3D noise for caves
pub fn terrain_sdf(p: Vec3, terrain: &TerrainParams) -> f32 {
    // Base terrain height from 2D heightmap or noise
    let height_2d = sample_heightmap(p.xz(), terrain);

    // Distance to ground plane at this height
    let ground_distance = p.y - height_2d;

    // Add 3D caves/overhangs
    let cave_noise = fbm_3d(p * terrain.cave_scale, terrain.cave_octaves);
    let cave_contribution = (cave_noise - terrain.cave_threshold) * terrain.cave_strength;

    // Combine: positive = air, negative = solid
    ground_distance + cave_contribution
}
```

---

## Mesh Generation Algorithms

### Marching Cubes

The standard algorithm for extracting surfaces from volumetric data.

#### Algorithm Overview

1. Divide space into a regular grid of cubes
2. For each cube, determine which corners are inside/outside
3. This gives 256 possible configurations (2^8 corners)
4. Look up triangle configuration in precomputed table
5. Interpolate vertex positions along edges

#### Pseudocode

```
function marching_cubes(density_field, isolevel=0):
    vertices = []
    indices = []

    for each cube in grid:
        // 1. Sample 8 corners
        corner_values[8] = sample_corners(cube, density_field)

        // 2. Determine cube index (which corners are inside)
        cube_index = 0
        for i in 0..8:
            if corner_values[i] < isolevel:
                cube_index |= (1 << i)

        // 3. Skip if entirely inside or outside
        if edge_table[cube_index] == 0:
            continue

        // 4. Find vertices on edges
        edge_vertices[12] = {}
        for edge in 0..12:
            if edge_table[cube_index] & (1 << edge):
                // Interpolate position along edge
                v1, v2 = edge_endpoints(edge)
                t = (isolevel - corner_values[v1]) / (corner_values[v2] - corner_values[v1])
                edge_vertices[edge] = lerp(cube.corner(v1), cube.corner(v2), t)

        // 5. Create triangles from lookup table
        for i in 0..15 step 3:
            if tri_table[cube_index][i] == -1:
                break
            a = edge_vertices[tri_table[cube_index][i]]
            b = edge_vertices[tri_table[cube_index][i+1]]
            c = edge_vertices[tri_table[cube_index][i+2]]
            add_triangle(vertices, indices, a, b, c)

    return Mesh { vertices, indices }
```

#### Rust Prototype (GPU Compute)

```rust
// WGSL Compute Shader for Marching Cubes
const EDGE_TABLE: array<u32, 256> = array<u32, 256>(...);  // Precomputed
const TRI_TABLE: array<array<i32, 16>, 256> = array<...>(...);

struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> density: array<f32>;
@group(0) @binding(1) var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(2) var<storage, read_write> vertex_count: atomic<u32>;

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid_size = vec3<u32>(CHUNK_SIZE, CHUNK_SIZE, CHUNK_SIZE);
    if (any(id >= grid_size - 1u)) {
        return;
    }

    // Sample 8 corners of this cube
    var corner_values: array<f32, 8>;
    for (var i = 0u; i < 8u; i++) {
        let offset = corner_offset(i);
        let sample_pos = id + offset;
        let index = sample_pos.x + sample_pos.y * grid_size.x + sample_pos.z * grid_size.x * grid_size.y;
        corner_values[i] = density[index];
    }

    // Calculate cube configuration
    var cube_index = 0u;
    for (var i = 0u; i < 8u; i++) {
        if (corner_values[i] < 0.0) {
            cube_index |= (1u << i);
        }
    }

    // Skip empty cubes
    if (EDGE_TABLE[cube_index] == 0u) {
        return;
    }

    // Generate vertices and triangles...
    // (Full implementation in actual shader)
}
```

### Dual Contouring

An alternative that produces sharper features.

#### Key Difference from Marching Cubes

- **Marching Cubes**: Vertices on edges, may lose sharp features
- **Dual Contouring**: Vertices inside cells, uses Hermite data (position + normal)

#### Advantages

- Preserves sharp edges and corners
- Better for architectural/man-made terrain
- More expensive to compute

#### When to Use

| Scenario | Algorithm |
|----------|-----------|
| Organic terrain (caves, hills) | Marching Cubes |
| Sharp features (cliffs, buildings) | Dual Contouring |
| Real-time modification | Marching Cubes (faster) |
| Quality priority | Dual Contouring |

---

## Cube-Sphere Planetary Projection

### The Problem

How to represent a spherical planet without:

- Polar singularities (like UV sphere)
- Excessive distortion
- Complex math

### Solution: Cube-Sphere

Project a cube onto a sphere surface.

```
        ┌───────┐
        │  TOP  │
        │ (Y+)  │
┌───────┼───────┼───────┬───────┐
│ LEFT  │ FRONT │ RIGHT │ BACK  │
│ (X-)  │ (Z+)  │ (X+)  │ (Z-)  │
└───────┼───────┼───────┴───────┘
        │BOTTOM │
        │ (Y-)  │
        └───────┘
```

### Projection Methods

#### Simple Normalization

```rust
/// Project cube point to sphere (simple but uneven distribution)
pub fn cube_to_sphere_simple(cube_point: Vec3) -> Vec3 {
    cube_point.normalize()
}
```

#### Tangent-Based (Better Distribution)

```rust
/// Project cube point to sphere (more even distribution)
pub fn cube_to_sphere_tangent(cube_point: Vec3) -> Vec3 {
    let x2 = cube_point.x * cube_point.x;
    let y2 = cube_point.y * cube_point.y;
    let z2 = cube_point.z * cube_point.z;

    Vec3::new(
        cube_point.x * (1.0 - y2 / 2.0 - z2 / 2.0 + y2 * z2 / 3.0).sqrt(),
        cube_point.y * (1.0 - z2 / 2.0 - x2 / 2.0 + z2 * x2 / 3.0).sqrt(),
        cube_point.z * (1.0 - x2 / 2.0 - y2 / 2.0 + x2 * y2 / 3.0).sqrt(),
    )
}
```

### Face Addressing

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubeFace {
    PositiveX = 0,  // Right
    NegativeX = 1,  // Left
    PositiveY = 2,  // Top
    NegativeY = 3,  // Bottom
    PositiveZ = 4,  // Front
    NegativeZ = 5,  // Back
}

/// Tile address on a cube-sphere planet
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanetTileId {
    pub face: CubeFace,
    pub lod: u8,       // 0 = whole face, 1 = 4 tiles, etc.
    pub x: u32,        // X position within face at this LOD
    pub y: u32,        // Y position within face at this LOD
}

impl PlanetTileId {
    /// Number of tiles per face at this LOD
    pub fn tiles_per_axis(&self) -> u32 {
        1 << self.lod  // 2^lod
    }

    /// Convert to cube-space coordinates (before projection)
    pub fn to_cube_bounds(&self) -> (Vec3, Vec3) {
        let tiles = self.tiles_per_axis() as f32;
        let tile_size = 2.0 / tiles;  // Cube spans -1 to 1

        let min_u = -1.0 + self.x as f32 * tile_size;
        let max_u = min_u + tile_size;
        let min_v = -1.0 + self.y as f32 * tile_size;
        let max_v = min_v + tile_size;

        let (min, max) = match self.face {
            CubeFace::PositiveX => (Vec3::new(1.0, min_v, min_u), Vec3::new(1.0, max_v, max_u)),
            CubeFace::NegativeX => (Vec3::new(-1.0, min_v, -max_u), Vec3::new(-1.0, max_v, -min_u)),
            CubeFace::PositiveY => (Vec3::new(min_u, 1.0, min_v), Vec3::new(max_u, 1.0, max_v)),
            CubeFace::NegativeY => (Vec3::new(min_u, -1.0, -max_v), Vec3::new(max_u, -1.0, -min_v)),
            CubeFace::PositiveZ => (Vec3::new(min_u, min_v, 1.0), Vec3::new(max_u, max_v, 1.0)),
            CubeFace::NegativeZ => (Vec3::new(-max_u, min_v, -1.0), Vec3::new(-min_u, max_v, -1.0)),
        };

        (min, max)
    }
}
```

### Seamless Face Transitions

```rust
/// Get neighboring tile across face boundary
pub fn get_neighbor_tile(tile: &PlanetTileId, direction: Direction) -> PlanetTileId {
    let max_coord = tile.tiles_per_axis() - 1;

    match direction {
        Direction::PosX => {
            if tile.x < max_coord {
                PlanetTileId { x: tile.x + 1, ..*tile }
            } else {
                // Transition to adjacent face
                transition_to_adjacent_face(tile, direction)
            }
        }
        // ... similar for other directions
    }
}

fn transition_to_adjacent_face(tile: &PlanetTileId, direction: Direction) -> PlanetTileId {
    // Face adjacency and coordinate transformations
    // This requires careful handling of rotations
    match (tile.face, direction) {
        (CubeFace::PositiveZ, Direction::PosX) => {
            // Front → Right face
            PlanetTileId {
                face: CubeFace::PositiveX,
                lod: tile.lod,
                x: 0,
                y: tile.y,
            }
        }
        // ... all 24 transitions (6 faces × 4 directions)
        _ => *tile
    }
}
```

---

## GPU-Driven Pipeline

### Overview

Move as much work as possible to the GPU:

```
Frame N:
├── CPU: Determine visible chunks, submit to queue
├── GPU: Generate density for queued chunks (compute)
├── GPU: Run marching cubes (compute)
├── GPU: Scatter vegetation instances (compute)
├── GPU: Render all chunks (DrawIndirect)
└── CPU: Prepare Frame N+1 while GPU renders
```

### Chunk Request System

```rust
#[derive(Clone, Copy)]
pub struct ChunkRequest {
    pub tile_id: PlanetTileId,
    pub priority: f32,        // Higher = generate first
    pub frame_requested: u64,
}

pub struct ChunkManager {
    pending_requests: PriorityQueue<ChunkRequest>,
    generating: HashSet<PlanetTileId>,
    ready: HashMap<PlanetTileId, ChunkData>,

    // GPU resources
    density_buffer: Buffer,
    vertex_buffer: Buffer,
    indirect_buffer: Buffer,
}

impl ChunkManager {
    pub fn update(&mut self, camera: &Camera, planet: &Planet) {
        // 1. Determine what should be visible
        let visible_tiles = self.calculate_visible_tiles(camera, planet);

        // 2. Request missing tiles
        for tile in visible_tiles {
            if !self.ready.contains_key(&tile) && !self.generating.contains(&tile) {
                let priority = self.calculate_priority(&tile, camera);
                self.pending_requests.push(ChunkRequest {
                    tile_id: tile,
                    priority,
                    frame_requested: self.current_frame,
                });
            }
        }

        // 3. Remove tiles that are too far
        self.ready.retain(|tile, _| {
            let distance = self.tile_distance(tile, camera);
            distance < self.unload_distance
        });
    }

    fn calculate_priority(&self, tile: &PlanetTileId, camera: &Camera) -> f32 {
        let distance = self.tile_distance(tile, camera);
        let in_view_frustum = self.is_in_frustum(tile, camera);

        // Higher priority for closer tiles and tiles in view
        let base_priority = 1.0 / (distance + 1.0);
        if in_view_frustum { base_priority * 2.0 } else { base_priority }
    }
}
```

### Compute Shader Workflow

```rust
// Density generation dispatch
pub fn dispatch_density_generation(
    encoder: &mut CommandEncoder,
    chunk: &ChunkRequest,
    density_pipeline: &ComputePipeline,
    bind_group: &BindGroup,
) {
    let mut pass = encoder.begin_compute_pass(&Default::default());
    pass.set_pipeline(density_pipeline);
    pass.set_bind_group(0, bind_group, &[]);

    // Dispatch: CHUNK_SIZE / WORKGROUP_SIZE per dimension
    let workgroups = (CHUNK_SIZE / 8, CHUNK_SIZE / 8, CHUNK_SIZE / 8);
    pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
}

// Marching cubes dispatch
pub fn dispatch_marching_cubes(
    encoder: &mut CommandEncoder,
    chunk: &ChunkRequest,
    mc_pipeline: &ComputePipeline,
    bind_group: &BindGroup,
) {
    let mut pass = encoder.begin_compute_pass(&Default::default());
    pass.set_pipeline(mc_pipeline);
    pass.set_bind_group(0, bind_group, &[]);

    // Dispatch: (CHUNK_SIZE-1) cubes per dimension
    let workgroups = ((CHUNK_SIZE - 1) / 4, (CHUNK_SIZE - 1) / 4, (CHUNK_SIZE - 1) / 4);
    pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
}
```

### DrawIndirect for Variable Geometry

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

// The marching cubes shader writes vertex_count
// CPU never needs to know how many vertices were generated
pub fn render_terrain(
    render_pass: &mut RenderPass,
    terrain_pipeline: &RenderPipeline,
    vertex_buffer: &Buffer,
    indirect_buffer: &Buffer,
    chunk_count: u32,
) {
    render_pass.set_pipeline(terrain_pipeline);
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

    // One indirect draw per chunk
    for i in 0..chunk_count {
        let offset = i as u64 * std::mem::size_of::<DrawIndirectCommand>() as u64;
        render_pass.draw_indirect(indirect_buffer, offset);
    }
}
```

---

## Level of Detail (LOD)

### Quadtree LOD per Face

Each cube face uses a quadtree for LOD:

```rust
pub struct QuadtreeNode {
    pub tile: PlanetTileId,
    pub children: Option<Box<[QuadtreeNode; 4]>>,
    pub mesh_data: Option<MeshHandle>,
}

impl QuadtreeNode {
    pub fn should_split(&self, camera: &Camera, planet: &Planet) -> bool {
        let center = self.tile.center_world_position(planet.radius);
        let distance = (center - camera.position).length();

        // Split if close enough and not at max LOD
        let threshold = planet.lod_distances[self.tile.lod as usize];
        distance < threshold && self.tile.lod < MAX_LOD
    }

    pub fn update(&mut self, camera: &Camera, planet: &Planet, chunk_manager: &mut ChunkManager) {
        if self.should_split(camera, planet) {
            // Ensure we have children
            if self.children.is_none() {
                self.create_children();
            }

            // Recursively update children
            for child in self.children.as_mut().unwrap().iter_mut() {
                child.update(camera, planet, chunk_manager);
            }
        } else {
            // Collapse children if we have them
            if self.children.is_some() {
                self.collapse_children(chunk_manager);
            }

            // Ensure this node's mesh is loaded
            chunk_manager.request(self.tile);
        }
    }
}
```

### LOD Transition (Geomorphing)

Smooth transitions between LOD levels:

```rust
// In vertex shader
fn calculate_geomorph_position(
    position: vec3<f32>,
    parent_position: vec3<f32>,
    morph_factor: f32,  // 0 = this LOD, 1 = parent LOD
) -> vec3<f32> {
    return mix(position, parent_position, morph_factor);
}

// CPU side: calculate morph factor based on distance
pub fn calculate_morph_factor(
    tile: &PlanetTileId,
    camera: &Camera,
    planet: &Planet,
) -> f32 {
    let distance = tile_distance(tile, camera);
    let lod_distance = planet.lod_distances[tile.lod as usize];
    let next_lod_distance = planet.lod_distances[(tile.lod + 1) as usize];

    // Morph in the transition zone
    let morph_start = lod_distance * 0.8;
    let morph_end = lod_distance;

    ((distance - morph_start) / (morph_end - morph_start)).clamp(0.0, 1.0)
}
```

---

## Terrain Modification

### Delta Storage

Store modifications as a sparse octree of changes:

```rust
pub enum DeltaType {
    Add(f32),       // Add material (fill)
    Remove(f32),    // Remove material (dig)
    Set(f32),       // Set absolute value
}

pub struct TerrainDelta {
    pub position: IVec3,  // Voxel coordinates
    pub delta_type: DeltaType,
    pub material: MaterialId,
}

pub struct DeltaStorage {
    // Sparse octree for efficient storage and lookup
    root: OctreeNode<Vec<TerrainDelta>>,
    // Dirty chunks that need regeneration
    dirty_chunks: HashSet<PlanetTileId>,
}

impl DeltaStorage {
    pub fn apply_edit(&mut self, edit: TerrainEdit) {
        // Convert edit to deltas
        let deltas = edit.to_deltas();

        for delta in deltas {
            // Insert into octree
            self.root.insert(delta.position, delta);

            // Mark affected chunks dirty
            let chunk = position_to_chunk(delta.position);
            self.dirty_chunks.insert(chunk);
        }
    }

    pub fn get_deltas_for_chunk(&self, chunk: &PlanetTileId) -> Vec<&TerrainDelta> {
        let bounds = chunk.to_voxel_bounds();
        self.root.query_range(bounds)
    }
}
```

### Applying Deltas in Density Shader

```wgsl
@group(0) @binding(2) var<storage, read> deltas: array<Delta>;
@group(0) @binding(3) var<uniform> delta_count: u32;

fn sample_density_with_deltas(world_pos: vec3<f32>) -> f32 {
    // Base procedural density
    var density = procedural_terrain(world_pos);

    // Apply deltas
    for (var i = 0u; i < delta_count; i++) {
        let delta = deltas[i];
        let delta_pos = vec3<f32>(delta.position);
        let distance = length(world_pos - delta_pos);

        if (distance < delta.radius) {
            let falloff = smoothstep(delta.radius, 0.0, distance);
            switch (delta.delta_type) {
                case 0u: { // Add
                    density = density - delta.strength * falloff;
                }
                case 1u: { // Remove
                    density = density + delta.strength * falloff;
                }
                case 2u: { // Set
                    density = mix(density, delta.strength, falloff);
                }
            }
        }
    }

    return density;
}
```

---

## Noise Functions

### Perlin/Simplex Noise

```rust
/// 3D Simplex noise (implementation based on Stefan Gustavson's work)
pub fn simplex_3d(p: Vec3) -> f32 {
    // Skewing factors for 3D
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;

    // Skew input space to determine simplex cell
    let s = (p.x + p.y + p.z) * F3;
    let i = (p.x + s).floor();
    let j = (p.y + s).floor();
    let k = (p.z + s).floor();

    // Unskew cell origin back to (x,y,z) space
    let t = (i + j + k) * G3;
    let x0 = p.x - i + t;
    let y0 = p.y - j + t;
    let z0 = p.z - k + t;

    // Determine which simplex we're in
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 { (1, 0, 0, 1, 1, 0) }
        else if x0 >= z0 { (1, 0, 0, 1, 0, 1) }
        else { (0, 0, 1, 1, 0, 1) }
    } else {
        if y0 < z0 { (0, 0, 1, 0, 1, 1) }
        else if x0 < z0 { (0, 1, 0, 0, 1, 1) }
        else { (0, 1, 0, 1, 1, 0) }
    };

    // Calculate contributions from corners
    // ... (full implementation)

    // Return in range [-1, 1]
    32.0 * (n0 + n1 + n2 + n3)
}
```

### Fractal Brownian Motion (fBm)

```rust
/// Fractal Brownian Motion - layered noise
pub fn fbm_3d(
    p: Vec3,
    octaves: u32,
    lacunarity: f32,    // Frequency multiplier (usually 2.0)
    persistence: f32,   // Amplitude multiplier (usually 0.5)
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        value += simplex_3d(p * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    value / max_value  // Normalize to [-1, 1]
}
```

### Domain Warping

```rust
/// Warp coordinates before sampling noise for more organic results
pub fn domain_warp(p: Vec3, warp_strength: f32) -> Vec3 {
    let warp = Vec3::new(
        fbm_3d(p + Vec3::new(0.0, 0.0, 0.0), 4, 2.0, 0.5),
        fbm_3d(p + Vec3::new(5.2, 1.3, 2.8), 4, 2.0, 0.5),
        fbm_3d(p + Vec3::new(1.7, 9.2, 3.1), 4, 2.0, 0.5),
    );

    p + warp * warp_strength
}

pub fn warped_terrain(p: Vec3) -> f32 {
    let warped_p = domain_warp(p, 0.3);
    fbm_3d(warped_p, 6, 2.0, 0.5)
}
```

### Ridged Noise (for mountains)

```rust
/// Ridged multifractal noise - creates sharp ridges
pub fn ridged_fbm(
    p: Vec3,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut weight = 1.0;

    for _ in 0..octaves {
        // Absolute value creates ridges, invert so ridges are peaks
        let signal = 1.0 - simplex_3d(p * frequency).abs();
        // Square to sharpen ridges
        let signal = signal * signal;
        // Weight by previous octave
        let signal = signal * weight;
        // Update weight for next octave
        weight = (signal * gain).clamp(0.0, 1.0);

        value += signal * amplitude;
        amplitude *= 0.5;
        frequency *= lacunarity;
    }

    value
}
```

---

## Implementation Notes

### Memory Budget

| Component | Budget | Notes |
|-----------|--------|-------|
| Density buffer | 256 MB | Reused ring buffer |
| Vertex buffer | 512 MB | Pooled, reused |
| Texture cache | 1 GB | Terrain textures |
| Delta storage | 256 MB | Player modifications |

### Chunk Size Considerations

```
Chunk size: 32³ voxels
- Too small: Too many draw calls, LOD seams
- Too big: Wasted computation, slow modification

Optimal: 32-64 voxels per side
- 32³ = 32,768 samples
- At most ~5,000 triangles per chunk (dense terrain)
- Good balance for modern GPUs
```

### Suggested Crate Structure

```
syn_terrain/
├── src/
│   ├── lib.rs
│   ├── sdf/
│   │   ├── mod.rs
│   │   ├── primitives.rs     # SDF shapes
│   │   ├── operations.rs     # CSG ops
│   │   └── terrain.rs        # Terrain-specific SDF
│   ├── meshing/
│   │   ├── mod.rs
│   │   ├── marching_cubes.rs
│   │   ├── dual_contouring.rs
│   │   └── tables.rs         # Lookup tables
│   ├── planetary/
│   │   ├── mod.rs
│   │   ├── cube_sphere.rs    # Projection
│   │   ├── tile.rs           # Tile addressing
│   │   └── quadtree.rs       # LOD tree
│   ├── chunk/
│   │   ├── mod.rs
│   │   ├── manager.rs        # Chunk lifecycle
│   │   ├── request.rs        # Request queue
│   │   └── cache.rs          # LRU cache
│   ├── modification/
│   │   ├── mod.rs
│   │   ├── delta.rs          # Delta storage
│   │   └── edit.rs           # Edit operations
│   └── gpu/
│       ├── mod.rs
│       ├── pipelines.rs      # Compute/render pipelines
│       └── shaders/
│           ├── density.wgsl
│           ├── marching_cubes.wgsl
│           └── terrain.wgsl
└── tests/
```

---

## References

### Papers

1. **Marching Cubes**
   - Lorensen, W. E., & Cline, H. E. (1987). "Marching cubes: A high resolution 3D surface construction algorithm"

2. **Dual Contouring**
   - Ju, T., et al. (2002). "Dual Contouring of Hermite Data"

3. **GPU Terrain**
   - Strugar, F. (2009). "Continuous Distance-Dependent Level of Detail for Rendering Heightmaps"

### Game Development

1. **No Man's Sky**
   - GDC 2017: "Building Worlds with Noise"

2. **Dreams (Media Molecule)**
   - GDC 2015: "Learning from Failure: The Trials of Tearaway"

3. **Sebastian Lague**
   - YouTube series on procedural terrain

### Online Resources

- Inigo Quilez (SDF master): https://iquilezles.org/articles/
- GPU Gems 3: Chapter 1 (Procedural Terrain)
- Scratchapixel: Marching Cubes tutorial

---

## Ideas & Future Work

### To Research

- [ ] **Transvoxel**: Seamless LOD transitions without cracks
- [ ] **Clipmaps**: Alternative to quadtree for terrain LOD
- [ ] **Async compute**: Better overlap of density gen and rendering
- [ ] **Mesh simplification**: Reduce triangle count in flat areas
- [ ] **Material blending**: Triplanar mapping for terrain textures

### Optimization Ideas

- [ ] **Frustum-based generation**: Only generate visible chunks
- [ ] **Prefetch**: Predict player movement, pre-generate chunks
- [ ] **Compression**: Compress density data for distant chunks
- [ ] **Instancing**: Instance similar terrain features (rocks, trees)

### Visual Quality Ideas

- [ ] **Normal smoothing**: Smooth normals across chunk boundaries
- [ ] **AO baking**: Ambient occlusion for caves
- [ ] **Detail layers**: Add surface detail at close range
- [ ] **Erosion simulation**: More realistic terrain shapes

### Notes

```
2026-01-22: Initial document creation
- Documented SDF and marching cubes basics
- Need to implement and benchmark dual contouring
- Cube-sphere transition handling needs more work
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
