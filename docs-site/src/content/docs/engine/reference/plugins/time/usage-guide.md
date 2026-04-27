---
title: "Time Plugin Usage Guide"
description: "Documentation for Time Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Time Plugin Usage Guide

## Purpose

Advances frame time state for runtime consumers.

## Entry Points

- Module: engine/src/plugins/time/mod.rs
- Entry: TimePlugin
- Local README: engine/src/plugins/time/README.md

## Minimal Setup

```rust
use engine::plugins::time::TimePlugin;

app.add_plugin(TimePlugin);
```

## Runtime Contract

- Schedule placement: PreUpdate (CoreSet::Time)
- Ownership: Frame time progression.
- Non-ownership: Fixed-step catchup loop semantics.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
