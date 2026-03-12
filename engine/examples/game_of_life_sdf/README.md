# Game of Life SDF Example

Public `RenderFlow` API example that shows:

- namespaced `compute_pass` + `fullscreen_pass` + `builtin_ui_composite_pass` chaining
- builtin compiled execution only (no custom executors, no low-level registry mutation); graphics/copy/present are also builtin-supported in the same runtime path
- windowed app wiring with `App::add_render_flow`
- a procedural Game of Life-style fullscreen shader (`assets/shaders/game_of_life_sdf.wgsl`)
- ECS-first render params (`ecs_resource`, `uniform_buffer`, `uniform_state`, `uniform_state_with_surface`)
- explicit simulation resource flow: `gol.simulate` writes `gol.cells`, `gol.compose` reads `gol.cells`

Shader lookup note:

- pass builders use shader registry IDs (for this file: `game_of_life_sdf`), not raw file paths.
- the example registers that exact shader ID explicitly at startup using an absolute path, so it works regardless of the current working directory.
- shader registration fails loudly if `ShaderRegistryResource` is missing.

Runtime state note:

- the app inserts `GameOfLifeRenderState` into ECS and registers it in `RenderFrameResourceBindings`.
- this allows state-to-uniform projection for:
  - `gol.compute.params`
  - `gol.compose.params`

## Run

```bash
cargo run -p engine --example game_of_life_sdf
```
