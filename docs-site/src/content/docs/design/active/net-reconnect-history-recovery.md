---
title: "Net Reconnect and History Recovery Design"
description: "Current boundary between RunenNet recovery authority, Runenwerk reconnect policy, replication resynchronization, and replay/history."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Reconnect and History Recovery Design

## Purpose

This design separates RunenNet session/replication recovery semantics from Runenwerk host reconnect
policy, gameplay resynchronization/realization, and Runenwerk replay/history.

It must not conflate RunenNet recovery time with reconnect attempts, transport backoff, wall-clock
scheduling, replay history, or gameplay recovery policy.

## Current Boundary

Standalone RunenNet owns:

- participant membership, connection binding/loss, retention, replacement, expiry, removal, and
  session closure;
- client replication consistency/history/full-recovery state;
- authority replication baseline/history/recovery state and pending delivery evidence;
- prediction/reconciliation lineage associated with the accepted client replication state.

Runenwerk host/application integration owns:

- whether and when to attempt a reconnect;
- reconnect attempt counters, timing, deployment, endpoint, and concrete transport policy;
- presentation and diagnostics for reconnect state;
- downstream realization of accepted replicated products;
- deciding when product/gameplay state should be restored, resynchronized, or presented as
  degraded.

`engine_history` is an independent Runenwerk replay/history substrate. It does not own networking
lifecycle or RunenNet recovery semantics and is not implicitly wired into multiplayer reconnect.

## Implemented Substrate

Implemented now:

- RunenNet `Session` loss, retention, replacement, expiry, removal, and closure semantics consumed
  through `RunenNetSessionCore`;
- read-only Engine session projection and owner-routing reconciliation;
- host-owned reconnect attempt/error/health diagnostics;
- RunenNet-backed client and authority replication recovery/full-snapshot fallback;
- lifecycle integration that cancels pending authority work on retained loss, forces full recovery
  after replacement, and removes terminal replication lineages;
- Engine streaming state reconciled from accepted connection projections;
- `engine_history` archive, recorder, controller, checkpoint-policy, and validation-report
  primitives.

The former `engine_net` admission/handoff state machines and `engine_net_quic`
reconnect/runtime ownership are deleted.

The Editor ↔ Runtime Preview control channel is a separate maintained application consumer of
standalone RunenNet and `runen-net-quic`; it does not make Engine gameplay networking a QUIC
runtime.

## Partial Contracts

Partial now:

- multiplayer replication recovery relies on RunenNet full-snapshot/resynchronization paths where
  retained state cannot continue safely;
- generic ECS checkpoint capture/restore hooks are not a standardized multiplayer recovery
  contract;
- `engine_history` is not the default multiplayer reconnect recovery path;
- checkpoint-backed or rollback-oriented gameplay recovery remains separately evidence-gated;
- richer user-facing reconnect/recovery explanation remains product work.

## Ownership Rules

RunenNet owns reusable membership retention/replacement/expiry and replication/prediction recovery
semantics.

Runenwerk host/application policy owns reconnect scheduling and concrete transport choices. A
RunenNet recovery-time value is not reconnect backoff, retry timing, or a wall clock.

`engine_history` owns its replay/archive/checkpoint/validation semantics independently of
networking.

Engine/gameplay integration owns:

- downstream world/product realization after accepted network state;
- which gameplay state is recoverable;
- how reconnect, resynchronization, and correction are presented to users.

## Invariants

- Reconnect must not make clients authoritative over replicated state.
- Engine reconnect diagnostics are projections/policy, not session membership authority.
- Missing or unusable replication state must recover deterministically rather than silently apply
  partial state.
- Replay/history reports do not mutate gameplay state or become networking authority by
  observation.
- Transport reconnect policy must not redefine RunenNet retention/replacement semantics.
- RunenNet must not absorb Runenwerk ECS/history/gameplay policy.
- No replacement Engine transport runtime may be introduced without a maintained gameplay consumer
  and an explicitly owned product integration boundary.

## Future Work Constraints

Potential future work may include checkpoint-backed gameplay recovery, richer recovery diagnostics,
or transport-specific reconnect behavior for a real gameplay consumer. Those concerns require
current consumer evidence and their own owner; they are not unfinished RN8 migration slices.

This design does not authorize wiring `engine_history` into RunenNet, adding an Engine QUIC
runtime, or inventing final Replicated View authoring syntax.

## Validation Plan

For this boundary, validate as applicable:

- RunenNet session loss/retention/replacement/expiry tests through Engine integration;
- host reconnect diagnostics tests that do not mutate membership authority;
- client/authority replication recovery and full-resynchronization tests;
- `engine_history` replay/archive tests independently of transport;
- transport tests only in the maintained product consumer that owns that realization;
- repository canonical validation;
- documentation validation.
