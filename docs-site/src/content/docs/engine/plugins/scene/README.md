---
title: "Scene Plugin"
description: "Documentation for Scene Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Scene Plugin

## Purpose

Coordinates scene registration, world/overlay scene lifecycle, authoritative scene simulation, and scene replay/snapshot boundaries.

## Usage

- Plugin: `ScenePlugin`
- App composition extension: `AppSceneExt`
- Timing provider: `TimePlugin` directly or through `default_plugins()`
- Scene catalog provider: `ScenePlugin`, or explicit `App::add_scene*` composition before plugin installation
- Schedules:
  - `Startup`: initialize the scene manager
  - `PreUpdate`: process transition commands and input-driven scene state
  - `FixedUpdate`: run authoritative world-scene simulation
  - `Update`: apply world-to-overlay message flow and republish scene state

The plugin consumes `Time` during scene transition/runtime processing but does not install or
advance frame timing state.

`SceneCatalog` is Scene-owned composition/runtime input. A bare `App` does not contain a scene
catalog. `ScenePlugin` installs an empty catalog when none exists, while `AppSceneExt::add_scene` and
`AppSceneExt::add_scene_template` are explicit composition conveniences that may create and populate the
same catalog before the plugin is selected. Plugin installation preserves those registrations.

Scene runtime-control admission uses explicit `ScenePlugin` activation; shared `SceneResource` presence from Replay or Render does not activate Scene semantics.

The plugin owns the runtime scene manager and republishes transport-neutral Scene observation
state through:

- `SceneRuntimeState`
- `SceneOverlayViewportState`

`SceneOverlayViewportState` is the Scene-owned observation of the active overlay viewport. It
contains only the overlay `screen_size` and UI `scale` derived from the Scene runtime. `ScenePlugin`
provides it; a bare `App` does not imply Scene viewport state. Consumers such as DebugMetrics may
read this projection without owning or mutating Scene internals.

Gameplay configuration remains Scene-owned runtime context and is captured/restored through the
Scene snapshot/replay boundary; it is not duplicated as a separate public runtime resource.

It also defines the current authoritative scene replay/snapshot DTOs:

- `SceneSimulationSnapshotV1`
- `SceneSimulationDeltaV1`
- `SceneReplayCommandFrame`
- `SceneReplayArchive`

## Ownership Boundaries

- Owns scene registration/catalog state, transition orchestration, and scene lifecycle event flow.
- Owns world scene runtime updates and overlay/world interaction state.
- Owns `SceneOverlayViewportState` publication as transport-neutral Scene viewport observation.
- Owns the authoritative scene snapshot/restore boundary used by replay and replication.
- Owns applying compiled scene/template authoring outputs to runtime state.
- Consumes `Time` supplied by `TimePlugin`.
- Does not own frame-time progression, render graph execution, or input device event collection.

## Extension Points

- Register new scene labels/aliases and transition commands.
- Add new world-to-overlay message types and formatting paths.
- Extend the authoritative snapshot boundary as real gameplay state grows.
- Add scene authoring schemas/compilers under scene-owned authoring modules.

## Guides

- Usage: [../../../docs/reference/plugins/scene/usage-guide.md](../../reference/plugins/scene/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/scene/advanced-guide.md](../../reference/plugins/scene/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/scene/architecture.md](../../reference/plugins/scene/architecture.md)
