---
title: "ECS Runtime Feature Inventory (April 2026)"
description: "Current implemented ECS/runtime/network foundations relevant to multiplayer and editor sequencing."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---

# ECS Runtime Feature Inventory (April 2026)

This inventory reflects the code currently in the repository (audit date: 2026-04-09).

Terminology alignment update (2026-04-09):

- ECS runtime queue surfaces use `WorkQueue*` naming.
- ECS runtime tick-buffer surfaces use `TickBuffer*` naming.

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

### Messaging/Broadcast Baseline

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| Typed world broadcast streams | Implemented | `domain/ecs/src/world/messaging/broadcast.rs` | `publish/read/drain/clear` APIs are present. |
| Bounded/unbounded streams + overflow policies | Implemented | `domain/ecs/src/world/messaging/broadcast.rs` | Capacity `Option<usize>`, overflow `DropOldest/DropNewest/Panic`. |
| Broadcast lifetimes | Partial | `domain/ecs/src/world/messaging/broadcast.rs` | Supports `FrameTransient`, `Manual`, `Persistent`; no tick-scoped lifetime. |
| Unread-per-consumer cursors | Implemented | `domain/ecs/src/system/params.rs` (`BroadcastReaderState`, `iter_new`) | Cursor tracked in runtime-owned system param state. |
| Cursor clamping across retention cleanup | Implemented | `domain/ecs/src/world/messaging/broadcast.rs` (`messages_ref_since`) | Cursor clamps to `[start_sequence, next_sequence]`. |
| Observer notifications + stats | Implemented | `domain/ecs/src/world/messaging/broadcast.rs`, `domain/ecs/src/world/messaging/diagnostics.rs` | `observe_broadcast`, trigger notifications, and diagnostics snapshots are available. |
| End-of-frame cleanup hook | Implemented | `domain/ecs/src/world/messaging/finalization.rs`, `engine/src/runtime/frame_lifecycle.rs` | Runtime frame lifecycle executes messaging frame finalization. |

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
| Runtime messaging split exists with aligned primitive vocabulary | Implemented | `domain/ecs/src/world/messaging/broadcast.rs`, `domain/ecs/src/world/messaging/work_queue.rs`, `domain/ecs/src/world/messaging/tick_buffer.rs` | Broadcast/work-queue/tick-buffer semantics are split in core ECS runtime with final naming alignment. |
| WorkQueue primitive has core ECS storage but minimal policy surface | Partial | `domain/ecs/src/world/messaging/work_queue.rs` | Reusable core primitive exists (`VecDeque`, capacity/backpressure, stats/diagnostics), but no pluggable overflow/priority/aging policies yet. |
| TickBuffer pipeline is typed and registry-backed with neutral provenance | Implemented | `domain/ecs/src/world/messaging/tick_buffer.rs`, `engine/src/plugins/net/prediction.rs`, `engine/src/plugins/net/runtime_io.rs` | Supports typed registration, per-tick buffering, dedup, sequence metadata, and opaque provenance in ECS core; net/prediction map domain-specific meaning externally. |
| Ownership registry exists, but typed input-to-owned-target routing is still plugin policy | Partial | `domain/ecs/src/world/ownership/*.rs`, `engine/src/plugins/net/runtime_io.rs` | ECS owns controller/entity/resource ownership and routing queries; per-target input dispatch policy is still integration-layer logic. |
| Broadcast retention policy surface remains narrow | Partial | `domain/ecs/src/world/messaging/broadcast.rs` | Current lifetime modes are useful but there is no tick-scoped lifetime or custom retention policy extension point yet. |

## MSDF Text: Current Inventory

| Capability | Status | Evidence | Notes |
| --- | --- | --- | --- |
| MSDF atlas/glyph metric data contracts | Implemented | `domain/ui/ui_text/src/atlas.rs`, `domain/ui/ui_text/src/font.rs` | Domain types exist for atlas + glyph metrics. |
| Text layouter abstraction | Implemented | `domain/ui/ui_text/src/layout.rs` (`TextLayouter`, `AtlasTextLayouter`) | Trait and atlas-backed implementation exist. |
| Runtime/editor text shaping from atlas metrics | Implemented in UI runtime output | `domain/ui/ui_runtime/src/output/build_ui_frame.rs` | UI frame construction uses `AtlasTextLayouter` and `FontAtlasSource`. |
| Current runtime/editor glyph placement | Implemented | `domain/ui/ui_runtime/src/output/build_ui_frame.rs` | Glyph runs are emitted from atlas metrics before render extraction. |
| Render backend support for real MSDF sampling | Missing in current path | `engine/src/plugins/render/renderer/extract.rs` | Glyph runs are flattened into rects; no atlas sampling stage in this path. |

## Inventory Takeaway

The repository already has a strong deterministic runtime base, world-owned messaging primitives, driver-based multiplayer contracts, and replay substrate pieces.

The largest remaining runtime gaps are convergence and extension quality in messaging, ownership routing, and extraction surfaces.
These gaps no longer block the first editor MVP from proceeding.

For first editor MVP sequencing, the immediate blockers are editor-side:

- readable text bring-up (MSDF path),
- live viewport picking result production,
- stale reference cleanup in planning/docs.

Runtime follow-up priorities in this document remain:

- keep broadcast streams vs destructive work queues vs typed tick buffers semantically distinct,
- make lifecycle/finalization runtime-owned (not manual),
- continue standardizing ownership routing and change extraction contracts for replication/editor sync.
