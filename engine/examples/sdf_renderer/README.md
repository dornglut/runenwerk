# 3D SDF Renderer Example

Compute-shader 3D SDF raymarching example for the engine.

## Current State

- Example entry and plugin wiring are in code: `engine/examples/sdf_renderer/main.rs`.
- The SDF compute shader is at `assets/shaders/sdf_compute_3d_example.wgsl`.
- Setup is loaded from `.ron` configs under `engine/examples/sdf_renderer/assets/`:
  - `sdf_params.ron`
  - `input_bindings.ron`
  - `render_graph.ron`
- Config parse/load failures emit concise `tracing` diagnostics and automatically fall back to typed defaults.

Current graph shape (registered by example):

1. `sdf.compute` (compute)
   - executor: `sdf.compute`
   - reads: `sdf.params`, `world.agents`
   - writes: `sdf.color`
2. `sdf.compose` (render)
   - executor: `sdf.compose`
   - reads: `sdf.color`
   - writes: `surface.color`
   - depends on: `sdf.compute`
3. `ui_composite` (render)
   - executor: `ui_composite`
   - reads: `ui.draw_list`
   - writes: `surface.color`
   - depends on: `sdf.compose`

Compatibility note:

- Executor ids are feature-owned in `render_graph.ron`.
- `executor_bindings` are used to register `register_custom` executors.
- `sdf.compute` and `sdf.compose` now use feature-owned executor implementations in this example.
- `ui_composite` also runs through a custom executor path in the example.
- Use `builtin_compute`, `builtin_compose`, and `builtin_ui_composite` labels in config.
- This keeps SDF pass ownership in the example while preserving parity.

## Target State (Planned)

Ownership rule for this example:

- No reuse of built-in `world_compute` / `world_compose` pass ids as the target architecture.
- SDF example should define its own resources, pass ids, pipeline ids, and executor ids.
- Engine should provide abstractions/tooling for custom renderer features, not require users to route through the default world renderer naming.

Current config split:

- `sdf_params.ron`
  - world and camera defaults
  - SDF controls/tuning
  - debug mode defaults
- `input_bindings.ron`
  - action -> key bindings for SDF example controls
- `render_graph.ron`
  - feature-owned pass/pipeline/executor/resource declarations

Authoring API direction:

- Primary: typed builder API (`RenderFeatureGraphSpec::builder(...)`).
- Secondary: `render_graph.ron` import that converts into the same runtime model.
- Load/validate/compile/apply should move into the shared engine authoring pipeline over time.

`sdf.params` clarification:

- `sdf.params` is a logical render resource id in the render graph.
- Data is authored in [sdf_params.ron](/Users/joshua/Projekte/grotto-quest/engine/examples/sdf_renderer/assets/sdf_params.ron) and falls back to typed Rust defaults in [main.rs](/Users/joshua/Projekte/grotto-quest/engine/examples/sdf_renderer/main.rs).
- Runtime parses/validates into a typed struct and writes frame data consumed by `sdf.compute`.

## Authoring Pipeline Direction

This example should also follow the shared authoring pipeline in [docs/authoring-layer.md](/Users/joshua/Projekte/grotto-quest/docs/authoring-layer.md).

Target split:

- authored assets:
  - `sdf_params.ron`
  - `input_bindings.ron`
  - `render_graph.ron`
- compiled artifacts:
  - validated params/config structs
  - compiled input bindings
  - validated render graph spec
- live runtime state:
  - render resources
  - input registry/resource state
  - render graph and executor registrations

Reload rules for this example:

- render graph authoring must track dependencies on referenced shaders, executors, pipelines, and logical resource ids
- reload should compute one affected bundle and only swap it in if the bundle compiles fully
- partial success must not leave mixed-generation render graph state active

Diagnostics should report:

- source asset path
- unresolved resource/pipeline/executor/shader references
- the dependency chain that produced the failed compiled graph
- actionable hints for fallback/default behavior when relevant

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
