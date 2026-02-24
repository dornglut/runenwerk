# 3D SDF Renderer Example

Compute-shader 3D SDF raymarching example for the engine.

## Current State

- Example entry and plugin wiring are in code: `engine/examples/sdf_renderer/main.rs`.
- The SDF compute shader is at `assets/shaders/sdf_compute_3d_example.wgsl`.
- Input mapping, render-graph registration, and default world/SDF tuning are currently hardcoded in the example plugin setup.

## Target State (Planned)

Move example setup data into `.ron` files under:

- `engine/examples/sdf_renderer/assets/`

Ownership rule for this example:

- No reuse of built-in `world_compute` / `world_compose` pass ids as the target architecture.
- SDF example should define its own resources, pass ids, pipeline ids, and executor ids.
- Engine should provide abstractions/tooling for custom renderer features, not require users to route through the default world renderer naming.

Planned config split:

- `sdf_params.ron`
  - world and camera defaults
  - SDF/raymarch parameters
  - debug mode defaults
- `input_bindings.ron`
  - action -> key bindings for SDF example controls
- `render_graph.ron`
  - fully plugin-owned pass/pipeline/executor/resource declarations for the example

The goal is that `main.rs` mostly loads config and registers systems, rather than encoding setup constants directly.

Authoring API direction:

- Primary: typed builder API (`RenderFeatureGraphSpec::builder(...)`).
- Secondary: `render_graph.ron` import that converts into the same typed runtime model.

`sdf.params` clarification:

- `sdf.params` is a logical render resource id in the render graph.
- Data should come from `sdf_params.ron` and/or typed Rust defaults.
- Runtime parses/validates into a typed struct and writes frame data consumed by `sdf.compute`.

## Run

```bash
cargo run -p engine --example sdf_renderer
```

## Planned Controls (from `input_bindings.ron`)

- Hold left mouse + move: rotate camera (yaw/pitch)
- Mouse wheel: zoom
- `W/A/S/D`: move camera target on ground plane
- `R/F`: move camera target up/down
- `E`: faster move speed (hold)
- `Q`: slower move speed (hold)
- `Tab`: next debug view (lit -> distance -> normals -> steps)
- `` ` ``: previous debug view
- `Esc`: pause/unpause animation time

## Planning Docs

- Scope and goals: `engine/examples/sdf_renderer/feature-request.md`
- Migration plan and engine-change checklist: `engine/examples/sdf_renderer/plan.md`
