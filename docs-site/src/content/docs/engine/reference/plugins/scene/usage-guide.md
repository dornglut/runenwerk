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

Owns scene registration/catalog state, scene lifecycle orchestration, and runtime scene state publication.

## Entry Points

- Module: engine/src/plugins/scene/plugin.rs
- Entry: ScenePlugin
- Scene composition extension: `AppSceneExt` (`add_scene`, `add_scene_template`, `registered_scene_count`)
- Local README: engine/src/plugins/scene/README.md

## Minimal Setup

```rust
use engine::plugins::{ScenePlugin, TimePlugin};

app.add_plugin(TimePlugin);
app.add_plugin(ScenePlugin);
```

The current Scene runtime consumes frame timing but does not own it. Applications that already
use `default_plugins()` get `TimePlugin` from that stack and only need to add `ScenePlugin`.

`ScenePlugin` installs an empty `SceneCatalog` when no scenes were registered explicitly. A bare
`App` does not imply Scene catalog state. Applications may register scenes before installing the
plugin; `AppSceneExt::add_scene` and `AppSceneExt::add_scene_template` create/populate the same Scene-owned catalog,
and later `ScenePlugin` installation preserves those registrations.

Scene runtime controls require explicit `ScenePlugin` activation. Replay/Render may share an empty Scene substrate without activating Scene manager semantics.

## Runtime Contract

- Schedule placement: Startup, PreUpdate, FixedUpdate, Update
- Timing prerequisite: `Time` supplied by `TimePlugin` (directly or through a plugin stack).
- Scene catalog: supplied by `ScenePlugin` when absent or populated explicitly through `App::add_scene*` composition.
- Ownership: Scene registration/catalog state, scene manager lifecycle, and runtime publication boundaries.
- Non-ownership: Frame-time progression and render graph submission.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
