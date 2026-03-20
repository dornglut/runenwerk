# Game of Life SDF Example

Public `RenderFlow` API example that shows:

- namespaced `compute_pass` + `fullscreen_pass` + `builtin_ui_composite_pass` chaining
- builtin compiled execution only (no custom executors, no low-level registry mutation); graphics/copy/present are also builtin-supported in the same runtime path
- windowed app wiring with `App::add_render_flow`
- a procedural Game of Life GPU simulation + compose pair:
  - `assets/shaders/game_of_life_compute.wgsl`
  - `assets/shaders/game_of_life_compose.wgsl`
- ECS-first render params (`ecs_resource`, `uniform_buffer`, `uniform_state`, `uniform_state_with_surface`)
- explicit ping-pong simulation resource flow:
  - `gol.simulate` reads/writes `gol.cells_a` + `gol.cells_b`
  - `gol.compose` reads `gol.cells_a` + `gol.cells_b`

Shader lookup note:

- pass builders use shader registry IDs (for this example: `game_of_life.compute` and `game_of_life.compose`), not raw file paths.
- the example registers shader IDs explicitly at startup using absolute paths, so it works regardless of the current working directory.
- shader registration fails loudly if `ShaderRegistryResource` is missing.

Runtime state note:

- the app inserts `GameOfLifeRenderState` into ECS.
- flow-declared `ecs_resource::<...>()` values are collected automatically at render submit time.
- this enables state-to-uniform projection for:
  - `gol.compute.params`
  - `gol.compose.params`

## Run

```bash
cargo run -p engine --example game_of_life_sdf
```
