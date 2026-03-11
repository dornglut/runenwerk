# ECS Usage Patterns

```rust
use ecs::prelude::*;

#[derive(ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(ecs::Component)]
struct Simulated;

#[derive(ecs::Component)]
struct DeltaTime(pub f32);

#[derive(ecs::Component)]
struct Frame(pub u64);
```

All ECS data types derive ecs::Component.

Components store entity data (Position, Velocity)

Tags are empty components used for filtering (Simulated)

Resources are global singleton components (DeltaTime, Frame)

System Example

Systems declare the data they require using system parameters.

```rust
use ecs::prelude::*;

fn tick(
    mut query: Query<(&mut Position, &Velocity), With<Simulated>>,
    dt: Res<DeltaTime>,
    mut frame: ResMut<Frame>,
) {
    for (pos, vel) in query.iter() {
        pos.x += vel.x * dt.0;
        pos.y += vel.y * dt.0;
    }

    frame.0 += 1;
}
```

This system:

Iterates all entities with Position, Velocity, and Simulated

Updates positions using velocity and delta time

Increments the global frame counter

Query Patterns
Read-only

```rust
fn system(query: Query<&Position>) {
    for pos in query.iter() {}
}
```

Read + write

```rust
fn system(mut query: Query<(&mut Position, &Velocity)>) {
    for (pos, vel) in query.iter() {}
}
```

Multiple mutable

```rust
fn system(mut query: Query<(&mut Position, &mut Velocity)>) {
    for (pos, vel) in query.iter() {}
}
```
Entity ids

```rust
fn system(query: Query<(Entity, &Position)>) {
    for (entity, pos) in query.iter() {}
}
```

Optional components

```rust
fn system(mut query: Query<(&mut Position, Option<&Velocity>)>) {
    for (pos, vel) in query.iter() {}
}
```

Query Filters

With

```rust
Query<&Position, With<Player>>
```

Without

```rust
Query<&Position, Without<Disabled>>
```

Multiple filters
```rust
Query<&Position, (With<Player>, Without<Disabled>)>
```

Resource Access

Resources are injected as system parameters.

```rust
fn system(dt: Res<DeltaTime>) {}
```

Mutable resources:

```rust
fn system(mut frame: ResMut<Frame>) {}
```
Design Principles

Components store data only

Systems contain logic

Tags filter entities

Resources store global state

Systems declare data dependencies explicitly

The scheduler can use this to run systems in parallel