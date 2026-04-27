---
title: "Time Plugin Architecture"
description: "Documentation for Time Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Time Plugin Architecture

## Ownership Boundary

- Owns: Frame time progression.
- Does not own: Fixed-step catchup loop semantics.

## Module Layout

- Primary module: engine/src/plugins/time/mod.rs
- Entry surface: TimePlugin
- Runtime schedule touchpoints: PreUpdate (CoreSet::Time)

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
