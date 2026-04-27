---
title: Advanced Guide
description: Engine-agnostic guide for ecs usage.
status: active
owner: ecs
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# ECS Advanced Guide

Audience: advanced users, runtime integrators, and users extending ECS behavior.

For normal day-to-day ECS usage, start with [usage-guide.md](usage-guide.md).
For internal implementation invariants, see [architecture.md](architecture.md).

## 1. Deferred Commands and Stage Visibility

`Commands` are deferred structural mutations.

Runtime rule:

1. each system run gets its own command queue
2. queues are collected in deterministic system execution order
3. queues are applied at stage end

Implication: systems in the same stage do not see each other's queued structural changes until the stage flush boundary.

Practical pitfall: if one system queues `spawn/insert/remove` and another must observe that in the same frame, place the observer in a later stage via set ordering.

## 2. Runtime Ordering and Configuration

`SystemConfigExt` enables explicit ordering:

- `in_set(...)`
- `before(...)`
- `after(...)`

`Runtime::plan_for::<L>()` returns the compiled execution plan and is useful for validating schedule shape during integration tests.

```rust
use ecs::prelude::*;
use scheduler::label::SystemSet;
use scheduler::ScheduleLabel;

#[derive(Copy, Clone)]
struct Update;
impl ScheduleLabel for Update {
    fn name() -> &'static str { "Update" }
}

#[derive(Copy, Clone)]
struct Gameplay;
impl SystemSet for Gameplay {
    fn name() -> &'static str { "Gameplay" }
}

#[derive(Copy, Clone)]
struct PostGameplay;
impl SystemSet for PostGameplay {
    fn name() -> &'static str { "PostGameplay" }
}

fn produce(mut commands: Commands) {
    commands.spawn(());
}

fn observe() {}

let mut world = World::new();
let mut runtime = Runtime::new();
runtime.add_systems::<Update, _, _>(&mut world, produce.in_set(Gameplay));
runtime.add_systems::<Update, _, _>(&mut world, observe.in_set(PostGameplay).after(Gameplay));

let _plan = runtime.plan_for::<Update>().unwrap().clone();
```

## 3. Advanced Event Channels

`BroadcastStreamConfig` controls queue semantics per event type:

- `capacity: Option<usize>`
- `overflow: BroadcastOverflowPolicy` (`DropOldest`, `DropNewest`, `Panic`)
- `lifetime: BroadcastLifetime` (`Persistent`, `FrameTransient`)
- `tracing: BroadcastTracingPolicy`

```rust
use ecs::{BroadcastStreamConfig, BroadcastLifetime, BroadcastTracingPolicy, BroadcastOverflowPolicy, World};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickEvent;

let mut world = World::new();
world.configure_broadcast_stream::<TickEvent>(BroadcastStreamConfig {
    capacity: Some(1),
    overflow: BroadcastOverflowPolicy::DropOldest,
    lifetime: BroadcastLifetime::FrameTransient,
    tracing: BroadcastTracingPolicy::Disabled,
});
```

Pitfalls:

- `capacity = Some(0)` drops all events (or panics if overflow policy is `Panic`).
- `FrameTransient` requires `finalize_frame_boundary()` to clear pending events at frame end.

## 4. Event Observers and Notifications

Observer APIs:

- `observe_events<T>(observer_id, trigger)`
- `remove_event_observer(observer_id)`
- `event_observer_invocations(observer_id)`
- `drain_event_observer_notifications()`

Triggers:

- `ObserverTrigger::OnEmit`
- `ObserverTrigger::OnDrain`
- `ObserverTrigger::EndOfFrame`

This is useful for diagnostics, auditing, and runtime tooling without coupling game logic to direct drain timing.

## 5. Event Drain Helpers

When draining events into derived outputs:

- `drain_events_map<T, U, F>(map)`
- `drain_events_filter<T, F>(predicate)`

These helpers preserve explicit-drain semantics while avoiding intermediate boilerplate loops.

## 6. Advanced Secondary Index Usage

Beyond basic lookups:

- named indexes: `ensure_component_index_named<T, K>(name, extractor)`
- multi-hit lookups: `find_entities_by_index*`
- direct component lookup: `find_component_by_index*`

Operational note: indexes are lazily rebuilt and dirtied by component churn. Integration code can call lookup helpers from `&World`; rebuild mutation is internal via interior mutability.

## 7. Change Semantics Boundary

Two separate models exist and should not be conflated:

- Query/filter semantics (`Changed<T>`, `Added<T>`): archetype-row ticks, per-query last-seen tick state
- Reporting/introspection (`component_changes_since`, `resource_changes_since`, `*_changed_since`): world-level history views

Guideline: use query filters for gameplay/system behavior and history APIs for diagnostics/reporting.

## 8. Custom `SystemParam` Extension Path

Extension trait: `ecs::SystemParam<'w>`.

Required pieces:

- `type State`
- `fn init_state(world: &mut World) -> Result<State, SystemParamError>`
- `fn access(state: &State) -> QueryAccess`
- `unsafe fn extract(state: &mut State, world: *mut World, commands: *mut Commands) -> Result<Self, SystemParamError>`

Safety requirement: `State` must remain lifetime-independent across extraction lifetimes; extraction must respect declared `QueryAccess`.

## 9. Telemetry Interpretation and Profiling Workflow

Enable telemetry:

```powershell
cargo bench -p ecs --bench phase6 --features telemetry -- --quick
cargo run -p ecs --example phase6_profile --features telemetry --release
```

Use telemetry counters/timers to separate:

- query iteration and filter cost
- schedule planning cost
- per-stage execution cost
- deferred command flush cost

Suggested workflow:

1. capture baseline snapshot
2. run targeted workload/benchmark
3. compare query/filter/runtime/flush counters
4. validate no semantic regressions with `cargo test -p ecs`

Related benchmark docs:

- [`../benchmarks/phase6/benchmark-suite.md`](./benchmarks/phase6/benchmark-suite.md)
- [`../benchmarks/phase6/progress-report.md`](./benchmarks/phase6/progress-report.md)
- [`../benchmarks/phase6/final-decision-report.md`](./benchmarks/phase6/final-decision-report.md)
