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
  - Focus: custom render feature graph, custom pass executors, config hot reload.
  - Assets/config: `engine/examples/sdf_renderer/assets/`
  - Run: `cargo run -p engine --example sdf_renderer`

## Related

- Crate navigation: `engine/README.md`
- Crate docs hub: `engine/docs/index.md`
- Usage guide: `engine/docs/reference/usage-guide.md`
- Plugin map: `engine/src/plugins/README.md`
- Integration tests: `engine/tests/README.md`
