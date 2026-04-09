---
title: "ECS Runtime Gap Summary (Capability Model Cross-Check)"
description: "Implemented vs partial vs missing capability audit for ECS/runtime/multiplayer foundations."
---

# ECS Runtime Gap Summary (Capability Model Cross-Check)

This document maps the requested capability model to the current repository state.

Audit date: 2026-04-09.

Terminology alignment update: 2026-04-09 (`Queue*` -> `WorkQueue*` and legacy tick-buffer names -> `TickBuffer*` in ECS runtime code).

Status labels:

- `Implemented`: present as reusable behavior now.
- `Partial`: present with important limits (or only at plugin level, not core ECS/runtime).
- `Implied`: architecture suggests direction, but concrete reusable API is absent.
- `Missing`: not implemented.

## 1) Execution and schedule model

- `Implemented` tick-first simulation model.
- `Partial` explicit fixed-step authority phase (`FixedUpdate` exists; authority split is plugin policy, not first-class schedule contract).
- `Implemented` explicit frame phase vs tick phase separation.
- `Implemented` deterministic schedule ordering.
- `Partial` stable system identity (string/type-based names only; no explicit stable runtime system ID).
- `Partial` stable param-slot identity (param state is tuple-slot based but no external slot identity API).
- `Implemented` runtime-owned per-system state.
- `Implemented` schedule phase introspection (`Runtime::plan_for`).
- `Implemented` tick finalization hooks (`fixed_step_executor` calls `world.finalize_tick_boundary` per simulated tick).
- `Implemented` frame finalization hooks (`run_frame` calls `world.finalize_frame_boundary` after `FrameEnd`).
- `Partial` configurable schedule barriers (set ordering exists; explicit barrier primitives do not).
- `Implemented` conflict-aware scheduling for non-component resources and messaging primitives (resource, broadcast, work-queue, tick-buffer domains with read/write/drain conflict modes).

## 2) Messaging model split

- `Implemented` broadcast streams for fan-out notifications (`Broadcast*` in ECS world messaging).
- `Implemented` destructive work queues for single-owner workflows (`WorkQueue*` in ECS world messaging).
- `Implemented` typed tick buffers for per-tick command/input flow (`TickBuffer*` registry in ECS world messaging).
- `Implemented` observer/diagnostic hooks for messaging streams (`observe_broadcast`, `messaging_diagnostics_snapshot`).
- `Missing` explicit gameplay messaging vs runtime/transport messaging split in ECS core.
- `Missing` explicit local messaging vs replicated/network-visible messaging split.

## 3) Broadcast stream features

- `Implemented` unread-per-consumer delivery.
- `Implemented` runtime-owned consumer cursors.
- `Implemented` multiple independent consumers.
- `Implemented` no global removal on read.
- `Implemented` bounded/unbounded capacity.
- `Implemented` overflow policies.
- `Partial` retention policies (manual/frame/persistent only).
- `Missing` tick-scoped lifetime.
- `Implemented` frame-scoped lifetime (via `FrameTransient` + frame cleanup hook call).
- `Implemented` manual lifetime.
- `Implemented` persistent lifetime.
- `Implemented` world/admin inspection APIs.
- `Implemented` direct emit APIs.
- `Implemented` direct clear/drain APIs for tools/tests.
- `Implemented` per-stream stats.
- `Missing` consumer lag metrics.
- `Implemented` dropped message counters.
- `Missing` end-of-tick cleanup.
- `Implemented` end-of-frame cleanup (executed from runtime frame lifecycle).
- `Implemented` scheduler access semantics for readers/writers/drainers.

## 4) WorkQueue features

- `Implemented` bounded work-queue types in ECS core (`WorkQueueConfig.capacity`).
- `Implemented` deque-backed storage (`VecDeque` in `work_queue.rs`).
- `Implemented` destructive drain semantics.
- `Partial` batch push/drain (drain-all exists; no explicit batch push API).
- `Partial` overflow strategies (backpressure reject is implemented; alternate overflow policies are not).
- `Implemented` structured backpressure reporting (`WorkQueueEnqueueError::Backpressure` + counters).
- `Implemented` queue occupancy/flow metrics (`WorkQueueStats`, diagnostics snapshot).
- `Missing` producer/consumer ownership rules.
- `Implemented` single-owner drain conflict modeling in scheduler access domains.
- `Missing` optional multi-priority queues.
- `Missing` optional per-tick drain limits.
- `Missing` optional queue aging/expiry policies.
- `Implemented` queue debug inspection APIs (`work_queue_iter`, `work_queue_peek`, stats/diagnostics).

## 5) Tick buffer features

- `Implemented` typed tick-buffer registration/configuration API (`ensure_tick_buffer`, `configure_tick_buffer`).
- `Implemented` per-tick tick-buffer buffering.
- `Implemented` local input producer API (`InputDriver::take_local_input`).
- `Implemented` simulation-facing typed tick-buffer view params (`TickBufferReader`, `TickBufferDrainer`).
- `Implemented` authoritative remote input ingestion.
- `Implemented` deterministic insertion order within a tick + global per-buffer sequence metadata.
- `Partial` ownership-aware routing (connection-aware ingestion; no entity ownership router).
- `Implemented` pending predicted input storage.
- `Implemented` replay of pending local input after correction.
- `Partial` input acknowledgement support hooks (snapshot ack exists; no dedicated input-ack channel semantics).
- `Implemented` per-buffer diagnostics (`TickBufferStats`, `TickBufferDiagnosticsSnapshot`).
- `Partial` input history windows (pending predicted history only).
- `Implemented` optional input deduplication hooks.
- `Implemented` optional input sequence metadata (`TickBufferMeta.sequence` + buffer key).
- `Implemented` multiple registered tick-buffer types.

## 6) Replication-facing ECS metadata

- `Implemented` component-level replication metadata.
- `Implemented` authority policy.
- `Partial` sync policy (encoded indirectly through profile presets).
- `Partial` prediction policy (profile + owner prediction flag).
- `Implemented` interest/relevancy policy.
- `Implemented` transport/profile policy.
- `Implemented` direction policy.
- `Implemented` owner-prediction flag.
- `Implemented` registration-based component replication registry.
- `Implemented` opt-in replicated component contract.
- `Partial` optional entity-level replication metadata (`NetEntity` marker + id map, not richer metadata registry).
- `Missing` optional resource-level replication metadata.
- `Partial` generic runtime metadata retrieval (component-descriptor lookup exists; no broader ECS metadata query model).

## 7) Ownership and routing

- `Implemented` generic owner contract (`OwnerState`, `OwnershipTarget`, `OwnershipRegistry`).
- `Implemented` connection/controller-to-target routing helpers (`NetworkControllerRouting` + `World::route_controller_*`).
- `Implemented` one-to-one ownership (each target has exactly one owner state).
- `Implemented` one-to-many ownership support (controllers own sets of targets).
- `Missing` many-to-one grouping support.
- `Implemented` transfer-of-ownership support (`assign_*`, `transfer_*`, transfer log).
- `Implemented` query helpers for owned entities/resources/targets.
- `Partial` query helpers for local-owned entities (controller-scoped helpers exist; no dedicated local-player abstraction in ECS core).
- `Missing` route typed input to owned entities (generic routing layer).
- `Implemented` support for no-owner/server-owned entities/resources (`OwnerState::NoOwner` / `OwnerState::ServerOwned`).
- `Implemented` support for spectators/controllers without entities (`ControllerRole` + controller registry).
- `Implemented` no character/controller shape assumptions in core ECS/runtime API.

## 8) Change extraction and diff support

- `Implemented` structural change feeds (`component_change_log` / `resource_change_log` + extraction API).
- `Implemented` spawn notifications.
- `Implemented` despawn notifications.
- `Implemented` component added feeds.
- `Implemented` component removed feeds.
- `Implemented` changed-component extraction.
- `Implemented` tick-relative change queries.
- `Implemented` frame-relative change queries.
- `Partial` diff-friendly world access.
- `Implemented` batched structural delta generation hooks (`StructuralDeltaBatch`).
- `Implemented` optional filtered change collection by type keys.
- `Implemented` optional filtered change collection by ownership/interest.
- `Partial` support for replication/state sync/editor/save systems (possible via logs or custom drivers, not standardized extraction API).

## 9) History, replay, and rollback support

- `Partial` optional world checkpoint hooks (scene replay plugin path, not generic ECS hook contract).
- `Partial` optional component snapshot hooks.
- `Partial` optional history buffers by tick.
- `Implemented` checkpoint tagging.
- `Implemented` state-history pruning policies.
- `Partial` replay-friendly deterministic consumption state.
- `Implemented` validation hooks.
- `Partial` world hash/checksum hooks (scene codec hash available; no generic ECS world hash hook).
- `Partial` rollback application boundaries.
- `Missing` selective history for chosen components/resources.
- `Partial` tooling APIs for replay inspection.

## 10) Runtime boundary integration

- `Implemented` standard bounded inbox resources.
- `Implemented` standard bounded outbox resources.
- `Implemented` typed inbound/outbound bridge queues.
- `Implemented` generic runtime-handle resource pattern.
- `Implemented` drain/flush lifecycle hooks.
- `Implemented` queue-to-runtime bridge helpers.
- `Implemented` runtime-to-queue bridge helpers.
- `Partial` backpressure/error handling hooks.
- `Implemented` diagnostics for flushed/received counts.

## 11) Observability and diagnostics

- `Implemented` per-stream stats (broadcast/work-queue/tick-buffer messaging diagnostics).
- `Implemented` per-work-queue stats.
- `Implemented` per-tick-buffer stats.
- `Partial` overflow/drop counters.
- `Missing` consumer lag metrics.
- `Implemented` schedule/tick timing metrics.
- `Implemented` correction/replay metrics.
- `Partial` replication/change extraction metrics.
- `Partial` debug taps.
- `Partial` observer notifications for infra events.
- `Partial` inspection APIs for current buffered contents.
- `Partial` tracing hooks.
- `Implemented` optional debug dumps for snapshot/delta payload structure (`engine_net` diagnostics).

## 12) Ergonomics and API shape

- `Implemented` concise gameplay-facing params.
- `Implemented` explicit world/admin APIs.
- `Partial` registration-driven configuration.
- `Partial` derive/attribute-based metadata.
- `Implemented` convention-based defaults.
- `Implemented` composable traits/registries.
- `Partial` batch APIs.
- `Missing` zero need for custom low-level drivers in common multiplayer cases.
- `Implemented` low-level escape hatches for special cases.
- `Implemented` no gameplay-specific assumptions in ECS core API.

## 13) Safety and correctness features

- `Partial` deterministic delivery semantics.
- `Partial` explicit lifecycle boundaries.
- `Implemented` no hidden cleanup (cleanup is explicit where present).
- `Implemented` stable cursor clamping across retention cleanup.
- `Partial` scheduler conflict modeling for streams/queues.
- `Partial` clear read vs write vs destructive-drain access semantics.
- `Implemented` bounded-memory options.
- `Implemented` explicit error surfaces for invalid baseline/history conditions.
- `Partial` tool-visible state for debugging desyncs.

## 14) Extensibility layer

- `Implemented` custom replication drivers as escape hatch.
- `Implemented` custom delta/snapshot codecs.
- `Missing` custom queue implementations as a first-class ECS/runtime extension point.
- `Partial` custom interest/ownership resolvers.
- `Implemented` custom history policies.
- `Partial` custom stream retention policies.
- `Partial` plugin-extensible schedule phases (schedule labels extensible, frame runner ordering fixed).
- `Partial` runtime registries instead of hardcoded feature sets.

## 15) What should not be ECS core features

Current codebase alignment is good here:

- `Implemented` no character-controller-specific multiplayer logic in ECS core.
- `Implemented` no movement-specific foundation abstractions in ECS core.
- `Implemented` transport protocol details are outside ECS core.
- `Implemented` generic ECS events are not direct wire-message models.
- `Implemented` no mandatory game-specific replication presets in ECS core API.

## Major Gap Conclusion

The highest-impact remaining gap is no longer naming convergence; that work is complete.

Current primary gap focus:

1. `WorkQueue*` extensibility (priority, aging, custom policies) is still intentionally minimal,
2. richer consumer-lag/inspection diagnostics remain partial in several runtime layers.

The current substrate now has world-owned messaging primitives with aligned naming (`Broadcast*`, `WorkQueue*`, `TickBuffer*`) and runtime-owned frame/tick finalization.
