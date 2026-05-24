# Planetary Simulation - Research Document

**Version**: 0.1.0
**Date**: 2026-01-22
**Status**: Active Research
**Crate**: `syn_planet_sim`

---

## Table of Contents

1. [Overview](#overview)
2. [Orbital Mechanics](#orbital-mechanics)
3. [Tectonic Simulation](#tectonic-simulation)
4. [Hydrology System](#hydrology-system)
5. [Atmospheric Simulation](#atmospheric-simulation)
6. [Climate Calculation](#climate-calculation)
7. [Biome Classification](#biome-classification)
8. [Implementation Notes](#implementation-notes)
9. [References](#references)
10. [Ideas & Future Work](#ideas--future-work)

---

## Overview

This document contains the research, equations, algorithms, and prototype code for the planetary simulation system of Synarion Engine.

### Goal

Create physically-plausible planets where:

- Climate is derived from orbital parameters, not random
- Biomes are classified from climate data, not painted
- Rivers flow downhill, always
- Weather patterns follow atmospheric physics

### Non-Goals (for v1)

- Perfect scientific accuracy (we're making games, not NASA simulations)
- Real-time plate tectonics (pre-computed at world generation)
- Full weather simulation (climate averages are sufficient)

---

## Orbital Mechanics

### Stellar Luminosity and Habitable Zone

The habitable zone (liquid water possible) depends on stellar luminosity.

#### Equations

**Stefan-Boltzmann Law** (stellar surface):

```
L = 4π R² σ T⁴

Where:
- L = Luminosity (W)
- R = Star radius (m)
- σ = Stefan-Boltzmann constant = 5.67 × 10⁻⁸ W·m⁻²·K⁻⁴
- T = Surface temperature (K)
```

**Flux at distance d**:

```
F = L / (4π d²)  [W/m²]
```

**Equilibrium temperature** (without atmosphere):

```
T_eq = (L (1 - A) / (16 π σ d²))^(1/4)

Where:
- A = planetary albedo (0.0-1.0, Earth ≈ 0.3)
- d = semi-major axis (m)
```

**Habitable zone boundaries** (simplified):

```
d_inner = √(L / L_sun) × 0.95 AU
d_outer = √(L / L_sun) × 1.37 AU
```

#### Rust Prototype

```rust
/// Solar constants
const STEFAN_BOLTZMANN: f64 = 5.670374419e-8; // W·m⁻²·K⁻⁴
const SOLAR_LUMINOSITY: f64 = 3.828e26;       // W
const AU_METERS: f64 = 1.496e11;              // m

/// Calculate equilibrium temperature for a planet
pub fn equilibrium_temperature(
    star_luminosity: f64,  // In solar luminosities
    distance_au: f64,
    albedo: f64,
) -> f64 {
    let l = star_luminosity * SOLAR_LUMINOSITY;
    let d = distance_au * AU_METERS;
    let numerator = l * (1.0 - albedo);
    let denominator = 16.0 * std::f64::consts::PI * STEFAN_BOLTZMANN * d * d;
    (numerator / denominator).powf(0.25)
}

/// Calculate habitable zone for a star
pub fn habitable_zone(star_luminosity: f64) -> (f64, f64) {
    let sqrt_l = star_luminosity.sqrt();
    let inner = sqrt_l * 0.95;  // AU
    let outer = sqrt_l * 1.37;  // AU
    (inner, outer)
}

#[test]
fn test_earth() {
    let t_eq = equilibrium_temperature(1.0, 1.0, 0.3);
    // Should be ~255K (-18°C) without greenhouse effect
    assert!((t_eq - 255.0).abs() < 5.0);
}
```

### Seasonal Variation

Seasons are caused by axial tilt + orbital position.

#### Equations

**Solar declination** (simplified, circular orbit):

```
δ = ε × sin(2π × (d - d_equinox) / P_orbital)

Where:
- δ = solar declination angle
- ε = axial tilt (radians)
- d = current day
- d_equinox = day of vernal equinox
- P_orbital = orbital period (days)
```

**Insolation at latitude φ**:

```
Q = S × (cos(φ) × cos(δ) × sin(H) + H × sin(φ) × sin(δ)) / π

Where:
- S = solar constant at planet distance
- φ = latitude
- δ = solar declination
- H = hour angle at sunset (complex calculation)
```

#### Rust Prototype

```rust
/// Calculate solar declination for a given day
pub fn solar_declination(
    axial_tilt: f64,      // radians
    day_of_year: f64,
    days_in_year: f64,
    day_of_vernal_equinox: f64,
) -> f64 {
    let fraction = (day_of_year - day_of_vernal_equinox) / days_in_year;
    axial_tilt * (2.0 * std::f64::consts::PI * fraction).sin()
}

/// Simplified insolation factor at latitude
pub fn insolation_factor(latitude: f64, declination: f64) -> f64 {
    let cos_lat = latitude.cos();
    let cos_dec = declination.cos();
    let sin_lat = latitude.sin();
    let sin_dec = declination.sin();

    // Simplified: this should include day length calculation
    let factor = cos_lat * cos_dec + sin_lat * sin_dec;
    factor.max(0.0) // Can't be negative (polar night)
}
```

### Coriolis Effect

The Coriolis effect deflects moving air/water due to planetary rotation.

```
f = 2 × Ω × sin(φ)

Where:
- f = Coriolis parameter (s⁻¹)
- Ω = angular velocity = 2π / rotation_period
- φ = latitude
```

```rust
/// Calculate Coriolis parameter at latitude
pub fn coriolis_parameter(rotation_period: f64, latitude: f64) -> f64 {
    let omega = 2.0 * std::f64::consts::PI / rotation_period;
    2.0 * omega * latitude.sin()
}
```

---

## Tectonic Simulation

### Plate Generation Algorithm

1. **Seed points**: Poisson disk sampling on sphere surface
2. **Voronoi**: Calculate Voronoi cells → plate boundaries
3. **Classification**: Continental (thick, light) vs Oceanic (thin, dense)
4. **Velocity assignment**: Random velocities with continuity constraints

#### Pseudocode

```
function generate_plates(seed, num_plates):
    // 1. Generate seed points on sphere
    points = poisson_disk_sample_sphere(num_plates * 1.5)

    // 2. Build spherical Voronoi
    voronoi = spherical_voronoi(points)
    plates = voronoi.cells[:num_plates]

    // 3. Classify plates
    for plate in plates:
        if random(seed, plate.id) < 0.3:
            plate.type = Continental
            plate.density = 2.7  // g/cm³
        else:
            plate.type = Oceanic
            plate.density = 3.0  // g/cm³

    // 4. Assign velocities (cm/year)
    for plate in plates:
        plate.velocity = random_direction(seed, plate.id) * random_magnitude(1, 10)

    return plates
```

### Boundary Type Detection

```rust
pub enum BoundaryType {
    Divergent,   // Plates moving apart
    Convergent,  // Plates moving together
    Transform,   // Plates sliding past each other
}

pub fn classify_boundary(
    plate_a: &TectonicPlate,
    plate_b: &TectonicPlate,
    boundary_normal: Vec3,
) -> BoundaryType {
    // Relative velocity along boundary normal
    let relative_vel = plate_a.velocity - plate_b.velocity;
    let normal_component = relative_vel.dot(boundary_normal);
    let tangent_component = (relative_vel - boundary_normal * normal_component).length();

    if normal_component.abs() < 0.1 * tangent_component {
        BoundaryType::Transform
    } else if normal_component < 0.0 {
        BoundaryType::Convergent
    } else {
        BoundaryType::Divergent
    }
}
```

### Orogeny (Mountain Building)

At convergent boundaries, mountains form based on:

- **Continent-continent**: Himalayas (highest, no volcanism)
- **Oceanic-continental**: Andes (volcanic arc)
- **Oceanic-oceanic**: Island arc (Japan)

```rust
pub fn calculate_elevation_contribution(
    boundary: &PlateBoundary,
    distance_from_boundary: f32,
) -> f32 {
    match boundary.boundary_type {
        BoundaryType::Convergent => {
            // Mountain height decays exponentially from boundary
            let max_height = match (boundary.plate_a.is_continental(), boundary.plate_b.is_continental()) {
                (true, true) => 8000.0,   // Continent-continent
                (true, false) | (false, true) => 5000.0, // Ocean-continent
                (false, false) => 3000.0, // Ocean-ocean
            };
            let decay_distance = 200_000.0; // 200km
            max_height * (-distance_from_boundary / decay_distance).exp()
        }
        BoundaryType::Divergent => {
            // Rift valley or mid-ocean ridge
            let depth = 2000.0;
            let width = 50_000.0;
            -depth * (-distance_from_boundary.powi(2) / (2.0 * width * width)).exp()
        }
        BoundaryType::Transform => {
            0.0 // No significant elevation change
        }
    }
}
```

---

## Hydrology System

### Drainage Basin Algorithm

The goal: find where water flows on a terrain.

#### Algorithm: Priority-Flood

A robust algorithm for calculating flow direction:

```
function calculate_flow_direction(heightmap):
    // Initialize
    flow_dir = array of NONE
    visited = array of false

    // Priority queue: (elevation, x, y) - min-heap
    queue = PriorityQueue()

    // Seed with ocean cells (or boundary cells)
    for each cell at boundary or with elevation < sea_level:
        queue.push((cell.elevation, cell.x, cell.y))
        visited[cell] = true

    // Flood fill from lowest to highest
    while queue not empty:
        (elev, x, y) = queue.pop()

        for each neighbor (nx, ny) of (x, y):
            if not visited[neighbor]:
                visited[neighbor] = true
                flow_dir[neighbor] = direction_to(neighbor, (x, y))

                // Use max to handle flat areas (water must flow somewhere)
                effective_elev = max(heightmap[neighbor], elev)
                queue.push((effective_elev, nx, ny))

    return flow_dir
```

#### Rust Prototype

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlowDirection {
    None,
    North, South, East, West,
    NorthEast, NorthWest, SouthEast, SouthWest,
}

pub fn calculate_flow_direction(
    heightmap: &[f32],
    width: usize,
    height: usize,
    sea_level: f32,
) -> Vec<FlowDirection> {
    let mut flow_dir = vec![FlowDirection::None; width * height];
    let mut visited = vec![false; width * height];

    // Min-heap: (elevation * 1000 as i32, index)
    let mut queue: BinaryHeap<Reverse<(i32, usize)>> = BinaryHeap::new();

    // Seed with boundary and underwater cells
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let is_boundary = x == 0 || x == width - 1 || y == 0 || y == height - 1;
            let is_underwater = heightmap[idx] < sea_level;

            if is_boundary || is_underwater {
                let elev_key = (heightmap[idx] * 1000.0) as i32;
                queue.push(Reverse((elev_key, idx)));
                visited[idx] = true;
            }
        }
    }

    // Process queue
    while let Some(Reverse((_, idx))) = queue.pop() {
        let x = idx % width;
        let y = idx / width;
        let current_elev = heightmap[idx];

        // Check all 8 neighbors
        for (dx, dy, dir) in neighbors() {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nidx = ny as usize * width + nx as usize;

                if !visited[nidx] {
                    visited[nidx] = true;
                    flow_dir[nidx] = opposite_direction(dir);

                    // Ensure water can flow (handle flat areas)
                    let effective_elev = heightmap[nidx].max(current_elev);
                    let elev_key = (effective_elev * 1000.0) as i32;
                    queue.push(Reverse((elev_key, nidx)));
                }
            }
        }
    }

    flow_dir
}

fn neighbors() -> [(i32, i32, FlowDirection); 8] {
    [
        (0, -1, FlowDirection::North),
        (0, 1, FlowDirection::South),
        (1, 0, FlowDirection::East),
        (-1, 0, FlowDirection::West),
        (1, -1, FlowDirection::NorthEast),
        (-1, -1, FlowDirection::NorthWest),
        (1, 1, FlowDirection::SouthEast),
        (-1, 1, FlowDirection::SouthWest),
    ]
}

fn opposite_direction(dir: FlowDirection) -> FlowDirection {
    match dir {
        FlowDirection::North => FlowDirection::South,
        FlowDirection::South => FlowDirection::North,
        FlowDirection::East => FlowDirection::West,
        FlowDirection::West => FlowDirection::East,
        FlowDirection::NorthEast => FlowDirection::SouthWest,
        FlowDirection::NorthWest => FlowDirection::SouthEast,
        FlowDirection::SouthEast => FlowDirection::NorthWest,
        FlowDirection::SouthWest => FlowDirection::NorthEast,
        FlowDirection::None => FlowDirection::None,
    }
}
```

### River Discharge Calculation

Water accumulates as it flows downstream:

```
function calculate_discharge(flow_dir, precipitation):
    discharge = copy(precipitation)  // Start with local precipitation

    // Process cells from highest to lowest
    sorted_cells = sort_by_elevation_descending(all_cells)

    for cell in sorted_cells:
        downstream = follow_flow_direction(cell, flow_dir)
        if downstream exists:
            discharge[downstream] += discharge[cell]

    return discharge
```

### River Width from Discharge

Empirical relationship (Leopold & Maddock, 1953):

```
W = a × Q^b

Where:
- W = channel width (m)
- Q = discharge (m³/s)
- a ≈ 2.7
- b ≈ 0.5
```

```rust
pub fn river_width(discharge_m3s: f32) -> f32 {
    2.7 * discharge_m3s.powf(0.5)
}

pub fn river_depth(discharge_m3s: f32) -> f32 {
    0.3 * discharge_m3s.powf(0.4)
}
```

---

## Atmospheric Simulation

### Pressure and Scale Height

Pressure decreases exponentially with altitude:

```
P(h) = P_0 × exp(-h / H)

Where:
- P_0 = surface pressure
- h = altitude
- H = scale height = kT / (mg) ≈ 8.5 km for Earth
```

```rust
pub fn pressure_at_altitude(
    surface_pressure: f32,
    altitude: f32,
    scale_height: f32,
) -> f32 {
    surface_pressure * (-altitude / scale_height).exp()
}

/// Calculate scale height from atmosphere parameters
pub fn scale_height(
    temperature: f32,      // K
    mean_molecular_mass: f32, // kg/mol
    surface_gravity: f32,  // m/s²
) -> f32 {
    const BOLTZMANN: f32 = 1.380649e-23; // J/K
    const AVOGADRO: f32 = 6.02214076e23;

    let k = BOLTZMANN;
    let m = mean_molecular_mass / AVOGADRO;
    let g = surface_gravity;

    (k * temperature) / (m * g)
}
```

### Atmospheric Cells (Hadley-Ferrel-Polar)

The number and position of cells depends on rotation speed:

```rust
/// Calculate number of atmospheric cells based on rotation
pub fn estimate_cell_count(rotation_period_hours: f32) -> u32 {
    // Earth (24h) has 3 cells per hemisphere
    // Fast rotation → more cells, slower → fewer
    let earth_period = 24.0;
    let ratio = earth_period / rotation_period_hours;

    // Rough approximation
    if ratio < 0.5 {
        1  // Very slow rotation: single cell (Venus-like)
    } else if ratio < 1.5 {
        3  // Earth-like
    } else if ratio < 3.0 {
        4  // Fast rotation
    } else {
        6  // Very fast (gas giant style)
    }
}

/// Get latitude boundaries for atmospheric cells (Earth-like, 3 cells)
pub fn cell_boundaries_3() -> [(f32, f32, &'static str); 3] {
    [
        (0.0, 30.0, "Hadley"),   // Tropical
        (30.0, 60.0, "Ferrel"),  // Mid-latitude
        (60.0, 90.0, "Polar"),   // Polar
    ]
}
```

### Wind Direction

```rust
/// Calculate dominant wind direction at latitude
pub fn dominant_wind_direction(
    latitude: f32,        // degrees, positive = north
    rotation_direction: f32, // 1.0 = prograde, -1.0 = retrograde
) -> Vec2 {
    let lat_abs = latitude.abs();
    let hemisphere_sign = if latitude >= 0.0 { 1.0 } else { -1.0 };

    // Base wind direction (without Coriolis)
    let (meridional, zonal) = if lat_abs < 30.0 {
        // Hadley cell: surface winds toward equator
        (-hemisphere_sign, 0.0) // Equatorward
    } else if lat_abs < 60.0 {
        // Ferrel cell: surface winds toward pole
        (hemisphere_sign, 0.0)  // Poleward
    } else {
        // Polar cell: surface winds toward equator
        (-hemisphere_sign, 0.0)
    };

    // Apply Coriolis deflection
    let coriolis_deflection = rotation_direction * hemisphere_sign;
    let zonal_coriolis = if lat_abs < 30.0 {
        -coriolis_deflection  // Trade winds: easterly
    } else if lat_abs < 60.0 {
        coriolis_deflection   // Westerlies
    } else {
        -coriolis_deflection  // Polar easterlies
    };

    Vec2::new(zonal + zonal_coriolis, meridional).normalize()
}
```

### Greenhouse Effect

Simple approximation:

```rust
/// Calculate greenhouse warming factor
pub fn greenhouse_factor(
    co2_ppm: f32,
    h2o_fraction: f32,
    ch4_ppm: f32,
) -> f32 {
    // Logarithmic relationship for CO2
    let co2_contribution = 0.15 * (co2_ppm / 280.0).ln().max(0.0);

    // Water vapor contribution (strong greenhouse gas)
    let h2o_contribution = 0.20 * h2o_fraction;

    // Methane (less significant)
    let ch4_contribution = 0.02 * (ch4_ppm / 1.7);

    // Total warming factor (Earth ≈ 0.33 = 33°C warming)
    co2_contribution + h2o_contribution + ch4_contribution
}
```

---

## Climate Calculation

### Temperature Map

```rust
pub fn calculate_temperature(
    latitude: f32,        // radians
    altitude: f32,        // meters
    distance_to_ocean: f32, // km (for continentality)
    season_factor: f32,   // -1 to 1 (winter to summer)
    base_temp: f32,       // equilibrium temp at equator, sea level
    axial_tilt: f32,      // radians
) -> f32 {
    // Latitude effect
    let lat_factor = latitude.cos();

    // Seasonal effect (stronger at higher latitudes)
    let seasonal = axial_tilt.sin() * season_factor * (1.0 - lat_factor);

    // Altitude lapse rate: ~6.5°C per 1000m
    let altitude_effect = -6.5 * (altitude / 1000.0);

    // Continentality: more extreme inland
    let continentality = 0.01 * distance_to_ocean; // ±°C amplitude

    // Combine
    base_temp * (lat_factor + seasonal * 0.3) + altitude_effect
}
```

### Precipitation Map

```rust
pub struct PrecipitationFactors {
    pub ocean_distance: f32,     // km
    pub wind_from_ocean: bool,
    pub altitude: f32,           // m
    pub orographic_factor: f32,  // 0-1 (windward vs leeward)
    pub latitude: f32,           // radians
    pub temperature: f32,        // °C
}

pub fn calculate_precipitation(factors: &PrecipitationFactors) -> f32 {
    // Base moisture from ocean
    let moisture = if factors.wind_from_ocean {
        1000.0 * (-factors.ocean_distance / 500.0).exp()
    } else {
        200.0 * (-factors.ocean_distance / 300.0).exp()
    };

    // Orographic effect (windward side gets rain)
    let orographic = if factors.orographic_factor > 0.5 {
        // Windward: enhanced precipitation
        1.0 + (factors.altitude / 2000.0).min(2.0)
    } else {
        // Leeward: rain shadow
        0.3
    };

    // ITCZ boost near equator
    let itcz_boost = if factors.latitude.abs() < 0.17 { // ~10°
        1.5
    } else {
        1.0
    };

    // Temperature affects capacity
    let temp_factor = 1.0 + 0.02 * factors.temperature.max(0.0);

    moisture * orographic * itcz_boost * temp_factor
}
```

### Orographic Effect Detection

To know if a location is windward or leeward:

```rust
pub fn calculate_orographic_factor(
    position: Vec2,
    heightmap: &Heightmap,
    wind_direction: Vec2,
    sample_distance: f32,
) -> f32 {
    // Sample terrain in upwind direction
    let upwind_pos = position - wind_direction * sample_distance;
    let upwind_height = heightmap.sample(upwind_pos);
    let current_height = heightmap.sample(position);

    // If terrain rises in upwind direction, we're on windward side
    let slope = (current_height - upwind_height) / sample_distance;

    // Convert to 0-1 factor (0 = strong leeward, 1 = strong windward)
    (slope * 100.0 + 0.5).clamp(0.0, 1.0)
}
```

---

## Biome Classification

### Köppen-Geiger Complete Implementation

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KoppenClimate {
    // Tropical (A)
    Af,  // Tropical rainforest
    Am,  // Tropical monsoon
    Aw,  // Tropical savanna (dry winter)
    As,  // Tropical savanna (dry summer)

    // Arid (B)
    BWh, // Hot desert
    BWk, // Cold desert
    BSh, // Hot semi-arid (steppe)
    BSk, // Cold semi-arid (steppe)

    // Temperate (C)
    Cfa, // Humid subtropical (no dry season, hot summer)
    Cfb, // Oceanic (no dry season, warm summer)
    Cfc, // Subpolar oceanic
    Csa, // Mediterranean (dry summer, hot)
    Csb, // Mediterranean (dry summer, warm)
    Csc, // Mediterranean (dry summer, cold)
    Cwa, // Humid subtropical (dry winter, hot summer)
    Cwb, // Subtropical highland
    Cwc, // Subtropical highland (cold)

    // Continental (D)
    Dfa, // Hot-summer humid continental
    Dfb, // Warm-summer humid continental
    Dfc, // Subarctic
    Dfd, // Extremely cold subarctic
    Dsa, // Mediterranean-influenced hot-summer
    Dsb, // Mediterranean-influenced warm-summer
    Dsc, // Mediterranean-influenced subarctic
    Dsd, // Mediterranean-influenced extremely cold
    Dwa, // Monsoon-influenced hot-summer
    Dwb, // Monsoon-influenced warm-summer
    Dwc, // Monsoon-influenced subarctic
    Dwd, // Monsoon-influenced extremely cold

    // Polar (E)
    ET,  // Tundra
    EF,  // Ice cap
}

pub struct ClimateInput {
    pub t_warmest_month: f32,     // °C
    pub t_coldest_month: f32,     // °C
    pub t_annual_mean: f32,       // °C
    pub p_annual: f32,            // mm/year
    pub p_driest_month: f32,      // mm
    pub p_wettest_month: f32,     // mm
    pub p_summer: f32,            // mm (6 warmest months)
    pub p_winter: f32,            // mm (6 coldest months)
}

pub fn classify_koppen(input: &ClimateInput) -> KoppenClimate {
    let t_hot = input.t_warmest_month;
    let t_cold = input.t_coldest_month;
    let t_mean = input.t_annual_mean;
    let p_ann = input.p_annual;
    let p_dry = input.p_driest_month;
    let p_wet = input.p_wettest_month;

    // === Group E: Polar ===
    if t_hot < 10.0 {
        return if t_hot < 0.0 { KoppenClimate::EF } else { KoppenClimate::ET };
    }

    // === Group B: Arid ===
    // Precipitation threshold depends on temperature and seasonality
    let p_threshold = calculate_aridity_threshold(input);
    if p_ann < p_threshold {
        let is_hot = t_mean >= 18.0;
        let is_desert = p_ann < p_threshold / 2.0;

        return match (is_desert, is_hot) {
            (true, true) => KoppenClimate::BWh,
            (true, false) => KoppenClimate::BWk,
            (false, true) => KoppenClimate::BSh,
            (false, false) => KoppenClimate::BSk,
        };
    }

    // === Group A: Tropical ===
    if t_cold >= 18.0 {
        return if p_dry >= 60.0 {
            KoppenClimate::Af  // Rainforest
        } else if p_ann >= 25.0 * (100.0 - p_dry) {
            KoppenClimate::Am  // Monsoon
        } else {
            KoppenClimate::Aw  // Savanna
        };
    }

    // === Groups C and D ===
    let is_continental = t_cold <= 0.0;  // Or <= -3.0 for stricter

    // Determine precipitation pattern
    let summer_dry = p_dry < 40.0 && p_dry < p_wet / 3.0 && input.p_summer < input.p_winter;
    let winter_dry = p_dry < p_wet / 10.0 && input.p_winter < input.p_summer;

    // Determine temperature subtype
    let temp_subtype = if t_hot >= 22.0 {
        'a'  // Hot summer
    } else if (0..4).filter(|_| t_hot >= 10.0).count() >= 4 {
        'b'  // Warm summer (at least 4 months above 10°C)
    } else if t_cold < -38.0 {
        'd'  // Extremely cold winter
    } else {
        'c'  // Cold summer
    };

    if is_continental {
        // Group D
        match (summer_dry, winter_dry, temp_subtype) {
            (true, _, 'a') => KoppenClimate::Dsa,
            (true, _, 'b') => KoppenClimate::Dsb,
            (true, _, 'c') => KoppenClimate::Dsc,
            (true, _, 'd') => KoppenClimate::Dsd,
            (_, true, 'a') => KoppenClimate::Dwa,
            (_, true, 'b') => KoppenClimate::Dwb,
            (_, true, 'c') => KoppenClimate::Dwc,
            (_, true, 'd') => KoppenClimate::Dwd,
            (_, _, 'a') => KoppenClimate::Dfa,
            (_, _, 'b') => KoppenClimate::Dfb,
            (_, _, 'c') => KoppenClimate::Dfc,
            (_, _, 'd') => KoppenClimate::Dfd,
            _ => KoppenClimate::Dfb, // fallback
        }
    } else {
        // Group C
        match (summer_dry, winter_dry, temp_subtype) {
            (true, _, 'a') => KoppenClimate::Csa,
            (true, _, 'b') => KoppenClimate::Csb,
            (true, _, 'c') => KoppenClimate::Csc,
            (_, true, 'a') => KoppenClimate::Cwa,
            (_, true, 'b') => KoppenClimate::Cwb,
            (_, true, 'c') => KoppenClimate::Cwc,
            (_, _, 'a') => KoppenClimate::Cfa,
            (_, _, 'b') => KoppenClimate::Cfb,
            (_, _, 'c') => KoppenClimate::Cfc,
            _ => KoppenClimate::Cfb, // fallback
        }
    }
}

fn calculate_aridity_threshold(input: &ClimateInput) -> f32 {
    // Threshold depends on when precipitation falls relative to temperature
    let summer_fraction = input.p_summer / input.p_annual;

    let base = if summer_fraction >= 0.7 {
        // Most rain in summer (when evaporation is high)
        2.0 * input.t_annual_mean + 28.0
    } else if summer_fraction <= 0.3 {
        // Most rain in winter (less evaporation)
        2.0 * input.t_annual_mean
    } else {
        // Even distribution
        2.0 * input.t_annual_mean + 14.0
    };

    base * 10.0  // Convert to mm
}
```

### Biome to Visual Parameters

```rust
pub struct BiomeVisuals {
    pub vegetation_density: f32,
    pub tree_height_range: (f32, f32),
    pub grass_color: [f32; 3],
    pub soil_color: [f32; 3],
    pub dominant_tree_types: Vec<TreeType>,
}

pub fn biome_visuals(climate: KoppenClimate) -> BiomeVisuals {
    match climate {
        KoppenClimate::Af => BiomeVisuals {
            vegetation_density: 0.95,
            tree_height_range: (20.0, 50.0),
            grass_color: [0.1, 0.4, 0.1],
            soil_color: [0.3, 0.2, 0.1],
            dominant_tree_types: vec![TreeType::TropicalBroadleaf],
        },
        KoppenClimate::BWh => BiomeVisuals {
            vegetation_density: 0.02,
            tree_height_range: (0.0, 0.0),
            grass_color: [0.8, 0.7, 0.5],
            soil_color: [0.9, 0.8, 0.6],
            dominant_tree_types: vec![],
        },
        KoppenClimate::Cfb => BiomeVisuals {
            vegetation_density: 0.70,
            tree_height_range: (10.0, 30.0),
            grass_color: [0.2, 0.5, 0.2],
            soil_color: [0.4, 0.3, 0.2],
            dominant_tree_types: vec![TreeType::DeciduousBroadleaf, TreeType::MixedForest],
        },
        KoppenClimate::ET => BiomeVisuals {
            vegetation_density: 0.15,
            tree_height_range: (0.0, 2.0),
            grass_color: [0.4, 0.5, 0.3],
            soil_color: [0.5, 0.5, 0.4],
            dominant_tree_types: vec![TreeType::Shrub],
        },
        // ... etc for other biomes
        _ => BiomeVisuals::default(),
    }
}
```

---

## Implementation Notes

### Performance Considerations

1. **Pre-computation**: Climate map should be computed once at world generation
2. **Resolution**: Climate doesn't need per-vertex resolution; 1km² tiles are sufficient
3. **GPU offloading**: Temperature/precipitation can be computed in compute shaders
4. **Caching**: Cache biome lookups; they don't change

### Suggested Crate Structure

```
syn_planet_sim/
├── src/
│   ├── lib.rs
│   ├── orbital.rs       # Orbital mechanics
│   ├── tectonic.rs      # Plate tectonics
│   ├── hydrology.rs     # Water flow
│   ├── atmosphere.rs    # Atmospheric physics
│   ├── climate.rs       # Climate computation
│   ├── biome.rs         # Köppen classification
│   └── prelude.rs
├── tests/
│   ├── earth_validation.rs  # Compare with real Earth data
│   └── edge_cases.rs
└── benches/
    └── climate_generation.rs
```

### Validation Strategy

Test against real Earth data:

- Temperature at known locations (Paris: 12°C avg, Sahara: 30°C avg)
- Precipitation patterns (Amazon: 3000mm, Atacama: 15mm)
- Biome classification for real locations

---

## References

### Scientific Papers & Books

1. **Köppen-Geiger Classification**
   - Peel, M. C., Finlayson, B. L., and McMahon, T. A. (2007). "Updated world map of the Köppen-Geiger climate classification"

2. **Atmospheric Circulation**
   - Holton, J. R., & Hakim, G. J. (2012). "An Introduction to Dynamic Meteorology"

3. **Hydrology**
   - Leopold, L. B., & Maddock, T. (1953). "The Hydraulic Geometry of Stream Channels"
   - Barnes, H. H. (1967). "Roughness Characteristics of Natural Channels"

4. **Procedural Generation**
   - Amit Patel - Red Blob Games: https://www.redblobgames.com/maps/terrain-from-noise/
   - "Polygonal Map Generation for Games": http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/

### Game Development References

1. **No Man's Sky** - Sean Murray GDC talks on procedural generation
2. **Dwarf Fortress** - Temperature and hydrology simulation
3. **Civilization** - Climate/terrain generation

### Online Resources

- NASA Climate Data: https://climate.nasa.gov/
- NOAA Wind Patterns: https://www.noaa.gov/
- World Biomes: https://www.worldbiomes.com/

---

## Ideas & Future Work

### To Research

- [ ] **Ocean currents**: Gulf Stream effect on European climate
- [ ] **Monsoon dynamics**: Seasonal wind reversal
- [ ] **Ice ages**: Long-term climate variation
- [ ] **Terraforming**: Dynamic climate change from player actions
- [ ] **Alien atmospheres**: Non-Earth-like compositions

### Optimization Ideas

- [ ] **Hierarchical climate**: Compute at multiple resolutions
- [ ] **GPU climate map**: Generate entire planet climate in one compute pass
- [ ] **Climate interpolation**: Only compute at sparse points, interpolate

### Gameplay Integration Ideas

- [ ] **Seasonal gameplay**: Different resources/dangers per season
- [ ] **Weather events**: Storms, droughts derived from climate
- [ ] **Agriculture**: Crop suitability based on climate
- [ ] **Creature migration**: Animals follow climate patterns

### Notes

```
2026-01-22: Initial document creation
- Started with orbital mechanics equations
- Added Köppen classification from Wikipedia/papers
- Need to validate against real Earth data
```

---

*This is a living research document. Add equations, notes, and prototypes as research progresses.*
