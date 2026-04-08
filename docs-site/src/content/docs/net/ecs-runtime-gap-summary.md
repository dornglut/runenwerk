---
title: "ECS Runtime Gap Summary (Capability Model Cross-Check)"
description: "Implemented vs partial vs missing capability audit for ECS/runtime/multiplayer foundations."
---

# ECS Runtime Gap Summary (Capability Model Cross-Check)

This document maps the requested capability model to the current repository state.

Audit date: 2026-04-08.

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
- `Missing` tick/finalization hooks.
- `Partial` frame/finalization hooks (`FrameEnd` exists; event finalization is not runtime-owned by default).
- `Partial` configurable schedule barriers (set ordering exists; explicit barrier primitives do not).
- `Partial` conflict-aware scheduling for non-component resources and streams (resources yes; stream/queue domains not first-class).

## 2) Messaging model split

- `Partial` broadcast streams for fan-out notifications (event channels provide this behavior).
- `Partial` destructive queues for single-owner workflows (plugin inbox/outbox queues, not ECS core primitive).
- `Partial` typed input streams for commands (driver-oriented input flow, no stream registry).
- `Partial` observer/diagnostic hooks for instrumentation only (event observers exist).
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
- `Partial` end-of-frame cleanup (available on `World`; not executed automatically by runtime frame lifecycle).
- `Partial` scheduler access semantics for readers vs writers (read/write modeled; destructive-drain semantics not scheduler-modeled).

## 4) Queue features

- `Partial` bounded queue resources/types (implemented in net plugin resources, not ECS core queue type).
- `Missing` ring-buffer/deque-backed storage (current `Vec` + `remove(0)`).
- `Implemented` destructive drain semantics.
- `Partial` batch push/drain (drain-all exists; no explicit batch push API).
- `Partial` overflow strategies (drop-oldest convention in plugin queues).
- `Partial` backpressure reporting (warnings, not structured counters/errors).
- `Partial` queue occupancy metrics (`len`, but no standardized metrics resource).
- `Missing` producer/consumer ownership rules.
- `Partial` single-owner drain guarantees (by system convention only).
- `Missing` optional multi-priority queues.
- `Missing` optional per-tick drain limits.
- `Missing` optional queue aging/expiry policies.
- `Partial` queue debug inspection APIs (slice accessors exist in plugin resources).

## 5) Input stream features

- `Missing` typed input registration API.
- `Partial` per-tick input buffering.
- `Implemented` local input producer API (`InputDriver::take_local_input`).
- `Partial` simulation-facing input view API (driver `apply_input`; no reusable typed stream view param).
- `Implemented` authoritative remote input ingestion.
- `Partial` deterministic input ordering within a tick (vector order only; no canonical sort/sequence policy).
- `Partial` ownership-aware routing (connection-aware ingestion; no entity ownership router).
- `Implemented` pending predicted input storage.
- `Implemented` replay of pending local input after correction.
- `Partial` input acknowledgement support hooks (snapshot ack exists; no dedicated input-ack channel semantics).
- `Partial` input stream diagnostics (global prediction diagnostics only).
- `Partial` input history windows (pending predicted history only).
- `Missing` optional input deduplication.
- `Missing` optional input sequence/cursor metadata.
- `Missing` multiple registered input stream types.

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

- `Missing` generic owner contract.
- `Partial` connection/entity routing helpers (plugin-level connection routing and streaming-interest maps).
- `Partial` one-to-one ownership (interest checks can use optional owner).
- `Missing` one-to-many ownership support.
- `Missing` many-to-one grouping support.
- `Missing` transfer-of-ownership support.
- `Missing` query helpers for owned entities.
- `Missing` query helpers for local-owned entities.
- `Missing` route typed input to owned entities (generic routing layer).
- `Partial` support for no-owner/server-owned entities (`Option<ConnectionId>` patterns in session/interest paths).
- `Partial` support for spectators/controllers without entities (session model supports connections; no generic ownership role contract).
- `Implemented` no character/controller shape assumptions in core ECS/runtime API.

## 8) Change extraction and diff support

- `Partial` structural change feeds (spawn/despawn events + change logs).
- `Implemented` spawn notifications.
- `Implemented` despawn notifications.
- `Partial` component added feeds.
- `Partial` component removed feeds.
- `Implemented` changed-component extraction.
- `Implemented` tick-relative change queries.
- `Missing` frame-relative change queries.
- `Partial` diff-friendly world access.
- `Missing` batched structural delta generation hooks.
- `Partial` optional filtered change collection by component type (manual filtering over logs).
- `Missing` optional filtered change collection by ownership/interest.
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

- `Implemented` per-stream stats (event channels).
- `Partial` per-queue stats.
- `Missing` per-input-stream stats.
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

The highest-impact gap is the messaging/lifecycle seam:

1. event channels currently carry multiple semantic roles,
2. queue/input/stream contracts are split across plugin conventions rather than reusable ECS/runtime primitives,
3. end-of-frame event cleanup is not runtime-owned by default.

This is the primary reason a major event-system redesign should happen before substantial additional editor features are built on top of the current messaging substrate.
