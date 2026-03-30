---
title: Feature Overview
description: Overview of the ecs features.
---


- ✅ Core-supported
- ⚠ Partial / limited support
- ❌ Missing


| Domain                    | Feature / Concept                                 | Current ECS Support | Notes / Gap Analysis                                                  |
| ------------------------- | ------------------------------------------------- | ------------------- | --------------------------------------------------------------------- |
| **Derives**               | `Component`                                       | ✅                   | Fully supported                                                       |
|                           | `Resource`                                        | ✅                   | Fully supported                                                       |
|                           | `Event`                                           | ❌                   | Not yet ECS-native; only ad-hoc event APIs exist                      |
|                           | `Flag` / zero-sized markers                       | ⚠                   | Can use empty `Component`s but no dedicated derive                    |
|                           | `StatefulComponent`                               | ❌                   | Automatic change tracking per component would help queries/reactivity |
|                           | `Inspectable`                                     | ❌                   | Useful for editor/UI panels                                           |
|                           | `NetworkSync`                                     | ❌                   | Component-level replication / serialization missing                   |
|                           | `BundlePartial`                                   | ❌                   | Optional component bundles missing                                    |
| **Queries**               | `Query<&T>` / `Query<&mut T>` / tuples            | ✅                   | Core queries supported                                                |
|                           | Filters: `With`, `Without`, `Added`, `Changed`    | ✅                   | Fully supported                                                       |
|                           | Detached / reusable `QueryState`                  | ✅                   | Supported                                                             |
|                           | `QueryMutOrDefault<T>`                            | ❌                   | Auto-insert default missing                                           |
|                           | `QueryOnce<T>`                                    | ❌                   | One-off queries missing                                               |
|                           | `QueryWithHistory<T>`                             | ❌                   | Useful for undo/redo or editor history                                |
|                           | `QueryOrphaned<T>`                                | ❌                   | Track removed components for transient operations                     |
|                           | `QueryIndexable<T>`                               | ❌                   | Combine query + secondary index for fast lookups                      |
| **Events / Reactivity**   | `emit_event`, `read_events`, `drain_events`       | ✅                   | Basic per-frame events exist                                          |
|                           | `#[derive(Event)]`                                | ❌                   | Type-level ECS event registration missing                             |
|                           | `Reactive<T>`                                     | ❌                   | Auto `Changed<T>` notifications across systems missing                |
|                           | `EventChannel<T>`                                 | ❌                   | Multi-consumer, per-system cursors missing                            |
|                           | `EventFilter<T>`                                  | ❌                   | Predicate-based event filtering missing                               |
|                           | `EventHistory<T>`                                 | ❌                   | Store N previous frames missing                                       |
| **Resources**             | `Resource` insert / read / remove / mut           | ✅                   | Core supported                                                        |
|                           | `ResHistory<T>`                                   | ❌                   | Historical snapshots for undo/redo                                    |
|                           | `ResView<T>`                                      | ❌                   | Read-only snapshot for safe access                                    |
|                           | `LazyResource<T>`                                 | ❌                   | Deferred initialization missing                                       |
|                           | `GlobalEventQueue<T>`                             | ❌                   | Cross-system event queue missing                                      |
| **Commands / Scheduling** | `commands().insert/spawn/apply`                   | ✅                   | Core supported                                                        |
|                           | `ScheduleLabel`                                   | ✅                   | Supported                                                             |
|                           | `DeferredCommand<T>`                              | ❌                   | Safe mutation after queries missing                                   |
|                           | `BatchCommands`                                   | ❌                   | Atomic batch application missing                                      |
|                           | `ConditionalCommands`                             | ❌                   | Apply only if query condition holds missing                           |
|                           | `ScheduleParam`                                   | ❌                   | Ergonomic scheduling with dependencies missing                        |
| **Editor / UI**           | `InspectableComponent`                            | ❌                   | Auto-bind to editor panels missing                                    |
|                           | `GizmoBundle`                                     | ❌                   | Editor handles / manipulators missing                                 |
|                           | `UiNode ECS`                                      | ❌                   | UI state in ECS missing                                               |
|                           | `UndoRedo`                                        | ❌                   | Requires `ResHistory` / `QueryWithHistory`                            |
|                           | `TransientComponent`                              | ❌                   | Auto-remove per frame for ephemeral editor objects                    |
| **Indexes**               | `ensure_component_index` / `find_entity_by_index` | ⚠                   | Single-index supported                                                |
|                           | `MultiIndex`                                      | ❌                   | Map one component to multiple keys missing                            |
|                           | `IndexChain`                                      | ❌                   | Chained indexes for complex lookups missing                           |
|                           | `SpatialIndex`                                    | ❌                   | Grid / octree for editor/world queries missing                        |
| **Debug / Telemetry**     | Telemetry APIs `reset()`, `snapshot()`            | ✅                   | Feature-gated profiling exists                                        |
|                           | `QueryProfiler`                                   | ❌                   | Per-query cost tracking missing                                       |
|                           | `ComponentUsageTracker`                           | ❌                   | Track hot components missing                                          |
|                           | `EventProfiler`                                   | ❌                   | Track event frequency missing                                         |
|                           | `ChangeHeatmap`                                   | ❌                   | Visualize component change density missing                            |
