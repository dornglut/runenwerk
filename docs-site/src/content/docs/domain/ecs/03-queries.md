---
title: Queries
description: Engine-agnostic guide for ecs queries.
---

# ECS Queries

Queries retrieve entities/components through typed access rules and filters.

## Purpose

- Select entity subsets by component composition.
- Support read and mutable access with scheduler conflict safety.
- Express change-driven logic with `Added<T>` and `Changed<T>`.
- Observe recent removals through `QueryOrphaned<T>`.

## Key Concepts

- `Query<Q, F>`: in-system typed query param.
- `QueryState<Q, F>`: detached reusable query state.
- Filters: `With<T>`, `Without<T>`, `Added<T>`, `Changed<T>`.
- `QueryOrphaned<T>` / `QueryOrphanedState<T>`: removed-component stage window.

## API Notes

Detached query state:

```rust
let query = world.query_state::<(&mut Position, &Velocity), ()>();
for (pos, vel) in query.iter(&mut world) {
    pos.x += vel.x;
    pos.y += vel.y;
}
```

Orphaned component window:

```rust
fn process_removed(mut orphaned: QueryOrphaned<Velocity>) {
    for removed in orphaned.iter() {
        let entity = removed.entity();
        let tick = removed.tick();
        let _ = (entity, tick);
    }
}
```

## Invariants

- Queries do not observe deferred structural changes until stage flush.
- `Added<T>`/`Changed<T>` are query-instance stateful and tick-based.
- `QueryOrphaned<T>` reports removals in the current stage flush window.
- Mutable query shapes must not alias the same component mutably.
