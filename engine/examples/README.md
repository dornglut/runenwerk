# Engine Examples

Use this map to pick an entry point quickly.

## Quick Pick

- Learn `App`, schedules, and ECS basics:
  - `runtime_minimal`
- Verify windowed runtime + input behavior:
  - `window_input_demo`
- Explore scene/UI orchestration:
  - `scene_manager_ui`
- Explore feature-owned render graph/executor wiring:
  - `sdf_renderer`
- Explore GPU cellular automata with SDF compose:
  - `game_of_life_sdf`
- Learn minimal fullscreen render-flow authoring:
  - `render_flow_fullscreen_minimal`
- Explore a postprocess compositor render flow:
  - `render_flow_postprocess_compositor`
- Explore merged plugin-style flow contributions:
  - `render_flow_contributions`
- Explore flow inspection and debug surfaces:
  - `render_flow_debug_inspect`

## Examples

- `runtime_minimal`
  - Entry: `engine/examples/runtime_minimal/main.rs`
  - Focus: headless run loop, startup/update scheduling, resource/query usage.
  - Run: `cargo run -p engine --example runtime_minimal`
- `window_input_demo`
  - Entry: `engine/examples/window_input_demo/main.rs`
  - Focus: windowed runtime path, default plugin stack, action-mapped input.
  - Run: `cargo run -p engine --example window_input_demo`
- `scene_manager_ui`
  - Entry: `engine/examples/scene_manager_ui/main.rs`
  - Focus: scene registration, UI template assets, scene transitions.
  - Assets: `engine/examples/scene_manager_ui/assets/`
  - Run: `cargo run -p engine --example scene_manager_ui`
- `sdf_renderer`
  - Entry: `engine/examples/sdf_renderer/main.rs`
  - Focus: `RenderFlow`-authored compute/fullscreen/UI chain with custom pass executors and config hot reload.
  - Assets/config: `engine/examples/sdf_renderer/assets/`
  - Run: `cargo run -p engine --example sdf_renderer`
- `game_of_life_sdf`
  - Entry: `engine/examples/game_of_life_sdf/main.rs`
  - Focus: `RenderFlow` + `GpuUniform` ECS-first params with feature-owned compute/compose executors.
  - Shader: `assets/shaders/game_of_life_sdf.wgsl`
  - Run: `cargo run -p engine --example game_of_life_sdf`
- `render_flow_fullscreen_minimal`
  - Entry: `engine/examples/render_flow_fullscreen_minimal/main.rs`
  - Focus: minimal namespaced fullscreen flow (`RenderFlow` + `fullscreen_pass`).
  - Run: `cargo run -p engine --example render_flow_fullscreen_minimal`
- `render_flow_postprocess_compositor`
  - Entry: `engine/examples/render_flow_postprocess_compositor/main.rs`
  - Focus: multi-pass postprocess chain with `copy_pass` + `present_pass`.
  - Run: `cargo run -p engine --example render_flow_postprocess_compositor`
- `render_flow_contributions`
  - Entry: `engine/examples/render_flow_contributions/main.rs`
  - Focus: mixed boids/post/ui `RenderFlowContribution` merge into a base flow.
  - Run: `cargo run -p engine --example render_flow_contributions`
- `render_flow_debug_inspect`
  - Entry: `engine/examples/render_flow_debug_inspect/main.rs`
  - Focus: graph/resource/texture inspection and per-pass timing summary API.
  - Run: `cargo run -p engine --example render_flow_debug_inspect`

## Related

- Crate navigation: `engine/README.md`
- Crate docs hub: `engine/docs/index.md`
- Usage guide: `engine/docs/reference/usage-guide.md`
- Plugin map: `engine/src/plugins/README.md`
- Integration tests: `engine/tests/README.md`
