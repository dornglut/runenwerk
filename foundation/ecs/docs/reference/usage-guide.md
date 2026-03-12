# ECS Usage Guide

This guide documents the target `foundation/ecs` API.

## 1. Import the API

```rust
use ecs::prelude::*;
```

## 2. Define Components, Tags, Resources

All ECS-managed data derives `ecs::Component`.

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

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct DeltaTime(f32);
```

- Components: per-entity data (`Position`, `Velocity`)
- Tags: empty components (`Player`)
- Resources: singleton components stored globally (`DeltaTime`)

## 3. World Lifecycle

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

let mut world = World::new();
let entity = world.spawn(Position { x: 1.0, y: 2.0 });
assert!(world.contains(entity));
world.despawn(entity).unwrap();
```

## 4. Entity Access

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

let mut world = World::new();
let entity = world.spawn(Position { x: 1.0, y: 2.0 });

assert_eq!(world.require::<Position>(entity).unwrap().x, 1.0);
world.require_mut::<Position>(entity).unwrap().x += 1.0;
assert_eq!(world.require::<Position>(entity).unwrap().x, 2.0);
```

## 5. QueryState Runtime Queries

`World` uses detached reusable query state for direct runtime querying:

- preferred constructor: `world.query_state::<Q, F>()`
- reusable alternative: `QueryState::<Q, F>::new(&world)`

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Velocity { x: f32, y: f32 }

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Disabled;

let mut world = World::new();
world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 2.0 }));
world.spawn((Position { x: 9.0, y: 9.0 }, Velocity { x: 9.0, y: 9.0 }, Disabled));

let query = world
    .query_state::<(&mut Position, &Velocity), ()>()
    .without::<Disabled>();
for (position, velocity) in query.iter(&mut world) {
    position.x += velocity.x;
    position.y += velocity.y;
}
```

`QueryState` uses one API shape:

- `iter(...)`
- `get(...)`
- `single(...)`

Mutability is driven by the query type `Q` (`&T` vs `&mut T`), not by separate query wrapper types.

## 6. Query Patterns

Supported forms include:

- `Query<&T>`
- `Query<&mut T>`
- `Query<(Entity, &T)>`
- `Query<(Entity, &mut T)>`
- `Query<(&A, &B)>`
- `Query<(&mut A, &B)>`
- `Query<(&A, &mut B)>`
- `Query<(&mut A, &mut B)>`
- `Query<Option<&T>>`
- `Query<Option<&mut T>>`
- `Query<(&mut A, Option<&B>)>`
- `Query<(&mut A, Option<&mut B>)>`
- `Query<(&A, Option<&B>)>`
- `Query<(&A, Option<&mut B>)>`
- `Query<(Entity, Option<&T>)>`

Recommended extended tuple forms are also implemented:

- `Query<(&A, &B, &C)>`
- `Query<(&mut A, &B, &C)>`
- `Query<(&mut A, &mut B, &C)>`

## 7. Query Filters

```rust
use ecs::prelude::*;

fn players(_query: Query<&Position, With<Player>>) {}
fn active_players(_query: Query<&Position, (With<Player>, Without<Disabled>)>) {}
fn changed_players(_query: Query<&Position, (Changed<Position>, With<Player>)>) {}
fn added_health(_query: Query<(Entity, &Health), Added<Health>>) {}
# #[derive(ecs::Component)] struct Position;
# #[derive(ecs::Component)] struct Player;
# #[derive(ecs::Component)] struct Disabled;
# #[derive(ecs::Component)] struct Health;
```

`Changed<T>` and `Added<T>` are stateful filters: each `QueryState` tracks its own last-seen tick.
The first run includes already-added components. Subsequent runs include only new changes since
the previous call.

## 8. Resources

Resources are components stored globally:

```rust
use ecs::prelude::*;

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct Frame(u64);

let mut world = World::new();
world.insert_resource(Frame(1));
assert!(world.has_resource::<Frame>());

{
    let mut frame = world.resource_mut::<Frame>().unwrap();
    frame.0 += 1;
}

assert_eq!(world.resource::<Frame>().unwrap().0, 2);
assert_eq!(world.remove_resource::<Frame>(), Some(Frame(2)));
```

## 9. Commands

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

let mut world = World::new();
let doomed = world.spawn(Position { x: 9.0, y: 9.0 });

let mut commands = world.commands();
commands.spawn(Position { x: 1.0, y: 2.0 });
commands.despawn(doomed);
commands.apply(&mut world).unwrap();
```

In runtime schedules, each system gets its own command queue. Queues are applied deterministically
at the end of each scheduler stage.

## 10. Runtime Scheduling

```rust
use ecs::prelude::*;
use scheduler::ScheduleLabel;

#[derive(Copy, Clone)]
struct Update;
impl ScheduleLabel for Update {
    fn name() -> &'static str { "Update" }
}

#[derive(Debug, PartialEq, Eq, ecs::Component)]
struct Frame(u64);

fn tick(mut frame: ResMut<Frame>) {
    frame.0 += 1;
}

let mut world = World::new();
world.insert_resource(Frame(0));

let mut runtime = Runtime::new();
runtime.add_systems::<Update, _, _>(&mut world, tick);
runtime.run_schedule::<Update>(&mut world).unwrap();
assert_eq!(world.resource::<Frame>().unwrap().0, 1);
```

## 11. Events

Runtime world APIs:

- `emit_event<T>(event)`
- `read_events<T>()`
- `drain_events<T>()`
- `clear_events<T>()`
- `event_count<T>()`
- `event_channel_stats<T>()`

Gameplay param APIs:

- `EventReader<T>::iter()`
- `EventWriter<T>::send(event)`

## 12. Change Tracking

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

let mut world = World::new();
let tick = world.current_change_tick();
let entity = world.spawn(Position { x: 0.0, y: 0.0 });
world.require_mut::<Position>(entity).unwrap().x = 1.0;

assert!(world.component_changed_since::<Position>(tick));
assert!(!world.component_changes_since(tick).is_empty());
```

Tracking model summary:

- Query/filter semantics (`Changed<T>`, `Added<T>`) are driven by archetype row metadata.
- Reporting/introspection APIs (`component_changed_since`, `component_changes_since`, and
  resource variants) are log/tick-based history helpers.

## 13. Secondary Indexes

```rust
use ecs::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component)]
struct Name(String);

let mut world = World::new();
world.ensure_component_index::<Name, String>(|name| name.0.clone());

let hero = world.spawn(Name("hero".to_string()));
assert_eq!(
    world.find_entity_by_index::<Name, String>(&"hero".to_string()),
    Some(hero)
);

let shared: &World = &world;
assert_eq!(
    shared.find_entity_by_index::<Name, String>(&"hero".to_string()),
    Some(hero)
);
```

## 14. Benchmarking and Telemetry (Phase 6)

`ecs` exposes feature-gated runtime telemetry for profiling and cost attribution.

Run the benchmark/profiling suite:

```powershell
cargo test -p ecs
cargo bench -p ecs --bench phase6 --features telemetry -- --quick
cargo run -p ecs --example phase6_profile --features telemetry --release
cargo bench -p engine --bench phase6_runtime -- --quick
```

Telemetry APIs:

- `ecs::telemetry::reset()`
- `ecs::telemetry::snapshot()`

The profiler example prints query/filter/scheduler/runtime/flush counters and timing totals so
hotspots can be attributed without changing runtime semantics.

Benchmark/profiling artifacts and decision reports are stored in:

- `foundation/ecs/benchmarks/phase6/` (raw `.txt` outputs)
- `foundation/ecs/docs/benchmarks/phase6/benchmark-suite.md`
- `foundation/ecs/docs/benchmarks/phase6/progress-report.md`
- `foundation/ecs/docs/benchmarks/phase6/final-decision-report.md`
