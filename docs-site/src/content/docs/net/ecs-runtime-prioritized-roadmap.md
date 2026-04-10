---
title: "ECS Runtime Prioritized Roadmap (Post-Runtime Rewrite)"
description: "Implementation-ordered roadmap for ECS/runtime/network convergence work after the runtime/network rewrite."
---

# ECS Runtime Prioritized Roadmap (Post-Runtime Rewrite)

This roadmap is implementation-ordered and intentionally biased toward remaining runtime convergence work.

Audit date: 2026-04-09.

## Recommendation Summary

This document is now partly historical.
The major messaging redesign has already landed in core ECS runtime (`Broadcast*`, `WorkQueue*`, `TickBuffer*`) with runtime-owned frame/tick finalization hooks.

Reason:

- the remaining high-impact gaps are extensibility and richer diagnostics, not naming convergence,
- some roadmap path references below still use pre-merge module paths and should be read as historical context.

Use this roadmap primarily to track remaining convergence work, not as a statement that the old event substrate is still current.

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

- [ ] define separate storage/config types for broadcast streams and queues in ECS world APIs.
- [ ] preserve unread cursor behavior for broadcast streams with runtime-owned cursor state.
- [ ] add bounded queue primitive with explicit overflow and backpressure reporting.
- [x] add deterministic tick-scoped tick-buffer buffering contract.
- [ ] keep direct world/admin APIs for inspection, clear, and deterministic test control.

### F2. Make lifecycle/finalization runtime-owned

Primary ownership locations:

- `engine/src/runtime/frame_lifecycle.rs` (`run_frame`),
- `domain/ecs/src/system/runtime.rs` (schedule/run hooks if needed),
- `domain/ecs/src/world/messaging/finalization.rs`.

Checklist:

- [ ] ensure frame finalization always executes exactly once per frame from runtime lifecycle.
- [ ] add explicit tick finalization boundary/hook for fixed-step phases.
- [ ] remove dependence on manual `world.finalize_frame_boundary()` calls in gameplay/editor flows.

### F3. Strengthen stable identity surfaces for runtime introspection

Primary ownership locations:

- `domain/scheduler/src/system.rs`,
- `domain/ecs/src/system/runtime.rs`.

Checklist:

- [ ] add stable system IDs separate from type-name strings.
- [ ] expose stable param-slot IDs for diagnostic tooling and stream cursor tracking.
- [ ] keep deterministic registration and plan reporting.

### F4. Add first-class work-queue/stream conflict semantics to scheduler

Primary ownership locations:

- `domain/scheduler/src/access.rs`,
- `domain/ecs/src/system/runtime.rs` (`query_access_to_system_access` mapping).

Checklist:

- [ ] model broadcast read/write and work-queue destructive drain as explicit access domains.
- [ ] preserve parallelism where safe while preventing drain/read races.
- [ ] expose conflicts in plan introspection for tooling.

## Priority 2: Multiplayer-Enabling

### M1. Typed tick-buffer registry and diagnostics

Primary ownership locations:

- `domain/ecs/src/world/messaging/tick_buffer.rs`,
- integration points in `engine/src/plugins/net/prediction.rs` and `engine/src/plugins/net/runtime_io.rs`.

Checklist:

- [x] register multiple tick-buffer types.
- [ ] deterministic per-tick ordering and sequence metadata.
- [ ] optional dedup hooks.
- [x] per-buffer diagnostics (buffer size, dropped, rejected, pending, sequence).

### M2. Generic ownership/routing contract

Primary ownership locations:

- new ownership module in `domain/ecs` or `net/engine_net` contract layer,
- integration in `engine/src/plugins/net/runtime_io.rs` and replication systems.

Checklist:

- [ ] generic owner model for entities/resources (including no-owner/server-owned).
- [ ] input routing helpers from connection/controller to owned targets.
- [ ] support spectator/controller roles without entity assumptions.
- [ ] ownership transfer support hooks.

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

- [ ] batched structural delta extraction API (spawn/despawn/upsert/remove style output).
- [ ] component-type filtering in extraction API.
- [ ] ownership/interest filtering hooks for replication and streaming.
- [ ] frame-relative and tick-relative extraction windows.

### M5. Diagnostics baseline for desync triage

Primary ownership locations:

- `domain/ecs/src/telemetry.rs`,
- net plugin diagnostics resources.

Checklist:

- [x] per-stream/per-queue/per-tick-buffer counters.
- [ ] consumer lag and backpressure counters.
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

The runtime/network rewrite is complete and editor MVP implementation can proceed now.
This roadmap is no longer a precondition gate for editor MVP sequencing.

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
- `engine/src/plugins/scene/runtime/overlay_ui.rs` (replace `estimate_glyph_run` path),
- `domain/editor/editor_shell/src/runtime/output/build_ui_frame.rs` (replace `estimate_glyph_run` path),
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
