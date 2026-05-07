---
title: "Engine Examples"
description: "Documentation for Engine Examples."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# Engine Examples

Use this map to pick an entry point quickly.

## Quick Pick

- Learn `App`, schedules, and ECS basics:
  - `runtime_minimal`
- Verify windowed runtime + input behavior:
  - `window_input_demo`
- Explore the canonical RenderFlow v2 sample:
  - `game_of_life_sdf`
- Explore boids-style compute simulation + graphics draw-buffer rendering:
  - `boids_render_flow`
- Explore a 3D SDF raymarching flow with compute preparation and history copy:
  - `sdf_render_flow`
- Explore procedural sky + noise terrain SDF rendering:
  - `procedural_sky_sdf_terrain`
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
- `game_of_life_sdf`
  - Entry: `engine/examples/game_of_life_sdf/main.rs`
  - Focus: semantic state + ping-pong simulation on the RenderFlow v2 path.
  - Shaders:
    - `assets/shaders/game_of_life_compute.wgsl`
    - `assets/shaders/game_of_life_compose.wgsl`
  - Run: `cargo run -p engine --example game_of_life_sdf`
- `boids_render_flow`
  - Entry: `engine/examples/boids_render_flow/main.rs`
  - Focus: boids compute simulation with storage ping-pong, public graphics instance-buffer binding, history copy, and explicit present.
  - Shaders:
    - `assets/shaders/boids_compute.wgsl`
    - `assets/shaders/boids_compose.wgsl`
  - Run: `cargo run -p engine --example boids_render_flow`
- `sdf_render_flow`
  - Entry: `engine/examples/sdf_render_flow/main.rs`
  - Focus: compute preparation, fullscreen 3D SDF raymarch composition, flow-owned history copy, explicit present, and Tab-cycled debug views.
  - Shader:
    - `assets/shaders/sdf_render_flow_3d_compose.wgsl`
  - Run: `cargo run -p engine --example sdf_render_flow`
- `procedural_sky_sdf_terrain`
  - Entry: `engine/examples/procedural_sky_sdf_terrain/main.rs`
  - Focus: fullscreen procedural sky and fBm-shaped terrain SDF raymarch with free-fly camera, Shift sprint, title FPS readout, and Tab-cycled debug views.
  - Shader:
    - `assets/shaders/procedural_sky_sdf_terrain_compose.wgsl`
  - Run: `cargo run -p engine --example procedural_sky_sdf_terrain`
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

- Crate docs hub: [`../index.md`](../index.md)
- Usage guide: [`../reference/usage-guide.md`](../reference/usage-guide.md)
- Plugin map: [`../plugins/README.md`](../plugins/README.md)
- Integration tests: [`../tests/README.md`](../tests/README.md)
