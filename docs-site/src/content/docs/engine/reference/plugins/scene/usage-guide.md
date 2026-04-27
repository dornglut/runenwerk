---
title: "Scene Plugin Usage Guide"
description: "Documentation for Scene Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
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
use engine::plugins::scene::ScenePlugin;

app.add_plugin(ScenePlugin);
```

## Runtime Contract

- Schedule placement: Startup, PreUpdate, FixedUpdate, Update
- Ownership: Scene manager lifecycle and runtime publication boundaries.
- Non-ownership: Render graph submission.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
