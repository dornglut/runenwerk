# ECS Crate

`ecs` is the runtime ECS foundation in the `foundation` domain.

## Purpose

- Provide a typed runtime API for entities, components, resources, and events.
- Keep storage internals behind `World`, `EntityRef`/`EntityMut`, and query surfaces.
- Support engine/runtime systems that need deterministic ECS behavior.

## Quick Start

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, ecs::Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
struct Velocity {
    x: f32,
    y: f32,
}

fn tick(world: &mut World) {
    let mut query = world.query_mut::<(&mut Position, &Velocity)>();
    for (position, velocity) in query.iter_mut() {
        position.x += velocity.x;
        position.y += velocity.y;
    }
}
```

## Public Runtime Surface

- `World`: spawn/despawn, insert/remove components, resources, queries, events, indexes.
- `Entity`: stable `(id, generation)` handle.
- `EntityRef` / `EntityMut`: entity-scoped read/write helpers.
- `QueryBorrow` / `QueryBorrowMut`: borrowed query wrappers from `world.query*()`.
- `QueryState<Q, F>`: detached query state reusable across ticks.
- `Commands`: deferred structural changes with explicit `apply`.
- `QueryAccess`: read/write access metadata for scheduler/runtime planning.

## Detailed Usage Documentation

- Usage guide: [`foundation/ecs/USAGE_GUIDE.md`](./USAGE_GUIDE.md)

## Current Query Support

Implemented query data forms:

- `&T`
- `&mut T`
- `(Entity, &T)`
- `(&A, &B)`
- `(&mut A, &B)`

Filters are additive via `.with::<T>()` and `.without::<T>()` on both borrowed and detached queries.

## Ownership Boundaries

- Owns runtime ECS state and typed access APIs.
- Owns change tracking, event channels, and component secondary indexes.
- Does not own scheduler orchestration, rendering, or authoring/editor pipelines.

## Extension Points

- Add additional `QueryData` implementations and filter composition.
- Extend `QueryAccess` metadata for scheduling and diagnostics.
- Add runtime adapters at domain boundaries (engine/net) without leaking storage internals.
