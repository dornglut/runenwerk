---
title: "Scene Plugin Usage Guide"
description: "Documentation for Scene Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Scene Plugin Usage Guide

## Purpose

Owns scene lifecycle orchestration and runtime scene state publication.

## Entry Points

- Module: engine/src/plugins/scene/plugin.rs
- Entry: ScenePlugin
- Local README: engine/src/plugins/scene/README.md

## Minimal Setup

```rust
use engine::plugins::{ScenePlugin, TimePlugin};

app.add_plugin(TimePlugin);
app.add_plugin(ScenePlugin);
```

The current Scene runtime consumes frame timing but does not own it. Applications that already
use `default_plugins()` get `TimePlugin` from that stack and only need to add `ScenePlugin`.

## Runtime Contract

- Schedule placement: Startup, PreUpdate, FixedUpdate, Update
- Timing prerequisite: `Time` supplied by `TimePlugin` (directly or through a plugin stack).
- Ownership: Scene manager lifecycle and runtime publication boundaries.
- Non-ownership: Frame-time progression and render graph submission.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
