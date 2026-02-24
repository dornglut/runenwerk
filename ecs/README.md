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

## Mutable Query Builder

Use `query_mut_components::<A>()` when you need mutable queries with `with/without` filters:

```rust
#[derive(Debug, Copy, Clone, ecs::Component)]
struct Disabled;

let mut world = ecs::World::new();
let _ = world.spawn_bundle((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.0 }));
let _ = world.spawn_bundle((Position { x: 0.0, y: 0.0 }, Velocity { x: 2.0, y: 0.0 }, Disabled));

world
    .query_mut_components::<Position>()
    .with::<Velocity>()
    .without::<Disabled>()
    .for_each_with::<Velocity, _>(|_, position, velocity| {
        position.x += velocity.x;
        position.y += velocity.y;
    });
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

## Events and Subscriptions

Events are typed and channel-backed. Channels are auto-created on first emit, so setup is minimal.

```rust
use ecs::{ObserverTrigger, World};

#[derive(Debug, Clone, PartialEq, Eq)]
struct UiActionEvent {
    action: &'static str,
}

let mut world = World::new();

// "Subscribe" by registering observers for typed events.
world.observe_events::<UiActionEvent>("ui_action_on_emit", ObserverTrigger::OnEmit);
world.observe_events::<UiActionEvent>("ui_action_on_drain", ObserverTrigger::OnDrain);
world.observe_events::<UiActionEvent>("ui_action_end_of_frame", ObserverTrigger::EndOfFrame);

// Publish events.
world.emit_event(UiActionEvent { action: "pause" });
world.emit_event(UiActionEvent { action: "settings" });

// Non-consuming read.
let pending = world.read_events::<UiActionEvent>();
assert_eq!(pending.len(), 2);

// Consume and clear this event type.
let drained = world.drain_events::<UiActionEvent>();
assert_eq!(drained.len(), 2);

// Observer notifications are plain data you can process in a system.
let notifications = world.drain_event_observer_notifications();
assert!(notifications
    .iter()
    .any(|n| n.observer_id == "ui_action_on_emit" && n.event_count > 0));
assert!(notifications
    .iter()
    .any(|n| n.observer_id == "ui_action_on_drain" && n.event_count == 2));
```

### Event drain helpers

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct UiEvent {
    action: &'static str,
}

let mut world = ecs::World::new();
world.emit_event(UiEvent { action: "pause" });
world.emit_event(UiEvent { action: "settings" });

let important = world.drain_events_filter::<UiEvent, _>(|event| event.action.len() > 5);
assert_eq!(important.len(), 1);

world.emit_event(UiEvent { action: "resume" });
let labels = world.drain_events_map::<UiEvent, String, _>(|event| format!("ui:{}", event.action));
assert_eq!(labels, vec!["ui:resume".to_string()]);
```

### Frame lifecycle

Call `world.finish_event_frame()` once per frame/tick if you use end-of-frame behavior:

- Fires `ObserverTrigger::EndOfFrame` for channels with pending events.
- Clears channels configured as `EventLifetime::FrameTransient`.

Example:

```rust
use ecs::{EventChannelConfig, EventLifetime, EventTracingPolicy, OverflowPolicy, World};

#[derive(Debug, Clone, PartialEq, Eq)]
struct TickEvent;

let mut world = World::new();
world.configure_event_channel::<TickEvent>(EventChannelConfig {
    capacity: None,
    overflow: OverflowPolicy::DropOldest,
    lifetime: EventLifetime::FrameTransient,
    tracing: EventTracingPolicy::Disabled,
});

world.emit_event(TickEvent);
assert_eq!(world.event_count::<TickEvent>(), 1);

world.finish_event_frame();
assert_eq!(world.event_count::<TickEvent>(), 0);
```

## Secondary Component Indexes

Use indexes to look up entities/components by component-derived keys.

```rust
#[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
struct ShaderAsset {
    id: String,
    path: String,
}

let mut world = ecs::World::new();
world.register_component::<ShaderAsset>();
world.ensure_component_index::<ShaderAsset, String>(|asset| asset.id.clone());

let entity = world.spawn_entity_typed(ShaderAsset {
    id: "ui_rect".to_string(),
    path: "assets/shaders/ui_rect.wgsl".to_string(),
});

assert_eq!(
    world.find_entity_by_index::<ShaderAsset, String>(&"ui_rect".to_string()),
    Some(entity)
);
```

Multiple indexes for the same component/key type can be registered by name:

```rust
world.ensure_component_index_named::<ShaderAsset, usize>("path_len", |asset| asset.path.len());
let found = world.find_entity_by_index_named::<ShaderAsset, usize>("path_len", &28usize);
```

## Change Tracking and Lifecycle Events

`World` maintains monotonic change ticks for component/resource writes and emits lifecycle events on spawn/despawn.

```rust
use ecs::{EntityDespawnedEvent, EntitySpawnedEvent, World};

#[derive(Debug, Copy, Clone, ecs::Component)]
struct Position(f32, f32);

let mut world = World::new();
world.register_component::<Position>();
let tick = world.current_change_tick();

let entity = world.spawn_entity_typed(Position(0.0, 0.0));
assert!(world.component_changed_since::<Position>(tick));

world.remove_entity(entity);
assert_eq!(world.drain_events::<EntitySpawnedEvent>().len(), 1);
assert_eq!(world.drain_events::<EntityDespawnedEvent>().len(), 1);

let component_changes = world.component_changes_since(tick);
assert!(!component_changes.is_empty());
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
- Event channel and observer examples are covered in:
  - `ecs/tests/events.rs`
