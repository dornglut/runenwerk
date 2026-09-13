---
title: "Debug Metrics Plugin Usage Guide"
description: "Documentation for Debug Metrics Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
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
use engine::plugins::{DebugMetricsPlugin, ScenePlugin, TimePlugin};

app.add_plugin(TimePlugin);
app.add_plugin(ScenePlugin);
app.add_plugin(DebugMetricsPlugin);
```

`DebugMetricsPlugin` consumes frame timing and scene runtime state for its diagnostics but owns
neither. Applications that already use `default_plugins()` get `TimePlugin` from that stack,
but `default_plugins()` does not include `ScenePlugin`; select `ScenePlugin` explicitly when
using the overlay's scene metrics.

## Runtime Contract

- Schedule placement: Startup, RenderPrepare
- Timing prerequisite: `Time` supplied by `TimePlugin` (directly or through a plugin stack).
- Scene prerequisite: `SceneRuntimeState` supplied by `ScenePlugin`.
- Ownership: Debug overlay state and draw-list publication.
- Non-ownership: Frame-time progression, scene runtime state, render submission execution, and input transport.
- Runtime timing surface: frame workload, preflight, flow encode, shader poll,
  diagnostics report, frame pacing mode/FPS cap, and preflight cache status are
  read from runtime/render inspection resources and displayed without owning
  render policy or backend handles.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
