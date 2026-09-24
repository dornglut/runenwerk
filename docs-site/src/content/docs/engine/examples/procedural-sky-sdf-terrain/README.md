---
title: "Procedural Sky + SDF Terrain Example"
description: "Documentation for Procedural Sky + SDF Terrain Example."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Procedural Sky + SDF Terrain Example

Windowed public `RenderFlow` sample that raymarches a noise-shaped SDF terrain and a fully procedural sky in one fullscreen pass.

The window title shows a smoothed FPS and frame-time readout while the example runs.

## Controls

- `W`, `A`, `S`, `D`: move forward/left/back/right
- `Space`: move up
- `Ctrl` (`Strg`): move down
- `Shift`: sprint (faster movement)
- mouse move: look around (free-fly camera)
- `Tab`: cycle debug view mode (`lit` -> `height` -> `normals` -> `steps`)

## Structure

- `main.rs`
  - entry point
- `rendering/graph.rs`
  - flow declaration (`with_state`, `with_surface_color`, fullscreen compose pass)
- `rendering/state.rs`
  - ECS-owned camera/time/view state and projected uniform payload
- `runtime/app.rs`
  - app/plugin wiring and per-frame animation + view switching
- shader:
  - `assets/shaders/procedural_sky_sdf_terrain_compose.wgsl`

## Run

```bash
cargo run -p engine --example procedural_sky_sdf_terrain
```
