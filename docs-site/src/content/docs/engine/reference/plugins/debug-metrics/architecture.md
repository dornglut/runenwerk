---
title: "Debug Metrics Plugin Architecture"
description: "Documentation for Debug Metrics Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Debug Metrics Plugin Architecture

## Ownership Boundary

- Owns: Debug overlay state and draw-list publication.
- Consumes: `Time`, `RenderReadinessState`, `SceneRuntimeState`, and render/runtime inspection state.
- Does not own: frame-time progression, render readiness, scene runtime state, render submission execution, or input transport.

## Module Layout

- Primary module: engine/src/plugins/debug_metrics/mod.rs
- Entry surface: DebugMetricsPlugin
- Timing provider: `TimePlugin` directly or through a plugin stack that contains it
- Scene-state provider: `ScenePlugin`
- Render-readiness provider: `RenderPlugin`
- Runtime schedule touchpoints: Startup, RenderPrepare

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- `TimePlugin` owns default `Time` installation and progression; DebugMetrics remains a consumer.
- `ScenePlugin` owns default `SceneRuntimeState` installation; DebugMetrics consumes scene diagnostics without manufacturing scene state.
- `RenderPlugin` owns default `RenderReadinessState` installation and Render submit advances it from warm-frame evidence; DebugMetrics consumes readiness diagnostics without manufacturing or advancing that state.
- `RenderReadinessState` is Render capability state, not the Runenwerk App `Startup` lifecycle.
- Cross-plugin coupling stays data-oriented through typed resource/event/state boundaries.
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
