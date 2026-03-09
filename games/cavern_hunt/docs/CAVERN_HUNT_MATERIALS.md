# Cavern Hunt Materials and Lighting

Updated: 2026-03-04

## Goal

`Cavern Hunt` now has a game-owned, asset-driven material pipeline for SDF rendering:

- material authoring in RON graph assets
- compiled graph runtime as compact op streams + constants
- triplanar procedural surface layer for floor/wall differentiation
- PBR-lite lighting (GGX/Smith/Schlick)
- staged GI modes (`Off`, `AoBentNormal`, `ProbeGi`)

This path is intentionally game-first in `cavern_hunt` and does not require an engine-wide material abstraction yet.

## Asset Layout

- Graph assets: [games/cavern_hunt/assets/materials/graphs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/assets/materials/graphs)
- Profile assets: [games/cavern_hunt/assets/materials/profiles](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/assets/materials/profiles)

Current graph IDs:

- `rock_terrain`
- `barrier_rock`
- `hazard_glow`
- `marker_signal`

Current profile IDs:

- `performance`
- `balanced`
- `quality`

## Quick Start

Use this for the active material path:

```bash
CAVERN_RENDER_MODE=material_graph CAVERN_MATERIAL_PROFILE=balanced scripts/run_cavern_client.sh
```

On successful load, logs should include:

- `cavern material runtime ready`
- non-zero counts for `graph_count` and `class_program_count`

## Runtime Modules

- Graph schema/compiler: [material_graph.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/src/domain/material_graph.rs)
- Runtime state and GPU payload assembly: [material_runtime.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/src/domain/material_runtime.rs)
- Loader/hot-reload plugin: [materials/plugin.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/src/features/materials/plugin.rs)
- Render integration: [render_sdf/plugin.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/src/features/render_sdf/plugin.rs)
- Shader path: [cavern_hunt_sdf.wgsl](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/assets/shaders/cavern_hunt_sdf.wgsl)

## Material Model

`Triplanar` and `PBR` are not alternatives here. They are layered:

- triplanar/noise/slope/height nodes generate material inputs
- PBR-lite consumes those outputs for lighting

Material graph outputs:

- `base_color`
- `roughness`
- `metallic`
- `normal_perturb` (optional, currently placeholder)
- `ao`
- `emissive`

## GI Staging

- `Off`: direct light + ambient baseline only
- `AoBentNormal`: SDF ambient occlusion and bent-normal ambient sampling
- `ProbeGi`: currently scaffolded in runtime/config and shader mode path; staged for incremental probe data integration

## Runtime Flags

- `CAVERN_RENDER_MODE=legacy|material_graph`
- `CAVERN_GI_MODE=off|ao|probes`
- `CAVERN_GI_QUALITY=low|medium|high`
- `CAVERN_GI_SAMPLE_BUDGET=<u32>`
- `CAVERN_MATERIAL_PROFILE=performance|balanced|quality`
- `CAVERN_MATERIAL_WATCH=true|false`
- `CAVERN_MATERIAL_POLL_SECONDS=<f32>`

The active profile sets defaults; env vars are applied as the final override layer.

## Asset Authoring Rules

The current RON schema is strict. Most common failure:

- `ConstantVec3` must use tuple syntax, not array syntax.

Correct:

```ron
kind: ConstantVec3(value: (0.42, 0.28, 0.20))
```

Incorrect:

```ron
kind: ConstantVec3(value: [0.42, 0.28, 0.20])
```

## Safety/Fallback Rules

- material graph compile failures keep the previous valid compiled program
- missing/invalid profiles fall back to legacy shading path
- render mode can always be forced to `legacy` with `CAVERN_RENDER_MODE=legacy`

## Known Gaps

- no visual node editor yet (asset-only RON authoring)
- no normal-map perturb output integration yet
- probe GI data population is scaffolded but not fully implemented
- this is still a game-owned system, not an extracted engine-wide framework

## Troubleshooting

If the cave still looks grey:

1. Check `logs/engine.log` for:
   - `cavern material runtime ready`
   - `graph_count`
   - `class_program_count`
2. If counts are `0`, fix parsing errors reported as:
   - `cavern material diagnostic`
3. Force fallback check:
   - `CAVERN_RENDER_MODE=legacy`
4. Return to material mode:
   - `CAVERN_RENDER_MODE=material_graph`
