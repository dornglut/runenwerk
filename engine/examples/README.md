# Engine Examples

Use this map to pick an entry point quickly.

## Quick Pick

- Learn `App`, schedules, and ECS basics:
  - `runtime_minimal`
- Verify windowed runtime + input behavior:
  - `window_input_demo`
- Explore scene/UI orchestration:
  - `scene_manager_ui`
- Explore the canonical RenderFlow v2 sample:
  - `game_of_life_sdf`
- Learn the smallest fullscreen render-flow declaration:
  - `render_flow_fullscreen_minimal`
- Explore compute + compose flow authoring:
  - `render_flow_postprocess_compositor`
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
- `game_of_life_sdf`
  - Entry: `engine/examples/game_of_life_sdf/main.rs`
  - Focus: semantic state + ping-pong simulation on the RenderFlow v2 path.
  - Shaders:
    - `assets/shaders/game_of_life_compute.wgsl`
    - `assets/shaders/game_of_life_compose.wgsl`
  - Run: `cargo run -p engine --example game_of_life_sdf`
- `render_flow_fullscreen_minimal`
  - Entry: `engine/examples/render_flow_fullscreen_minimal/main.rs`
  - Focus: minimal `with_surface_color` + `fullscreen_pass` + `write_surface_color` chain.
  - Run: `cargo run -p engine --example render_flow_fullscreen_minimal`
- `render_flow_postprocess_compositor`
  - Entry: `engine/examples/render_flow_postprocess_compositor/main.rs`
  - Focus: compute + fullscreen pass chaining with double-buffer storage.
  - Run: `cargo run -p engine --example render_flow_postprocess_compositor`
- `render_flow_debug_inspect`
  - Entry: `engine/examples/render_flow_debug_inspect/main.rs`
  - Focus: graph/resource inspection and per-pass timing summaries.
  - Run: `cargo run -p engine --example render_flow_debug_inspect`

## Related

- Crate navigation: `engine/README.md`
- Crate docs hub: `engine/docs/index.md`
- Usage guide: `engine/docs/reference/usage-guide.md`
- Plugin map: `engine/src/plugins/README.md`
- Integration tests: `engine/tests/README.md`
