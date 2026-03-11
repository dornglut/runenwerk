# ECS Usage Guide

This guide documents the currently supported runtime API in `foundation/ecs`.
Examples are aligned with `foundation/ecs/tests/world.rs`.

## 1. Import the API

```rust
use ecs::prelude::*;
```

The prelude re-exports `World`, query types, event/index APIs, error types, and derive macros.

## 2. Define Components and Bundles

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Player;

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
struct Name(String);

#[derive(Debug, PartialEq, ecs::Bundle)]
struct CombatBundle {
    health: Health,
    name: Name,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Health(i32);
```

Notes:

- Single components can be inserted as bundles.
- Tuple bundles are supported up to 6 components.
- Custom bundle structs can derive `ecs::Bundle`.

## 3. Create a World and Spawn Entities

```rust
use ecs::prelude::*;

let mut world = World::new();

let player = world.spawn((
    Player,
    Position { x: 1.0, y: 2.0 },
    Velocity { x: 0.5, y: -1.0 },
));

assert!(world.contains(player));
```

## 4. Entity-Scoped Reads and Writes

```rust
use ecs::prelude::*;

let position = world.require::<Position>(player)?;
assert_eq!(position.x, 1.0);

{
    let mut entity = world.entity_mut(player)?;
    entity.require_mut::<Position>()?.x += 10.0;
}

assert_eq!(world.require::<Position>(player)?.x, 11.0);
```

Use:

- `world.get::<T>(entity)` for optional reads
- `world.require::<T>(entity)` for fallible reads (`EntityError`)
- `world.entity(entity)` / `world.entity_mut(entity)` for grouped entity operations

## 5. Borrowed Queries

### Read-only query

```rust
use ecs::prelude::*;

let seen: Vec<_> = world
    .query::<(Entity, &Position)>()
    .with::<Player>()
    .iter()
    .map(|(entity, position)| (entity, position.x, position.y))
    .collect();
```

### Mutable query with filter

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Disabled;

let mut query = world
    .query_mut::<(&mut Position, &Velocity)>()
    .without::<Disabled>();

for (position, velocity) in query.iter_mut() {
    position.x += velocity.x;
    position.y += velocity.y;
}
```

### Single/get helpers

```rust
use ecs::prelude::*;

let maybe_pos = world.query::<&Position>().get(player);
let only_player_pos = world.query::<&Position>().with::<Player>().single();
```

`single()` / `single_mut()` return `QueryError::NoResults` or `QueryError::MultipleResults`.

## 6. Detached QueryState

`QueryState<Q, F>` is reusable across frames and detached from a world borrow.

```rust
use ecs::prelude::*;

let position_query = QueryState::<(Entity, &Position)>::new(&mut world).with::<Player>();

for (entity, position) in position_query.iter_on(&world) {
    let _ = (entity, position.x);
}
```

Mutable detached query:

```rust
use ecs::prelude::*;

let mut movement_query = QueryState::<(&mut Position, &Velocity)>::new(&mut world);
for (position, velocity) in movement_query.iter_mut_on(&mut world) {
    position.x += velocity.x;
    position.y += velocity.y;
}
```

## 7. Resources

```rust
use ecs::prelude::*;

#[derive(Debug, PartialEq, Eq)]
struct Frame(u64);

world.insert_resource(Frame(1));
assert!(world.has_resource::<Frame>());

{
    let mut frame = world.resource_mut::<Frame>()?;
    frame.0 += 1;
}

assert_eq!(world.resource::<Frame>()?.0, 2);
assert_eq!(world.remove_resource::<Frame>(), Some(Frame(2)));
```

All resource failures use `ResourceError::Missing`.

## 8. Deferred Structural Changes with Commands

```rust
use ecs::prelude::*;

let existing = world.spawn(Position { x: 1.0, y: 1.0 });
let doomed = world.spawn(Position { x: 99.0, y: 99.0 });

let mut commands = world.commands();
commands.spawn((Position { x: 3.0, y: 4.0 }, Velocity { x: 0.0, y: 1.0 }));
commands.insert(existing, Velocity { x: 5.0, y: 6.0 });
commands.despawn(doomed);
commands.apply(&mut world)?;
```

`Commands::apply` executes queued operations in order and returns `CommandError` if one fails.

## 9. Secondary Component Indexes

Use indexes for key-based lookups on a component field.

```rust
use ecs::prelude::*;

world.ensure_component_index::<Name, String>(|name| name.0.clone());

let hero = world.spawn(Name("hero".to_string()));
assert_eq!(
    world.find_entity_by_index::<Name, String>(&"hero".to_string()),
    Some(hero)
);
```

Named indexes:

```rust
use ecs::prelude::*;

world.ensure_component_index_named::<Name, char>("initial", |name| {
    name.0.chars().next().unwrap_or_default()
});

let h_entities = world.find_entities_by_index_named::<Name, char>("initial", &'h');
```

Lookup methods take `&mut World` because index rebuilds are lazy and performed on access.

## 10. Event Channels and Observers

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickEvent;

world.configure_event_channel::<TickEvent>(EventChannelConfig {
    capacity: Some(1),
    overflow: OverflowPolicy::DropOldest,
    lifetime: EventLifetime::FrameTransient,
    tracing: EventTracingPolicy::Disabled,
});

world.observe_events::<TickEvent>("tick_emit", ObserverTrigger::OnEmit);

world.emit_event(TickEvent);
world.emit_event(TickEvent); // one pending because capacity=1 + DropOldest

assert_eq!(world.event_count::<TickEvent>(), 1);
let drained = world.drain_events::<TickEvent>();
assert_eq!(drained.len(), 1);

let notifications = world.drain_event_observer_notifications();
assert!(!notifications.is_empty());
```

For `EventLifetime::FrameTransient`, call `world.finish_event_frame()` each frame to clear pending events.

## 11. Change Tracking

```rust
use ecs::prelude::*;

let start_tick = world.current_change_tick();
let entity = world.spawn(Position { x: 0.0, y: 0.0 });
world.require_mut::<Position>(entity)?.x = 1.0;

assert!(world.component_changed_since::<Position>(start_tick));

let component_changes = world.component_changes_since(start_tick);
assert!(!component_changes.is_empty());
```

Resource changes are available through:

- `resource_changed_since::<R>(tick)`
- `resource_changes_since(tick)`

## 12. Current Runtime Constraints

- Query data implementations are currently limited to:
  - `&T`
  - `&mut T`
  - `(Entity, &T)`
  - `(&A, &B)`
  - `(&mut A, &B)`
- Component tuple bundles are implemented up to arity 6.
- `Resource` is a marker for `'static` types.
- Event channels are type-based (`T: 'static`) and created lazily on first use.

