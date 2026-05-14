---
title: "ECS Runtime Prioritized Roadmap (Post-Runtime Rewrite)"
description: "Implementation-ordered roadmap for ECS/runtime/network convergence work after the runtime/network rewrite."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-12
related_roadmaps:
  - ../workspace/sdf-first-execution-roadmap.md
---

# ECS Runtime Prioritized Roadmap (Post-Runtime Rewrite)

This roadmap is implementation-ordered and intentionally biased toward remaining runtime convergence work.

Audit date: 2026-05-05.

## Recommendation Summary

This document is now partly historical.
The major messaging redesign has already landed in core ECS runtime (`Broadcast*`, `WorkQueue*`, `TickBuffer*`) with runtime-owned frame/tick finalization hooks.
The canonical multiplayer replication implementation sequence now lives
in [multiplayer-replication-implementation-roadmap.md](multiplayer-replication-implementation-roadmap.md).

For current cross-track sequencing, the remaining ECS/runtime convergence work
feeds the [SDF-first execution roadmap](../workspace/sdf-first-execution-roadmap.md).
The old editor MVP sequencing notes below are historical; the active priority
is product jobs, query snapshots, deterministic barriers, and diagnostics that
support SDF-first formed products.

Reason:

- the remaining high-impact gaps are extensibility and richer diagnostics, not naming convergence,
- some roadmap path references below still use pre-merge module paths and should be read as historical context.

Use this roadmap primarily to track remaining convergence work, not as a statement that the old event substrate is still current or that editor MVP work is still the sequencing driver.
Use the active design package for networking architecture:

- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Plugin runtime bridge](../design/active/net-plugin-runtime-bridge.md)
- [Diagnostics and inspection](../design/active/net-diagnostics-inspection.md)

## Priority 1: Foundation

### F1. Split messaging model in ECS/runtime core

Target outcome:

- explicit `BroadcastStream<T>` (fan-out, unread cursors),
- explicit `WorkQueue<T>` (single-owner destructive drain),
- explicit `TickBuffer<T>` (tick-buffered command/input flow).

Terminology note:

- `WorkQueue<T>` and `TickBuffer<T>` are the current ECS runtime terms in code.

Primary ownership locations:

- `domain/ecs/src/world/messaging/*`,
- `domain/ecs/src/system/params.rs` (new params for stream/queue/input read/write/drain semantics),
- `domain/scheduler/src/access.rs` (access domains for stream read/write/drain conflicts).

Checklist:

- [x] define separate storage/config types for broadcast streams and queues in ECS world APIs.
- [x] preserve unread cursor behavior for broadcast streams with runtime-owned cursor state.
- [x] add bounded queue primitive with explicit overflow and backpressure reporting.
- [x] add deterministic tick-scoped tick-buffer buffering contract.
- [x] keep direct world/admin APIs for inspection, clear, and deterministic test control.

### F2. Make lifecycle/finalization runtime-owned

Primary ownership locations:

- `engine/src/runtime/frame_lifecycle.rs` (`run_frame`),
- `domain/ecs/src/system/runtime.rs` (schedule/run hooks if needed),
- `domain/ecs/src/world/messaging/finalization.rs`.

Checklist:

- [ ] ensure frame finalization always executes exactly once per frame from runtime lifecycle.
- [x] add explicit tick finalization boundary/hook for fixed-step phases.
- [ ] remove dependence on manual `world.finalize_frame_boundary()` calls in gameplay/editor flows.

Tick-buffer lifecycle contract:

- `World::finalize_tick_boundary(N)` closes all ticks `<= N`.
- For non-retained tick buffers, finalizing tick `N` purges all buffered messages for ticks
  `<= N`; gap finalization is therefore explicit and closes the whole prefix, not only
  the single named tick.
- Repeating finalization for an already-finalized tick is idempotent and does not rewind
  `current_buffer_tick` or increment tick-boundary counters again.
- A push for `tick <= finalized_buffer_tick` is rejected with
  `TickBufferPushError::FinalizedTick` and increments the buffer rejected counter; late
  writes are not silently dropped and do not remain pending.
- Retained tick buffers keep already-finalized messages for inspection, but finalized
  ticks are still closed to new writes.

### F3. Strengthen stable identity surfaces for runtime introspection

Primary ownership locations:

- `domain/scheduler/src/system.rs`,
- `domain/ecs/src/system/runtime.rs`.

Checklist:

- [x] add stable system IDs separate from type-name strings.
- [x] expose stable param-slot IDs for diagnostic tooling and stream cursor tracking.
- [ ] keep deterministic registration and plan reporting.

Current scope note: system and param-slot IDs are first-class runtime diagnostics
surfaces. Richer external identity for every scheduler access key remains constrained by
TypeId-backed domains, so conflict ordering uses deterministic registration order rather
than human-readable names as canonical identity.

### F4. Add first-class work-queue/stream conflict semantics to scheduler

Primary ownership locations:

- `domain/scheduler/src/access.rs`,
- `domain/ecs/src/system/runtime.rs` (`query_access_to_system_access` mapping).

Checklist:

- [x] model broadcast read/write and work-queue destructive drain as explicit access domains.
- [x] preserve parallelism where safe while preventing drain/read races.
- [x] expose conflicts in plan introspection for tooling.

## Priority 2: Multiplayer-Enabling

### M1. Typed tick-buffer registry and diagnostics

Primary ownership locations:

- `domain/ecs/src/world/messaging/tick_buffer.rs`,
- integration points in `engine/src/plugins/net/prediction.rs` and `engine/src/plugins/net/runtime_io.rs`.

Checklist:

- [x] register multiple tick-buffer types.
- [x] deterministic per-tick ordering and sequence metadata.
- [x] optional dedup hooks.
- [x] per-buffer diagnostics (buffer size, dropped, rejected, pending, sequence).

### M2. Generic ownership/routing contract

Primary ownership locations:

- new ownership module in `domain/ecs` or `net/engine_net` contract layer,
- integration in `engine/src/plugins/net/runtime_io.rs` and replication systems.

Checklist:

- [x] generic owner model for entities/resources (including no-owner/server-owned).
- [x] input routing helpers from connection/controller to owned targets.
- [x] support spectator/controller roles without entity assumptions.
- [x] ownership transfer support hooks.

### M3. Replication metadata registry expansion

Primary ownership locations:

- `net/engine_net/src/replication/model.rs`,
- `net/engine_net_macros/src/lib.rs`.

Checklist:

- [ ] keep current component registry and expand with clearer sync/prediction semantics.
- [ ] add optional entity/resource metadata registration pathways.
- [ ] expose runtime metadata retrieval APIs for extractors and tools.

### M4. Standardized change extraction pipeline

Primary ownership locations:

- `domain/ecs/src/world/*` change tracking modules,
- replication bridge modules consuming change deltas.

Checklist:

- [x] batched structural delta extraction API (spawn/despawn/upsert/remove style output).
- [x] component-type filtering in extraction API.
- [x] ownership/interest filtering hooks for replication and streaming.
- [x] frame-relative and tick-relative extraction windows.

### M5. Diagnostics baseline for desync triage

Primary ownership locations:

- `domain/ecs/src/telemetry.rs`,
- net plugin diagnostics resources.

Checklist:

- [x] per-stream/per-queue/per-tick-buffer counters.
- [x] consumer lag and backpressure counters.
- [x] correction/replay counters in net diagnostics resources (`PredictionDiagnostics` / `ReplicationDiagnostics`).

## Priority 3: Advanced / Later

### A1. History/checkpoint/rollback formalization

- [ ] generic ECS checkpoint/snapshot hooks (not scene-only).
- [ ] selective history policies by component/resource type.
- [ ] rollback application boundaries for runtime phases.

### A2. Replay and validation hardening

- [ ] generalized world hash/checksum hooks.
- [ ] richer mismatch reports (stream cursors, queue states, ownership state).

### A3. Extensibility upgrades

- [ ] pluggable queue/storage policies.
- [ ] pluggable stream retention policies.
- [ ] explicit codec/delta extension registry (beyond trait overrides).

## Editor Sequencing Guidance

The runtime/network rewrite is complete and editor MVP implementation is historical.
Remaining ECS runtime convergence is now relevant because it feeds the
SDF-first execution roadmap: lifecycle finalization, deterministic plan
reporting, conflict diagnostics, consumer lag/backpressure counters, and
runtime inspection surfaces support product jobs, query snapshots, replay, and
network authority.

## MSDF Text Assessment

### Should MSDF be added now?

Recommendation: for the editor MVP, treat readable MSDF text as an early bring-up blocker.

Why:

- current editor panel text is not a usable text path,
- current renderer still flattens glyph runs into rect proxies,
- readable panel text is required before deeper editor interaction milestones.

### Prerequisites

- stable frame/tick lifecycle finalization boundaries,
- clear ownership of runtime text update flow (avoid coupling to legacy event semantics),
- deterministic render input contract for glyph runs.

### Integration approach

Primary ownership locations:

- `domain/ui/ui_text/src/layout.rs` (`TextLayouter` implementation),
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs` (UI frame glyph run emission),
- `engine/src/plugins/render/renderer/extract.rs` (move from glyph-as-rect approximation to atlas-backed glyph rendering path).

Incremental steps:

1. implement atlas-backed layouter in `ui_text` using `MsdfFontAtlas` metrics,
2. switch runtime/editor glyph run builders to layouter output,
3. add renderer path for MSDF atlas sampling and glyph quads,
4. keep old approximate path behind a fallback toggle until parity is validated.

### Blocking status

Full MSDF text is a blocker for the first editor MVP interaction slice.

## Concise Do-Next List

1. continue Priority 1 and 2 runtime convergence work where still open.
2. execute editor MVP in implementation order: stale reference cleanup, text bring-up, viewport scene bring-up, picking producer + debug instrumentation, then interaction slice expansion.
