---
title: Domain Map
description: Domain Map
---

# Domain Map

This map tracks crate ownership and allowed dependency direction for the active workspace.

## Foundation Domain

- `foundation/ecs` (`ecs`): entity/component/resource storage and typed world/query APIs
- `foundation/ecs_macros` (`ecs_macros`): proc-macro support for ECS derives
- `foundation/scheduler` (`scheduler`): schedule/stage/runtime execution graph utilities
- `foundation/grid` (`grid`): grid/math/runtime helpers shared by engine and gameplay

## Engine Domain

- `engine` (`engine`): `engine::App`, plugin composition, scene/render/input/time/UI runtime integration

Primary plugin modules live under:

- `engine/src/plugins/scene`
- `engine/src/plugins/render`
- `engine/src/plugins/input`
- `engine/src/plugins/time`
- `engine/src/plugins/ui`
- `engine/src/plugins/grid`
- `engine/src/plugins/shared`
- `engine/src/plugins/debug_metrics`
- `engine/src/plugins/scheduler_diagnostics`

## Networking and Replay Domain

- `net/engine_net` (`engine_net`): protocol/session/replication contracts
- `net/engine_net_quic` (`engine_net_quic`): QUIC transport/runtime adapter
- `net/engine_sim` (`engine_sim`): shared simulation network contracts/types
- `net/engine_history` (`engine_replay`): replay checkpoints/journals/archive flow

## Game Domain

- `games/cavern_hunt` (`cavern_hunt`): game-specific rules, progression, content flow, and net sync mapping

Game documentation:

- `games/cavern_hunt/docs/CAVERN_HUNT_GDD.md`
- `games/cavern_hunt/docs/CAVERN_HUNT_MATERIALS.md`
- `games/cavern_hunt/docs/CAVERN_HUNT_PLAYTEST.md`

## Application Domain

- `apps/grotto_client` (`grotto_client`): client binary/bootstrap
- `apps/grotto_server` (`grotto_server`): dedicated-authority server binary/bootstrap
- `apps/grotto_online` (`grotto_online`): Axiom integration and runtime handoff services
- `apps/grotto_fleet_control` (`grotto_fleet_control`): operator/fleet control binary/services

## Non-Crate Supporting Domains

- `assets/`: authoring/runtime data assets
- `docs/`: architecture and runbooks
- `ops/`: deployment/runtime operational definitions

## Dependency Rules

Preferred dependency direction:

- `foundation` <- `engine`
- `foundation` <- `net`
- `foundation` <- `games`
- `engine` <- `games`
- `net` <- `games`
- `games` <- `apps`
- `engine` <- `apps`
- `net` <- `apps`

Disallowed direction:

- `foundation` depending on `engine`, `net`, `games`, or `apps`
- `engine` depending on `apps`
- private cross-app dependencies (`apps/*` directly depending on another `apps/*`)
