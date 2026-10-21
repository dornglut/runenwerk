---
title: "Render Plugin Usage Guide"
description: "Documentation for Render Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: primary
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
use engine::prelude::AppRenderExt;

app.add_plugins(default_plugins());
app.add_plugin(ScenePlugin);
app.add_plugin(RenderPlugin);
```

## App integration ownership

Render-specific App ergonomics live on the Render-owned `AppRenderExt` extension:

- `add_render_flow`
- `update_render_debug_control`
- `update_render_debug_config`

Import `AppRenderExt` directly or through `engine::prelude::*`. These owner operations may materialize Render-owned configuration/debug resources before `RenderPlugin` is selected; that state does not itself activate Render integration, and later plugin installation preserves it.

## Runtime Ownership

`RenderPlugin` initializes render runtime resources and systems, including:

- flow registry
- feature registry
- prepared frame resource
- pipeline cache stats resource
- renderer-owned nonblocking GPU progress in `PreUpdate`
- render prepare/submit systems

Feature modules should register feature-owned flow declarations and state resources; they should not duplicate plugin bootstrap wiring.

## Frame Pipeline Contract

`RenderPlugin` wires renderer GPU completion observation before producer verification, then a two-phase render frame contract:

1. `PreUpdate`: the invoker-thread `frame_render_gpu_progress_before_update` advances the current Gfx context once through the existing nonblocking RunenGPU G5 progress/reconciliation contract when Gfx exists. GPU completion can then be verified by source-owned `Update` producers. No Gfx means no poll; no drawable frame does not inhibit this progress.
2. `RenderPrepare`:
   - `sync_render_feature_registry_system`
   - `sync_render_flow_registry_system`
   - `frame_render_prepare_system`
3. `RenderSubmit`:
   - `frame_render_submit_system`

`frame_render_prepare_system` publishes an owned `PreparedRenderFrame` into `PreparedRenderFrameResource`.

`frame_render_submit_system` consumes the prepared frame and submits through renderer integration state and public RunenGPU execution contracts. It does not perform live extraction of flow-declared ECS resources. The first matching Gfx render consumes the earlier GPU progress observation instead of polling twice; subsequent surfaces retain their normal progress, and an incompatible device affinity falls back to a normal render-time observation. This observation alone never verifies producer steps or authorizes retained-resource reuse.

Feature fallback policy is resolved in prepare (`Ready | Stale | Disabled | Missing`) and submit/runtime executes the encoded policy without ECS back-fills.

Runtime performance policy is split by ownership:

- window redraw cadence is owned by `FramePacingPolicyResource` in the runtime
  domain and defaults to capped continuous 60 FPS for windowed apps;
- shader live reload stays enabled by default, but `ShaderRegistryResource`
  throttles normal directory polling to 500 ms after the first poll while
  `request_reload()` still bypasses the throttle;
- render diagnostics are tiered by `RenderFrameDiagnosticsPolicyResource`.
  Cheap CPU timings, pass samples, GPU timing capability/diagnostics, shader
  poll state, pass-shape guard diagnostics, preflight cache state, and pacing
  state remain available every frame. Full `RenderDebugFrameReport` mapping runs for
  provenance/capture/readback/pixel probes/texture diffs/export, slow frames,
  explicit requests, or full-every-frame policy.

## Per-surface presentation policy

`RenderSurfaceRegistryResource` stores a declarative
`RenderSurfacePresentationRequest` for each `RenderSurfaceId`. Native windows
default to `GpuSurfacePresentMode::Fifo` with a desired maximum frame-latency
hint of 2. RunenGPU defines `Fifo`, `FifoRelaxed`, `Immediate`, and `Mailbox`.
Requests are strict: unsupported modes are rejected without implicit fallback.

For an existing or reserved Render surface, before or after native attachment:

```rust
let request = RenderSurfacePresentationRequest::new(
    GpuSurfacePresentMode::Mailbox, 1,
)?;
let revision = registry.set_presentation_request(surface_id, request)?;
```

The revision records intent, not physical application. The existing RenderSubmit
path reconciles requests before surface acquisition; it does not trigger Host redraws.

`Gfx::surface_presentation(record)` returns a
`RenderSurfacePresentationSnapshot` with requested, attachment-supported,
selected and applied settings, the last attempt, and a
`Pending | Applied | Rejected | Unavailable` disposition. A rejected new request
may leave a previously applied configuration valid. The latency field remains
a requested backend hint, not an observed effective frame latency.

Supported modes are attachment-time facts, not live monitor capabilities.
Physical loss or failed configuration invalidates cached facts and requires
safe reattachment/requalification before claiming application; an unsupported
mode request alone does not require recreation. Non-drawable surfaces defer
configuration. Retired logical surfaces reject further policy updates.

Host redraw pacing is independent of Render Present mode. Present submission,
GPU completion, and physical display scanout are distinct; no selection
guarantees FPS, tearing behavior, or input-to-photon latency.

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
