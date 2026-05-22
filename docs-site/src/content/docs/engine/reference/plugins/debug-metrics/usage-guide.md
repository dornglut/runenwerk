---
title: "Debug Metrics Plugin Usage Guide"
description: "Documentation for Debug Metrics Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
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
use engine::plugins::debug_metrics::DebugMetricsPlugin;

app.add_plugin(DebugMetricsPlugin);
```

## Runtime Contract

- Schedule placement: Startup, RenderPrepare
- Ownership: Debug overlay state and draw-list publication.
- Non-ownership: Render submission execution and input transport.
- Runtime timing surface: frame workload, preflight, flow encode, shader poll,
  diagnostics report, frame pacing mode/FPS cap, and preflight cache status are
  read from runtime/render inspection resources and displayed without owning
  render policy or backend handles.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
