---
title: "Render Plugin Usage Guide"
description: "Documentation for Render Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

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
- feature registry
- prepared frame resource
- pipeline cache stats resource
- render prepare/submit systems

Feature modules should register feature-owned flow declarations and state resources; they should not duplicate plugin bootstrap wiring.

## Frame Pipeline Contract

`RenderPlugin` wires a two-phase render frame contract:

1. `RenderPrepare`:
   - `sync_render_feature_registry_system`
   - `sync_render_flow_registry_system`
   - `frame_render_prepare_system`
2. `RenderSubmit`:
   - `frame_render_submit_system`

`frame_render_prepare_system` publishes an owned `PreparedRenderFrame` into `PreparedRenderFrameResource`.

`frame_render_submit_system` consumes the prepared frame and submits through renderer/backend runtime state. It does not perform live extraction of flow-declared ECS resources.

Feature fallback policy is resolved in prepare (`Ready | Stale | Disabled | Missing`) and submit/runtime executes the encoded policy without ECS back-fills.

## Prepared Frame Surface

`PreparedRenderFrame` carries the prepare/submit boundary payload:

- frame context (`PreparedFrameContext`)
- surface snapshot (`PreparedSurfaceInfo`)
- view container (`PreparedViewFrame`)
- flow-projected uniforms and dispatch state (`PreparedFlowInputs`)
- feature/domain contributions (`PreparedFrameContributions`)
- shader revision snapshot (`PreparedShaderSnapshot`)

UI note:

- UI travels as `PreparedFeaturePayload::Ui` carrying `PreparedUiFrameContribution`.
- Each submission carries an owned `UiFrame` payload with ordered route/layer/priority metadata.
