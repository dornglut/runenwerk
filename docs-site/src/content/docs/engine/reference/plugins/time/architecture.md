---
title: "Time Plugin Architecture"
description: "Documentation for Time Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Time Plugin Architecture

## Ownership Boundary

- Owns: default `Time` resource installation and frame time progression.
- Does not own: fixed-step catchup loop semantics or simulation tick identity.

Bare `App` construction does not manufacture `Time`. `TimePlugin` initializes the resource
through `App::init_resource`, so an explicitly pre-inserted `Time` value remains authoritative
and is not overwritten during plugin composition.

## Module Layout

- Primary module: engine/src/plugins/time/mod.rs
- Entry surface: TimePlugin
- Owned runtime resource: `Time`
- Runtime schedule touchpoints: PreUpdate (CoreSet::Time)

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Consumers that require frame timing must select `TimePlugin` directly or through a plugin stack that contains it.
- Fixed-step execution may operate without `Time` according to its own fallback contract; that does not transfer timing ownership to App or fixed-step code.
- Cross-plugin coupling stays data-oriented through typed resource/event/state boundaries.
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
