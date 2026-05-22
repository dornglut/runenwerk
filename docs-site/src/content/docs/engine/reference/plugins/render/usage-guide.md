---
title: "Render Plugin Usage Guide"
description: "Documentation for Render Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# Render Plugin Usage Guide

This page covers plugin/runtime wiring. For `RenderFlow` authoring, use:

- [`render-flow-usage-guide.md`](render-flow-usage-guide.md)
- [`gpu-params-guide.md`](gpu-params-guide.md)

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

Runtime performance policy is split by ownership:

- window redraw cadence is owned by `FramePacingPolicyResource` in the runtime
  domain and defaults to capped continuous 60 FPS for windowed apps;
- shader live reload stays enabled by default, but `ShaderRegistryResource`
  throttles normal directory polling to 500 ms after the first poll while
  `request_reload()` still bypasses the throttle;
- render diagnostics are tiered by `RenderFrameDiagnosticsPolicyResource`.
  Cheap timings, pass samples, shader poll state, preflight cache state, and
  pacing state remain available every frame. Full `RenderDebugFrameReport`
  mapping runs for provenance/capture/readback/pixel probes/texture diffs/export,
  slow frames, explicit requests, or full-every-frame policy.

## Prepared Frame Surface

`PreparedRenderFrame` carries the prepare/submit boundary payload:

- frame context (`PreparedFrameContext`)
- surface snapshot (`PreparedSurfaceInfo`)
- view container (`PreparedViewFrame`)
- flow-projected uniforms and dispatch state (`PreparedFlowInputs`)
- prepared flow invocations with view ids, target alias bindings, and history signatures (`PreparedFlowInvocation`)
- dynamic texture target request snapshot (`RenderDynamicTextureTargetDescriptor`)
- viewport surface binding registry for UI/product sampling
- feature/domain contributions (`PreparedFrameContributions`)
- shader revision snapshot (`PreparedShaderSnapshot`)

UI note:

- UI travels as `PreparedFeaturePayload::Ui` carrying `PreparedUiFrameContribution`.
- Each submission carries an owned `UiFrame` payload with ordered route/layer/priority metadata.
