# Render Flow Usage Guide

## Happy Path

1. Add `RenderPlugin`.
2. Model frame-owned simulation/render values as ECS `Resource` types.
3. Build a `RenderFlow` with ergonomic declarations:
   - `with_state`, `with_surface_color`, `with_surface_depth` (optional), `with_builtin_ui`
   - `double_buffer_storage_array`
   - pass builders (`compute_pass`, `fullscreen_pass`, `builtin_ui_composite_pass`)
4. Validate (`.validate()?`) and register with `App::add_render_flow(...)`.

## Minimal Flow

```rust
use engine::plugins::render::RenderFlow;

let flow = RenderFlow::new("minimal.flow")
    .with_surface_color()
    .fullscreen_pass("minimal.compose")
    .write_surface_color()
    .finish()
    .validate()?;
```

## State-Projected Simulation + Compose

```rust
use engine::plugins::render::{GpuStorage, GpuUniform, RenderFlow};
use engine::prelude::Resource;

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Cell {
    alive: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface: [u32; 2],
}

#[derive(Debug, Clone, Resource)]
struct State {
    tick: u32,
}

impl State {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams { tick: self.tick }
    }

    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            surface: [surface.0, surface.1],
        }
    }

    fn dispatch_workgroups(&self) -> [u32; 3] {
        [8, 8, 1]
    }
}

let flow = RenderFlow::new("sim.flow")
    .with_state::<State>()
    .with_surface_color()
    .with_builtin_ui()
    .double_buffer_storage_array::<Cell>("cells", 1024)
    .compute_pass("simulate")
    .uniform_from_state(State::compute_params)
    .bind_ping_pong_storage("cells")
    .dispatch_from_state(State::dispatch_workgroups)
    .finish()
    .fullscreen_pass("compose")
    .uniform_from_state_with_surface(State::compose_params)
    .bind_ping_pong_storage("cells")
    .write_surface_color()
    .depends_on("simulate")
    .finish()
    .builtin_ui_composite_pass("ui")
    .depends_on("compose")
    .finish()
    .validate()?;
```

## Validation and Contract Inspection

`RenderFlow` keeps contracts inspectable:

- `flow.validation_report()` returns pass order and validation result details.
- `flow.graph()` exposes declared pass/resource topology for tests and tooling.
- `flow.project_uniforms(frame_data, surface_size)` verifies state projection at frame time.

Import-model contract:

- Use typed imports from the public flow API (`with_surface_color`, `with_surface_depth`, `with_builtin_ui`).
- Avoid generic `RenderResourceDescriptor::imported_texture(...)` / `imported_buffer(...)` for active runtime flows.
- Active validation rejects external/generic import semantics in the runtime path.

Runtime boundary note:

- `RenderFrameDataRegistry` remains a compatibility helper for projection tests/tools.
- Active frame execution uses `PreparedRenderFrame` produced in `RenderPrepare`.

Advanced feature-tagged pass note:

- `compute_pass(...)` and `fullscreen_pass(...)` expose optional `.for_feature("feature.id")` tagging.
- Feature-tagged passes execute through the same compiled path but are gated by prepared feature status/fallback policy.
- Only tag passes when the corresponding feature contribution is prepared for the frame; otherwise policy may skip those passes.

Current multi-view scope:

- active runtime execution is single-view only; prepare may carry view containers, but multi-view execution remains explicitly deferred.

## Related Examples

- `engine/examples/render_flow_fullscreen_minimal/main.rs`
- `engine/examples/render_flow_postprocess_compositor/main.rs`
- `engine/examples/game_of_life_sdf/main.rs`
- `engine/examples/boids_render_flow/main.rs`
- `engine/examples/sdf_render_flow/main.rs`
- `engine/examples/procedural_sky_sdf_terrain/main.rs`
- `engine/examples/render_flow_debug_inspect/main.rs`
