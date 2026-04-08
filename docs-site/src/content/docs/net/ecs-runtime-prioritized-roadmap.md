---
title: "ECS Runtime Prioritized Roadmap (Pre-Editor Gate)"
description: "Implementation-ordered roadmap for ECS/runtime/network foundations required before substantial editor expansion."
---

# ECS Runtime Prioritized Roadmap (Pre-Editor Gate)

This roadmap is implementation-ordered and intentionally biased toward foundational runtime correctness before broader editor feature growth.

Audit date: 2026-04-08.

## Recommendation Summary

A major event-system redesign should happen now, before substantial additional editor work.

Reason:

- current messaging roles are conflated (broadcast and workflow semantics share one channel model),
- queue/input/stream behavior is split across plugin conventions rather than reusable ECS/runtime primitives,
- end-of-frame cleanup is not runtime-owned by default.

Without fixing this first, editor features will continue binding to unstable messaging/runtime seams and increase migration cost.

## Priority 1: Foundation (Do Before More Editor Feature Expansion)

### F1. Split messaging model in ECS/runtime core

Target outcome:

- explicit `BroadcastStream<T>` (fan-out, unread cursors),
- explicit `Queue<T>` (single-owner destructive drain),
- explicit `InputStream<T>` (tick-buffered command/input flow).

Primary ownership locations:

- `domain/ecs/src/world/events/*` (replace/reshape current event substrate),
- `domain/ecs/src/system/params.rs` (new params for stream/queue/input read/write/drain semantics),
- `domain/scheduler/src/access.rs` (access domains for stream read/write/drain conflicts).

Checklist:

- [ ] define separate storage/config types for broadcast streams and queues in ECS world APIs.
- [ ] preserve unread cursor behavior for broadcast streams with runtime-owned cursor state.
- [ ] add bounded queue primitive with explicit overflow and backpressure reporting.
- [ ] add deterministic tick-scoped input stream buffering contract.
- [ ] keep direct world/admin APIs for inspection, clear, and deterministic test control.

### F2. Make lifecycle/finalization runtime-owned

Primary ownership locations:

- `engine/src/runtime/frame_lifecycle.rs` (`run_frame`),
- `domain/ecs/src/system/runtime.rs` (schedule/run hooks if needed),
- `domain/ecs/src/world/events/dispatch.rs` (or replacement lifecycle endpoints).

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

### F4. Add first-class queue/stream conflict semantics to scheduler

Primary ownership locations:

- `domain/scheduler/src/access.rs`,
- `domain/ecs/src/system/runtime.rs` (`query_access_to_system_access` mapping).

Checklist:

- [ ] model broadcast read/write and queue destructive drain as explicit access domains.
- [ ] preserve parallelism where safe while preventing drain/read races.
- [ ] expose conflicts in plan introspection for tooling.

## Priority 2: Multiplayer-Enabling (After Foundation Split, Before Large Net/Editor Features)

### M1. Typed input stream registry and diagnostics

Primary ownership locations:

- new input-stream module in `domain/ecs` (recommended),
- integration points in `engine/src/plugins/net/prediction.rs` and `engine/src/plugins/net/runtime_io.rs`.

Checklist:

- [ ] register multiple input stream types.
- [ ] deterministic per-tick ordering and sequence metadata.
- [ ] optional dedup hooks.
- [ ] per-stream diagnostics (buffer size, dropped, replayed, acked).

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

- [ ] per-stream/per-queue/per-input-stream counters.
- [ ] consumer lag and backpressure counters.
- [ ] correction/replay counters tied to stream IDs.

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

## Editor Gating Guidance

Editor work can continue in limited form (UI composition, tooling UX, non-runtime presentation), but editor systems that depend on messaging/runtime semantics should wait until Priority 1 is complete.

Editor areas that should wait:

- new editor workflows relying on event-driven state propagation,
- editor multiplayer/session synchronization features,
- editor undo/redo or timeline features that expect stable stream/queue contracts.

## MSDF Text Assessment

### Should MSDF be added now?

Recommendation: add full MSDF text after Priority 1 foundation work, not before.

Why:

- current text path is approximate but serviceable for interim editor/runtime UI,
- event/runtime messaging and lifecycle redesign is the higher-risk architectural dependency,
- MSDF rollout will be cleaner once runtime boundaries and diagnostics are stabilized.

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

Full MSDF text is not a blocker for Priority 1 runtime/event redesign.

Full MSDF text should follow (or run in parallel late in Priority 2) after messaging/lifecycle foundations are stable.

## Concise Do-Next List

1. implement Priority 1 (messaging split + runtime-owned lifecycle + scheduler conflict semantics + stable IDs).
2. implement Priority 2 core multiplayer enablers (typed input streams, ownership routing, replication metadata expansion, change extraction, diagnostics).
3. resume substantial editor runtime-integrated feature expansion.
4. add full MSDF text integration on top of stabilized runtime contracts.
