---
title: "Render Flow Usage Guide"
description: "Documentation for Render Flow Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-05
---

# Render Flow Usage Guide

## Happy Path

1. Add `RenderPlugin`.
2. Model frame-owned simulation/render values as ECS `Resource` types.
3. Build a `RenderFlow` with ergonomic declarations:
   - `with_state`, `with_surface_color`, `with_builtin_ui`
   - `with_color_target`, `with_color_target_exact`, `with_depth_target`, `with_history_texture`
   - `double_buffer_storage_array`
   - pass builders (`compute_pass`, `fullscreen_pass`, `graphics_pass`, `copy_pass`, `present_pass`, `builtin_ui_composite_pass`)
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
use engine::plugins::render::{
    GpuStorage, GpuUniform, RenderFlow, RenderVertexBufferLayout, RenderVertexFormat,
};
use engine::prelude::Resource;

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Cell {
    alive: u32,
}

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Instance {
    position: [f32; 2],
    radius: f32,
    _pad: u32,
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

let (flow, instances) = RenderFlow::new("sim.flow")
    .with_state::<State>()
    .with_surface_color()
    .with_color_target("scene.color")
    .with_history_texture("scene.history")
    .double_buffer_storage_array::<Cell>("cells", 1024)
    .storage_array::<Instance>("instances", 128);

let flow = flow
    .compute_pass("simulate")
    .uniform_from_state(State::compute_params)
    .bind_ping_pong_storage("cells")
    .dispatch_from_state(State::dispatch_workgroups)
    .finish()
    .graphics_pass("draw")
    .uniform_from_state_with_surface(State::compose_params)
    .bind_ping_pong_storage("cells")
    .instance_buffer(
        instances,
        RenderVertexBufferLayout::instance(0, 16)
            .attribute(0, 0, RenderVertexFormat::Float32x2)
            .attribute(1, 8, RenderVertexFormat::Float32),
    )
    .write_color_target("scene.color")
    .draw(3, 128)
    .depends_on("simulate")
    .finish()
    .copy_pass("history")
    .source("scene.color")
    .destination("scene.history")
    .depends_on("draw")
    .finish()
    .present_pass("present")
    .source("scene.color")
    .depends_on("history")
    .finish()
    .validate()?;
```

## Fullscreen Compose + Present

Fullscreen raymarching and postprocess flows should stay on `fullscreen_pass(...)` when they draw a screen triangle without mesh/instance buffers. Use a flow-owned color target when the frame should end with an explicit terminal present pass:

```rust
use engine::plugins::render::RenderFlow;

let flow = RenderFlow::new("sdf.flow")
    .with_surface_color()
    .with_color_target("sdf.color")
    .fullscreen_pass("sdf.compose")
    .write_color_target("sdf.color")
    .finish()
    .present_pass("sdf.present")
    .source("sdf.color")
    .depends_on("sdf.compose")
    .finish()
    .validate()?;
```

Compatibility path:

- A fullscreen or graphics pass may still write `surface.color` directly with `.write_surface_color()`.
- Use `present_pass(...)` when the flow needs a first-class terminal pass for inspection, ordering, or copy/history work.

## Surface-Format And Exact-Format Color Targets

Use `with_color_target("...")` for presentation-style color targets that should inherit the platform-selected surface/swapchain format. This keeps ordinary scene, viewport, and UI presentation paths aligned with the active adapter and surface capabilities.

Use `with_color_target_exact("...", format)` for surface-sized, flow-owned targets whose byte format is part of the data contract. Exact means exact texture format only; it does not mean fixed size. CPU proof data, deterministic product bytes, and intermediate byte-truth targets should declare the required format explicitly:

```rust
use engine::plugins::render::{RenderFlow, RenderTextureTargetFormat};

let flow = RenderFlow::new("proof.flow")
    .with_color_target_exact("proof.rgba8", RenderTextureTargetFormat::Rgba8Unorm)
    .validate()?;
```

Future fixed-size exact targets should be added as a separate API if a flow needs exact dimensions as well as exact format.

## Product Targets, Aliases, And Prepared Invocations

Use flow-owned targets when one compiled flow writes one local product:

```rust
let flow = RenderFlow::new("product.flow")
    .with_surface_color()
    .with_color_target("product.color")
    .with_history_texture("product.history")
    .fullscreen_pass("product.compose")
    .write_color_target("product.color")
    .finish()
    .copy_pass("product.history")
    .source("product.color")
    .destination("product.history")
    .depends_on("product.compose")
    .finish()
    .present_pass("product.present")
    .source("product.color")
    .depends_on("product.history")
    .finish()
    .validate()?;
```

Use target aliases when authored flow topology should stay static while prepared invocations bind concrete product targets. This is the intended product-surface API shape; active runtime execution must stay on flow-owned targets until target alias validation and renderer resolution are landed:

```rust
use engine::plugins::render::{
    PreparedFlowInvocationId, PreparedFlowInvocationRequest, PreparedTargetBinding,
    PreparedViewFrame, RenderDynamicTextureTargetKey, RenderFlow,
};
use std::collections::BTreeMap;

let flow = RenderFlow::new("viewport.product.flow")
    .with_color_target_alias("viewport.scene_color")
    .fullscreen_pass("viewport.compose")
    .offscreen_products_only()
    .write_color_target("viewport.scene_color")
    .finish();

let view = PreparedViewFrame::offscreen_product("viewport.1", (1280, 720));
let mut target_alias_bindings = BTreeMap::new();
target_alias_bindings.insert(
    "viewport.scene_color".to_string(),
    PreparedTargetBinding::DynamicTexture(RenderDynamicTextureTargetKey::new(
        "editor.viewport.1",
        "scene_color",
    )),
);

let invocation = PreparedFlowInvocationRequest {
    invocation_id: PreparedFlowInvocationId::new("viewport.1.scene"),
    flow_id: flow.id(),
    view_id: view.view_id.clone(),
    target_alias_bindings,
    uniform_overrides: BTreeMap::new(),
    history_signature: Some("camera:v1:1280x720".to_string()),
};
```

Prepared render frame requests are written before `RenderPrepare`. `RenderPrepare` snapshots requested views, prepared flow invocations, target alias bindings, dynamic target descriptors, projected uniform bytes, dispatch workgroups, and history signatures into `PreparedRenderFrame`. `RenderSubmit` must consume that packet rather than rediscovering product targets from live ECS state.

Current implementation boundary:

- `RenderDynamicTextureTargetRequestRegistryResource` validates producer-scoped dynamic target descriptor contributions and snapshots them into `PreparedRenderFrame`.
- `PreparedRenderFrameRequestResource` carries producer-scoped offscreen product views and per-flow invocation requests.
- Target alias execution and renderer-owned dynamic texture cache work are implemented foundation behavior; do not model dynamic products by cloning flows or suffixing static flow resource labels.

History retention should be expressed through explicit history resources or dynamic target retention policy:

- flow-owned history textures use `with_history_texture(...)` plus a `copy_pass(...)` and are scoped per prepared invocation;
- dynamic product targets use `RenderDynamicTextureRetention` and prepared view/invocation history signatures;
- prepared views and invocations carry history signatures so resize, camera, product, or descriptor changes can invalidate only the affected product/history scope.

## Copy Pass Raw Transfer Policy

`copy_pass(...)` is a raw texture transfer. It allows identical color formats and color formats that are identical after removing the sRGB suffix, such as `Rgba8Unorm <-> Rgba8UnormSrgb` and `Bgra8Unorm <-> Bgra8UnormSrgb`.

No color-space conversion happens during `copy_pass(...)`. Unrelated color formats and depth/stencil formats are rejected. Any actual color-space conversion must be modeled as a future explicit shader blit/convert pass family, never hidden inside `copy_pass(...)`.

## UI Composite After Direct Surface Writes

`builtin_ui_composite_pass(...)` writes to the surface color import. It is appropriate after flows that render directly to `surface.color`:

```rust
let flow = RenderFlow::new("ui.flow")
    .with_surface_color()
    .with_builtin_ui()
    .fullscreen_pass("compose")
    .write_surface_color()
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

- Use typed imports from the public flow API (`with_surface_color`, `with_builtin_ui`) for active runtime flows.
- Use flow-owned `with_depth_target(...)` plus `.depth_target(...)` for runtime-backed graphics depth attachments.
- `with_surface_depth` / `RenderResourceDescriptor::imported_surface_depth` remain declaration compatibility for the typed import model, but imported surface depth is not accepted as a graphics depth target until the renderer exposes a prepared surface-depth texture.
- Avoid generic `RenderResourceDescriptor::imported_texture(...)` / `imported_buffer(...)` for active runtime flows.
- Active validation rejects external/generic import semantics in the runtime path.

Graphics contract:

- `graphics_pass(...)` must declare exactly one color output and explicit draw parameters with `.draw(...)` or `.draw_with_offsets(...)`.
- Vertex and instance buffers are runtime-backed when authored with `RenderVertexBufferLayout::{vertex, instance}` attributes. Layout slots must be dense from `0`, shader locations must be unique, and every buffer must have a matching layout.
- Graphics storage bindings are bind-group storage reads/writes. Raster color/depth writes must use `.write_color_target(...)` / `.depth_target(...)`; storage bindings are not color attachments.

Runtime boundary note:

- `RenderFrameDataRegistry` remains a compatibility helper for projection tests/tools.
- Active frame execution uses `PreparedRenderFrame` produced in `RenderPrepare`.

Advanced feature-tagged pass note:

- `compute_pass(...)`, `fullscreen_pass(...)`, and `graphics_pass(...)` expose optional `.for_feature("feature.id")` tagging.
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
