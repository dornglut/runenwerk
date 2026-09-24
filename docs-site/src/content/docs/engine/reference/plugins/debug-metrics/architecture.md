---
title: "Debug Metrics Plugin Architecture"
description: "Documentation for Debug Metrics Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Debug Metrics Plugin Architecture

## Ownership Boundary

- Owns: Debug overlay state plus producer-local diagnostics frame generation/publication.
- Consumes: `Time`, `RenderReadinessState`, `SceneRuntimeState`, `SceneOverlayViewportState`, and render/runtime inspection state.
- Does not own: frame-time progression, render readiness, scene runtime/viewport state, render submission execution, or input transport.

## Module Layout

- Primary module: engine/src/plugins/debug_metrics/mod.rs
- Entry surface: DebugMetricsPlugin
- Timing provider: `TimePlugin` directly or through a plugin stack that contains it
- Scene-state and viewport provider: `ScenePlugin`
- Render-readiness provider: `RenderPlugin`
- Runtime schedule touchpoints: Startup, RenderPrepare

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- `TimePlugin` owns default `Time` installation and progression; DebugMetrics remains a consumer.
- `ScenePlugin` owns default `SceneRuntimeState` and `SceneOverlayViewportState` installation; DebugMetrics consumes scene diagnostics and viewport facts without manufacturing Scene state.
- `RenderPlugin` owns default `RenderReadinessState` installation and Render submit advances it from warm-frame evidence; DebugMetrics consumes readiness diagnostics without manufacturing or advancing that state.
- DebugMetrics generates its `UiFrame` locally and publishes producer id 2 directly through `SurfaceFrameSubmissionRegistryResource`; no shared Scene viewport resource stores the diagnostics frame.
- `RenderReadinessState` is Render capability state, not the Runenwerk App `Startup` lifecycle.
- Cross-plugin coupling stays data-oriented through typed resource/event/state boundaries.
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
