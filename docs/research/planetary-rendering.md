# Planetary Rendering - Research Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Active Research
**Crates**: `syn_renderer`, `syn_atmosphere`, `syn_ocean`

---

## Table of Contents

1. [Overview](#overview)
2. [Atmospheric Scattering](#atmospheric-scattering)
3. [Volumetric Clouds](#volumetric-clouds)
4. [Ocean Rendering](#ocean-rendering)
5. [Planetary LOD System](#planetary-lod-system)
6. [Special Effects](#special-effects)
7. [Performance Optimization](#performance-optimization)
8. [Implementation Notes](#implementation-notes)
9. [References](#references)
10. [Ideas & Future Work](#ideas--future-work)

---

## Overview

### Goals

Render planets that look believable from:
- **Space** (10,000+ km): Planet as sphere with atmosphere halo
- **Orbit** (100-1000 km): Cloud patterns, terrain features visible
- **High altitude** (10-100 km): Atmosphere scattering, curvature visible
- **Ground level** (0-10 km): Full detail, realistic sky

### Key Challenges

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PLANETARY RENDERING CHALLENGES                        │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   ATMOSPHERE    │  │     CLOUDS      │  │     OCEAN       │         │
│  │                 │  │                 │  │                 │         │
│  │ • Scattering    │  │ • Volumetric    │  │ • FFT Waves     │         │
│  │ • Sky gradient  │  │ • Weather       │  │ • Reflections   │         │
│  │ • Sunset colors │  │ • Shadows       │  │ • Subsurface    │         │
│  │ • Aerial persp  │  │ • Day/night     │  │ • Foam/spray    │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   SCALE LOD     │  │    LIGHTING     │  │    EFFECTS      │         │
│  │                 │  │                 │  │                 │         │
│  │ • Space to      │  │ • Sun/star      │  │ • Aurora        │         │
│  │   ground        │  │ • Moon bounce   │  │ • Rings         │         │
│  │ • Impostors     │  │ • City lights   │  │ • Eclipses      │         │
│  │ • LOD blending  │  │ • Night sky     │  │ • Meteor shower │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Atmospheric Scattering

### Physics Background

Light scattering in atmosphere is caused by two main phenomena:

#### Rayleigh Scattering
- Caused by **molecules** (N₂, O₂)
- Wavelength dependent: shorter wavelengths scatter more
- Why sky is blue, sunsets are red

```
Scattering coefficient:
β_R(λ) = (8π³(n²-1)²) / (3Nλ⁴)

Where:
- n = refractive index of air (~1.0003)
- N = molecular number density
- λ = wavelength

Simplified (Earth sea level):
β_R(λ) = β_R0 × (λ0/λ)⁴

With:
β_R0 = (5.8, 13.5, 33.1) × 10⁻⁶ m⁻¹  for RGB at 680nm, 550nm, 440nm
```

#### Mie Scattering
- Caused by **aerosols** (dust, water droplets)
- Wavelength independent (mostly)
- Creates haze, sun glare

```
Mie scattering is complex, but simplified:
β_M ≈ 2.0 × 10⁻⁵ m⁻¹  (typical Earth value)

Phase function (Henyey-Greenstein):
P_HG(θ, g) = (1 - g²) / (4π × (1 + g² - 2g×cos(θ))^1.5)

Where:
- θ = scattering angle
- g = asymmetry parameter (0.76 for Earth aerosols)
```

### Single Scattering Model

The classic approach, still used for performance:

```rust
/// Atmospheric parameters
pub struct AtmosphereParams {
    /// Planet radius (m)
    pub planet_radius: f32,
    /// Atmosphere thickness (m)
    pub atmosphere_height: f32,

    /// Rayleigh scattering coefficients at sea level (m⁻¹)
    pub rayleigh_coeff: Vec3,
    /// Rayleigh scale height (m) - where density = 1/e
    pub rayleigh_scale_height: f32,

    /// Mie scattering coefficient at sea level (m⁻¹)
    pub mie_coeff: f32,
    /// Mie scale height (m)
    pub mie_scale_height: f32,
    /// Mie asymmetry parameter
    pub mie_g: f32,

    /// Sun intensity
    pub sun_intensity: f32,
}

impl AtmosphereParams {
    pub fn earth() -> Self {
        Self {
            planet_radius: 6_371_000.0,
            atmosphere_height: 100_000.0,
            rayleigh_coeff: Vec3::new(5.8e-6, 13.5e-6, 33.1e-6),
            rayleigh_scale_height: 8_500.0,
            mie_coeff: 2.0e-5,
            mie_scale_height: 1_200.0,
            mie_g: 0.76,
            sun_intensity: 20.0,
        }
    }

    pub fn mars() -> Self {
        Self {
            planet_radius: 3_389_500.0,
            atmosphere_height: 50_000.0,
            // Mars has more dust (Mie) than Rayleigh
            rayleigh_coeff: Vec3::new(19.918e-6, 13.57e-6, 5.75e-6),
            rayleigh_scale_height: 11_100.0,
            mie_coeff: 2.1e-5,
            mie_scale_height: 2_000.0,
            mie_g: 0.65,
            sun_intensity: 10.0,
        }
    }
}
```

### WGSL Shader: Single Scattering

```wgsl
struct AtmosphereUniforms {
    planet_center: vec3<f32>,
    planet_radius: f32,
    atmosphere_radius: f32,
    rayleigh_coeff: vec3<f32>,
    rayleigh_scale_height: f32,
    mie_coeff: f32,
    mie_scale_height: f32,
    mie_g: f32,
    sun_direction: vec3<f32>,
    sun_intensity: f32,
}

@group(0) @binding(0) var<uniform> atm: AtmosphereUniforms;

const PI: f32 = 3.14159265359;
const NUM_SAMPLES: i32 = 16;
const NUM_LIGHT_SAMPLES: i32 = 8;

// Ray-sphere intersection
fn ray_sphere_intersect(origin: vec3<f32>, dir: vec3<f32>, center: vec3<f32>, radius: f32) -> vec2<f32> {
    let oc = origin - center;
    let b = dot(oc, dir);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - c;

    if (discriminant < 0.0) {
        return vec2<f32>(-1.0, -1.0);
    }

    let sqrt_d = sqrt(discriminant);
    return vec2<f32>(-b - sqrt_d, -b + sqrt_d);
}

// Density at height
fn density_at_height(height: f32, scale_height: f32) -> f32 {
    return exp(-height / scale_height);
}

// Rayleigh phase function
fn rayleigh_phase(cos_theta: f32) -> f32 {
    return (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta);
}

// Mie phase function (Henyey-Greenstein)
fn mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let num = 1.0 - g2;
    let denom = pow(1.0 + g2 - 2.0 * g * cos_theta, 1.5);
    return (3.0 / (8.0 * PI)) * num / denom;
}

// Main scattering calculation
fn calculate_scattering(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
) -> vec3<f32> {
    // Find atmosphere intersection
    let atm_intersect = ray_sphere_intersect(
        ray_origin, ray_dir, atm.planet_center, atm.atmosphere_radius
    );

    if (atm_intersect.x > atm_intersect.y) {
        return vec3<f32>(0.0); // No intersection
    }

    // Clamp to valid range
    let t_min = max(atm_intersect.x, 0.0);
    let t_max = atm_intersect.y;

    // Check for planet intersection (in shadow)
    let planet_intersect = ray_sphere_intersect(
        ray_origin, ray_dir, atm.planet_center, atm.planet_radius
    );
    var segment_length = t_max - t_min;
    if (planet_intersect.x > 0.0) {
        segment_length = min(segment_length, planet_intersect.x - t_min);
    }

    let step_size = segment_length / f32(NUM_SAMPLES);
    var t = t_min + step_size * 0.5;

    var rayleigh_sum = vec3<f32>(0.0);
    var mie_sum = vec3<f32>(0.0);
    var optical_depth_r = 0.0;
    var optical_depth_m = 0.0;

    // March along view ray
    for (var i = 0; i < NUM_SAMPLES; i++) {
        let sample_pos = ray_origin + ray_dir * t;
        let height = length(sample_pos - atm.planet_center) - atm.planet_radius;

        // Local density
        let density_r = density_at_height(height, atm.rayleigh_scale_height);
        let density_m = density_at_height(height, atm.mie_scale_height);

        optical_depth_r += density_r * step_size;
        optical_depth_m += density_m * step_size;

        // Light ray to sun
        let sun_ray_intersect = ray_sphere_intersect(
            sample_pos, atm.sun_direction, atm.planet_center, atm.atmosphere_radius
        );

        let light_step_size = sun_ray_intersect.y / f32(NUM_LIGHT_SAMPLES);
        var light_t = light_step_size * 0.5;
        var light_optical_depth_r = 0.0;
        var light_optical_depth_m = 0.0;

        for (var j = 0; j < NUM_LIGHT_SAMPLES; j++) {
            let light_pos = sample_pos + atm.sun_direction * light_t;
            let light_height = length(light_pos - atm.planet_center) - atm.planet_radius;

            light_optical_depth_r += density_at_height(light_height, atm.rayleigh_scale_height) * light_step_size;
            light_optical_depth_m += density_at_height(light_height, atm.mie_scale_height) * light_step_size;

            light_t += light_step_size;
        }

        // Transmittance
        let tau = atm.rayleigh_coeff * (optical_depth_r + light_optical_depth_r)
                + vec3<f32>(atm.mie_coeff) * (optical_depth_m + light_optical_depth_m);
        let attenuation = exp(-tau);

        rayleigh_sum += density_r * attenuation * step_size;
        mie_sum += density_m * attenuation * step_size;

        t += step_size;
    }

    // Phase functions
    let cos_theta = dot(ray_dir, atm.sun_direction);
    let phase_r = rayleigh_phase(cos_theta);
    let phase_m = mie_phase(cos_theta, atm.mie_g);

    // Final color
    let scatter = atm.sun_intensity * (
        rayleigh_sum * atm.rayleigh_coeff * phase_r +
        mie_sum * atm.mie_coeff * phase_m
    );

    return scatter;
}

@fragment
fn fs_main(@location(0) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
    let ray_origin = camera.position;
    let ray_dir = normalize(world_pos - ray_origin);

    let scatter = calculate_scattering(ray_origin, ray_dir);

    // Tone mapping
    let color = vec3<f32>(1.0) - exp(-scatter);

    return vec4<f32>(color, 1.0);
}
```

### Precomputed LUT Approach

For better performance, precompute scattering into lookup tables:

```rust
/// Precomputed atmosphere lookup tables
pub struct AtmosphereLUT {
    /// Transmittance LUT: (height, view_zenith) → RGB transmittance
    /// Size: 256 × 64 × 3
    pub transmittance: Texture2D,

    /// Inscattering LUT: (height, view_zenith, sun_zenith, azimuth) → RGB
    /// Size: 32 × 128 × 32 × 8 (4D texture as 3D array)
    pub inscatter: Texture3D,

    /// Irradiance LUT: (height, sun_zenith) → RGB ground irradiance
    /// Size: 64 × 16 × 3
    pub irradiance: Texture2D,
}

impl AtmosphereLUT {
    pub fn compute(params: &AtmosphereParams, device: &Device) -> Self {
        // Compute transmittance LUT
        let transmittance = Self::compute_transmittance(params, device);

        // Compute inscatter using transmittance
        let inscatter = Self::compute_inscatter(params, &transmittance, device);

        // Compute ground irradiance
        let irradiance = Self::compute_irradiance(params, &transmittance, device);

        Self { transmittance, inscatter, irradiance }
    }

    fn compute_transmittance(params: &AtmosphereParams, device: &Device) -> Texture2D {
        // Dispatch compute shader to fill transmittance texture
        // For each (height, view_zenith), integrate optical depth along ray
        todo!("Implement compute shader")
    }
}
```

### Aerial Perspective

Apply atmosphere scattering to distant objects:

```wgsl
fn apply_aerial_perspective(
    surface_color: vec3<f32>,
    world_pos: vec3<f32>,
    camera_pos: vec3<f32>,
) -> vec3<f32> {
    let view_dir = normalize(world_pos - camera_pos);
    let distance = length(world_pos - camera_pos);

    // Sample transmittance along view ray
    let transmittance = sample_transmittance_lut(camera_pos, world_pos);

    // Sample inscattering along view ray
    let inscatter = sample_inscatter_lut(camera_pos, view_dir, distance);

    // Apply: color * transmittance + inscatter
    return surface_color * transmittance + inscatter;
}
```

---

## Volumetric Clouds

### Cloud Density Model

Clouds are modeled as density fields sampled with raymarching.

#### Noise-Based Density

```rust
/// Cloud layer parameters
pub struct CloudLayer {
    /// Altitude range (m)
    pub altitude_min: f32,
    pub altitude_max: f32,

    /// Coverage (0-1)
    pub coverage: f32,

    /// Density multiplier
    pub density: f32,

    /// Noise parameters
    pub noise_scale: f32,
    pub detail_scale: f32,

    /// Wind offset (animated)
    pub wind_offset: Vec3,
}

pub struct CloudParams {
    pub layers: Vec<CloudLayer>,

    /// Light absorption coefficient
    pub absorption: f32,

    /// Scattering coefficient
    pub scattering: f32,

    /// Phase function asymmetry
    pub phase_g: f32,

    /// Ambient light
    pub ambient: Vec3,
}
```

#### WGSL Cloud Density

```wgsl
struct CloudUniforms {
    altitude_min: f32,
    altitude_max: f32,
    coverage: f32,
    density: f32,
    noise_scale: f32,
    detail_scale: f32,
    wind_offset: vec3<f32>,
    time: f32,
}

@group(0) @binding(0) var<uniform> cloud: CloudUniforms;
@group(0) @binding(1) var noise_3d: texture_3d<f32>;
@group(0) @binding(2) var noise_sampler: sampler;

// Sample 3D noise
fn sample_noise(pos: vec3<f32>, scale: f32) -> f32 {
    return textureSample(noise_3d, noise_sampler, pos * scale).r;
}

// FBM noise
fn fbm_noise(pos: vec3<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var i = 0; i < octaves; i++) {
        value += amplitude * sample_noise(pos, frequency);
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    return value;
}

// Height-based density falloff
fn height_gradient(height: f32) -> f32 {
    let h = (height - cloud.altitude_min) / (cloud.altitude_max - cloud.altitude_min);
    // Anvil shape: dense at bottom, wispy at top
    return smoothstep(0.0, 0.1, h) * smoothstep(1.0, 0.7, h);
}

// Main cloud density function
fn cloud_density(world_pos: vec3<f32>) -> f32 {
    let height = length(world_pos) - PLANET_RADIUS;

    // Check altitude bounds
    if (height < cloud.altitude_min || height > cloud.altitude_max) {
        return 0.0;
    }

    // Animate with wind
    let animated_pos = world_pos + cloud.wind_offset * cloud.time;

    // Base shape noise (low frequency)
    let shape = fbm_noise(animated_pos * cloud.noise_scale, 4);

    // Detail noise (high frequency, subtractive)
    let detail = fbm_noise(animated_pos * cloud.detail_scale, 3);

    // Combine with coverage and height
    let height_factor = height_gradient(height);
    var density = shape * height_factor;

    // Remap with coverage
    density = remap(density, 1.0 - cloud.coverage, 1.0, 0.0, 1.0);

    // Subtract detail at edges
    density = max(0.0, density - detail * 0.3);

    return density * cloud.density;
}

fn remap(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    return new_min + (value - old_min) / (old_max - old_min) * (new_max - new_min);
}
```

### Cloud Raymarching

```wgsl
const CLOUD_STEPS: i32 = 64;
const LIGHT_STEPS: i32 = 6;

struct CloudResult {
    color: vec3<f32>,
    transmittance: f32,
}

fn raymarch_clouds(
    ray_origin: vec3<f32>,
    ray_dir: vec3<f32>,
    max_distance: f32,
) -> CloudResult {
    // Find cloud layer intersection
    let inner_sphere = ray_sphere_intersect(
        ray_origin, ray_dir, vec3<f32>(0.0), PLANET_RADIUS + cloud.altitude_min
    );
    let outer_sphere = ray_sphere_intersect(
        ray_origin, ray_dir, vec3<f32>(0.0), PLANET_RADIUS + cloud.altitude_max
    );

    var t_min = max(0.0, inner_sphere.y);
    var t_max = min(max_distance, outer_sphere.y);

    if (t_min >= t_max) {
        return CloudResult(vec3<f32>(0.0), 1.0);
    }

    let step_size = (t_max - t_min) / f32(CLOUD_STEPS);
    var t = t_min;

    var accumulated_color = vec3<f32>(0.0);
    var transmittance = 1.0;

    // Raymarch through clouds
    for (var i = 0; i < CLOUD_STEPS; i++) {
        if (transmittance < 0.01) {
            break;
        }

        let pos = ray_origin + ray_dir * t;
        let density = cloud_density(pos);

        if (density > 0.001) {
            // Light sampling (simplified)
            let light_energy = sample_light(pos);

            // Beer-Lambert absorption
            let sample_transmittance = exp(-density * step_size * cloud_params.absorption);

            // Accumulate color
            let sample_color = light_energy * cloud_params.scattering * density;
            accumulated_color += sample_color * transmittance * (1.0 - sample_transmittance);

            transmittance *= sample_transmittance;
        }

        t += step_size;
    }

    return CloudResult(accumulated_color, transmittance);
}

fn sample_light(pos: vec3<f32>) -> vec3<f32> {
    // March toward sun, accumulate optical depth
    var optical_depth = 0.0;
    let light_step = 200.0; // meters

    for (var i = 0; i < LIGHT_STEPS; i++) {
        let light_pos = pos + sun_direction * light_step * f32(i + 1);
        optical_depth += cloud_density(light_pos) * light_step;
    }

    // Beer-Lambert
    let light_transmittance = exp(-optical_depth * cloud_params.absorption);

    // Add ambient
    return sun_color * light_transmittance + cloud_params.ambient;
}
```

### Cloud Shadow Map

For shadows on terrain:

```rust
/// Render cloud shadows to a texture from sun's perspective
pub fn render_cloud_shadow_map(
    encoder: &mut CommandEncoder,
    cloud_params: &CloudParams,
    sun_direction: Vec3,
    shadow_map: &Texture,
) {
    // Orthographic projection from sun
    let shadow_size = 50_000.0; // 50km coverage
    let projection = Mat4::orthographic_rh(
        -shadow_size, shadow_size,
        -shadow_size, shadow_size,
        0.1, 100_000.0
    );

    // Raymarch clouds, output transmittance
    // Result is used in terrain shader as shadow
}
```

---

## Ocean Rendering

### Wave Simulation (FFT)

Realistic ocean waves using Fast Fourier Transform:

#### Phillips Spectrum

```rust
/// Ocean wave spectrum
pub fn phillips_spectrum(k: Vec2, wind: Vec2, amplitude: f32, gravity: f32) -> f32 {
    let k_len = k.length();
    if k_len < 0.0001 {
        return 0.0;
    }

    let k_len2 = k_len * k_len;
    let k_len4 = k_len2 * k_len2;

    let wind_speed = wind.length();
    let l = wind_speed * wind_speed / gravity; // Largest possible wave
    let l2 = l * l;

    let k_dot_w = k.dot(wind.normalize());
    let k_dot_w2 = k_dot_w * k_dot_w;

    // Phillips spectrum
    let phillips = amplitude * (-1.0 / (k_len2 * l2)).exp() / k_len4 * k_dot_w2;

    // Suppress small wavelengths
    let small_waves = 0.001;
    let suppression = (-k_len2 * small_waves * small_waves).exp();

    phillips * suppression
}

/// Initial spectrum H0
pub fn generate_initial_spectrum(
    size: usize,
    patch_size: f32,
    wind: Vec2,
    amplitude: f32,
) -> Vec<Complex32> {
    let mut h0 = vec![Complex32::new(0.0, 0.0); size * size];

    for y in 0..size {
        for x in 0..size {
            // Wave vector
            let kx = (2.0 * PI * (x as f32 - size as f32 / 2.0)) / patch_size;
            let ky = (2.0 * PI * (y as f32 - size as f32 / 2.0)) / patch_size;
            let k = Vec2::new(kx, ky);

            // Random Gaussian
            let xi = Complex32::new(rand_gaussian(), rand_gaussian());

            // Spectrum amplitude
            let p = phillips_spectrum(k, wind, amplitude, 9.81);
            let h = xi * (p / 2.0).sqrt();

            h0[y * size + x] = h;
        }
    }

    h0
}
```

#### Time Evolution

```rust
/// Evolve spectrum over time
pub fn evolve_spectrum(
    h0: &[Complex32],
    h0_conj: &[Complex32],
    size: usize,
    patch_size: f32,
    time: f32,
) -> Vec<Complex32> {
    let mut ht = vec![Complex32::new(0.0, 0.0); size * size];

    for y in 0..size {
        for x in 0..size {
            let kx = (2.0 * PI * (x as f32 - size as f32 / 2.0)) / patch_size;
            let ky = (2.0 * PI * (y as f32 - size as f32 / 2.0)) / patch_size;
            let k = Vec2::new(kx, ky).length();

            // Dispersion relation
            let omega = (9.81 * k).sqrt();

            // Time evolution
            let exp_pos = Complex32::from_polar(1.0, omega * time);
            let exp_neg = Complex32::from_polar(1.0, -omega * time);

            let idx = y * size + x;
            let idx_conj = ((size - y) % size) * size + ((size - x) % size);

            ht[idx] = h0[idx] * exp_pos + h0_conj[idx_conj] * exp_neg;
        }
    }

    ht
}

/// Compute displacement and normal maps from spectrum
pub fn compute_ocean_maps(
    ht: &[Complex32],
    size: usize,
    patch_size: f32,
) -> (Vec<Vec3>, Vec<Vec3>) {
    // IFFT for height
    let height = ifft_2d(ht, size);

    // Compute derivative spectrums for normal
    let dx_spectrum = compute_dx_spectrum(ht, size, patch_size);
    let dy_spectrum = compute_dy_spectrum(ht, size, patch_size);

    let dx = ifft_2d(&dx_spectrum, size);
    let dy = ifft_2d(&dy_spectrum, size);

    // Build displacement and normal vectors
    let mut displacement = vec![Vec3::ZERO; size * size];
    let mut normals = vec![Vec3::ZERO; size * size];

    for i in 0..size * size {
        displacement[i] = Vec3::new(
            dx[i].re * 0.5, // Horizontal displacement (choppiness)
            height[i].re,
            dy[i].re * 0.5,
        );

        let normal = Vec3::new(-dx[i].re, 1.0, -dy[i].re).normalize();
        normals[i] = normal;
    }

    (displacement, normals)
}
```

### Ocean Shader

```wgsl
struct OceanUniforms {
    sun_direction: vec3<f32>,
    sun_color: vec3<f32>,
    ocean_color_shallow: vec3<f32>,
    ocean_color_deep: vec3<f32>,
    fresnel_power: f32,
    roughness: f32,
    foam_threshold: f32,
}

@group(0) @binding(0) var<uniform> ocean: OceanUniforms;
@group(0) @binding(1) var displacement_map: texture_2d<f32>;
@group(0) @binding(2) var normal_map: texture_2d<f32>;
@group(0) @binding(3) var foam_map: texture_2d<f32>;
@group(0) @binding(4) var env_map: texture_cube<f32>;

@fragment
fn fs_ocean(
    @location(0) world_pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    // Sample displacement for LOD
    let displacement = textureSample(displacement_map, sampler_linear, uv).xyz;
    let displaced_pos = world_pos + displacement;

    // Sample normal
    let normal = textureSample(normal_map, sampler_linear, uv).xyz * 2.0 - 1.0;
    let N = normalize(normal);

    // View direction
    let V = normalize(camera.position - displaced_pos);

    // Fresnel
    let fresnel = pow(1.0 - max(0.0, dot(N, V)), ocean.fresnel_power);

    // Reflection
    let R = reflect(-V, N);
    let reflection = textureSample(env_map, sampler_linear, R).rgb;

    // Refraction (subsurface color)
    let depth_factor = 1.0 - exp(-displacement.y * 0.1);
    let refraction = mix(ocean.ocean_color_shallow, ocean.ocean_color_deep, depth_factor);

    // Combine with Fresnel
    var color = mix(refraction, reflection, fresnel);

    // Specular highlight (sun)
    let H = normalize(V + ocean.sun_direction);
    let spec = pow(max(0.0, dot(N, H)), 1.0 / ocean.roughness);
    color += ocean.sun_color * spec;

    // Foam
    let foam_amount = textureSample(foam_map, sampler_linear, uv).r;
    let foam = smoothstep(ocean.foam_threshold, ocean.foam_threshold + 0.1, foam_amount);
    color = mix(color, vec3<f32>(1.0), foam);

    return vec4<f32>(color, 1.0);
}
```

### Foam and Spray

```rust
/// Generate foam where waves break
pub fn compute_foam_map(
    displacement: &[Vec3],
    size: usize,
    jacobian_threshold: f32,
) -> Vec<f32> {
    let mut foam = vec![0.0; size * size];

    for y in 0..size {
        for x in 0..size {
            // Compute Jacobian (measures wave folding)
            let dx = displacement[(y * size + (x + 1) % size)].x
                   - displacement[(y * size + (x + size - 1) % size)].x;
            let dy = displacement[(((y + 1) % size) * size + x)].z
                   - displacement[(((y + size - 1) % size) * size + x)].z;

            let jacobian = 1.0 + dx + dy + dx * dy;

            // Foam where waves fold
            if jacobian < jacobian_threshold {
                foam[y * size + x] = 1.0 - jacobian / jacobian_threshold;
            }
        }
    }

    foam
}
```

---

## Planetary LOD System

### Distance-Based Representation

```rust
pub enum PlanetRepresentation {
    /// Very far: simple billboard/impostor
    Impostor {
        texture: TextureId,
        atmosphere_color: Vec3,
    },

    /// Far: low-poly sphere with texture
    LowPolySphere {
        mesh: MeshId,
        albedo_map: TextureId,
        atmosphere: AtmosphereParams,
    },

    /// Medium: quadtree terrain visible
    QuadtreeTerrain {
        visible_tiles: Vec<PlanetTileId>,
        atmosphere: AtmosphereParams,
        clouds: Option<CloudParams>,
    },

    /// Close: full detail terrain + all effects
    FullDetail {
        terrain: TerrainRenderer,
        atmosphere: AtmosphereRenderer,
        clouds: CloudRenderer,
        ocean: Option<OceanRenderer>,
    },
}

pub fn select_representation(distance_to_surface: f64, planet_radius: f64) -> PlanetRepresentation {
    let normalized_distance = distance_to_surface / planet_radius;

    if normalized_distance > 100.0 {
        // > 100 radii away: impostor
        PlanetRepresentation::Impostor { .. }
    } else if normalized_distance > 10.0 {
        // 10-100 radii: low poly sphere
        PlanetRepresentation::LowPolySphere { .. }
    } else if normalized_distance > 0.1 {
        // 0.1-10 radii: quadtree terrain
        PlanetRepresentation::QuadtreeTerrain { .. }
    } else {
        // Close: full detail
        PlanetRepresentation::FullDetail { .. }
    }
}
```

### Impostor Generation

```rust
/// Generate planet impostor texture
pub fn generate_planet_impostor(
    planet: &Planet,
    resolution: u32,
    device: &Device,
) -> Texture {
    // Render planet from multiple angles
    // Store: albedo + atmosphere rim

    let views = [
        // Front, back, left, right, top, bottom
        Quat::IDENTITY,
        Quat::from_rotation_y(PI),
        Quat::from_rotation_y(FRAC_PI_2),
        Quat::from_rotation_y(-FRAC_PI_2),
        Quat::from_rotation_x(-FRAC_PI_2),
        Quat::from_rotation_x(FRAC_PI_2),
    ];

    // Render each view to texture atlas
    // Include atmosphere glow in alpha channel
    todo!("Implement impostor rendering")
}
```

### Transition Blending

```rust
/// Blend between LOD levels smoothly
pub struct PlanetLodBlender {
    current_lod: u32,
    target_lod: u32,
    blend_factor: f32,
    blend_duration: f32,
}

impl PlanetLodBlender {
    pub fn update(&mut self, dt: f32, target_distance: f64, planet_radius: f64) {
        let new_target_lod = self.calculate_target_lod(target_distance, planet_radius);

        if new_target_lod != self.target_lod {
            self.current_lod = self.target_lod;
            self.target_lod = new_target_lod;
            self.blend_factor = 0.0;
        }

        self.blend_factor = (self.blend_factor + dt / self.blend_duration).min(1.0);
    }

    pub fn render(&self, planet: &Planet, renderer: &mut Renderer) {
        if self.blend_factor < 1.0 {
            // Render both LODs and blend
            renderer.render_planet_lod(planet, self.current_lod, 1.0 - self.blend_factor);
            renderer.render_planet_lod(planet, self.target_lod, self.blend_factor);
        } else {
            renderer.render_planet_lod(planet, self.target_lod, 1.0);
        }
    }
}
```

---

## Special Effects

### Aurora Borealis

```wgsl
fn aurora_density(pos: vec3<f32>, time: f32) -> f32 {
    let altitude = length(pos) - PLANET_RADIUS;

    // Aurora occurs at ~100-300 km altitude
    if (altitude < 100000.0 || altitude > 300000.0) {
        return 0.0;
    }

    // Only near magnetic poles (simplified: near actual poles)
    let latitude = asin(pos.y / length(pos));
    if (abs(latitude) < 1.0) { // < ~60° from equator
        return 0.0;
    }

    // Curtain-like noise
    let noise_pos = vec3<f32>(pos.x * 0.00001, time * 0.1, pos.z * 0.00001);
    let curtain = fbm_noise(noise_pos, 4);

    // Vertical streaks
    let streak_noise = sin(pos.x * 0.0001 + time) * sin(pos.z * 0.0001 + time * 0.7);

    let density = curtain * (0.5 + streak_noise * 0.5);

    // Height falloff
    let height_factor = exp(-(altitude - 150000.0) * (altitude - 150000.0) / (50000.0 * 50000.0));

    return density * height_factor;
}

fn aurora_color(density: f32, altitude: f32) -> vec3<f32> {
    // Green at lower altitudes (oxygen), red/purple higher
    let t = (altitude - 100000.0) / 200000.0;
    let green = vec3<f32>(0.2, 1.0, 0.3);
    let red = vec3<f32>(1.0, 0.2, 0.3);
    let purple = vec3<f32>(0.5, 0.2, 1.0);

    var color = mix(green, red, smoothstep(0.3, 0.7, t));
    color = mix(color, purple, smoothstep(0.7, 1.0, t));

    return color * density;
}
```

### Planetary Rings

```rust
pub struct PlanetaryRing {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub color: Vec3,
    pub opacity_texture: TextureId,
    pub normal_map: TextureId,
    pub thickness: f32,
}

impl PlanetaryRing {
    pub fn render(&self, encoder: &mut CommandEncoder, planet_pos: Vec3, sun_dir: Vec3) {
        // Render ring as a disc
        // Sample opacity texture radially
        // Apply shadow from planet
        // Self-shadowing for multiple ring bands
    }
}
```

### Eclipse Effects

```rust
pub fn calculate_eclipse_factor(
    point: DVec3,
    sun_pos: DVec3,
    occluder_pos: DVec3,
    occluder_radius: f64,
    sun_radius: f64,
) -> f32 {
    let to_sun = (sun_pos - point).normalize();
    let to_occluder = (occluder_pos - point).normalize();

    // Angular sizes
    let sun_distance = (sun_pos - point).length();
    let occluder_distance = (occluder_pos - point).length();

    let sun_angular = (sun_radius / sun_distance).atan();
    let occluder_angular = (occluder_radius / occluder_distance).atan();

    // Angular separation
    let separation = to_sun.dot(to_occluder).acos();

    // Calculate coverage
    if separation > sun_angular + occluder_angular {
        // No eclipse
        1.0
    } else if separation < occluder_angular - sun_angular {
        // Total eclipse (if occluder > sun angular)
        if occluder_angular > sun_angular {
            0.0 // Total
        } else {
            // Annular eclipse
            let covered_ratio = (occluder_angular / sun_angular).powi(2);
            1.0 - covered_ratio as f32
        }
    } else {
        // Partial eclipse - compute overlap area
        compute_circle_overlap(sun_angular, occluder_angular, separation) as f32
    }
}
```

---

## Performance Optimization

### Budget Allocation

```
Target: 16ms frame time (60 FPS)

Budget allocation:
├── Atmosphere: 2ms
│   └── LUT lookup, aerial perspective
├── Clouds: 4ms
│   └── Raymarching with temporal reprojection
├── Ocean: 2ms
│   └── FFT (async), shading
├── Terrain: 5ms
│   └── Already budgeted separately
├── Post-processing: 2ms
│   └── Tone mapping, bloom
└── Overhead: 1ms
```

### Temporal Reprojection

Reuse previous frame's results where possible:

```rust
pub struct TemporalReprojector {
    history_buffer: Texture,
    motion_vectors: Texture,
    frame_count: u32,
}

impl TemporalReprojector {
    pub fn reproject_clouds(
        &mut self,
        current_frame: &Texture,
        camera_motion: &CameraMotion,
    ) -> Texture {
        // Sample history at reprojected position
        // Blend with current frame
        // Handle disocclusion (new areas)
        todo!()
    }
}
```

### Resolution Scaling

```rust
pub struct AdaptiveResolution {
    /// Target frame time
    target_ms: f32,

    /// Current resolution scale (0.5 - 1.0)
    scale: f32,

    /// Per-effect scales
    atmosphere_scale: f32,
    cloud_scale: f32,
    ocean_scale: f32,
}

impl AdaptiveResolution {
    pub fn update(&mut self, last_frame_ms: f32) {
        if last_frame_ms > self.target_ms * 1.1 {
            // Reduce quality
            self.scale = (self.scale - 0.05).max(0.5);
        } else if last_frame_ms < self.target_ms * 0.9 {
            // Increase quality
            self.scale = (self.scale + 0.01).min(1.0);
        }
    }
}
```

---

## Implementation Notes

### Suggested Crate Structure

```
engine/crates/
├── syn_atmosphere/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── params.rs         # AtmosphereParams
│   │   ├── scattering.rs     # Scattering math
│   │   ├── lut.rs            # LUT generation
│   │   ├── render.rs         # Rendering
│   │   └── shaders/
│   │       ├── atmosphere.wgsl
│   │       └── aerial_perspective.wgsl
├── syn_clouds/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── density.rs        # Cloud density
│   │   ├── raymarch.rs       # Raymarching
│   │   ├── shadows.rs        # Cloud shadows
│   │   └── shaders/
│   │       └── clouds.wgsl
├── syn_ocean/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── fft.rs            # FFT waves
│   │   ├── spectrum.rs       # Wave spectrum
│   │   ├── foam.rs           # Foam generation
│   │   └── shaders/
│   │       ├── ocean.wgsl
│   │       └── fft.wgsl
└── syn_planet_render/
    ├── src/
    │   ├── lib.rs
    │   ├── lod.rs            # LOD system
    │   ├── impostor.rs       # Planet impostors
    │   ├── effects.rs        # Aurora, rings, eclipse
    │   └── compositor.rs     # Combine all effects
```

---

## References

### Papers

1. **Atmospheric Scattering**
   - Bruneton, E., & Neyret, F. (2008). "Precomputed Atmospheric Scattering"
   - Hillaire, S. (2020). "A Scalable and Production Ready Sky and Atmosphere Rendering Technique" (Unreal)

2. **Clouds**
   - Schneider, A. (2015). "The Real-time Volumetric Cloudscapes of Horizon Zero Dawn"
   - Hillaire, S. (2016). "Physically Based Sky, Atmosphere and Cloud Rendering in Frostbite"

3. **Ocean**
   - Tessendorf, J. (2001). "Simulating Ocean Water"
   - Bruneton, E. et al. (2010). "Real-time Realistic Ocean Lighting using Seamless Transitions from Geometry to BRDF"

### Games

- **No Man's Sky** - Planet-scale atmospheric rendering
- **Sea of Thieves** - Real-time ocean
- **Horizon Zero Dawn** - Volumetric clouds
- **Red Dead Redemption 2** - Atmospheric effects

### Online Resources

- GPU Gems 2: Chapter 16 (Atmosphere)
- Alan Zucconi's atmosphere tutorials
- Sebastian Lague's cloud rendering video

---

## Ideas & Future Work

### To Research

- [ ] **Multiple scattering**: More accurate but expensive
- [ ] **Cloud shadows on ocean**: Correct light sampling
- [ ] **Underwater rendering**: Caustics, volumetric light
- [ ] **Weather transitions**: Rain, storms, snow
- [ ] **Gas giant rendering**: Different atmosphere model

### Optimization Ideas

- [ ] **Async FFT**: Compute ocean FFT across frames
- [ ] **Hierarchical cloud raymarching**: Skip empty regions
- [ ] **Atmosphere LUT streaming**: Load on approach
- [ ] **Variable rate shading**: Lower quality at screen edges

### Visual Quality Ideas

- [ ] **God rays**: Volumetric light shafts
- [ ] **Rainbow**: After rain, correct position
- [ ] **Starfield**: Procedural at night
- [ ] **Satellites**: Visible points of light

### Notes

```
2026-01-22: Initial document creation
- Started with atmospheric scattering equations
- Need to implement and test FFT ocean
- Cloud raymarching needs optimization
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
