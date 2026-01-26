# Ecosystem & Fauna System - Research Document

**Version**: 0.1.0
**Status**: Active Research
**Related Crates**: `syn_ecosystem`, `syn_fauna`, `syn_ai`
**Last Updated**: 2026-01-24

---

## 1. Overview

### 1.1 Goals

Create a living, breathing ecosystem where:
- **Fauna behaves realistically** with food chains, territories, and survival instincts
- **Ecosystems are self-regulating** through predator-prey dynamics
- **Player actions have consequences** on wildlife populations
- **Industrial activity impacts biodiversity** (core Groundbreak theme)
- **Procedural spawning** creates diverse, believable wildlife distribution

### 1.2 Design Pillars

1. **Emergence over Scripting**: Simple rules create complex behaviors
2. **Simulation Economy**: Not every creature needs full AI every frame
3. **Observable Consequences**: Player sees ecosystem changes over time
4. **Performance First**: Thousands of creatures, minimal CPU cost

### 1.3 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Ecosystem Manager                           │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Biome      │  │  Population  │  │   Impact     │          │
│  │   Layer      │  │   Dynamics   │  │   Tracker    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Spawn      │  │  Behavior    │  │   Food       │          │
│  │   System     │  │   Trees      │  │   Web        │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                     Creature Instances                          │
│  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐      │
│  │ 🦌  │ │ 🐺  │ │ 🐰  │ │ 🦅  │ │ 🐟  │ │ 🦎  │ │ ...│      │
│  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Biomes & Species Distribution

### 2.1 Biome Definition

Each biome defines which species can exist and their abundance:

```rust
/// Biome types derived from Köppen climate classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BiomeType {
    // Tropical
    TropicalRainforest,
    TropicalMonsoon,
    TropicalSavanna,

    // Arid
    HotDesert,
    ColdDesert,
    Steppe,

    // Temperate
    TemperateOceanic,
    TemperateContinental,
    Mediterranean,

    // Cold
    Boreal,          // Taiga
    Tundra,
    IceCap,

    // Aquatic
    Ocean,
    CoastalShallow,
    FreshwaterLake,
    River,
    Wetland,

    // Transitional
    MountainAlpine,
    Cave,
    Volcanic,
}

/// Biome environmental parameters
#[derive(Clone, Debug)]
pub struct BiomeParameters {
    pub temperature_range: (f32, f32),  // Celsius
    pub humidity_range: (f32, f32),     // 0.0 - 1.0
    pub altitude_range: (f32, f32),     // Meters
    pub vegetation_density: f32,        // 0.0 - 1.0
    pub water_availability: f32,        // 0.0 - 1.0
    pub light_level: f32,               // 0.0 - 1.0 (caves are dark)
}

impl BiomeType {
    pub fn parameters(&self) -> BiomeParameters {
        match self {
            BiomeType::TropicalRainforest => BiomeParameters {
                temperature_range: (20.0, 35.0),
                humidity_range: (0.7, 1.0),
                altitude_range: (0.0, 1000.0),
                vegetation_density: 0.95,
                water_availability: 0.9,
                light_level: 0.6, // Canopy blocks light
            },
            BiomeType::HotDesert => BiomeParameters {
                temperature_range: (15.0, 50.0),
                humidity_range: (0.0, 0.2),
                altitude_range: (0.0, 2000.0),
                vegetation_density: 0.05,
                water_availability: 0.05,
                light_level: 1.0,
            },
            BiomeType::Boreal => BiomeParameters {
                temperature_range: (-40.0, 20.0),
                humidity_range: (0.4, 0.8),
                altitude_range: (0.0, 1500.0),
                vegetation_density: 0.7,
                water_availability: 0.6,
                light_level: 0.8,
            },
            // ... other biomes
            _ => BiomeParameters::default(),
        }
    }
}
```

### 2.2 Species-Biome Compatibility

```rust
/// Species compatibility with biomes
#[derive(Clone, Debug)]
pub struct SpeciesHabitat {
    /// Primary biomes where species thrives
    pub preferred_biomes: Vec<BiomeType>,
    /// Secondary biomes where species can survive
    pub tolerated_biomes: Vec<BiomeType>,
    /// Environmental tolerances
    pub temperature_tolerance: (f32, f32),
    pub humidity_tolerance: (f32, f32),
    pub altitude_tolerance: (f32, f32),
}

/// Calculate habitat suitability score
pub fn habitat_suitability(
    species: &SpeciesHabitat,
    biome: BiomeType,
    local_conditions: &LocalConditions,
) -> f32 {
    // Base score from biome preference
    let biome_score = if species.preferred_biomes.contains(&biome) {
        1.0
    } else if species.tolerated_biomes.contains(&biome) {
        0.5
    } else {
        0.0
    };

    // Temperature fitness (Gaussian falloff)
    let temp_mid = (species.temperature_tolerance.0 + species.temperature_tolerance.1) / 2.0;
    let temp_range = species.temperature_tolerance.1 - species.temperature_tolerance.0;
    let temp_score = gaussian_falloff(local_conditions.temperature, temp_mid, temp_range / 2.0);

    // Humidity fitness
    let humid_mid = (species.humidity_tolerance.0 + species.humidity_tolerance.1) / 2.0;
    let humid_range = species.humidity_tolerance.1 - species.humidity_tolerance.0;
    let humid_score = gaussian_falloff(local_conditions.humidity, humid_mid, humid_range / 2.0);

    // Combine scores
    biome_score * temp_score * humid_score
}

fn gaussian_falloff(value: f32, center: f32, sigma: f32) -> f32 {
    let diff = value - center;
    (-diff * diff / (2.0 * sigma * sigma)).exp()
}
```

---

## 3. Food Web & Trophic Levels

### 3.1 Ecological Roles

```rust
/// Trophic level in the food chain
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrophicLevel {
    /// Primary producers (plants, algae)
    Producer,
    /// Primary consumers (herbivores)
    PrimaryConsumer,
    /// Secondary consumers (small predators, omnivores)
    SecondaryConsumer,
    /// Tertiary consumers (apex predators)
    TertiaryConsumer,
    /// Break down dead matter
    Decomposer,
}

/// Dietary classification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diet {
    Herbivore,
    Carnivore,
    Omnivore,
    Insectivore,
    Piscivore,      // Fish eater
    Scavenger,
    Detritivore,    // Eats dead organic matter
    Parasite,
}

/// Species definition in the food web
#[derive(Clone, Debug)]
pub struct SpeciesDefinition {
    pub id: SpeciesId,
    pub name: String,
    pub trophic_level: TrophicLevel,
    pub diet: Diet,
    pub body_mass_kg: f32,          // Average adult mass
    pub metabolic_rate: f32,        // kcal/day requirement
    pub prey_species: Vec<SpeciesId>,
    pub predator_species: Vec<SpeciesId>,
    pub competition_species: Vec<SpeciesId>, // Same niche
}
```

### 3.2 Energy Flow Equations

Energy transfer between trophic levels follows ecological efficiency rules:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Energy Pyramid                                │
│                                                                  │
│                    ▲ Apex Predators                             │
│                   ╱ ╲  (~0.1% of base)                          │
│                  ╱   ╲                                          │
│                 ╱─────╲ Predators                               │
│                ╱       ╲  (~1% of base)                         │
│               ╱         ╲                                       │
│              ╱───────────╲ Herbivores                           │
│             ╱             ╲  (~10% of base)                     │
│            ╱               ╲                                    │
│           ╱─────────────────╲ Producers (100%)                  │
│          ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔                                   │
└─────────────────────────────────────────────────────────────────┘
```

**Lindeman's 10% Rule**: Only ~10% of energy transfers between trophic levels.

```rust
/// Calculate energy available at each trophic level
pub struct EnergyFlow {
    pub transfer_efficiency: f32,  // Typically 0.10 (10%)
}

impl EnergyFlow {
    /// Calculate sustainable population based on energy
    pub fn carrying_capacity(
        &self,
        producer_energy: f32,      // Total plant energy in kcal/day
        trophic_level: TrophicLevel,
        metabolic_rate: f32,       // Species requirement kcal/day
    ) -> u32 {
        let level = match trophic_level {
            TrophicLevel::Producer => 0,
            TrophicLevel::PrimaryConsumer => 1,
            TrophicLevel::SecondaryConsumer => 2,
            TrophicLevel::TertiaryConsumer => 3,
            TrophicLevel::Decomposer => 1,
        };

        // Energy available at this level
        let available_energy = producer_energy * self.transfer_efficiency.powi(level);

        // Number of individuals supportable
        (available_energy / metabolic_rate) as u32
    }
}

/// Metabolic rate from body mass (Kleiber's Law)
/// P = P₀ * M^(3/4)
/// Where P₀ ≈ 70 for mammals (kcal/day per kg^0.75)
pub fn metabolic_rate_kleiber(body_mass_kg: f32) -> f32 {
    const KLEIBER_CONSTANT: f32 = 70.0;  // For mammals
    KLEIBER_CONSTANT * body_mass_kg.powf(0.75)
}
```

### 3.3 Predator-Prey Dynamics (Lotka-Volterra)

Classic predator-prey oscillations:

```
dN/dt = rN - aNP    (Prey growth - predation)
dP/dt = baNP - mP   (Predator growth from prey - mortality)

Where:
  N = Prey population
  P = Predator population
  r = Prey intrinsic growth rate
  a = Predation rate (attack success)
  b = Conversion efficiency (prey → predator biomass)
  m = Predator mortality rate
```

```rust
/// Lotka-Volterra population dynamics
#[derive(Clone, Debug)]
pub struct LotkaVolterraParams {
    pub prey_growth_rate: f32,      // r
    pub predation_rate: f32,        // a
    pub conversion_efficiency: f32, // b
    pub predator_mortality: f32,    // m
}

impl LotkaVolterraParams {
    /// Simulate one time step
    pub fn step(
        &self,
        prey_pop: f32,
        predator_pop: f32,
        dt: f32,
    ) -> (f32, f32) {
        let prey_change = (self.prey_growth_rate * prey_pop
            - self.predation_rate * prey_pop * predator_pop) * dt;

        let predator_change = (self.conversion_efficiency * self.predation_rate
            * prey_pop * predator_pop
            - self.predator_mortality * predator_pop) * dt;

        let new_prey = (prey_pop + prey_change).max(0.0);
        let new_predator = (predator_pop + predator_change).max(0.0);

        (new_prey, new_predator)
    }

    /// Calculate equilibrium populations
    pub fn equilibrium(&self) -> (f32, f32) {
        let prey_eq = self.predator_mortality
            / (self.conversion_efficiency * self.predation_rate);
        let predator_eq = self.prey_growth_rate / self.predation_rate;
        (prey_eq, predator_eq)
    }
}
```

---

## 4. Creature Behavior System

### 4.1 Behavioral States (FSM)

```rust
/// High-level behavioral states
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorState {
    /// Searching for food
    Foraging,
    /// Moving to food/water source
    Traveling,
    /// Actively eating/drinking
    Consuming,
    /// Avoiding threats
    Fleeing,
    /// Chasing prey
    Hunting,
    /// Resting to conserve energy
    Resting,
    /// Seeking mates
    Mating,
    /// Caring for offspring
    Nurturing,
    /// Defending territory
    Territorial,
    /// Group social behaviors
    Socializing,
    /// Hiding from danger
    Hiding,
    /// Migrating to new area
    Migrating,
}

/// Creature needs that drive behavior
#[derive(Clone, Debug)]
pub struct CreatureNeeds {
    pub hunger: f32,        // 0.0 = full, 1.0 = starving
    pub thirst: f32,        // 0.0 = hydrated, 1.0 = dehydrated
    pub energy: f32,        // 0.0 = exhausted, 1.0 = full energy
    pub safety: f32,        // 0.0 = terrified, 1.0 = secure
    pub social: f32,        // 0.0 = lonely, 1.0 = satisfied
    pub reproduction: f32,  // 0.0 = no urge, 1.0 = strong urge
}

impl CreatureNeeds {
    /// Get the most urgent need
    pub fn priority_need(&self) -> (NeedType, f32) {
        let needs = [
            (NeedType::Safety, 1.0 - self.safety),    // Danger is top priority
            (NeedType::Thirst, self.thirst),
            (NeedType::Hunger, self.hunger),
            (NeedType::Energy, 1.0 - self.energy),
            (NeedType::Reproduction, self.reproduction),
            (NeedType::Social, 1.0 - self.social),
        ];

        needs.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NeedType {
    Hunger,
    Thirst,
    Energy,
    Safety,
    Social,
    Reproduction,
}
```

### 4.2 Behavior Tree Structure

```rust
/// Behavior tree node types
#[derive(Clone, Debug)]
pub enum BehaviorNode {
    /// Execute children in order until one succeeds
    Selector(Vec<BehaviorNode>),
    /// Execute children in order until one fails
    Sequence(Vec<BehaviorNode>),
    /// Run child while condition is true
    While {
        condition: Condition,
        child: Box<BehaviorNode>,
    },
    /// Invert child result
    Inverter(Box<BehaviorNode>),
    /// Leaf actions
    Action(BehaviorAction),
    /// Check a condition
    Condition(Condition),
}

#[derive(Clone, Debug)]
pub enum Condition {
    IsHungry(f32),              // Threshold
    IsThirsty(f32),
    IsThreatNearby(f32),        // Distance
    IsPreyVisible,
    HasReachedDestination,
    IsDaytime,
    IsInTerritory,
    HasMate,
    IsHealthy(f32),             // Health threshold
}

#[derive(Clone, Debug)]
pub enum BehaviorAction {
    // Movement
    MoveToward(TargetType),
    FleeFrom(TargetType),
    Wander,
    FollowPath,

    // Interaction
    Attack(TargetType),
    Eat(TargetType),
    Drink,
    PickUp(TargetType),

    // Social
    CallMate,
    FollowLeader,
    FormGroup,

    // State
    Rest,
    Hide,
    AlertOthers,
}

#[derive(Clone, Debug)]
pub enum TargetType {
    NearestFood,
    NearestWater,
    NearestPrey,
    NearestThreat,
    Home,
    Mate,
    Leader,
    SafeSpot,
}
```

### 4.3 Example Behavior Tree: Herbivore

```rust
/// Create herbivore behavior tree
pub fn herbivore_behavior() -> BehaviorNode {
    BehaviorNode::Selector(vec![
        // Priority 1: Flee from danger
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(Condition::IsThreatNearby(50.0)),
            BehaviorNode::Action(BehaviorAction::AlertOthers),
            BehaviorNode::Action(BehaviorAction::FleeFrom(TargetType::NearestThreat)),
        ]),

        // Priority 2: Drink if thirsty
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(Condition::IsThirsty(0.7)),
            BehaviorNode::Action(BehaviorAction::MoveToward(TargetType::NearestWater)),
            BehaviorNode::Action(BehaviorAction::Drink),
        ]),

        // Priority 3: Eat if hungry
        BehaviorNode::Sequence(vec![
            BehaviorNode::Condition(Condition::IsHungry(0.6)),
            BehaviorNode::Action(BehaviorAction::MoveToward(TargetType::NearestFood)),
            BehaviorNode::Action(BehaviorAction::Eat(TargetType::NearestFood)),
        ]),

        // Priority 4: Rest if tired (at night)
        BehaviorNode::Sequence(vec![
            BehaviorNode::Inverter(Box::new(
                BehaviorNode::Condition(Condition::IsDaytime)
            )),
            BehaviorNode::Action(BehaviorAction::MoveToward(TargetType::SafeSpot)),
            BehaviorNode::Action(BehaviorAction::Rest),
        ]),

        // Default: Wander and graze
        BehaviorNode::Action(BehaviorAction::Wander),
    ])
}
```

### 4.4 Flocking Behavior (Boids)

For herds, flocks, and schools:

```rust
/// Boid flocking parameters
#[derive(Clone, Debug)]
pub struct FlockingParams {
    pub separation_distance: f32,   // Minimum distance between individuals
    pub alignment_radius: f32,      // Radius to align velocity with neighbors
    pub cohesion_radius: f32,       // Radius to move toward center of flock
    pub separation_weight: f32,     // Strength of separation force
    pub alignment_weight: f32,      // Strength of alignment force
    pub cohesion_weight: f32,       // Strength of cohesion force
    pub max_speed: f32,
    pub max_steering_force: f32,
}

/// Calculate flocking forces for a boid
pub fn calculate_flocking(
    boid: &Creature,
    neighbors: &[&Creature],
    params: &FlockingParams,
) -> Vec3 {
    let mut separation = Vec3::ZERO;
    let mut alignment = Vec3::ZERO;
    let mut cohesion = Vec3::ZERO;
    let mut separation_count = 0;
    let mut alignment_count = 0;
    let mut cohesion_count = 0;

    for neighbor in neighbors {
        let distance = boid.position.distance(neighbor.position);

        // Separation: Steer away from close neighbors
        if distance < params.separation_distance && distance > 0.0 {
            let diff = (boid.position - neighbor.position).normalize() / distance;
            separation += diff;
            separation_count += 1;
        }

        // Alignment: Match velocity with nearby neighbors
        if distance < params.alignment_radius {
            alignment += neighbor.velocity;
            alignment_count += 1;
        }

        // Cohesion: Move toward center of nearby flock
        if distance < params.cohesion_radius {
            cohesion += neighbor.position;
            cohesion_count += 1;
        }
    }

    let mut steering = Vec3::ZERO;

    if separation_count > 0 {
        separation /= separation_count as f32;
        steering += separation.normalize_or_zero() * params.separation_weight;
    }

    if alignment_count > 0 {
        alignment /= alignment_count as f32;
        let desired = alignment.normalize_or_zero() * params.max_speed;
        steering += (desired - boid.velocity).clamp_length_max(params.max_steering_force)
            * params.alignment_weight;
    }

    if cohesion_count > 0 {
        cohesion /= cohesion_count as f32;
        let desired = (cohesion - boid.position).normalize_or_zero() * params.max_speed;
        steering += (desired - boid.velocity).clamp_length_max(params.max_steering_force)
            * params.cohesion_weight;
    }

    steering.clamp_length_max(params.max_steering_force)
}
```

---

## 5. Procedural Spawning

### 5.1 Population Distribution

```rust
/// Controls how populations are distributed in the world
#[derive(Clone, Debug)]
pub struct PopulationDistribution {
    pub species_id: SpeciesId,
    /// Base population density (individuals per km²)
    pub base_density: f32,
    /// Clustering behavior
    pub cluster_radius: f32,
    pub cluster_density_multiplier: f32,
    /// Territory requirements
    pub territory_size: f32,        // m² per individual
    pub home_range: f32,            // Daily movement range
}

/// Spawn point generation using Poisson disk sampling
pub fn generate_spawn_points(
    area: &AreaBounds,
    biome_map: &BiomeMap,
    species: &SpeciesDefinition,
    distribution: &PopulationDistribution,
) -> Vec<SpawnPoint> {
    let mut spawns = Vec::new();
    let mut active_list: Vec<Vec2> = Vec::new();
    let cell_size = distribution.territory_size.sqrt() / std::f32::consts::SQRT_2;

    // Grid for fast neighbor lookup
    let grid_width = (area.width() / cell_size).ceil() as usize;
    let grid_height = (area.height() / cell_size).ceil() as usize;
    let mut grid: Vec<Option<Vec2>> = vec![None; grid_width * grid_height];

    // Start with random point in suitable biome
    if let Some(start) = find_suitable_start(area, biome_map, &species.habitat) {
        active_list.push(start);
        insert_to_grid(&mut grid, start, cell_size, grid_width);
    }

    let mut rng = thread_rng();

    while let Some(point) = active_list.pop() {
        for _ in 0..30 {  // k samples around each point
            let angle = rng.gen::<f32>() * std::f32::consts::TAU;
            let radius = distribution.territory_size.sqrt() * (1.0 + rng.gen::<f32>());

            let candidate = Vec2::new(
                point.x + radius * angle.cos(),
                point.y + radius * angle.sin(),
            );

            if !area.contains(candidate) {
                continue;
            }

            // Check biome suitability
            let biome = biome_map.get_biome(candidate);
            let suitability = habitat_suitability(&species.habitat, biome,
                &biome_map.get_conditions(candidate));

            if suitability < 0.3 {
                continue;
            }

            // Check distance to existing spawns
            if !has_nearby_spawn(&grid, candidate, distribution.territory_size.sqrt(),
                cell_size, grid_width)
            {
                active_list.push(candidate);
                insert_to_grid(&mut grid, candidate, cell_size, grid_width);

                spawns.push(SpawnPoint {
                    position: candidate,
                    species_id: species.id,
                    suitability,
                });
            }
        }
    }

    spawns
}
```

### 5.2 Dynamic Population Management

```rust
/// Manages creature populations across the world
pub struct PopulationManager {
    /// Population counts per species per region
    populations: HashMap<(RegionId, SpeciesId), PopulationData>,
    /// Carrying capacity per region
    carrying_capacities: HashMap<RegionId, HashMap<SpeciesId, u32>>,
    /// Migration pressure between regions
    migration_pressure: HashMap<(RegionId, RegionId), f32>,
}

#[derive(Clone, Debug)]
pub struct PopulationData {
    pub count: u32,
    pub birth_rate: f32,
    pub death_rate: f32,
    pub migration_in: u32,
    pub migration_out: u32,
    pub health_index: f32,  // Overall population health
}

impl PopulationManager {
    /// Update population for one time step (game day)
    pub fn update(&mut self, species_data: &SpeciesDatabase, delta_days: f32) {
        for ((region, species), pop) in &mut self.populations {
            let species_def = species_data.get(*species);
            let carrying_cap = self.carrying_capacities
                .get(region)
                .and_then(|caps| caps.get(species))
                .copied()
                .unwrap_or(0);

            // Logistic growth model
            // dN/dt = rN(1 - N/K)
            let r = pop.birth_rate - pop.death_rate;
            let n = pop.count as f32;
            let k = carrying_cap as f32;

            let growth = if k > 0.0 {
                r * n * (1.0 - n / k)
            } else {
                -pop.death_rate * n  // Population decline without capacity
            };

            // Apply growth
            pop.count = ((n + growth * delta_days).max(0.0)) as u32;

            // Migration (if overpopulated or underpopulated)
            self.process_migration(*region, *species, pop, carrying_cap);
        }
    }

    fn process_migration(
        &mut self,
        region: RegionId,
        species: SpeciesId,
        pop: &mut PopulationData,
        carrying_cap: u32,
    ) {
        let pressure = if pop.count > carrying_cap {
            (pop.count - carrying_cap) as f32 / carrying_cap as f32
        } else {
            0.0
        };

        // Store migration pressure for neighboring regions to pull from
        // (Simplified - real implementation would track neighbor connections)
        pop.migration_out = (pop.count as f32 * pressure * 0.1) as u32;
    }
}
```

### 5.3 LOD Population Simulation

Not all creatures need full AI. Use level-of-detail for distant populations:

```rust
/// Level of detail for creature simulation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureLOD {
    /// Full behavior tree, animation, physics
    Full,
    /// Simplified AI, basic animation
    Medium,
    /// Statistical movement, no animation
    Low,
    /// Just population count, no individuals
    Abstract,
}

impl CreatureLOD {
    pub fn from_distance(distance_to_player: f32) -> Self {
        match distance_to_player {
            d if d < 100.0 => CreatureLOD::Full,
            d if d < 500.0 => CreatureLOD::Medium,
            d if d < 2000.0 => CreatureLOD::Low,
            _ => CreatureLOD::Abstract,
        }
    }
}

/// Simulate creature based on LOD
pub fn update_creature(creature: &mut Creature, lod: CreatureLOD, dt: f32) {
    match lod {
        CreatureLOD::Full => {
            // Full behavior tree evaluation
            creature.behavior_tree.evaluate(creature, dt);
            // Full physics
            creature.physics.update(dt);
            // Animation blending
            creature.animator.update(dt);
        }
        CreatureLOD::Medium => {
            // Simplified behavior (FSM only)
            creature.simple_behavior.update(creature, dt);
            // Basic movement
            creature.position += creature.velocity * dt;
            // Keyframe animation
            creature.animator.update_simple(dt);
        }
        CreatureLOD::Low => {
            // Random walk toward goals
            creature.statistical_movement(dt);
            // No animation
        }
        CreatureLOD::Abstract => {
            // Nothing - creature is just a population statistic
        }
    }
}
```

---

## 6. Creature Lifecycle

### 6.1 Life Stages

```rust
/// Life stage of a creature
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifeStage {
    Egg,        // Laid, incubating
    Juvenile,   // Young, vulnerable, learning
    Subadult,   // Maturing, can't reproduce yet
    Adult,      // Full size, can reproduce
    Elder,      // Old, reduced capabilities
}

/// Age and lifecycle data
#[derive(Clone, Debug)]
pub struct LifecycleData {
    pub current_age: f32,           // Days
    pub life_expectancy: f32,       // Days
    pub stage_thresholds: [f32; 5], // Age at each stage transition
    pub growth_rate: f32,           // Size increase per day
    pub current_size: f32,          // 0.0 = newborn, 1.0 = adult
    pub fertility_window: (f32, f32), // Age range for reproduction
    pub gestation_period: f32,      // Days
    pub offspring_count: (u32, u32), // Min/max offspring
}

impl LifecycleData {
    pub fn current_stage(&self) -> LifeStage {
        if self.current_age < self.stage_thresholds[0] {
            LifeStage::Egg
        } else if self.current_age < self.stage_thresholds[1] {
            LifeStage::Juvenile
        } else if self.current_age < self.stage_thresholds[2] {
            LifeStage::Subadult
        } else if self.current_age < self.stage_thresholds[3] {
            LifeStage::Adult
        } else {
            LifeStage::Elder
        }
    }

    pub fn can_reproduce(&self) -> bool {
        self.current_age >= self.fertility_window.0
            && self.current_age <= self.fertility_window.1
    }

    /// Death probability increases with age (Gompertz mortality)
    pub fn daily_death_probability(&self) -> f32 {
        let age_ratio = self.current_age / self.life_expectancy;

        // Gompertz: μ(x) = α * e^(β*x)
        const ALPHA: f32 = 0.0001;  // Baseline mortality
        const BETA: f32 = 0.1;      // Aging rate

        ALPHA * (BETA * age_ratio * 10.0).exp()
    }
}
```

### 6.2 Reproduction System

```rust
/// Reproduction strategy
#[derive(Clone, Copy, Debug)]
pub enum ReproductionStrategy {
    /// Internal fertilization, live birth
    Viviparous,
    /// Eggs laid and incubated
    Oviparous,
    /// Eggs hatch inside, live birth
    Ovoviviparous,
    /// Asexual reproduction
    Parthenogenesis,
}

/// Mating system
#[derive(Clone, Copy, Debug)]
pub enum MatingSystem {
    Monogamous,         // One partner per season
    Polygynous,         // One male, multiple females
    Polyandrous,        // One female, multiple males
    Promiscuous,        // No pair bonds
}

/// Handle reproduction between two creatures
pub fn attempt_reproduction(
    parent_a: &mut Creature,
    parent_b: &mut Creature,
    species: &SpeciesDefinition,
    rng: &mut impl Rng,
) -> Option<Vec<Creature>> {
    // Check compatibility
    if parent_a.species_id != parent_b.species_id {
        return None;
    }

    // Check fertility
    if !parent_a.lifecycle.can_reproduce() || !parent_b.lifecycle.can_reproduce() {
        return None;
    }

    // Check reproductive urge
    if parent_a.needs.reproduction < 0.7 || parent_b.needs.reproduction < 0.7 {
        return None;
    }

    // Mating success based on health and fitness
    let success_chance = parent_a.health * parent_b.health;
    if rng.gen::<f32>() > success_chance {
        return None;
    }

    // Reset reproduction urge
    parent_a.needs.reproduction = 0.0;
    parent_b.needs.reproduction = 0.0;

    // Generate offspring
    let offspring_count = rng.gen_range(
        species.lifecycle.offspring_count.0..=species.lifecycle.offspring_count.1
    );

    let mut offspring = Vec::new();
    for _ in 0..offspring_count {
        let child = Creature::new_offspring(parent_a, parent_b, species, rng);
        offspring.push(child);
    }

    Some(offspring)
}
```

---

## 7. Player Interactions

### 7.1 Hunting & Combat

```rust
/// Creature's reaction to player
#[derive(Clone, Copy, Debug)]
pub enum PlayerReaction {
    Ignore,         // Player not perceived as threat/prey
    Curious,        // Investigates player
    Wary,           // Keeps distance, watches
    Flee,           // Runs away
    Aggressive,     // Attacks player
    Territorial,    // Defends territory
    Neutral,        // Coexists peacefully
}

/// Determine creature reaction to player
pub fn player_reaction(
    creature: &Creature,
    player: &Player,
    species: &SpeciesDefinition,
) -> PlayerReaction {
    let distance = creature.position.distance(player.position);

    // Check if player is in territory
    let in_territory = creature.territory
        .map(|t| t.contains(player.position))
        .unwrap_or(false);

    // Predator sees player as prey?
    if species.diet == Diet::Carnivore && creature.needs.hunger > 0.7 {
        if distance < species.attack_range * 2.0 {
            return PlayerReaction::Aggressive;
        }
    }

    // Player has weapon drawn?
    let threat_level = player.calculate_threat_level();

    // Territorial defense
    if in_territory && threat_level > 0.3 {
        return PlayerReaction::Territorial;
    }

    // Flee behavior for prey animals
    if species.trophic_level == TrophicLevel::PrimaryConsumer {
        if distance < species.flee_distance && threat_level > 0.2 {
            return PlayerReaction::Flee;
        }
        if distance < species.awareness_distance {
            return PlayerReaction::Wary;
        }
    }

    // Curious creatures
    if species.curiosity > 0.5 && distance < species.awareness_distance {
        return PlayerReaction::Curious;
    }

    PlayerReaction::Ignore
}
```

### 7.2 Domestication System

```rust
/// Domestication progress for a creature
#[derive(Clone, Debug)]
pub struct DomesticationData {
    pub trust_level: f32,           // 0.0 = wild, 1.0 = tame
    pub familiarity_with_player: f32,
    pub feeding_count: u32,
    pub time_near_player: f32,      // Hours
    pub negative_experiences: u32,  // Being attacked, etc.
}

impl DomesticationData {
    /// Can this creature be tamed?
    pub fn can_domesticate(species: &SpeciesDefinition) -> bool {
        matches!(species.domestication_potential,
            DomesticationPotential::Easy | DomesticationPotential::Moderate)
    }

    /// Update trust based on player action
    pub fn process_interaction(&mut self, interaction: PlayerInteraction) {
        match interaction {
            PlayerInteraction::Feed(food_quality) => {
                self.trust_level += 0.05 * food_quality;
                self.feeding_count += 1;
            }
            PlayerInteraction::Pet => {
                if self.trust_level > 0.3 {
                    self.trust_level += 0.02;
                }
            }
            PlayerInteraction::Attack => {
                self.trust_level = (self.trust_level - 0.3).max(0.0);
                self.negative_experiences += 1;
            }
            PlayerInteraction::NearbyPresence(duration) => {
                if self.trust_level > 0.2 {
                    self.time_near_player += duration;
                    self.familiarity_with_player += 0.001 * duration;
                }
            }
        }

        self.trust_level = self.trust_level.clamp(0.0, 1.0);
    }

    /// Is creature considered tame?
    pub fn is_tame(&self) -> bool {
        self.trust_level >= 0.8 && self.negative_experiences < 3
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DomesticationPotential {
    Impossible,     // Cannot be tamed (wild predators)
    VeryHard,       // Extremely difficult
    Moderate,       // Possible with effort
    Easy,           // Natural companions
}
```

---

## 8. Industrial Impact on Ecosystem

### 8.1 Pollution Effects

This is central to Groundbreak's theme: player actions affect the ecosystem.

```rust
/// Types of industrial pollution
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PollutionType {
    AirParticulates,    // Smoke, dust
    ChemicalSpill,      // Toxic liquids
    Radiation,          // Nuclear/radioactive
    Noise,              // Disturbs wildlife
    Light,              // Disrupts nocturnal animals
    Thermal,            // Heat pollution
    WaterContamination, // Affects aquatic life
    SoilContamination,  // Affects plants and burrowers
}

/// Pollution level at a location
#[derive(Clone, Debug)]
pub struct PollutionLevel {
    pub levels: HashMap<PollutionType, f32>,  // 0.0 = none, 1.0 = lethal
}

impl PollutionLevel {
    /// Calculate effect on creature
    pub fn effect_on_creature(&self, species: &SpeciesDefinition) -> PollutionEffect {
        let mut health_drain = 0.0;
        let mut fertility_reduction = 0.0;
        let mut behavior_disruption = 0.0;
        let mut flee_trigger = false;

        for (pollution_type, level) in &self.levels {
            let sensitivity = species.pollution_sensitivity.get(pollution_type)
                .copied().unwrap_or(1.0);

            let effective_level = level * sensitivity;

            match pollution_type {
                PollutionType::Radiation | PollutionType::ChemicalSpill => {
                    health_drain += effective_level * 0.1;
                    fertility_reduction += effective_level * 0.5;
                }
                PollutionType::Noise => {
                    behavior_disruption += effective_level;
                    if effective_level > 0.5 {
                        flee_trigger = true;
                    }
                }
                PollutionType::AirParticulates => {
                    health_drain += effective_level * 0.05;
                }
                PollutionType::WaterContamination => {
                    if species.habitat.preferred_biomes.iter()
                        .any(|b| matches!(b, BiomeType::FreshwaterLake | BiomeType::River))
                    {
                        health_drain += effective_level * 0.2;
                    }
                }
                _ => {}
            }
        }

        PollutionEffect {
            health_drain_per_day: health_drain,
            fertility_multiplier: 1.0 - fertility_reduction.min(0.9),
            behavior_disruption,
            triggers_flee: flee_trigger,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PollutionEffect {
    pub health_drain_per_day: f32,
    pub fertility_multiplier: f32,
    pub behavior_disruption: f32,
    pub triggers_flee: bool,
}
```

### 8.2 Habitat Destruction

```rust
/// Track habitat changes from player activity
#[derive(Clone, Debug)]
pub struct HabitatTracker {
    /// Original biome before modification
    pub original_biome: BiomeType,
    /// Current effective biome
    pub current_biome: BiomeType,
    /// Vegetation coverage reduction
    pub deforestation: f32,         // 0.0 = pristine, 1.0 = clear-cut
    /// Ground disturbance
    pub terrain_modification: f32,  // 0.0 = natural, 1.0 = completely altered
    /// Water system disruption
    pub hydrology_disruption: f32,  // 0.0 = natural flow, 1.0 = completely diverted
    /// Species that have fled this area
    pub displaced_species: HashSet<SpeciesId>,
}

impl HabitatTracker {
    /// Calculate habitat quality for a species
    pub fn habitat_quality(&self, species: &SpeciesDefinition) -> f32 {
        let base = if species.habitat.preferred_biomes.contains(&self.current_biome) {
            1.0
        } else if species.habitat.tolerated_biomes.contains(&self.current_biome) {
            0.5
        } else {
            0.1
        };

        // Reduce quality based on disturbance
        let vegetation_factor = if species.requires_vegetation {
            1.0 - self.deforestation * 0.8
        } else {
            1.0
        };

        let terrain_factor = if species.burrows || species.nests_on_ground {
            1.0 - self.terrain_modification * 0.9
        } else {
            1.0 - self.terrain_modification * 0.3
        };

        let water_factor = if species.habitat.preferred_biomes.iter()
            .any(|b| matches!(b, BiomeType::FreshwaterLake | BiomeType::River | BiomeType::Wetland))
        {
            1.0 - self.hydrology_disruption * 0.95
        } else {
            1.0
        };

        base * vegetation_factor * terrain_factor * water_factor
    }

    /// Process terrain modification event
    pub fn on_terrain_modified(&mut self, modification_type: TerrainModification) {
        match modification_type {
            TerrainModification::TreeRemoved => {
                self.deforestation += 0.01;
            }
            TerrainModification::GroundDug { volume } => {
                self.terrain_modification += volume * 0.001;
            }
            TerrainModification::BuildingPlaced => {
                self.terrain_modification += 0.1;
            }
            TerrainModification::WaterDiverted => {
                self.hydrology_disruption += 0.2;
            }
        }

        self.deforestation = self.deforestation.min(1.0);
        self.terrain_modification = self.terrain_modification.min(1.0);
        self.hydrology_disruption = self.hydrology_disruption.min(1.0);
    }
}
```

### 8.3 Extinction Events

```rust
/// Track species extinction risk
#[derive(Clone, Debug)]
pub struct ExtinctionTracker {
    pub global_population: u32,
    pub population_trend: PopulationTrend,
    pub viable_habitats: u32,
    pub genetic_diversity: f32,     // 0.0 = inbred, 1.0 = diverse
    pub conservation_status: ConservationStatus,
}

#[derive(Clone, Copy, Debug)]
pub enum PopulationTrend {
    Increasing,
    Stable,
    Declining,
    CriticallyDeclining,
}

#[derive(Clone, Copy, Debug)]
pub enum ConservationStatus {
    LeastConcern,
    NearThreatened,
    Vulnerable,
    Endangered,
    CriticallyEndangered,
    ExtinctInWild,
    Extinct,
}

impl ExtinctionTracker {
    /// Update conservation status based on population data
    pub fn update_status(&mut self) {
        self.conservation_status = match (self.global_population, self.population_trend) {
            (0, _) => ConservationStatus::Extinct,
            (1..=10, _) => ConservationStatus::ExtinctInWild,
            (11..=50, PopulationTrend::CriticallyDeclining) =>
                ConservationStatus::CriticallyEndangered,
            (11..=250, PopulationTrend::Declining | PopulationTrend::CriticallyDeclining) =>
                ConservationStatus::Endangered,
            (51..=250, _) => ConservationStatus::Vulnerable,
            (251..=1000, PopulationTrend::Declining) => ConservationStatus::NearThreatened,
            _ => ConservationStatus::LeastConcern,
        };
    }

    /// Trigger extinction event
    pub fn check_extinction(&mut self) -> Option<ExtinctionEvent> {
        if self.global_population == 0 &&
           self.conservation_status != ConservationStatus::Extinct
        {
            self.conservation_status = ConservationStatus::Extinct;
            Some(ExtinctionEvent {
                cause: ExtinctionCause::HabitatDestruction,
                timestamp: current_game_time(),
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExtinctionEvent {
    pub cause: ExtinctionCause,
    pub timestamp: GameTime,
}

#[derive(Clone, Copy, Debug)]
pub enum ExtinctionCause {
    HabitatDestruction,
    Overhunting,
    Pollution,
    InvasiveSpecies,
    ClimateChange,
    NaturalDisaster,
}
```

### 8.4 Ecosystem Health UI

```rust
/// Data for ecosystem health display
#[derive(Clone, Debug)]
pub struct EcosystemHealthReport {
    pub region_name: String,
    pub overall_health: f32,        // 0.0 = dead, 1.0 = pristine
    pub species_diversity: u32,     // Number of species present
    pub original_diversity: u32,    // Number before player arrival
    pub endangered_species: Vec<SpeciesId>,
    pub extinct_species: Vec<SpeciesId>,
    pub pollution_levels: PollutionLevel,
    pub habitat_status: HabitatHealthStatus,
}

#[derive(Clone, Copy, Debug)]
pub enum HabitatHealthStatus {
    Pristine,
    LightlyDisturbed,
    ModeratelyDamaged,
    SeverelyDamaged,
    Destroyed,
}

impl EcosystemHealthReport {
    pub fn health_color(&self) -> Color {
        match self.overall_health {
            h if h >= 0.8 => Color::GREEN,
            h if h >= 0.6 => Color::YELLOW_GREEN,
            h if h >= 0.4 => Color::YELLOW,
            h if h >= 0.2 => Color::ORANGE,
            _ => Color::RED,
        }
    }

    pub fn summary_text(&self) -> String {
        let lost = self.original_diversity.saturating_sub(self.species_diversity);
        format!(
            "{} - Health: {:.0}% | Species: {}/{} ({} lost) | Endangered: {}",
            self.region_name,
            self.overall_health * 100.0,
            self.species_diversity,
            self.original_diversity,
            lost,
            self.endangered_species.len()
        )
    }
}
```

---

## 9. Implementation Notes

### 9.1 Performance Considerations

```rust
/// Spatial partitioning for creature queries
pub struct CreatureSpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<CreatureId>>,
}

impl CreatureSpatialGrid {
    /// Get creatures within radius
    pub fn query_radius(&self, center: Vec2, radius: f32) -> Vec<CreatureId> {
        let min_cell = self.world_to_cell(center - Vec2::splat(radius));
        let max_cell = self.world_to_cell(center + Vec2::splat(radius));

        let mut results = Vec::new();
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(creatures) = self.cells.get(&(x, y)) {
                    results.extend(creatures.iter().copied());
                }
            }
        }
        results
    }

    fn world_to_cell(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }
}
```

### 9.2 Data-Oriented Design

```rust
/// Component arrays for creature data (SoA layout)
pub struct CreatureComponents {
    pub positions: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
    pub species_ids: Vec<SpeciesId>,
    pub health: Vec<f32>,
    pub needs: Vec<CreatureNeeds>,
    pub states: Vec<BehaviorState>,
    pub lod_levels: Vec<CreatureLOD>,
}

impl CreatureComponents {
    /// Batch update positions for all creatures
    pub fn update_positions(&mut self, dt: f32) {
        for (pos, vel) in self.positions.iter_mut().zip(&self.velocities) {
            *pos += *vel * dt;
        }
    }

    /// Batch update needs
    pub fn update_needs(&mut self, dt: f32, species_db: &SpeciesDatabase) {
        for (needs, species_id) in self.needs.iter_mut().zip(&self.species_ids) {
            let species = species_db.get(*species_id);
            let metabolic = metabolic_rate_kleiber(species.body_mass_kg);

            // Increase hunger based on metabolism
            needs.hunger += (metabolic / 10000.0) * dt;
            needs.thirst += 0.01 * dt;
            needs.energy -= 0.005 * dt;

            // Clamp values
            needs.hunger = needs.hunger.clamp(0.0, 1.0);
            needs.thirst = needs.thirst.clamp(0.0, 1.0);
            needs.energy = needs.energy.clamp(0.0, 1.0);
        }
    }
}
```

### 9.3 Save/Load

```rust
/// Serializable ecosystem state
#[derive(Serialize, Deserialize)]
pub struct EcosystemSaveData {
    pub version: u32,
    pub populations: HashMap<(RegionId, SpeciesId), PopulationData>,
    pub extinction_events: Vec<(SpeciesId, ExtinctionEvent)>,
    pub habitat_modifications: HashMap<RegionId, HabitatTracker>,
    pub individual_creatures: Vec<CreatureSaveData>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatureSaveData {
    pub id: CreatureId,
    pub species_id: SpeciesId,
    pub position: Vec3,
    pub health: f32,
    pub age: f32,
    pub needs: CreatureNeeds,
    pub domestication: Option<DomesticationData>,
    pub genetics: GeneticData,
}
```

---

## 10. References

### Academic Papers
- Reynolds, C. W. (1987). "Flocks, herds and schools: A distributed behavioral model" - Boids algorithm
- Lotka-Volterra equations for predator-prey dynamics
- Kleiber's Law for metabolic scaling

### Game References
- **Spore**: Creature evolution and ecosystem
- **Dwarf Fortress**: Detailed creature simulation, population dynamics
- **Eco**: Player impact on ecosystem
- **No Man's Sky**: Procedural fauna generation
- **Rain World**: Food chain gameplay

### Online Resources
- [Red3D Boids](http://www.red3d.com/cwr/boids/)
- [Nature of Code - Autonomous Agents](https://natureofcode.com/book/chapter-6-autonomous-agents/)
- [Wikipedia - Lotka-Volterra equations](https://en.wikipedia.org/wiki/Lotka%E2%80%93Volterra_equations)

---

## 11. Ideas & Future Work

### Short Term
- [ ] Basic creature spawning with biome awareness
- [ ] Simple FSM behaviors (eat, drink, flee, wander)
- [ ] Player interaction (hunting, being hunted)
- [ ] Basic population tracking

### Medium Term
- [ ] Full behavior trees
- [ ] Flocking/herding behaviors
- [ ] Reproduction and lifecycle
- [ ] Domestication system
- [ ] Pollution impact on wildlife

### Long Term
- [ ] Emergent food webs
- [ ] Migration patterns
- [ ] Genetic system for creature variation
- [ ] Ecosystem recovery after player leaves area
- [ ] Invasive species from player actions
- [ ] Symbiotic relationships

### Research Questions
1. How to balance simulation depth vs. performance for 10,000+ creatures?
2. How to make player feel consequences without frustrating gameplay?
3. How to handle creature persistence across save/load boundaries?
4. How to generate believable creature designs procedurally?

---

*This document is part of the Synarion Engine research documentation.*
