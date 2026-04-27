---
title: "Grid Plugin Usage Guide"
description: "Documentation for Grid Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Grid Plugin Usage Guide

## Purpose

Projects gameplay grid settings into runtime grid render config.

## Entry Points

- Module: engine/src/plugins/grid/mod.rs
- Entry: GridPlugin
- Local README: engine/src/plugins/grid/README.md

## Minimal Setup

```rust
use engine::plugins::grid::GridPlugin;

app.add_plugin(GridPlugin);
```

## Runtime Contract

- Schedule placement: Update (CoreSet::Scene)
- Ownership: Grid runtime config publication.
- Non-ownership: Simulation authority and render pass execution.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
