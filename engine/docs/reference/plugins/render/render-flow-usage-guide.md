# Render Flow Usage Guide

## Happy Path

1. Add `RenderPlugin`.
2. Declare a `RenderFlow` with namespaced resource/pass IDs.
3. Declare ECS resources and GPU param buffers.
4. Add passes (`compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`).
5. Register the flow with `App::add_render_flow`.

Current runtime support in the hard-cutover path:

- builtin compiled execution: `compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`, `builtin_ui_composite_pass`
- `copy_pass` executes texture->texture and buffer->buffer copies for flow-owned resources
- unsupported builtin subfeatures still fail loudly (for example graphics vertex/index/instance/indirect bindings and non-surface imported resources)

## Minimal Flow

```rust
use engine::plugins::render::RenderFlow;

let flow = RenderFlow::new("main.flow")
    .import_texture("surface.color")
    .import_texture("scene.color")
    .fullscreen_pass("main.compose")
    .reads("scene.color")
    .writes("surface.color")
    .finish();
```

## ECS-First Uniform Projection

```rust
let flow = RenderFlow::new("sim.flow")
    .ecs_resource::<GameState>()
    .uniform_buffer::<ComputeParams>("sim.params")
    .compute_pass("sim.compute")
    .uniform_state(GameState::compute_params)
    .reads("sim.params")
    .finish();
```

Use `uniform_state_with_surface` when params depend on surface size.

## Validation

`RenderFlow::validate()` checks:

- namespaced IDs
- duplicate IDs
- missing resources/passes
- dependency cycles
- pass-shape constraints (`copy_pass`/`present_pass`)
- incompatible resource usage (for example texture-as-vertex-buffer)

## Related Examples

- `engine/examples/render_flow_fullscreen_minimal/main.rs`
- `engine/examples/render_flow_postprocess_compositor/main.rs`
- `engine/examples/boids_render_flow/main.rs`
- `engine/examples/sdf_render_flow/main.rs`
- `engine/examples/game_of_life_sdf/main.rs`
