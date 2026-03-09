# ECS Crate

`ecs` is the engine ECS foundation.

## Purpose

- Provide the engine ECS API.
- Hide storage internals behind typed world/entity/query surfaces.
- Serve as the runtime ECS for engine and networking systems.

## Usage

- Crate: `ecs`
- Primary surfaces:
  - `World`
  - `Entity`
  - `EntityRef` / `EntityMut`
  - `QueryState`
  - `Commands`

## Ownership Boundaries

- Owns runtime entity/component/resource/event state and typed access APIs.
- Does not own engine scheduling, rendering, or authoring pipelines.

## Extension Points

- Add richer query/filter/fetch implementations.
- Add typed scheduler-facing access metadata.
- Add richer runtime-facing adapters where engine features need them.

## Current Runtime Hooks

`ecs` now exposes the core pieces needed by the engine runtime:

- detached `QueryState<Q, F>` caches
- borrowed `world.query()` / `world.query_mut()` wrappers
- query access metadata via `QueryAccess`
- deferred `Commands` with explicit `apply`
