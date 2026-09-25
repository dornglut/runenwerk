---
title: "Debug Metrics Plugin Usage Guide"
description: "Documentation for Debug Metrics Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
publication: primary
---

# Debug Metrics Plugin Usage Guide

## Purpose

Renders runtime diagnostics into overlay draw commands and supports visibility toggling.

## Entry Points

- Module: engine/src/plugins/debug_metrics/mod.rs
- Entry: DebugMetricsPlugin
- Local README: engine/src/plugins/debug_metrics/README.md

## Minimal Setup

```rust
use engine::plugins::{DebugMetricsPlugin, RenderPlugin, ScenePlugin, TimePlugin};

app.add_plugin(TimePlugin);
app.add_plugin(ScenePlugin);
app.add_plugin(RenderPlugin);
app.add_plugin(DebugMetricsPlugin);
```

`DebugMetricsPlugin` consumes frame timing, scene runtime state, and Render-owned readiness state for
its diagnostics but owns none of them. Applications that already use `default_plugins()` get
`TimePlugin` from that stack, but `default_plugins()` does not include `ScenePlugin` or
`RenderPlugin`; select those capabilities explicitly when using the maintained overlay diagnostics.

## Runtime Contract

- Schedule placement: Startup, RenderPrepare
- Timing prerequisite: `Time` supplied by `TimePlugin` (directly or through a plugin stack).
- Scene prerequisite: `SceneRuntimeState` supplied by `ScenePlugin`.
- Render-readiness prerequisite: `RenderReadinessState` supplied by `RenderPlugin`.
- Ownership: Debug overlay state and draw-list publication.
- Non-ownership: Frame-time progression, render readiness, scene runtime state, render submission execution, and input transport.
- Readiness semantics: DebugMetrics only displays Render readiness; it does not install or advance `RenderReadinessState`, and that state is distinct from the Runenwerk App `Startup` lifecycle.
- Runtime timing surface: frame workload, preflight, flow encode, shader poll,
  diagnostics report, frame pacing mode/FPS cap, and preflight cache status are
  read from runtime/render inspection resources and displayed without owning
  render policy or backend handles.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
