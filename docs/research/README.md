# Synarion Engine - Research Documents

This directory contains research documents, equations, prototypes, and technical notes for advanced engine features.

## Purpose

These documents serve as:

- **Living research notes** during feature development
- **Technical reference** for equations and algorithms
- **Prototype repository** for experimental code
- **Knowledge base** for future contributors

## Document Status

| Document | Status | Related Crate |
|----------|--------|---------------|
| [planetary-simulation.md](./planetary-simulation.md) | Active Research | `syn_planet_sim` |
| [terrain-generation.md](./terrain-generation.md) | Active Research | `syn_terrain` |
| [world-graph.md](./world-graph.md) | Active Research | `syn_world` |
| [creation-tools.md](./creation-tools.md) | Active Research | `syn_world_editor` |
| [planetary-rendering.md](./planetary-rendering.md) | Active Research | `syn_atmosphere`, `syn_ocean`, `syn_clouds` |
| [gpu-terrain-shaders.md](./gpu-terrain-shaders.md) | Active Research | `syn_terrain` |
| [resource-system.md](./resource-system.md) | Active Research | `syn_resources` |
| [ecosystem-fauna.md](./ecosystem-fauna.md) | Active Research | `syn_ecosystem`, `syn_fauna`, `syn_ai` |

## Document Structure

Each research document should include:

1. **Overview** - What problem are we solving?
2. **Equations** - Mathematical foundations
3. **Algorithms** - Pseudocode and Rust prototypes
4. **Implementation Notes** - Performance considerations
5. **References** - Papers, books, online resources
6. **Ideas & Future Work** - What to explore next

## Contributing

When adding research:

1. Start with equations and mathematical foundations
2. Write pseudocode before Rust code
3. Include references for non-trivial algorithms
4. Add tests to validate against real-world data
5. Keep "Ideas & Future Work" updated

## Naming Convention

- `{feature}-{aspect}.md` for feature-specific docs
- Use kebab-case for filenames
- Include version and date at document top
