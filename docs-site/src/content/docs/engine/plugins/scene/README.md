---
title: "Scene Plugin"
description: "Documentation for Scene Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Scene Plugin

## Purpose

Coordinates world/overlay scene lifecycle, authoritative scene simulation, and scene replay/snapshot boundaries.

## Usage

- Plugin: `ScenePlugin`
- Schedules:
  - `Startup`: initialize the scene manager
  - `PreUpdate`: process transition commands and input-driven scene state
  - `FixedUpdate`: run authoritative world-scene simulation
  - `Update`: apply world-to-overlay message flow and republish scene state

The plugin owns the runtime scene manager and republishes transport-neutral scene state through:

- `SceneRuntimeState`
- `GameplayRuntimeConfig`
- `UiOverlayState`

It also defines the current authoritative scene replay/snapshot DTOs:

- `SceneSimulationSnapshotV1`
- `SceneSimulationDeltaV1`
- `SceneReplayCommandFrame`
- `SceneReplayArchive`

## Ownership Boundaries

- Owns scene transition orchestration and scene lifecycle event flow.
- Owns world scene runtime updates and overlay/world interaction state.
- Owns the authoritative scene snapshot/restore boundary used by replay and replication.
- Owns applying compiled scene/template authoring outputs to runtime state.
- Does not own render graph execution or input device event collection.

## Extension Points

- Register new scene labels/aliases and transition commands.
- Add new world-to-overlay message types and formatting paths.
- Extend the authoritative snapshot boundary as real gameplay state grows.
- Add scene authoring schemas/compilers under scene-owned authoring modules.

## Guides

- Usage: [../../../docs/reference/plugins/scene/usage-guide.md](../../reference/plugins/scene/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/scene/advanced-guide.md](../../reference/plugins/scene/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/scene/architecture.md](../../reference/plugins/scene/architecture.md)

