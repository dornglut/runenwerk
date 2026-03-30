---
title: Roadmap Overview
description: Overview of the engine-agnostic domain roadmaps.
---

## ECS Feature Roadmap / Prioritization


| Priority     | Domain                | Feature / Concept                                           | Impact / Value                                                | Complexity / Notes                                          |
| ------------ | --------------------- | ----------------------------------------------------------- | ------------------------------------------------------------- | ----------------------------------------------------------- |
| **1 (High)** | Events / Reactivity   | `#[derive(Event)]`                                          | Standardizes ECS events, enables send/read pattern            | Medium – requires event registration system                 |
|              | Events / Reactivity   | `Reactive<T>`                                               | Auto `Changed<T>` notifications for systems & editor bindings | Medium – integrates with change-tracking                    |
|              | Editor / UI           | `InspectableComponent`                                      | Auto-bind ECS state to editor UI panels                       | Medium – needs reflection / editor API                      |
|              | Queries               | `QueryWithHistory<T>`                                       | Undo/redo, editor history                                     | Medium – relies on history storage                          |
|              | Commands / Scheduling | `DeferredCommand<T>`                                        | Safe mutation after queries                                   | Medium – can extend existing commands                       |
| **2 (Mid)**  | Editor / UI           | `TransientComponent`                                        | Ephemeral per-frame objects (gizmos, temporary UI)            | Low-Medium – integrates with commands                       |
|              | Resources             | `ResHistory<T>`                                             | Snapshot / undo-redo support                                  | Medium – similar to `QueryWithHistory`                      |
|              | Indexes               | `MultiIndex`                                                | Map one component to multiple keys                            | Medium-High – improves lookups, needed for editor/world     |
|              | Queries               | `QueryMutOrDefault<T>`                                      | Auto-insert default component if missing                      | Low – ergonomic improvement                                 |
|              | Commands / Scheduling | `BatchCommands`                                             | Atomic application of multiple commands                       | Medium – improves performance & safety                      |
| **3 (Long)** | Editor / UI           | `GizmoBundle`                                               | Handles, manipulators for editor objects                      | Medium-High – editor-specific                               |
|              | Editor / UI           | `UiNode ECS`                                                | ECS-driven UI state                                           | High – requires UI integration                              |
|              | Indexes               | `SpatialIndex`                                              | Grid/octree spatial queries                                   | High – important for world streaming / editor               |
|              | Events / Reactivity   | `EventHistory<T>` / `EventChannel<T>`                       | Multi-consumer, frame history                                 | Medium-High – useful for editor & gameplay signals          |
|              | Commands / Scheduling | `ConditionalCommands`                                       | Apply commands only if query conditions hold                  | Medium – ergonomic improvement                              |
|              | Debug / Telemetry     | `QueryProfiler` / `ComponentUsageTracker` / `ChangeHeatmap` | Deep profiling & visualization                                | Medium-High – optional but valuable for large worlds/editor |
|              | Derives               | `StatefulComponent`                                         | Auto-change tracking per component                            | Medium – enables editor reactivity                          |
|              | Derives               | `NetworkSync`                                               | Component-level replication / serialization                   | High – for multiplayer integration                          |
|              | Derives               | `BundlePartial`                                             | Optional bundles                                              | Low-Medium – ergonomics                                     |
