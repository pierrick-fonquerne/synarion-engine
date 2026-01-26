# GPU Terrain Shaders - Research Document

**Version**: 0.1.0
**Date**: 2026-01-23
**Status**: Active Research
**Crate**: `syn_terrain`

---

## Table of Contents

1. [Overview](#overview)
2. [Compute Shader Fundamentals](#compute-shader-fundamentals)
3. [Density Generation](#density-generation)
4. [Marching Cubes GPU](#marching-cubes-gpu)
5. [Mesh Buffer Management](#mesh-buffer-management)
6. [Vegetation Instancing](#vegetation-instancing)
7. [LOD & Streaming](#lod--streaming)
8. [Performance Optimization](#performance-optimization)
9. [Implementation Notes](#implementation-notes)
10. [References](#references)
11. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### GPU-Driven Terrain Pipeline

The entire terrain pipeline runs on the GPU, with CPU only orchestrating:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      GPU TERRAIN PIPELINE                                    │
│                                                                              │
│  Frame N                                                                     │
│  ───────                                                                     │
│                                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   DENSITY   │───▶│  MARCHING   │───▶│  COMPACT &  │───▶│   RENDER    │  │
│  │ GENERATION  │    │   CUBES     │    │   UPLOAD    │    │  (Indirect) │  │
│  │  (Compute)  │    │  (Compute)  │    │  (Compute)  │    │             │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│        │                                                                     │
│        ▼                                                                     │
│  ┌─────────────┐    ┌─────────────┐                                         │
│  │ VEGETATION  │───▶│  INSTANCE   │───────────────────────────▶ RENDER     │
│  │  SCATTER    │    │   CULL      │                                         │
│  │  (Compute)  │    │  (Compute)  │                                         │
│  └─────────────┘    └─────────────┘                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

CPU Role:
- Determine visible chunks (frustum culling)
- Submit chunk requests to priority queue
- Dispatch compute shaders
- Issue DrawIndirect calls
```

### Key Principles

1. **CPU never reads back**: GPU generates, GPU renders
2. **Indirect everything**: DrawIndirect, DispatchIndirect
3. **Async compute**: Overlap generation with rendering
4. **Ring buffers**: Reuse memory without stalls

---

## Compute Shader Fundamentals

### WGSL Compute Basics

```wgsl
// Workgroup configuration
// - Total threads = workgroup_size × num_workgroups
// - Threads in same workgroup can share memory
// - Typical workgroup sizes: 64, 128, 256

@compute @workgroup_size(8, 8, 8)  // 512 threads per workgroup
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    // global_id = workgroup_id * workgroup_size + local_id
    // local_index = linear index within workgroup (0..511 for 8×8×8)
}
```

### Workgroup Shared Memory

```wgsl
// Shared memory: fast, visible to all threads in workgroup
// Limited size (~16-48 KB depending on GPU)

var<workgroup> shared_density: array<f32, 512>;  // 8×8×8
var<workgroup> shared_counter: atomic<u32>;

@compute @workgroup_size(8, 8, 8)
fn compute_with_shared(
    @builtin(local_invocation_index) local_index: u32,
) {
    // Write to shared memory
    shared_density[local_index] = calculate_density();

    // MUST synchronize before reading other threads' data
    workgroupBarrier();

    // Now safe to read neighbors
    let left = shared_density[local_index - 1];
    let right = shared_density[local_index + 1];
}
```

### Atomic Operations

```wgsl
// Atomics for thread-safe counting, allocation

var<storage, read_write> vertex_counter: atomic<u32>;
var<storage, read_write> vertices: array<Vertex>;

fn emit_vertex(v: Vertex) {
    // Atomically allocate slot
    let index = atomicAdd(&vertex_counter, 1u);

    // Write vertex to allocated slot
    vertices[index] = v;
}
```

### Buffer Binding

```wgsl
// Storage buffers: large, read/write from compute
@group(0) @binding(0) var<storage, read> input_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_data: array<f32>;

// Uniform buffers: small, read-only, faster
@group(0) @binding(2) var<uniform> params: ComputeParams;

// Textures
@group(0) @binding(3) var noise_texture: texture_3d<f32>;
@group(0) @binding(4) var noise_sampler: sampler;
```

---

## Density Generation

### Chunk Parameters

```wgsl
struct ChunkParams {
    // World position of chunk origin
    world_origin: vec3<f32>,
    _pad0: f32,

    // Chunk dimensions
    chunk_size: vec3<u32>,
    voxel_size: f32,

    // Noise parameters
    noise_scale: f32,
    noise_octaves: u32,
    noise_persistence: f32,
    noise_lacunarity: f32,

    // Terrain parameters
    base_height: f32,
    height_scale: f32,
    cave_threshold: f32,
    cave_scale: f32,
}

@group(0) @binding(0) var<uniform> chunk: ChunkParams;
@group(0) @binding(1) var<storage, read_write> density: array<f32>;
@group(0) @binding(2) var noise_3d: texture_3d<f32>;
@group(0) @binding(3) var noise_sampler: sampler;
```

### Noise Functions on GPU

```wgsl
// Hash function for procedural noise
fn hash31(p: vec3<f32>) -> f32 {
    var p3 = fract(p * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

// Value noise
fn value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Smooth interpolation
    let u = f * f * (3.0 - 2.0 * f);

    // Sample corners
    let n000 = hash31(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash31(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash31(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash31(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash31(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash31(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash31(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash31(i + vec3<f32>(1.0, 1.0, 1.0));

    // Trilinear interpolation
    let n00 = mix(n000, n100, u.x);
    let n01 = mix(n001, n101, u.x);
    let n10 = mix(n010, n110, u.x);
    let n11 = mix(n011, n111, u.x);

    let n0 = mix(n00, n10, u.y);
    let n1 = mix(n01, n11, u.y);

    return mix(n0, n1, u.z);
}

// Simplex noise (better but more complex)
fn simplex_3d(p: vec3<f32>) -> f32 {
    // Skewing factors
    let F3 = 1.0 / 3.0;
    let G3 = 1.0 / 6.0;

    let s = (p.x + p.y + p.z) * F3;
    let i = floor(p + s);

    let t = (i.x + i.y + i.z) * G3;
    let x0 = p - (i - t);

    // Determine simplex
    var i1: vec3<f32>;
    var i2: vec3<f32>;

    if (x0.x >= x0.y) {
        if (x0.y >= x0.z) {
            i1 = vec3<f32>(1.0, 0.0, 0.0);
            i2 = vec3<f32>(1.0, 1.0, 0.0);
        } else if (x0.x >= x0.z) {
            i1 = vec3<f32>(1.0, 0.0, 0.0);
            i2 = vec3<f32>(1.0, 0.0, 1.0);
        } else {
            i1 = vec3<f32>(0.0, 0.0, 1.0);
            i2 = vec3<f32>(1.0, 0.0, 1.0);
        }
    } else {
        if (x0.y < x0.z) {
            i1 = vec3<f32>(0.0, 0.0, 1.0);
            i2 = vec3<f32>(0.0, 1.0, 1.0);
        } else if (x0.x < x0.z) {
            i1 = vec3<f32>(0.0, 1.0, 0.0);
            i2 = vec3<f32>(0.0, 1.0, 1.0);
        } else {
            i1 = vec3<f32>(0.0, 1.0, 0.0);
            i2 = vec3<f32>(1.0, 1.0, 0.0);
        }
    }

    let x1 = x0 - i1 + G3;
    let x2 = x0 - i2 + 2.0 * G3;
    let x3 = x0 - 1.0 + 3.0 * G3;

    // Gradient contributions
    var n0 = 0.0;
    var n1 = 0.0;
    var n2 = 0.0;
    var n3 = 0.0;

    var t0 = 0.6 - dot(x0, x0);
    if (t0 >= 0.0) {
        t0 *= t0;
        let g0 = hash33(i) * 2.0 - 1.0;
        n0 = t0 * t0 * dot(g0, x0);
    }

    var t1 = 0.6 - dot(x1, x1);
    if (t1 >= 0.0) {
        t1 *= t1;
        let g1 = hash33(i + i1) * 2.0 - 1.0;
        n1 = t1 * t1 * dot(g1, x1);
    }

    var t2 = 0.6 - dot(x2, x2);
    if (t2 >= 0.0) {
        t2 *= t2;
        let g2 = hash33(i + i2) * 2.0 - 1.0;
        n2 = t2 * t2 * dot(g2, x2);
    }

    var t3 = 0.6 - dot(x3, x3);
    if (t3 >= 0.0) {
        t3 *= t3;
        let g3 = hash33(i + 1.0) * 2.0 - 1.0;
        n3 = t3 * t3 * dot(g3, x3);
    }

    return 32.0 * (n0 + n1 + n2 + n3);
}

// FBM (Fractal Brownian Motion)
fn fbm_3d(p: vec3<f32>, octaves: u32, persistence: f32, lacunarity: f32) -> f32 {
    var value = 0.0;
    var amplitude = 1.0;
    var frequency = 1.0;
    var max_value = 0.0;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * simplex_3d(p * frequency);
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    return value / max_value;
}
```

### Terrain Density Function

```wgsl
fn terrain_density(world_pos: vec3<f32>) -> f32 {
    // === Base terrain (heightmap-like) ===
    let terrain_noise = fbm_3d(
        world_pos * chunk.noise_scale,
        chunk.noise_octaves,
        chunk.noise_persistence,
        chunk.noise_lacunarity
    );

    let terrain_height = chunk.base_height + terrain_noise * chunk.height_scale;
    var density = world_pos.y - terrain_height;

    // === 3D features (caves, overhangs) ===
    let cave_noise = fbm_3d(
        world_pos * chunk.cave_scale,
        4u,
        0.5,
        2.0
    );

    // Carve caves where noise exceeds threshold
    let cave_contribution = smoothstep(
        chunk.cave_threshold - 0.1,
        chunk.cave_threshold + 0.1,
        cave_noise
    );

    // Caves only below surface (mostly)
    let cave_depth_factor = smoothstep(terrain_height + 20.0, terrain_height - 50.0, world_pos.y);
    density += cave_contribution * cave_depth_factor * 30.0;

    return density;
}

// Main compute shader
@compute @workgroup_size(8, 8, 8)
fn generate_density(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // Check bounds
    if (any(global_id >= chunk.chunk_size)) {
        return;
    }

    // Calculate world position
    let local_pos = vec3<f32>(global_id) * chunk.voxel_size;
    let world_pos = chunk.world_origin + local_pos;

    // Calculate density
    let density_value = terrain_density(world_pos);

    // Store in buffer
    let index = global_id.x
              + global_id.y * chunk.chunk_size.x
              + global_id.z * chunk.chunk_size.x * chunk.chunk_size.y;

    density[index] = density_value;
}
```

### Using 3D Texture for Noise

Pre-computed noise texture for better performance:

```rust
/// Generate 3D noise texture on CPU (once at startup)
pub fn generate_noise_texture(size: u32, device: &Device, queue: &Queue) -> Texture {
    let mut data = vec![0u8; (size * size * size) as usize];

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let p = Vec3::new(x as f32, y as f32, z as f32) / size as f32;
                let noise = simplex_3d(p * 8.0); // 8 = frequency
                let value = ((noise * 0.5 + 0.5) * 255.0) as u8;
                data[(z * size * size + y * size + x) as usize] = value;
            }
        }
    }

    let texture = device.create_texture(&TextureDescriptor {
        label: Some("noise_3d"),
        size: Extent3d { width: size, height: size, depth_or_array_layers: size },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        texture.as_image_copy(),
        &data,
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(size),
            rows_per_image: Some(size),
        },
        Extent3d { width: size, height: size, depth_or_array_layers: size },
    );

    texture
}
```

```wgsl
// Sample pre-computed noise (much faster than calculating)
fn sample_noise(p: vec3<f32>) -> f32 {
    return textureSampleLevel(noise_3d, noise_sampler, p, 0.0).r * 2.0 - 1.0;
}

fn fbm_texture(p: vec3<f32>, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var i = 0u; i < octaves; i++) {
        value += amplitude * sample_noise(p * frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}
```

---

## Marching Cubes GPU

### Lookup Tables

```wgsl
// Edge table: which edges are intersected for each configuration
// 256 entries, each is a 12-bit mask
const EDGE_TABLE: array<u32, 256> = array<u32, 256>(
    0x000, 0x109, 0x203, 0x30a, 0x406, 0x50f, 0x605, 0x70c,
    0x80c, 0x905, 0xa0f, 0xb06, 0xc0a, 0xd03, 0xe09, 0xf00,
    // ... (full table in actual implementation)
);

// Triangle table: which edges form triangles for each configuration
// 256 × 16 entries (-1 terminated)
const TRI_TABLE: array<array<i32, 16>, 256> = array<array<i32, 16>, 256>(
    array<i32, 16>(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    array<i32, 16>(0, 8, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    array<i32, 16>(0, 1, 9, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    array<i32, 16>(1, 8, 3, 9, 8, 1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1),
    // ... (full table in actual implementation)
);

// Edge vertices: which two corners each edge connects
const EDGE_VERTICES: array<vec2<u32>, 12> = array<vec2<u32>, 12>(
    vec2<u32>(0u, 1u), vec2<u32>(1u, 2u), vec2<u32>(2u, 3u), vec2<u32>(3u, 0u),
    vec2<u32>(4u, 5u), vec2<u32>(5u, 6u), vec2<u32>(6u, 7u), vec2<u32>(7u, 4u),
    vec2<u32>(0u, 4u), vec2<u32>(1u, 5u), vec2<u32>(2u, 6u), vec2<u32>(3u, 7u),
);

// Corner offsets within a cube
const CORNER_OFFSETS: array<vec3<u32>, 8> = array<vec3<u32>, 8>(
    vec3<u32>(0u, 0u, 0u),
    vec3<u32>(1u, 0u, 0u),
    vec3<u32>(1u, 1u, 0u),
    vec3<u32>(0u, 1u, 0u),
    vec3<u32>(0u, 0u, 1u),
    vec3<u32>(1u, 0u, 1u),
    vec3<u32>(1u, 1u, 1u),
    vec3<u32>(0u, 1u, 1u),
);
```

### Vertex Structure

```wgsl
struct Vertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    material: u32,
    _pad: u32,
}

struct MarchingCubesOutput {
    vertex_count: atomic<u32>,
    vertices: array<Vertex>,
}

@group(0) @binding(0) var<uniform> chunk: ChunkParams;
@group(0) @binding(1) var<storage, read> density: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: MarchingCubesOutput;
```

### Main Marching Cubes Shader

```wgsl
// Sample density at grid position
fn sample_density(pos: vec3<u32>) -> f32 {
    let index = pos.x + pos.y * chunk.chunk_size.x + pos.z * chunk.chunk_size.x * chunk.chunk_size.y;
    return density[index];
}

// Interpolate vertex position along edge
fn interpolate_vertex(p1: vec3<f32>, p2: vec3<f32>, v1: f32, v2: f32) -> vec3<f32> {
    if (abs(v1 - v2) < 0.00001) {
        return p1;
    }
    let t = -v1 / (v2 - v1);
    return p1 + t * (p2 - p1);
}

// Calculate normal from gradient
fn calculate_normal(pos: vec3<u32>) -> vec3<f32> {
    let d = 1u;

    let dx = sample_density(pos + vec3<u32>(d, 0u, 0u)) - sample_density(pos - vec3<u32>(d, 0u, 0u));
    let dy = sample_density(pos + vec3<u32>(0u, d, 0u)) - sample_density(pos - vec3<u32>(0u, d, 0u));
    let dz = sample_density(pos + vec3<u32>(0u, 0u, d)) - sample_density(pos - vec3<u32>(0u, 0u, d));

    return normalize(vec3<f32>(dx, dy, dz));
}

@compute @workgroup_size(4, 4, 4)
fn marching_cubes(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // Each thread processes one cube (needs 8 corners, so size-1 cubes per axis)
    if (any(global_id >= chunk.chunk_size - 1u)) {
        return;
    }

    // Sample 8 corners
    var corner_values: array<f32, 8>;
    for (var i = 0u; i < 8u; i++) {
        let corner_pos = global_id + CORNER_OFFSETS[i];
        corner_values[i] = sample_density(corner_pos);
    }

    // Build cube index (which corners are inside surface)
    var cube_index = 0u;
    for (var i = 0u; i < 8u; i++) {
        if (corner_values[i] < 0.0) {
            cube_index |= (1u << i);
        }
    }

    // Skip empty cubes
    let edge_mask = EDGE_TABLE[cube_index];
    if (edge_mask == 0u) {
        return;
    }

    // Calculate world position of cube origin
    let cube_world_pos = chunk.world_origin + vec3<f32>(global_id) * chunk.voxel_size;

    // Compute vertex positions on intersected edges
    var edge_vertices: array<vec3<f32>, 12>;
    for (var i = 0u; i < 12u; i++) {
        if ((edge_mask & (1u << i)) != 0u) {
            let v0_idx = EDGE_VERTICES[i].x;
            let v1_idx = EDGE_VERTICES[i].y;

            let p0 = cube_world_pos + vec3<f32>(CORNER_OFFSETS[v0_idx]) * chunk.voxel_size;
            let p1 = cube_world_pos + vec3<f32>(CORNER_OFFSETS[v1_idx]) * chunk.voxel_size;

            let d0 = corner_values[v0_idx];
            let d1 = corner_values[v1_idx];

            edge_vertices[i] = interpolate_vertex(p0, p1, d0, d1);
        }
    }

    // Generate triangles from lookup table
    var i = 0;
    loop {
        let edge_a = TRI_TABLE[cube_index][i];
        if (edge_a == -1) {
            break;
        }

        let edge_b = TRI_TABLE[cube_index][i + 1];
        let edge_c = TRI_TABLE[cube_index][i + 2];

        let v0 = edge_vertices[edge_a];
        let v1 = edge_vertices[edge_b];
        let v2 = edge_vertices[edge_c];

        // Calculate face normal
        let face_normal = normalize(cross(v1 - v0, v2 - v0));

        // Allocate 3 vertices atomically
        let base_index = atomicAdd(&output.vertex_count, 3u);

        // Write vertices
        output.vertices[base_index + 0u] = Vertex(v0, face_normal, 0u, 0u);
        output.vertices[base_index + 1u] = Vertex(v1, face_normal, 0u, 0u);
        output.vertices[base_index + 2u] = Vertex(v2, face_normal, 0u, 0u);

        i += 3;
    }
}
```

### Smooth Normals Pass

After marching cubes, run a pass to smooth normals:

```wgsl
// Pass 2: Calculate smooth normals from gradient (optional)
@compute @workgroup_size(64)
fn smooth_normals(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let vertex_index = global_id.x;
    if (vertex_index >= atomicLoad(&output.vertex_count)) {
        return;
    }

    let vertex = output.vertices[vertex_index];

    // Convert world position back to grid position
    let grid_pos = (vertex.position - chunk.world_origin) / chunk.voxel_size;
    let grid_pos_u = vec3<u32>(grid_pos);

    // Sample gradient for smooth normal
    let smooth_normal = calculate_normal(grid_pos_u);

    output.vertices[vertex_index].normal = smooth_normal;
}
```

---

## Mesh Buffer Management

### Ring Buffer System

```rust
/// Ring buffer for terrain mesh data
pub struct TerrainMeshPool {
    /// GPU buffer for vertices
    vertex_buffer: Buffer,
    /// Current write offset (in vertices)
    write_offset: u32,
    /// Total capacity (in vertices)
    capacity: u32,
    /// Allocated ranges (chunk_id → range)
    allocations: HashMap<ChunkId, BufferRange>,
    /// Pending frees (frame → ranges)
    pending_frees: VecDeque<(u64, Vec<BufferRange>)>,
    /// Current frame
    current_frame: u64,
    /// Frames to keep before reusing
    frames_in_flight: u64,
}

#[derive(Clone, Copy)]
pub struct BufferRange {
    pub offset: u32,
    pub size: u32,
}

impl TerrainMeshPool {
    pub fn allocate(&mut self, vertex_count: u32) -> Option<BufferRange> {
        // Simple bump allocator with wraparound
        let available = if self.write_offset + vertex_count <= self.capacity {
            vertex_count
        } else {
            // Wrap to beginning
            self.write_offset = 0;
            if vertex_count <= self.capacity {
                vertex_count
            } else {
                return None; // Too big
            }
        };

        let range = BufferRange {
            offset: self.write_offset,
            size: vertex_count,
        };

        self.write_offset += vertex_count;
        Some(range)
    }

    pub fn free(&mut self, chunk_id: ChunkId) {
        if let Some(range) = self.allocations.remove(&chunk_id) {
            // Defer free until frames_in_flight have passed
            let free_frame = self.current_frame + self.frames_in_flight;

            if let Some((frame, ranges)) = self.pending_frees.back_mut() {
                if *frame == free_frame {
                    ranges.push(range);
                    return;
                }
            }

            self.pending_frees.push_back((free_frame, vec![range]));
        }
    }

    pub fn begin_frame(&mut self, frame: u64) {
        self.current_frame = frame;

        // Process pending frees
        while let Some((free_frame, _)) = self.pending_frees.front() {
            if *free_frame <= frame {
                self.pending_frees.pop_front();
                // Ranges are now available for reuse
            } else {
                break;
            }
        }
    }
}
```

### Indirect Draw Commands

```rust
/// Generate draw indirect commands from mesh pool
pub struct IndirectDrawBuffer {
    /// GPU buffer for indirect commands
    buffer: Buffer,
    /// CPU staging data
    commands: Vec<DrawIndirectArgs>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndirectArgs {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

impl IndirectDrawBuffer {
    pub fn add_chunk(&mut self, range: BufferRange, chunk_id: u32) {
        self.commands.push(DrawIndirectArgs {
            vertex_count: range.size,
            instance_count: 1,
            first_vertex: range.offset,
            first_instance: chunk_id, // Use for per-chunk data lookup
        });
    }

    pub fn upload(&mut self, queue: &Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.commands));
    }

    pub fn draw(&self, render_pass: &mut RenderPass, count: u32) {
        render_pass.multi_draw_indirect(&self.buffer, 0, count);
    }
}
```

---

## Vegetation Instancing

### Scatter Points Generation

```wgsl
struct VegetationParams {
    chunk_world_origin: vec3<f32>,
    chunk_size: f32,
    density_base: f32,
    density_variation: f32,
    min_slope: f32,
    max_slope: f32,
    min_height: f32,
    max_height: f32,
    seed: u32,
    max_instances: u32,
}

struct VegetationInstance {
    position: vec3<f32>,
    scale: f32,
    rotation: vec4<f32>,  // Quaternion
    type_id: u32,
    _pad: vec3<u32>,
}

@group(0) @binding(0) var<uniform> params: VegetationParams;
@group(0) @binding(1) var<storage, read> terrain_density: array<f32>;
@group(0) @binding(2) var<storage, read_write> instance_count: atomic<u32>;
@group(0) @binding(3) var<storage, read_write> instances: array<VegetationInstance>;

fn pcg_hash(input: u32) -> u32 {
    var state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_float(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
}

@compute @workgroup_size(8, 8, 1)
fn scatter_vegetation(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    // Each thread handles a grid cell
    let cell_size = params.chunk_size / 64.0;  // 64×64 grid
    let cell_origin = params.chunk_world_origin + vec3<f32>(
        f32(global_id.x) * cell_size,
        0.0,
        f32(global_id.y) * cell_size
    );

    // Random seed for this cell
    let cell_seed = params.seed ^ (global_id.x * 73856093u) ^ (global_id.y * 19349663u);

    // Determine number of instances in this cell
    let density = params.density_base + params.density_variation * random_float(cell_seed);
    let instance_count_cell = u32(density * cell_size * cell_size);

    for (var i = 0u; i < instance_count_cell; i++) {
        let instance_seed = cell_seed ^ (i * 83492791u);

        // Random position within cell
        let local_x = random_float(instance_seed) * cell_size;
        let local_z = random_float(instance_seed ^ 1u) * cell_size;
        let position_xz = cell_origin + vec3<f32>(local_x, 0.0, local_z);

        // Sample terrain height at this position
        let terrain_height = sample_terrain_height(position_xz);
        let position = vec3<f32>(position_xz.x, terrain_height, position_xz.z);

        // Check height constraints
        if (terrain_height < params.min_height || terrain_height > params.max_height) {
            continue;
        }

        // Check slope constraints
        let normal = sample_terrain_normal(position_xz);
        let slope = acos(normal.y);
        if (slope < params.min_slope || slope > params.max_slope) {
            continue;
        }

        // Random scale and rotation
        let scale = 0.8 + random_float(instance_seed ^ 2u) * 0.4;
        let rotation_angle = random_float(instance_seed ^ 3u) * 6.28318;
        let rotation_quat = vec4<f32>(0.0, sin(rotation_angle * 0.5), 0.0, cos(rotation_angle * 0.5));

        // Vegetation type based on biome/conditions
        let type_id = select_vegetation_type(position, normal, instance_seed ^ 4u);

        // Allocate instance
        let index = atomicAdd(&instance_count, 1u);
        if (index >= params.max_instances) {
            return;
        }

        instances[index] = VegetationInstance(
            position,
            scale,
            rotation_quat,
            type_id,
            vec3<u32>(0u),
        );
    }
}
```

### Frustum Culling (GPU)

```wgsl
struct CullParams {
    view_proj: mat4x4<f32>,
    frustum_planes: array<vec4<f32>, 6>,
    camera_position: vec3<f32>,
    lod_distances: array<f32, 4>,
}

struct CulledInstance {
    position: vec3<f32>,
    scale: f32,
    rotation: vec4<f32>,
    type_id: u32,
    lod: u32,
    _pad: vec2<u32>,
}

@group(0) @binding(0) var<uniform> cull: CullParams;
@group(0) @binding(1) var<storage, read> input_instances: array<VegetationInstance>;
@group(0) @binding(2) var<uniform> input_count: u32;
@group(0) @binding(3) var<storage, read_write> output_count: atomic<u32>;
@group(0) @binding(4) var<storage, read_write> output_instances: array<CulledInstance>;

fn test_frustum(position: vec3<f32>, radius: f32) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = cull.frustum_planes[i];
        let distance = dot(plane.xyz, position) + plane.w;
        if (distance < -radius) {
            return false;  // Outside frustum
        }
    }
    return true;
}

fn calculate_lod(position: vec3<f32>) -> u32 {
    let distance = length(position - cull.camera_position);

    for (var i = 0u; i < 4u; i++) {
        if (distance < cull.lod_distances[i]) {
            return i;
        }
    }
    return 4u;  // Too far, cull
}

@compute @workgroup_size(64)
fn cull_instances(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let index = global_id.x;
    if (index >= input_count) {
        return;
    }

    let instance = input_instances[index];

    // Approximate bounding sphere radius
    let radius = instance.scale * 2.0;

    // Frustum test
    if (!test_frustum(instance.position, radius)) {
        return;
    }

    // LOD selection
    let lod = calculate_lod(instance.position);
    if (lod >= 4u) {
        return;  // Too far
    }

    // Output visible instance
    let out_index = atomicAdd(&output_count, 1u);
    output_instances[out_index] = CulledInstance(
        instance.position,
        instance.scale,
        instance.rotation,
        instance.type_id,
        lod,
        vec2<u32>(0u),
    );
}
```

### Instance Rendering

```wgsl
// Vertex shader for instanced vegetation
struct InstanceData {
    @location(5) instance_pos: vec3<f32>,
    @location(6) instance_scale: f32,
    @location(7) instance_rotation: vec4<f32>,
    @location(8) instance_type_lod: vec2<u32>,
}

fn rotate_by_quat(v: vec3<f32>, q: vec4<f32>) -> vec3<f32> {
    let u = q.xyz;
    let s = q.w;
    return 2.0 * dot(u, v) * u
         + (s * s - dot(u, u)) * v
         + 2.0 * s * cross(u, v);
}

@vertex
fn vs_vegetation(
    @location(0) local_pos: vec3<f32>,
    @location(1) local_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    instance: InstanceData,
) -> VertexOutput {
    // Apply instance transform
    var world_pos = rotate_by_quat(local_pos * instance.instance_scale, instance.instance_rotation);
    world_pos += instance.instance_pos;

    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.world_pos = world_pos;
    out.normal = rotate_by_quat(local_normal, instance.instance_rotation);
    out.uv = uv;
    out.type_id = instance.instance_type_lod.x;
    out.lod = instance.instance_type_lod.y;

    return out;
}
```

---

## LOD & Streaming

### Chunk Priority System

```rust
/// Calculate priority for chunk generation
pub fn calculate_chunk_priority(
    chunk: &ChunkId,
    camera: &Camera,
    world: &World,
) -> f32 {
    let chunk_center = chunk.center_world_position(world);
    let to_chunk = chunk_center - camera.position;
    let distance = to_chunk.length();

    // Base priority: closer = higher
    let distance_priority = 1.0 / (distance + 1.0);

    // Direction bonus: chunks in view direction get priority
    let direction = to_chunk.normalize();
    let facing = direction.dot(camera.forward());
    let direction_bonus = facing.max(0.0) * 0.5;

    // LOD urgency: higher LOD chunks (more detail) get priority when close
    let lod_factor = 1.0 / (chunk.lod as f32 + 1.0);
    let lod_bonus = if distance < chunk.size() * 2.0 { lod_factor * 0.3 } else { 0.0 };

    distance_priority + direction_bonus + lod_bonus
}
```

### Async Compute Dispatch

```rust
/// Dispatch terrain generation across multiple frames
pub struct TerrainGenerationQueue {
    pending: PriorityQueue<ChunkRequest>,
    in_flight: Vec<InFlightChunk>,
    max_dispatches_per_frame: usize,
}

pub struct InFlightChunk {
    chunk_id: ChunkId,
    density_pass: ComputePass,
    mesh_pass: ComputePass,
    fence: QuerySet,  // For timing
}

impl TerrainGenerationQueue {
    pub fn update(&mut self, encoder: &mut CommandEncoder, device: &Device) {
        // Dispatch up to N chunks per frame
        let dispatch_count = self.pending.len().min(self.max_dispatches_per_frame);

        for _ in 0..dispatch_count {
            if let Some(request) = self.pending.pop() {
                let in_flight = self.dispatch_chunk(request, encoder, device);
                self.in_flight.push(in_flight);
            }
        }

        // Check completed chunks
        self.in_flight.retain(|chunk| {
            if chunk.is_complete() {
                self.on_chunk_complete(chunk);
                false
            } else {
                true
            }
        });
    }

    fn dispatch_chunk(
        &self,
        request: ChunkRequest,
        encoder: &mut CommandEncoder,
        device: &Device,
    ) -> InFlightChunk {
        // Pass 1: Generate density
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("density_gen"),
            });
            pass.set_pipeline(&self.density_pipeline);
            pass.set_bind_group(0, &request.density_bind_group, &[]);
            let workgroups = (request.chunk_size + 7) / 8;
            pass.dispatch_workgroups(workgroups, workgroups, workgroups);
        }

        // Pass 2: Marching cubes
        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("marching_cubes"),
            });
            pass.set_pipeline(&self.mesh_pipeline);
            pass.set_bind_group(0, &request.mesh_bind_group, &[]);
            let workgroups = (request.chunk_size - 1 + 3) / 4;
            pass.dispatch_workgroups(workgroups, workgroups, workgroups);
        }

        InFlightChunk {
            chunk_id: request.chunk_id,
            // ... track completion
        }
    }
}
```

---

## Performance Optimization

### Occupancy Optimization

```wgsl
// Good: 8×8×8 = 512 threads, good for 3D work
@compute @workgroup_size(8, 8, 8)
fn density_gen_3d(...) { }

// Good: 64 threads, good for linear work
@compute @workgroup_size(64)
fn process_vertices(...) { }

// Bad: Too few threads, low occupancy
@compute @workgroup_size(4, 4, 4)  // Only 64 threads
fn bad_example(...) { }

// Bad: Too many threads, may exceed limits
@compute @workgroup_size(32, 32, 32)  // 32768 threads!
fn also_bad(...) { }
```

### Memory Coalescing

```wgsl
// Good: Sequential memory access
fn good_access(index: u32) {
    let value = data[index];  // Threads 0,1,2,3... access 0,1,2,3...
}

// Bad: Strided access
fn bad_access(index: u32) {
    let value = data[index * 16];  // Memory not coalesced
}

// Optimization: Use shared memory for non-coalesced patterns
var<workgroup> shared: array<f32, 256>;

fn optimized_access(local_id: u32, global_id: u32) {
    // Load to shared memory (coalesced)
    shared[local_id] = data[global_id];
    workgroupBarrier();

    // Random access in shared (fast)
    let value = shared[(local_id * 7) % 256];
}
```

### Avoiding Divergence

```wgsl
// Bad: Heavy divergence
fn divergent(id: u32) {
    if (id % 2 == 0) {
        // Heavy computation
        expensive_function_a();
    } else {
        // Different heavy computation
        expensive_function_b();
    }
}

// Better: Separate dispatches or reorganize
fn less_divergent(id: u32) {
    // Same path for all threads in warp
    let result = computation_all_threads();

    // Only diverge for light operations
    if (id % 2 == 0) {
        output_a[id / 2] = result;
    } else {
        output_b[id / 2] = result;
    }
}
```

### Budget Guidelines

```
Target: 16ms frame (60 FPS)

GPU Budget (RTX 3070 class):
├── Density generation: 1-2ms per chunk
├── Marching cubes: 1-2ms per chunk
├── Vegetation scatter: 0.5ms per chunk
├── Vegetation cull: 0.2ms for all
├── Terrain render: 3-4ms
├── Vegetation render: 2-3ms
└── Headroom: 4-6ms

Chunk counts per frame:
├── Generate: 2-4 chunks
├── Mesh: 2-4 chunks
├── Render: 50-200 chunks
└── Vegetation: 10,000-100,000 instances
```

---

## Implementation Notes

### Suggested Shader Organization

```
engine/crates/syn_terrain/
├── src/
│   ├── gpu/
│   │   ├── mod.rs
│   │   ├── density.rs      # Density generation dispatch
│   │   ├── mesh.rs         # Marching cubes dispatch
│   │   ├── vegetation.rs   # Vegetation scatter/cull
│   │   └── pipelines.rs    # Pipeline creation
│   └── shaders/
│       ├── density_gen.wgsl
│       ├── marching_cubes.wgsl
│       ├── smooth_normals.wgsl
│       ├── vegetation_scatter.wgsl
│       ├── vegetation_cull.wgsl
│       ├── terrain.wgsl          # Render shader
│       └── vegetation.wgsl       # Render shader
```

### Testing Strategy

```rust
#[test]
fn test_marching_cubes_sphere() {
    // Generate sphere SDF
    let density = generate_sphere_density(32, 10.0);

    // Run marching cubes on GPU
    let mesh = run_marching_cubes(&density);

    // Verify mesh is roughly spherical
    for vertex in &mesh.vertices {
        let dist = vertex.position.length();
        assert!((dist - 10.0).abs() < 1.0);
    }
}

#[test]
fn test_vegetation_culling() {
    // Generate test instances
    let instances = generate_test_instances(1000);

    // Set up frustum that should cull half
    let frustum = create_half_space_frustum();

    // Run culling
    let visible = run_vegetation_cull(&instances, &frustum);

    // Should have roughly half
    assert!(visible.len() > 400 && visible.len() < 600);
}
```

---

## References

### Papers

1. **Marching Cubes**
   - Lorensen & Cline (1987) - Original paper
   - Lewiner et al. (2003) - "Efficient Implementation of Marching Cubes"

2. **GPU Terrain**
   - Losasso & Hoppe (2004) - "Geometry Clipmaps"
   - Dick et al. (2009) - "GPU Ray-Casting for Scalable Terrain Rendering"

3. **Vegetation**
   - Deussen et al. (1998) - "Realistic Modeling of Plant Ecosystems"
   - Bruneton & Neyret (2012) - "Real-time Realistic Rendering of Nature Scenes"

### GPU Programming

- WGSL Specification: https://www.w3.org/TR/WGSL/
- WebGPU Best Practices: https://toji.dev/webgpu-best-practices/
- GPU Gems (NVidia): https://developer.nvidia.com/gpugems/

### Implementations

- **Transvoxel**: Eric Lengyel's seamless LOD algorithm
- **GPU Pro**: Various GPU terrain articles
- **Dreams (Media Molecule)**: SDF sculpting inspiration

---

## Ideas & Future Work

### To Research

- [ ] **Transvoxel**: Seamless LOD transitions
- [ ] **Mesh simplification**: Reduce triangles in flat areas
- [ ] **Tessellation shaders**: Runtime detail on GPU
- [ ] **Raymarching terrain**: Skip mesh entirely for distant terrain
- [ ] **Nanite-style**: Virtualized geometry

### Optimization Ideas

- [ ] **Hierarchical culling**: Octree-based chunk culling on GPU
- [ ] **Persistent threads**: Keep compute workgroups running
- [ ] **Async readback**: Get vertex count without stall
- [ ] **Mesh shaders**: Replace vertex instancing (newer GPUs)

### Visual Quality Ideas

- [ ] **Ambient occlusion**: Bake AO during mesh generation
- [ ] **Displacement mapping**: Add micro-detail in vertex shader
- [ ] **Parallax occlusion**: Fake depth on flat surfaces

### Notes

```
2026-01-23: Initial document creation
- Complete marching cubes shader
- Need to test on actual hardware
- Vegetation culling needs optimization
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
