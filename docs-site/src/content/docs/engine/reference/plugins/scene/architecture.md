---
title: "Scene Plugin Architecture"
description: "Documentation for Scene Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Scene Plugin Architecture

## Ownership Boundary

- Owns: Scene manager lifecycle and runtime publication boundaries.
- Does not own: Render graph submission.

## Module Layout

- Primary module: engine/src/plugins/scene/plugin.rs
- Entry surface: ScenePlugin
- Runtime schedule touchpoints: Startup, PreUpdate, FixedUpdate, Update

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
