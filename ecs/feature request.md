# Feature Request: ECS Event System (Roadmap-Oriented)

## Date

2026-02-23

## Context

`ecs` currently has components, queries, bundles, and resources, but no first-class event system. Engine code therefore uses ad-hoc vectors/channels in resources for transient messages.

That pattern is already becoming brittle in UI/scene flows and will not scale well for hot reload, networking, editor tooling, or multi-threaded runtime expansion.

## Core Intent

Introduce a typed ECS event system that starts simple in V1, but is explicitly designed to grow into:

- cross-thread event dispatch,
- network transport/replication,
- save/load persistence policies,
- and topic-based pub-sub where needed.

## Goals

- Typed, deterministic event channels integrated into `World`.
- Auto-create channels on first use (no required manual `ensure`).
- Observer triggers for reactive system behavior.
- Clear lifecycle semantics for frame-scoped vs persistent events.
- Backward-compatible migration path from current ad-hoc vectors.

## V1 (Immediate)

### Required Behavior

- Per-event-type channel storage.
- FIFO ordering within each event type.
- `emit`, `read`, `drain`, `clear`, `count` APIs.
- Channels are automatically created on first `emit_event::<T>()`.
- Optional explicit channel configuration for capacity/overflow.
- Observer registration for typed events.

### Suggested V1 API

```rust
#[derive(Debug, Clone)]
struct SceneUiEvent {
    action: String,
}

let mut world = ecs::World::new();

// Auto-creates channel for SceneUiEvent if missing.
world.emit_event(SceneUiEvent {
    action: "main_menu".to_string(),
});

for event in world.read_events::<SceneUiEvent>() {
    // inspect
}

for event in world.drain_events::<SceneUiEvent>() {
    // consume
}

world.observe_events::<SceneUiEvent>("scene_flow", ObserverTrigger::OnDrain);
```

### V1 API Surface (target)

- `emit_event<T: 'static>(event: T)`
- `read_events<T: 'static>() -> &[T]`
- `drain_events<T: 'static>() -> Drain<T>` or `Vec<T>`
- `clear_events<T: 'static>() -> usize`
- `event_count<T: 'static>() -> usize`
- `has_event_channel<T: 'static>() -> bool`
- `configure_event_channel<T: 'static>(EventChannelConfig)`
- `observe_events<T: 'static>(observer_id: &str, trigger: ObserverTrigger)`
- `remove_event_observer(observer_id: &str) -> bool`

## Observer Triggers

`ObserverTrigger` (initial proposal):

- `OnEmit`: observer runs when an event of type `T` is emitted.
- `OnDrain`: observer runs when a drain/read stage for `T` executes.
- `EndOfFrame`: observer runs once if channel had any events this frame.

Observer execution should be deterministic, with stable ordering by registration order (or explicit priority if added).

## Resource Hot Reload Use Cases

The event system should directly support resource/config hot reload flows, for example:

- `FileChangedEvent { path, modified_at }`
- `ResourceReloadRequested { key }`
- `ResourceReloaded { key, revision, status }`
- `ResourceReloadFailed { key, reason }`

This allows decoupling watchers, reloaders, and UI notifications without ad-hoc channel wiring.

## Deferred but Planned (V2/V3)

These are not rejected; they are staged for later implementation:

### V2 (Threading + Runtime Scale)

- Cross-thread event producers with safe queue handoff into world-owned channels.
- Optional lock-free ingestion for high-frequency event types.
- Backpressure metrics and overflow observability.

### V3 (Distributed + Persistence + Topics)

- Network transport/replication adapters for selected event types.
- Persistence policies for replayable or save-relevant events.
- Topic-based pub-sub layer for dynamic/editor-driven workflows.

## Event Lifetime Model

Each event type should support lifecycle policy configuration:

- `FrameTransient`: clear automatically at end-of-frame unless drained earlier.
- `Manual`: retained until explicit drain/clear.
- `Persistent`: retained with snapshot/restore hooks (for future save/load integration).

## Configuration

`EventChannelConfig` (proposed):

- `capacity: Option<usize>`
- `overflow: OverflowPolicy`
- `lifetime: EventLifetime`
- `tracing: EventTracingPolicy`

`OverflowPolicy`:

- `DropOldest`
- `DropNewest`
- `Panic`

## Scheduler Integration

Short term:

- Systems explicitly read/drain/clear.

Target:

- Optional auto phases:
  - `event_phase_begin_frame`
  - `event_phase_end_frame`
- Observer triggers scheduled in a deterministic phase.

## Migration Plan

1. Add V1 event channels to `ecs::World`.
2. Migrate one high-friction flow first (scene/UI button transitions).
3. Migrate hot-reload notifications to typed events.
4. Keep compatibility wrappers around legacy vector channels during transition.
5. Remove duplicated ad-hoc channels once parity is validated.

## Test Matrix

- Channel auto-create on first emit.
- Emit/read/drain/clear semantics.
- FIFO ordering.
- Type isolation across multiple event types.
- Overflow behavior per policy.
- Observer trigger correctness (`OnEmit`, `OnDrain`, `EndOfFrame`).
- Deterministic ordering of observers.
- Frame lifetime behavior (`FrameTransient` vs `Manual`).
- Hot-reload style integration scenario test.

## Acceptance Criteria

- No manual `ensure_*` required for normal event usage.
- Typed events with deterministic behavior and documented lifecycle.
- Observer triggers available and test-covered.
- V1 integrated in `ecs::World` with migration path to engine call sites.
- Roadmap explicitly keeps threading/replication/persistence/topic support in scope for later phases.
