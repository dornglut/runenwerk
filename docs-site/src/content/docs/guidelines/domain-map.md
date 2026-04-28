---
title: Domain Map
description: Domain Map
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# Domain Map

This map tracks crate ownership and allowed dependency direction for the active workspace.

## Domain Layer

- `foundation/id`: shared typed-id primitives and allocation contracts
- `domain/ecs`: entity/component/resource storage, reflection, and typed world/query APIs
- `domain/scheduler`: schedule/stage/runtime execution graph utilities
- `domain/scene`: scene-domain data contracts
- `domain/editor/*`: editor-facing domain logic (inspector, scene editing, viewport)
- `domain/id_macros`: derive/proc-macro support for id newtypes used across domains

## Engine Layer

- `engine` (`engine`): `engine::App`, plugin composition, scene/render/input/time/UI runtime integration

Primary plugin modules live under:

- `engine/src/plugins/scene`
- `engine/src/plugins/render`
- `engine/src/plugins/input`
- `engine/src/plugins/time`
- `domain/ui/*` with engine scene/render integration
- `engine/src/plugins/grid`
- `engine/src/plugins/shared`
- `engine/src/plugins/debug_metrics`
- `engine/src/plugins/scheduler_diagnostics`

## Networking Layer

- `net/engine_net` (`engine_net`): protocol/session/replication contracts
- `net/engine_net_quic` (`engine_net_quic`): QUIC transport/runtime adapter
- `net/engine_net_macros` (`engine_net_macros`): proc-macro support for replication/network derives

## App and Adapter Layer

- `apps/*`: runnable binaries and tools (for example `apps/runenwerk_editor`)
- `adapters/*`: integration bridges for external hosts (for example `adapters/godot_chunking_addon`)

## Non-Crate Supporting Domains

- `assets/`: authoring/runtime data assets
- `docs-site/`: documentation source and navigation pages

## Dependency Rules

Preferred dependency direction:

- `foundation` <- `domain`
- `foundation` <- `engine`
- `foundation` <- `net`
- `foundation` <- `apps`
- `foundation` <- `adapters`
- `domain` <- `engine`
- `domain` <- `net`
- `domain` <- `apps`
- `engine` <- `apps`
- `net` <- `apps`
- `domain` <- `adapters`

Disallowed direction:

- `domain` depending on `engine`, `net`, `apps`, or `adapters`
- `engine` depending on `apps`
- private cross-app dependencies (`apps/*` directly depending on another `apps/*`)
