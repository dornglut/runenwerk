---
title: "Fixed Step Plugin Architecture"
description: "Documentation for Fixed Step Plugin Architecture."
---

# Fixed Step Plugin Architecture

## Ownership Boundary

- Owns: Fixed-step resource installation contract.
- Does not own: Fixed-step loop execution logic.

## Module Layout

- Primary module: engine/src/plugins/fixed_step.rs
- Entry surface: FixedStepPlugin
- Runtime schedule touchpoints: Resource-only (no systems)

## Runtime Coupling

- Depends on engine runtime schedules and resources through typed system params.
- Should keep cross-plugin coupling data-oriented (resource/event/state boundaries).
- Architecture changes should stay narrow and avoid broad app or plugin redesign.
