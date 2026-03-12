# GPU Params Guide

## Purpose

`ToGpuValue` + `GpuParams` provide explicit, typed conversion from ECS/app state to GPU-uploadable raw data.

## Traits and Derives

- `ToGpuValue`: field-level conversion contract (scalar and array primitives).
- `GpuParams`: type-level conversion to an internal raw type.
- `#[derive(GpuUniform)]`: uniform-friendly params.
- `#[derive(GpuStorage)]`: storage-buffer params.

## Example

```rust
use engine::plugins::render::GpuUniform;

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface: [u32; 2],
    exposure: f32,
    enabled: bool,
}
```

The derive generates:

- a stable raw layout type
- a `GpuParams` impl
- field conversion through `ToGpuValue`

## Usage with `RenderFlow`

```rust
let flow = RenderFlow::new("main.flow")
    .ecs_resource::<State>()
    .uniform_buffer::<ComposeParams>("main.params")
    .fullscreen_pass("main.compose")
    .uniform_state_with_surface(State::compose_params)
    .reads("main.params")
    .finish();
```
