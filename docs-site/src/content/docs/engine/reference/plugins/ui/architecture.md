---
title: "UI Domain Surface Architecture"
description: "Documentation for UI Domain Surface Architecture."
---

# UI Domain Surface Architecture

## Ownership Boundary

- Owns: UI data contracts and domain helpers.
- Does not own: Render scheduling and execution.

## Module Layout

- Primary module: engine/src/plugins/ui/mod.rs
- Entry surface: module surface only; no standalone Plugin implementation
- Runtime schedule touchpoints: No direct schedule registration

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
