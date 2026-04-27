---
title: "Grid Plugin Architecture"
description: "Documentation for Grid Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Grid Plugin Architecture

## Ownership Boundary

- Owns: Grid runtime config publication.
- Does not own: Simulation authority and render pass execution.

## Module Layout

- Primary module: engine/src/plugins/grid/mod.rs
- Entry surface: GridPlugin
- Runtime schedule touchpoints: Update (CoreSet::Scene)

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
