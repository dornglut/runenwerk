# GPU Params Guide

## Purpose

`GpuUniform` and `GpuStorage` derive macros turn semantic Rust structs into GPU-uploadable layouts.

## Traits and Derives

- `GpuParams`: type-level conversion to an internal raw representation.
- `ToGpuValue`: field-level conversion for supported primitives.
- `#[derive(GpuUniform)]`: uniform-buffer projection.
- `#[derive(GpuStorage)]`: storage-buffer projection.

## Uniform Padding Ownership

Uniform ABI padding is engine-owned. User structs should stay semantic and should not declare manual `_pad` fields.

```rust
use engine::math::UVec2;
use engine::plugins::render::GpuUniform;

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    grid_size: UVec2,
    surface_size: [f32; 2],
    alive_mix: f32,
    current_is_a: u32,
}
```

## Usage with RenderFlow v2

```rust
use engine::plugins::render::{GpuUniform, RenderFlow};
use engine::prelude::Resource;

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface: [u32; 2],
}

#[derive(Debug, Clone, Resource)]
struct State;

impl State {
    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            surface: [surface.0, surface.1],
        }
    }
}

let flow = RenderFlow::new("main.flow")
    .with_state::<State>()
    .with_surface_color()
    .fullscreen_pass("main.compose")
    .uniform_from_state_with_surface(State::compose_params)
    .write_surface_color()
    .finish()
    .validate()?;
```

## Contract Testing

Use `flow.project_uniforms(...)` in tests to verify that expected pass uniforms are produced for your state type.
