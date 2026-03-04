# Game Folder

## Purpose

Contains game-specific direction, design, and eventually game-owned content/runtime plans that sit above the generic engine/runtime layers.

## Usage

- Start from [CAVERN_HUNT_GDD.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_GDD.md) for the current primary game concept.
- Use [CAVERN_HUNT_MATERIALS.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_MATERIALS.md) for the material graph, PBR-lite, and GI rollout details.
- Keep engine-agnostic gameplay vision here instead of mixing it into engine/plugin docs.
- Treat this folder as the home for game-specific plans, progression rules, content direction, and run structure.

## Ownership Boundaries

- Owns the game concept, player fantasy, content direction, and game-specific delivery goals.
- Does not own generic engine/runtime contracts, ECS internals, or transport implementation details.

## Extension Points

- Add deeper design docs for progression, loot, monsters, biomes, and encounter structure.
- Add game-specific technical plans once the runtime-facing gameplay implementation starts.
- Split into subdocs as the game direction becomes more concrete.
