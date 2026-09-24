---
title: "Net Reconnect and History Recovery Design"
description: "Current boundary between RunenNet session recovery semantics, Runenwerk reconnect policy, retained replication recovery, and history substrate."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Reconnect and History Recovery Design

## Purpose

This design defines the current separation between RunenNet session recovery semantics, Runenwerk host reconnect policy, retained replication resynchronization, and Runenwerk replay/history substrate.

It must not conflate RunenNet retention/recovery time with reconnect attempts, transport backoff, wall-clock scheduling, or gameplay recovery policy.

## Current Boundary

Standalone RunenNet Core owns:

- participant membership;
- connection binding and loss;
- retention for recovery;
- replacement binding;
- recovery-time advancement and membership expiry;
- session closure.

Runenwerk host/application integration owns:

- whether and when to attempt a reconnect;
- reconnect attempt counters, timing, deployment, endpoint, and transport policy;
- presentation and diagnostics for reconnect state;
- deciding when product/gameplay state should be restored or resynchronized.

Retained Runenwerk replication integration owns current baseline/checkpoint and full-snapshot fallback behavior until a later authorized RN8 cut disposes that residue.

`engine_history` remains an independent Runenwerk replay/history substrate. It does not own networking lifecycle and is not implicitly wired into RunenNet session recovery.

## Implemented Substrate

Implemented now:

- RunenNet `Session` membership loss, retention, replacement, expiry, removal, and closure semantics consumed by engine integration;
- engine `RunenNetSessionCore` placement/invocation of public RunenNet Core owners;
- engine read-only session projection and owner-routing reconciliation;
- host-owned reconnect attempt/error/health diagnostics;
- per-connection retained replication checkpoints and full-snapshot fallback for unusable baselines;
- engine streaming state with per-connection cursor/resync markers;
- `engine_history` archive, recorder, controller, checkpoint policy, and validation-report primitives.

The former `engine_net` admission/handoff state machines and `engine_net_quic` reconnect/runtime ownership are not part of the current engine architecture.

The already-migrated runtime-preview control channel is a separate maintained RunenNet transport consumer from RN8 N1; it does not make the engine lifecycle boundary a QUIC runtime.

## Partial Contracts

Partial now:

- retained replication recovery is primarily full-snapshot/resync based;
- generic ECS checkpoint capture/restore hooks are not standardized;
- `engine_history` is not the default multiplayer reconnect recovery path;
- selective history by component/resource type remains future work;
- rollback application boundaries remain separately sequenced.

## Ownership Rules

RunenNet owns reusable session retention/replacement/expiry semantics.

Runenwerk host/application policy owns reconnect scheduling and concrete transport choices. A `RecoveryTime` value supplied to RunenNet is not reconnect backoff, retry timing, or a wall clock.

`engine_history` owns:

- replay archives;
- checkpoints;
- journal frames;
- validation reports.

Engine/gameplay integration owns:

- when to restore a world checkpoint;
- which gameplay state is recoverable;
- whether retained replication state falls back to full resync;
- how reconnect/correction is presented to users.

## Invariants

- Reconnect must not make clients authoritative over replicated state.
- Engine reconnect diagnostics are projections/policy, not session membership authority.
- Unknown or unusable retained replication baselines recover deterministically rather than silently applying partial state.
- Replay/history validation reports explain divergence; they do not mutate gameplay state by themselves.
- Transport reconnect policy must not redefine RunenNet retention/replacement semantics.
- RunenNet must not absorb Runenwerk ECS/history/gameplay policy.
- No replacement engine transport runtime may be introduced without a maintained consumer and an explicitly authorized boundary.

## Future Work Constraints

Potential future work may include checkpoint-backed multiplayer recovery, richer recovery diagnostics, or transport-specific reconnect behavior for a real consumer. Those concerns must be derived from live authority when their RN8 boundary is authorized.

This design does not authorize wiring `engine_history` into RunenNet, adding an engine QUIC runtime, or selecting the next RN8 slice.

## Validation Plan

For the current boundary, validate as applicable:

- RunenNet Core admission/loss/retention/replacement/expiry tests through engine integration;
- host reconnect diagnostics tests that do not mutate membership authority;
- retained per-connection baseline/full-resync tests;
- `engine_history` replay/archive tests independently of transport;
- repository canonical validation;
- documentation validation.
