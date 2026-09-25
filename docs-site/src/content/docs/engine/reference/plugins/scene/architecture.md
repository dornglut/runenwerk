---
title: "Scene Plugin Architecture"
description: "Documentation for Scene Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
publication: primary
---

# Scene Plugin Architecture

## Ownership Boundary

- Owns: Scene registration/catalog state, scene manager lifecycle, and runtime publication boundaries.
- Consumes: `Time` plus current App/runtime input/window state used by scene transitions.
- Does not own: frame-time progression or render graph submission.

`SceneCatalog` is owner capability state, not universal App state. `ScenePlugin` supplies an empty
catalog when no explicit Scene composition has created one. `AppSceneExt::add_scene` and
`AppSceneExt::add_scene_template` are composition conveniences over that same owner resource and may
populate it before plugin installation; the plugin's non-overwriting initialization preserves
those registrations.

## Module Layout

- Primary module: engine/src/plugins/scene/plugin.rs
- Entry surface: ScenePlugin
- Scene composition surface: `AppSceneExt`
- Timing provider: `TimePlugin` directly or through a plugin stack that contains it
- Runtime schedule touchpoints: Startup, PreUpdate, FixedUpdate, Update

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- `TimePlugin` owns default `Time` installation and progression; Scene remains a consumer.
- Bare App construction does not manufacture `SceneCatalog`; selecting or explicitly configuring Scene materializes Scene-owned catalog state.
- Cross-plugin coupling stays data-oriented through typed resource/event/state boundaries.
- Shared `SceneResource` presence is substrate only; explicit private `ScenePlugin` activation gates Scene manager/runtime-control semantics.
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
