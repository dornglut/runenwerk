---
title: "ECS Runtime Feature Inventory (April 2026)"
description: "Current implemented ECS/runtime/network foundations relevant to multiplayer and editor sequencing."
---

# ECS Runtime Feature Inventory (April 2026)

This inventory reflects the code currently in the repository (audit date: 2026-04-08).

Status labels used in this document:

- `Implemented`: available and exercised in code/tests.
- `Partial`: implemented with clear constraints or only in a narrower layer.
- `Implied`: architecture direction exists, but no concrete reusable implementation.
- `Missing`: no implementation found.

## Scope

Audit coverage spans:

- `domain/ecs`
- `domain/scheduler`
- `engine/src/runtime`
- `engine/src/plugins/net`
- `net/engine_net`
- `net/engine_net_quic`
- `net/engine_history`
- `domain/ui/ui_text`

## Strong Existing Foundations

### Execution and Scheduling

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Tick-first fixed-step loop | Implemented | `engine/src/runtime/fixed_step_executor.rs` (`run_fixed_update_frame`) | `SimulationTick` advances before each `FixedUpdate` step. |
| Explicit frame-vs-tick phases | Implemented | `engine/src/runtime/frame_lifecycle.rs` (`run_frame`) | Canonical order: `PreUpdate -> FixedUpdate -> Update -> RenderPrepare -> RenderSubmit -> FrameEnd`. |
| Deterministic schedule planning and stage ordering | Implemented | `domain/scheduler/src/plan.rs` (`ExecutionScheduler::build_plan`) | Set ordering + access conflicts drive deterministic stage plan. |
| Resource/component conflict analysis | Implemented | `domain/scheduler/src/access.rs` | Reads/writes for components/resources are conflict checked. |
| Runtime introspection of schedule plans | Implemented | `domain/ecs/src/system/runtime.rs` (`Runtime::plan_for`) | Stage list + conflict list available at runtime. |
| Runtime-owned per-system param state | Implemented | `domain/ecs/src/system/runtime.rs` + `domain/ecs/src/system/params.rs` | Param `State` is cached per registered system closure. |
| Stage-bound deferred command flush | Implemented | `domain/ecs/src/system/runtime.rs` (`flush_stage_commands`) | Structural mutations become visible only after stage boundary. |

### Event/Broadcast Baseline

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Typed world event channels | Implemented | `domain/ecs/src/world/events/dispatch.rs` | `emit/read/drain/clear` APIs are present. |
| Bounded/unbounded channels + overflow policies | Implemented | `domain/ecs/src/world/events/types.rs` | Capacity `Option<usize>`, overflow `DropOldest/DropNewest/Panic`. |
| Channel lifetimes | Partial | `domain/ecs/src/world/events/types.rs` | Supports `FrameTransient`, `Manual`, `Persistent`; no tick-scoped lifetime. |
| Unread-per-consumer cursors | Implemented | `domain/ecs/src/system/params.rs` (`EventChannelState`, `iter_new`) | Cursor tracked in runtime-owned system param state. |
| Cursor clamping across retention cleanup | Implemented | `domain/ecs/src/world/events/types.rs` (`events_ref_since`) | Cursor clamps to `[start_sequence, next_sequence]`. |
| Observer notifications + stats | Implemented | `domain/ecs/src/world/events/dispatch.rs` | `observe_events`, trigger notifications, `event_channel_stats`. |
| End-of-frame cleanup hook | Partial | `domain/ecs/src/world/events/dispatch.rs` (`finish_event_frame`) | Exists on `World`, but not called by engine runtime frame lifecycle by default. |

### Change Tracking and Structural Signals

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Component/resource change logs with ticks | Implemented | `domain/ecs/src/world/component/access.rs`, `domain/ecs/src/world/resource/access.rs` | `component_changes_since` / `resource_changes_since`. |
| Spawn/despawn notifications | Implemented | `domain/ecs/src/world/entity/lifecycle.rs` | Emits `EntitySpawnedEvent` and `EntityDespawnedEvent`. |
| Removed-component stage window queries | Implemented | `domain/ecs/src/query/orphaned.rs` + `begin_stage_command_flush` | `QueryOrphaned<T>` backed by current flush window records. |
| World change tick surface | Implemented | `domain/ecs/src/world/runtime.rs` (`current_change_tick`) | Tick-relative change checks supported. |

### Multiplayer/Runtime Bridge Foundations

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Runtime handle resource pattern | Implemented | `engine/src/plugins/net/resources.rs` (`NetworkRuntimeHandle`) | ECS resource wraps runtime command/event channels. |
| Bounded inbox/outbox and bridge queues | Implemented | `engine/src/plugins/net/resources.rs` | `Network*Inbox/Outbox` + `NetworkInboundQueue` / `NetworkOutboundQueue`. |
| Explicit receive/drain/flush lifecycle systems | Implemented | `engine/src/plugins/net/runtime_io.rs` | Receive in `PreUpdate`, flush in `FrameEnd`. |
| Connection-aware server outbox routing | Implemented | `engine/src/plugins/net/resources.rs` (`OutboundServerMessage`) | Supports `ToConnection` and `Broadcast`. |
| Snapshot baseline checkpoints per connection | Implemented | `engine/src/plugins/net/resources.rs` (`ConnectionBaselineCheckpoint`) | Tracks per-connection ack/sent/full checkpoint state. |

### Input/Prediction/Replication Foundations

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Generic replication/input driver contracts | Implemented | `net/engine_net/src/replication/driver.rs` | `ReplicationDriver`, `SnapshotApplyDriver`, `InputDriver`. |
| Authoritative remote input ingestion hook | Implemented | `InputDriver::receive_remote_input` + `engine/src/plugins/net/runtime_io.rs` | Server path decodes and forwards remote input frames. |
| Pending predicted input storage + replay on correction | Implemented | `engine/src/plugins/net/prediction.rs` | `PredictionState<TInput>.pending_frames` replayed after snapshot/delta apply. |
| Per-component replication metadata registry | Implemented | `net/engine_net/src/replication/model.rs` | `ReplicationRegistry`, `ReplicatedComponentDescriptor`. |
| Replication metadata macros | Implemented | `net/engine_net_macros/src/lib.rs` | `#[net_component]` and `#[net_entity]`. |
| Interest policy vocabulary | Implemented | `net/engine_net/src/replication/interest.rs` | Includes `Global/OwnerOnly/Spatial/Team/Distance`. |

### History/Replay Foundations

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Generic replay archive/recorder/controller substrate | Implemented | `net/engine_history/src/*` | Checkpointed archive, journal, controller, validation types. |
| Checkpoint retention/storage policies | Implemented | `net/engine_history/src/policy.rs` | `CheckpointPolicy`, `ReplayStoragePolicy`. |
| Validation mismatch model with hash checks | Implemented | `net/engine_history/src/validation/mod.rs` | Includes missing-checkpoint and tick-hash mismatch reports. |
| Engine replay plugin integration | Partial | `engine/src/plugins/replay.rs` | Integrated for scene simulation flow, not generic ECS rollback hooks. |

## Valuable Partial Foundations (Need Generalization)

| Capability | Status | Evidence | Why It Matters |
| --- | --- | --- | --- |
| Event messaging acts as both broadcast and workflow queue | Partial | `domain/ecs/src/world/events/*` | Works now, but conflates semantics needed for deterministic multiplayer/runtime bridging. |
| Queue primitives are plugin-local and Vec-backed | Partial | `engine/src/plugins/net/resources.rs` (`push_bounded`) | Useful immediately, but not reusable ECS queue abstraction (no ring/deque, no explicit backpressure metrics). |
| Input pipeline is driver-centric, not stream-registry-centric | Partial | `net/engine_net/src/replication/driver.rs`, `engine/src/plugins/net/prediction.rs` | Works for one driver type; lacks typed multi-stream registration + diagnostics contract. |
| Ownership/routing exists as connection-level convention | Partial | `engine/src/plugins/net/runtime_io.rs`, `engine/src/plugins/world/streaming/interest.rs` | Needed for real workloads, but not a reusable ECS ownership contract. |
| End-of-frame event finalization is not runtime-owned | Partial | `domain/ecs/src/world/events/dispatch.rs` vs `engine/src/runtime/frame_lifecycle.rs` | Leaves lifecycle cleanup correctness to call-site discipline. |

## MSDF Text: Current Inventory

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| MSDF atlas/glyph metric data contracts | Implemented | `domain/ui/ui_text/src/atlas.rs`, `domain/ui/ui_text/src/font.rs` | Domain types exist for atlas + glyph metrics. |
| Text layouter abstraction | Implied | `domain/ui/ui_text/src/layout.rs` (`TextLayouter`) | Trait exists; no implementation found. |
| Runtime/editor text shaping from atlas metrics | Missing | Search shows no `impl TextLayouter` | Current UI paths approximate glyphs with fixed advance. |
| Current runtime/editor glyph placement | Implemented (approximation) | `engine/src/plugins/scene/runtime/overlay_ui.rs`, `domain/editor/editor_shell/src/runtime/output/build_ui_frame.rs` | Uses `estimate_glyph_run` heuristics (not full MSDF shaping). |
| Render backend support for real MSDF sampling | Missing in current path | `engine/src/plugins/render/renderer/extract.rs` | Glyph runs are flattened into rects; no atlas sampling stage in this path. |

## Inventory Takeaway

The repository already has a strong deterministic runtime base, an event channel substrate, driver-based multiplayer contracts, and replay substrate pieces.

The largest foundational gap before scaling editor work is not “add more editor features”; it is finishing generic runtime messaging boundaries:

- split broadcast streams vs destructive queues vs typed input streams,
- make lifecycle/finalization runtime-owned (not manual),
- standardize ownership routing and change extraction contracts for replication/editor sync.
