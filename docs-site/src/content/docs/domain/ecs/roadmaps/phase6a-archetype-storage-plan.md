---
title: "Phase 6A Archetype Storage Plan"
description: "Plan for ECS archetype-owned storage completion and invariants."
---

# Phase 6A Archetype Storage Plan

Roadmap reference (workspace): [Domain roadmap](../../01-roadmap.md)
Companion full-switch plan: [`phase6-archetype-full-switch-plan.md`](./phase6-archetype-full-switch-plan.md)
Closeout roadmap: [`phase6-closeout-roadmap.md`](./phase6-closeout-roadmap.md)

## Implemented

- Archetype identity/registry keyed by canonical component type sets.
- Entity location tracking (`Entity -> { archetype_id, row }`) with swap-remove repair.
- Dense primitives are implemented (not future-only):
  - `DenseEntityColumn`
  - `DenseColumn<T>` with row metadata (`added_tick`, `changed_tick`)
- Structural membership transitions are archetype-tracked for spawn/insert/remove/despawn.
- Archetype-owned typed columns now store component values for spawned/inserted entities, with row-preserving migration on insert/remove/despawn.
- `Query<&mut T>` fetch is bound to archetype-owned typed columns (no legacy typed-store value fetch in that switched path).
- `Query<(&mut A, &B)>` fetch is bound to archetype-owned typed columns (both tuple sides are archetype-native in the switched path).
- `Query<(&mut A, &mut B)>` fetch is bound to archetype-owned typed columns (both mutable tuple sides are archetype-native in the switched path).
- Transitional typed-store pointer index ownership has been removed; query fetch paths now resolve values from archetype-owned columns.

## Reporting Boundary (Resolved)

- Archetype row metadata is canonical for query/filter semantics:
  - `Changed<T>`
  - `Added<T>`
- Dead per-entity tick maps for changed/added bookkeeping are removed.
- Reporting and introspection remain explicit type-level APIs backed by ticks/logs:
  - `World::component_changed_since`
  - `World::resource_changed_since`
  - `World::component_changes_since`
  - `World::resource_changes_since`

## Final Phase 6 End State

- Archetypes are the canonical storage model.
- Component values are owned by archetype dense typed columns.
- Dominant mutable query execution is archetype-row-native:
  - `Query<&mut T>`
  - `Query<(&mut A, &B)>`
  - `Query<(&mut A, &mut B)>`
- Other supported query shapes remain valid and execute through the entity-list fallback matcher.
- Structural mutation (spawn/insert/remove/despawn + command flush application) mutates archetype-owned storage directly.
- Legacy typed-store ownership and legacy hot-path query fetch are removed.

## Required Invariants

1. ECS semantics remain exact:
   - `Changed<T>`
   - `Added<T>`
   - remove/reinsert behavior
   - command flush timing
   - query correctness
   - reused `QueryState` safety
   - mutable alias safety for tuple queries
2. Swap-remove repair must keep entity location maps correct.
3. Row metadata must remain aligned with value rows through migration/removal.

## Exit Criteria For This Plan

- No permanent legacy fallback for storage ownership or hot-path query fetch.
- Transitional bridge code is explicitly marked and deleted once replaced.
- Phase 6 docs and closeout reports remain aligned with `phase6-closeout-roadmap.md`.
