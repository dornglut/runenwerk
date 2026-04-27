---
title: "Game of Life SDF Example"
description: "Documentation for Game of Life SDF Example."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Game of Life SDF Example

Public `RenderFlow` v2 example that shows:

- semantic simulation state as an ECS `Resource`
- render flow declaration with:
  - `with_state`, `with_surface_color`, `with_builtin_ui`
  - `double_buffer_storage_array`
  - `compute_pass` + `fullscreen_pass` + `builtin_ui_composite_pass`
  - state-projected uniforms (`uniform_from_state`, `uniform_from_state_with_surface`)
  - chainable validation (`.validate().expect(...)`)
- no manual shader registration ceremony in app bootstrap
- common-path flow declarations (no explicit `.for_feature(...)` tagging required)
- explicit WGSL bindings for compute/compose shaders:
  - `assets/shaders/game_of_life_compute.wgsl`
  - `assets/shaders/game_of_life_compose.wgsl`

Runtime state note:

- the app inserts `GameOfLifeRenderState` as a resource.
- update systems advance the clock/tick and ping-pong phase.
- pass uniforms are projected from state methods in `rendering/state.rs`.
- unit tests keep `RenderFrameDataRegistry` usage only as projection-helper compatibility coverage.

## Run

```bash
cargo run -p engine --example game_of_life_sdf
```
