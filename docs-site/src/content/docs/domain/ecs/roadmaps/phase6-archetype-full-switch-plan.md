---
title: "Phase 6 Archetype Full Switch Plan"
description: "Detailed plan for completing the ECS archetype full switch."
---

# Phase 6 Archetype Full Switch Plan

Roadmap reference (workspace): [Domain roadmap](../../01-roadmap.md)
Phase 6A companion: [`phase6a-archetype-storage-plan.md`](./phase6a-archetype-storage-plan.md)
Closeout roadmap: [`phase6-closeout-roadmap.md`](./phase6-closeout-roadmap.md)

## Implemented

- Archetype registry and canonical archetype keys exist.
- Entity location tracking exists and is updated on structural membership transitions.
- Dense column primitives exist (`DenseEntityColumn`, `DenseColumn<T>`).
- Archetype-row-directed execution scaffolding exists in query state/runtime.
- Archetype-owned typed columns now own component values in world mutation paths (spawn/insert/remove/despawn).
- `Query<&mut T>` now fetches directly from archetype-owned typed columns.
- `Query<(&mut A, &B)>` now fetches directly from archetype-owned typed columns.
- `Query<(&mut A, &mut B)>` now fetches directly from archetype-owned typed columns.
- Transitional typed-store pointer index and query fast-cache store fields have been removed.

## Reporting Boundary (Resolved)

- `Changed<T>` and `Added<T>` filter evaluation is driven by archetype row metadata ticks.
- Dead per-entity changed/added tick maps are removed.
- Reporting/introspection APIs remain type-level tick/log APIs:
  - `World::component_changed_since`
  - `World::resource_changed_since`
  - `World::component_changes_since`
  - `World::resource_changes_since`

## Final End State (Non-Negotiable)

- Archetype-owned dense typed columns are the only component value source.
- Dominant query forms run archetype-native:
  - `Query<&mut T>`
  - `Query<(&mut A, &B)>`
  - `Query<(&mut A, &mut B)>`
- Filter/tracking hot paths run archetype-native:
  - `With<T>`
  - `Without<T>`
  - `Changed<T>`
  - `Added<T>`
- Structural mutation is archetype-native for spawn/insert/remove/despawn and command flush apply.
- Legacy typed-store ownership and legacy hot-path query fetch are removed.
- Non-dominant query shapes remain supported through the entity-list fallback matcher.

## Migration Sequence

1. Keep docs current with explicit `Implemented` / `Transitional` / `Final`.
2. Cut query matching fully to archetype-native constraints (required + excluded).
3. Move component value ownership into archetype typed columns.
4. Cut dominant query fetch paths to archetype-native typed bindings.
5. Cut filter/tracking fetch/evaluation paths to archetype-native rows.
6. Remove typed-store ownership and dead query/store fallback code.
7. Validate with `cargo test -p ecs` and Phase 6 telemetry benches/profile artifacts.

## Validation Gates

- Semantics parity for `Changed<T>`, `Added<T>`, remove/reinsert, command flush timing.
- Reused `QueryState` safety after structural changes.
- Mutable alias safety in tuple mutable queries.
- Storage invariants:
  - row alignment
  - swap-remove moved-row repair
  - metadata alignment
