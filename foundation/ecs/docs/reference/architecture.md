# ECS Architecture

This document covers `foundation/ecs` internals for runtime execution, scheduling, safety
invariants, and deterministic behavior.

## 1. Runtime Execution Model

`ecs::Runtime` executes function systems using system params:

- `Query<Q, F = ()>`
- `Res<T>`
- `ResMut<T>`
- `Commands`
- `EventReader<T>`
- `EventWriter<T>`

Function system tuple arity is supported up to 8 parameters.

Each registered system owns cached param state. State is initialized once at registration time and
reused on each run.

## 2. Access and Scheduling Model

System param metadata is mapped into scheduler access keys:

- component read/write
- resource read/write
- structural deferred mutation (`Commands`)
- event read/write (modeled as resource-like by event type)

The scheduler computes stage plans from:

- explicit set dependencies (`in_set`, `before`, `after`)
- strict access conflicts for component/resource/event domains (read/write and write/write)
- explicit structural-mutation metadata for deferred command producers

Policy for structural mutation:

- multiple deferred structural mutation producers can coexist in one stage
- deterministic command queue merge happens in system execution order
- deferred structural mutations are visible only after the stage-end flush boundary
- component/resource/event read/write conflicts still force stage separation

## 3. Deterministic Deferred Commands

Every system run gets a per-system `Commands` queue. Runtime execution order is deterministic:

1. systems execute in stage order
2. within each stage, command queues are collected in system execution order
3. queues are applied at stage end in that same order

This enforces the rule: systems inside the same stage do not observe each other's deferred
structural changes before the stage flush boundary.

## 4. Query Internals

`QueryState<Q, F>` stores:

- required-present component requirements (`Q` + `With<T>`)
- excluded component filters (`Without<T>`)
- access metadata
- per-query `last_run_tick`
- archetype-row scratch + entity-list scratch
- optional fast-fetch cache (`QueryFastCache`)

Execution paths:

- Archetype-row path (dominant mutable shapes):
  - `Query<&mut T>`
  - `Query<(&mut A, &B)>`
  - `Query<(&mut A, &mut B)>`
  - query state asks the archetype registry for matching bindings and iterates rows directly.
- Entity-list fallback path (all other supported shapes):
  - query state asks the archetype registry for matching entities and then fetches via `QueryData`.

`Changed<T>` and `Added<T>` filters compare against the query state's last seen tick and compose
with `With<T>` / `Without<T>` via tuple filters.

Change-aware filter path:

- `Changed<T>` checks `changed_tick` from archetype row metadata.
- `Added<T>` checks `added_tick` from archetype row metadata.
- Query/filter semantics do not depend on global change logs.

`QueryData` remains an internal query implementation trait and is not re-exported from the public
query module.

## 5. Reporting and Introspection Boundary

Type-level change reporting is intentionally separate from query/filter semantics:

- Query/filter semantics:
  - archetype row metadata (`added_tick`, `changed_tick`)
  - APIs: `Changed<T>`, `Added<T>`
- Reporting/introspection history:
  - `component_change_ticks`, `resource_change_ticks`
  - `component_change_log`, `resource_change_log`
  - APIs: `component_changed_since`, `resource_changed_since`,
    `component_changes_since`, `resource_changes_since`

## 6. Event Model

Phase-2 event semantics:

- `EventWriter<T>::send(event)` appends to the world channel
- `EventReader<T>::iter()` reads currently visible events without draining
- draining remains an explicit world API (`World::drain_events<T>()`)

## 7. Secondary Index Model

Secondary indexes are lazily rebuilt and marked dirty on component changes.

Read helpers (`find_entity_by_index*`, `find_entities_by_index*`, `find_component_by_index*`) are
`&self` APIs. Internal mutation for lazy rebuild is hidden behind interior mutability.

## 8. Unsafe Invariants

Unsafe sites are concentrated in query and system param extraction paths:

- `query/query_data_impls.rs`: typed fetches from erased component stores
- `query/traits_and_state.rs`: world-pointer based query iteration
- `system/runtime.rs`: cached-state lifetime bridge for `SystemParam`
- `system/params.rs`: world pointer extraction for runtime params
- `world/handles_and_commands.rs`: borrowed command queue forwarding

Required invariants:

- `SystemParam::State` is lifetime-independent for all extraction lifetimes
- world pointers passed into extraction are valid for the full extraction call
- mutable query forms never alias the same component type mutably
- borrowed command owners always point to a live owning `Commands` value

Unsafe blocks in these files include inline invariant comments and are covered by focused runtime
and query tests.

## 9. Phase 6 Telemetry

Feature-gated telemetry (`--features telemetry`) provides hot-path cost attribution:

- query matching/iteration/get/single timing
- changed/added filter check timing
- runtime plan lookup, per-stage execution, and stage-end flush timing
- scheduler plan build timing, conflict check counts, and stage counts
- event reader/writer call and volume counters

This enables separating query cost, filter cost, scheduler planning cost, and flush cost for
before/after comparisons without changing runtime semantics.
