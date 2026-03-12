# 3D SDF Renderer Example

Compute-shader 3D SDF raymarching example for the engine.

## Current State

- Example entry and plugin wiring are in code: `engine/examples/sdf_renderer/main.rs`.
- The SDF compute shader is at `assets/shaders/sdf_compute_3d_example.wgsl`.
- Setup is loaded from `.ron` configs under `engine/examples/sdf_renderer/assets/`:
  - `sdf_params.ron`
  - `input_bindings.ron`
- Render flow is authored with `RenderFlow` in `engine/examples/sdf_renderer/rendering/graph.rs`.
- GPU params use `GpuUniform`/`GpuStorage` derives in `engine/examples/sdf_renderer/rendering/params.rs`.
- ECS state projection lives on `SdfWorldState` methods in `engine/examples/sdf_renderer/runtime/state.rs`.
- Config parse/load failures emit concise `tracing` diagnostics and automatically fall back to typed defaults.
- `sdf_params.ron` is hot-reloaded at runtime when the file changes.

Current flow shape:

1. `sdf.compute` (compute)
   - reads: `sdf.params`, `world.agents`
   - writes: `sdf.color`
2. `sdf.compose` (render)
   - reads: `sdf.color`
   - writes: `surface.color`
   - depends on: `sdf.compute`
3. `ui.composite` (builtin ui composite)
   - reads: `ui.draw_list`
   - writes: `surface.color`
   - depends on: `sdf.compose`

Compatibility note:

- `sdf.compute` and `sdf.compose` are feature-owned executor ids registered by the example plugin.
- `ui.composite` uses the render plugin builtin UI executor path.
- Input bindings are installed through the app/input API (`App::add_input_bindings`), not from render flow authoring.

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
  - display fit controls:
    - `stretch`
    - `contain` (letterbox/pillarbox)
    - `cover` (crop)
    - `fixed_height`
    - `fixed_width`
  - display quality controls:
    - `render_scale` (supersampling scale; `1.0` = native surface resolution)

Fit mode behavior:

- `contain`: preserves aspect and adds bars when needed.
- `cover`: preserves aspect and fills window by cropping.
- `stretch`: fills window and allows distortion.
- For sharper `cover` at small window sizes, increase `display.render_scale` (example: `1.5`).
- `input_bindings.ron`
  - action -> key bindings for SDF example controls
`sdf.params` clarification:

- `sdf.params` is a logical render resource id in the render graph.
- Data is authored in [sdf_params.ron](./assets/sdf_params.ron) and falls back to typed Rust defaults in [main.rs](./main.rs).
- Runtime parses/validates into a typed struct and writes frame data consumed by `sdf.compute`.

## Authoring Pipeline Direction

This example should also follow the shared authoring pipeline in [docs/authoring-layer.md](../../../docs/authoring-layer.md).

Target split:

- authored assets:
  - `sdf_params.ron`
  - `input_bindings.ron`
- compiled artifacts:
  - validated params/config structs
  - compiled input bindings
  - validated render flow
- live runtime state:
  - render resources
  - input registry/resource state
  - render flow and executor registrations

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
