---
title: "Time Plugin Usage Guide"
description: "Documentation for Time Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Time Plugin Usage Guide

## Purpose

Installs and advances frame time state for runtime consumers.

## Entry Points

- Module: engine/src/plugins/time/mod.rs
- Entry: TimePlugin
- Resource: `Time`
- Local README: engine/src/plugins/time/README.md

## Minimal Setup

```rust
use engine::plugins::time::TimePlugin;

app.add_plugin(TimePlugin);
```

`TimePlugin` initializes `Time` if it is absent and preserves a value explicitly inserted before
plugin installation. `default_plugins()` already contains `TimePlugin` for the ordinary engine
stack. Bare `App` construction does not provide `Time` by itself.

## Runtime Contract

- Installation: `TimePlugin` owns default `Time` installation.
- Schedule placement: PreUpdate (CoreSet::Time)
- Ownership: Frame time progression.
- Non-ownership: Fixed-step catchup loop semantics and simulation tick identity.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
