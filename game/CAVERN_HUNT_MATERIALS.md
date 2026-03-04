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

- Graph assets: [game/assets/materials/graphs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/assets/materials/graphs)
- Profile assets: [game/assets/materials/profiles](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/assets/materials/profiles)

Current graph IDs:

- `rock_terrain`
- `barrier_rock`
- `hazard_glow`
- `marker_signal`

Current profile IDs:

- `performance`
- `balanced`
- `quality`

## Runtime Modules

- Graph schema/compiler: [material_graph.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/cavern_hunt/src/domain/material_graph.rs)
- Runtime state and GPU payload assembly: [material_runtime.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/cavern_hunt/src/domain/material_runtime.rs)
- Loader/hot-reload plugin: [materials.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/cavern_hunt/src/plugins/materials.rs)
- Render integration: [render_sdf.rs](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/cavern_hunt/src/plugins/render_sdf.rs)
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

## Safety/Fallback Rules

- material graph compile failures keep the previous valid compiled program
- missing/invalid profiles fall back to legacy shading path
- render mode can always be forced to `legacy` with `CAVERN_RENDER_MODE=legacy`

## Known Gaps

- no visual node editor yet (asset-only RON authoring)
- no normal-map perturb output integration yet
- probe GI data population is scaffolded but not fully implemented
- this is still a game-owned system, not an extracted engine-wide framework
