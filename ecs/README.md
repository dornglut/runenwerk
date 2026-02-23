# ECS Crate

`ecs` is the project-local entity component system used by the engine runtime.

## What It Provides

- Entity allocation and lifecycle management.
- Typed components via `#[derive(ecs::Component)]`.
- Bundle spawning for multi-component entities.
- Query iteration over archetypes.
- World resources (singleton state per world).

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

fn main() {
    let mut world = World::new();
    let entity = world.spawn_bundle((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.5 },
    ));

    world.query_mut::<Position, Velocity, _>(|_, position, velocity| {
        position.x += velocity.x;
        position.y += velocity.y;
    });

    let position = world
        .get_component::<Position>(entity)
        .expect("position should exist");
    assert_eq!(position.x, 1.0);
    assert_eq!(position.y, 0.5);
}
```

## Resources

Resources are world-level singletons:

```rust
#[derive(Debug, PartialEq)]
struct FrameCounter(u64);

let mut world = ecs::World::new();
world.insert_resource(FrameCounter(1));
assert_eq!(world.get_resource::<FrameCounter>(), Some(&FrameCounter(1)));
```

## Run Tests

```bash
cargo test -p ecs
```

## Notes

- Query and resource examples are covered in:
  - `ecs/tests/query.rs`
  - `ecs/tests/resources.rs`
  - `ecs/tests/world.rs`
- Event-bus style messaging is currently not part of this crate. See `ecs/feature request.md`.
