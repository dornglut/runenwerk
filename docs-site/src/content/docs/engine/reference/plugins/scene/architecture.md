---
title: "Scene Plugin Architecture"
description: "Documentation for Scene Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Scene Plugin Architecture

## Ownership Boundary

- Owns: Scene manager lifecycle and runtime publication boundaries.
- Consumes: `Time` plus current App/runtime input/window state used by scene transitions.
- Does not own: frame-time progression or render graph submission.

## Module Layout

- Primary module: engine/src/plugins/scene/plugin.rs
- Entry surface: ScenePlugin
- Timing provider: `TimePlugin` directly or through a plugin stack that contains it
- Runtime schedule touchpoints: Startup, PreUpdate, FixedUpdate, Update

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- `TimePlugin` owns default `Time` installation and progression; Scene remains a consumer.
- Cross-plugin coupling stays data-oriented through typed resource/event/state boundaries.
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
