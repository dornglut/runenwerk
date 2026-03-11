# ECS Crate

`ecs` is the runtime ECS foundation in the `foundation` domain.

## Target API

- One derive for ECS-managed data: `#[derive(ecs::Component)]`
- Tags are empty components
- Resources are globally stored singleton components
- Gameplay params: `Query<Q, F = ()>`, `Res<T>`, `ResMut<T>`, `Commands`, `EventReader<T>`, `EventWriter<T>`
- Query filters: `With<T>`, `Without<T>`, `Changed<T>`, `Added<T>`
- Runtime world API stays low-level and explicit (`World`, `QueryState`, entity handles, events, indexes)

## Canonical Gameplay Shape

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

## Runtime QueryState Example

```rust
use ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Velocity { x: f32, y: f32 }

let mut world = World::new();
world.spawn((Position { x: 1.0, y: 2.0 }, Velocity { x: 0.5, y: -1.0 }));

let query = world.query_state::<(&mut Position, &Velocity), ()>();
for (position, velocity) in query.iter(&mut world) {
    position.x += velocity.x;
    position.y += velocity.y;
}
```

## Runtime Execution Example

```rust
use ecs::prelude::*;
use scheduler::ScheduleLabel;

#[derive(Copy, Clone)]
struct Update;
impl ScheduleLabel for Update {
    fn name() -> &'static str { "Update" }
}

#[derive(ecs::Component)]
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

## Runtime Surface

- `World`: entity lifecycle, bundle insert/remove, resources, events, indexes
- `EntityRef` / `EntityMut`: entity-scoped access helpers
- `QueryState<Q, F = ()>`: reusable detached query state (`world.query_state::<Q, F>()`)
- `QueryAccess`: component/resource/deferred mutation metadata
- `Commands`: deferred structural changes

## Phase 5B Benchmarking and Profiling

Benchmark harness and profiling artifacts live in:

- `foundation/ecs/benches/phase35.rs` (ECS microbench workloads W1-W5)
- `engine/benches/phase35_runtime.rs` (engine/runtime scenario workload)
- `foundation/ecs/examples/phase35_profile.rs` (telemetry-attributed workload profiler)
- `foundation/ecs/benches/phase4.rs` (Phase 4 suite including C2/C3 broad read/write workloads)
- `engine/benches/phase4_runtime.rs` (Phase 4 engine/runtime scenario target)
- `foundation/ecs/examples/phase4_profile.rs` (Phase 4 profiler with C2/C3 attribution)
- `foundation/ecs/benches/phase5b.rs` (Phase 5B suite including dominant C4 double-mutable workload)
- `engine/benches/phase5b_runtime.rs` (Phase 5B engine/runtime scenario target)
- `foundation/ecs/examples/phase5b_profile.rs` (Phase 5B profiler with C4 attribution)
- `foundation/ecs/benchmarks/phase4/` (historical Phase 4 artifacts and Phase 5 decision report)
- `foundation/ecs/benchmarks/phase5b/` (Phase 5B baseline refresh, optimized outputs, and Phase 6 decision report)

Run commands:

```powershell
cargo bench -p ecs --bench phase35 --features telemetry -- --quick
cargo run -p ecs --example phase35_profile --features telemetry --release
cargo bench -p engine --bench phase35_runtime -- --quick

cargo bench -p ecs --bench phase4 --features telemetry -- --quick
cargo run -p ecs --example phase4_profile --features telemetry --release
cargo bench -p engine --bench phase4_runtime -- --quick

cargo bench -p ecs --bench phase5b --features telemetry -- --quick
cargo run -p ecs --example phase5b_profile --features telemetry --release
cargo bench -p engine --bench phase5b_runtime -- --quick
```

Notes:

- `telemetry` is feature-gated and records query/filter/runtime/scheduler hot path counters and timings.
- Quick-mode Criterion runs are used for repeatable local before/after comparisons during roadmap execution.
- Command flush, event visibility, scheduler conflict semantics, and change tracking semantics remain unchanged.

## Detailed Usage

- Usage guide: [`foundation/ecs/USAGE_GUIDE.md`](./USAGE_GUIDE.md)
- Architecture and invariants: [`foundation/ecs/ARCHITECTURE.md`](./ARCHITECTURE.md)
- Phase 3.5 benchmarks/report: [`foundation/ecs/benchmarks/phase35/README.md`](./benchmarks/phase35/README.md)
- Phase 4 benchmarks/report: [`foundation/ecs/benchmarks/phase4/README.md`](./benchmarks/phase4/README.md)
- Phase 5B benchmarks/report: [`foundation/ecs/benchmarks/phase5b/README.md`](./benchmarks/phase5b/README.md)
