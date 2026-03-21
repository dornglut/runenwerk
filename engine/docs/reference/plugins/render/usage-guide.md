# Render Plugin Usage Guide

This page covers plugin/runtime wiring. For `RenderFlow` authoring, use:

- `engine/docs/reference/plugins/render/render-flow-usage-guide.md`
- `engine/docs/reference/plugins/render/gpu-params-guide.md`

## Purpose

`RenderPlugin` owns runtime render wiring: resources, builtin executor registrations, and render schedules.

## Setup

```rust
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};

app.add_plugins(default_plugins());
app.add_plugin(ScenePlugin);
app.add_plugin(RenderPlugin);
```

## Runtime Ownership

`RenderPlugin` initializes render runtime resources and systems, including:

- flow registry
- pass executor registry
- pipeline/resource registries
- render prepare/submit systems

Feature modules should register feature-owned flow declarations and state resources; they should not duplicate plugin bootstrap wiring.

## Frame Pipeline Contract

`RenderPlugin` wires a two-phase render frame contract:

1. `RenderPrepare`:
   - `sync_render_flow_registry_system`
   - `frame_render_prepare_system`
2. `RenderSubmit`:
   - `ui_render_submit_system`

`frame_render_prepare_system` publishes an owned `PreparedRenderFrame` into `PreparedRenderFrameResource`.

`ui_render_submit_system` consumes the prepared frame and submits through renderer/backend runtime state. It should not perform live extraction of flow-declared ECS resources.

## Prepared Frame Surface

`PreparedRenderFrame` carries the prepare/submit boundary payload:

- surface snapshot (`PreparedSurfaceInfo`)
- scene labels (`PreparedSceneInfo`)
- UI input (`PreparedUiInput`)
- flow-projected uniforms and dispatch state (`PreparedFlowInputs`)
- shader revision snapshot (`PreparedShaderSnapshot`)

Phase-1 note:

- `PreparedUiInput::RawDrawList` is the current transport format.
- Long-term direction is extracted backend-neutral prepared UI input.
