---
title: "Debug Metrics Plugin"
description: "Documentation for Debug Metrics Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
publication: primary
---

# Debug Metrics Plugin

## Purpose

Provides an on-screen diagnostics overlay for frame timing and runtime state.

## Usage

- Plugin: `DebugMetricsPlugin`
- Timing provider: `TimePlugin` directly or through `default_plugins()`
- Scene-state and viewport provider: `ScenePlugin`
- Render-readiness provider: `RenderPlugin`
- Toggle overlay action: `debug.metrics.toggle`
- Default key: `F10`

The plugin publishes its diagnostics frame directly through
`SurfaceFrameSubmissionRegistryResource` when enabled. It consumes `Time` for frame-delta
diagnostics, `SceneRuntimeState` for scene diagnostics, `SceneOverlayViewportState` for the active
overlay viewport size/scale, and `RenderReadinessState` for render-readiness diagnostics. It does
not install or advance those owner states. `default_plugins()` supplies `TimePlugin`; applications
using the maintained diagnostics surface must also select `ScenePlugin` and `RenderPlugin`.

DebugMetrics does not store its frame in shared Scene viewport state. Frame generation is
producer-local; producer id 2 is published directly to the generic surface-frame submission
registry.

## Ownership Boundaries

- Owns debug metrics visibility toggle and diagnostics frame content/publication.
- Consumes `Time`, `RenderReadinessState`, `SceneRuntimeState`, `SceneOverlayViewportState`, and render/runtime inspection state.
- Does not own frame-time progression, render readiness, scene runtime/viewport state, frame submission execution, or UI extraction orchestration.
- `RenderReadinessState` is Render capability state derived from render warm-frame evidence; it is distinct from the Runenwerk App `Startup` lifecycle.

## Extension Points

- Add additional diagnostic lines/sections in `debug_metrics_overlay_system`.
- Rebind the toggle action through input binding changes.

## Guides

- Usage: [../../../docs/reference/plugins/debug-metrics/usage-guide.md](../../reference/plugins/debug-metrics/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/debug-metrics/advanced-guide.md](../../reference/plugins/debug-metrics/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/debug-metrics/architecture.md](../../reference/plugins/debug-metrics/architecture.md)
