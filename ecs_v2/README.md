# ECS V2 Crate

`ecs_v2` is the next-generation engine ECS foundation.

## Purpose

- Provide a stricter public ECS API than `ecs`.
- Hide storage internals behind typed world/entity/query surfaces.
- Serve as the target runtime ECS for future engine migration.

## Usage

- Crate: `ecs_v2`
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
- Add compatibility adapters for engine runtime migration.

## Current Runtime Hooks

`ecs_v2` now exposes the core pieces needed by the typed engine runtime:

- detached `QueryState<Q, F>` caches
- borrowed `world.query()` / `world.query_mut()` wrappers
- query access metadata via `QueryAccess`
- deferred `Commands` with explicit `apply`
