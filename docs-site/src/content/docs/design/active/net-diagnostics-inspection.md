---
title: "Net Diagnostics and Inspection Design"
description: "Current diagnostics boundary for RunenNet lifecycle projections, retained replication/prediction state, and host-owned networking policy."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Diagnostics and Inspection Design

## Purpose

This design defines diagnostics and inspection surfaces for the current RunenNet/Runenwerk integration without allowing diagnostics, ECS resources, or presentation views to become a second source of networking truth.

## Current Authority

Standalone RunenNet owns connection/session lifecycle truth. Engine diagnostics may project successful RunenNet bindings and lifecycle outcomes, but they do not authorize admission, loss, retention, replacement, expiry, or closure.

Runenwerk host/application policy owns reconnect attempts, timing, deployment, and presentation. Those diagnostics must remain distinct from RunenNet retention/recovery semantics.

Retained Runenwerk replication/prediction state may continue to expose diagnostics while its maintained consumers remain.

## Implemented Substrate

Implemented now:

- retained `SnapshotDebugDump`, `DeltaDebugDump`, `EntityMapTrace`, and replication statistics;
- engine `NetworkDiagnostics`, `ReplicationDiagnostics`, `PredictionDiagnostics`, `ConnectionHealth`, `RoundTripMetrics`, and `NetDiagnosticsView`;
- `NetworkSessionStatus` as a read-only engine status/host-policy projection whose connected/count fields are synchronized from `RunenNetSessionProjection`;
- engine owner-routing state reconciled from RunenNet-authorized connection bindings;
- ECS messaging diagnostics for work queues and tick buffers;
- replay validation mismatch reports in `engine_history`;
- transport-specific diagnostics in the separately maintained transport consumers that actually own them.

The old engine `SessionPhase`, admission state machine, JoinAccepted projection, session runtime events, generic engine transport runtime, synthetic lane-route trace, and replication-runtime event vocabulary are not current diagnostics surfaces because those duplicate/dead authorities have been removed.

## Partial Contracts

Partial now:

- rejection reasons are counted coarsely in some retained replication layers;
- per-connection replication health remains split between engine checkpoint resources and aggregate diagnostics;
- interest decisions are not yet explainable per entity/component;
- queue pressure is warning-heavy and not yet a complete inspection model;
- replay validation does not include all retained network cursor/queue state.

## Ownership Rules

Diagnostics may observe:

- RunenNet-derived active connection/participant projections;
- host-owned reconnect attempts and errors;
- retained replication cursors, baselines, ACK/resync outcomes, prediction counters, and owner routing;
- runtime work queues;
- retained snapshot/delta payload shape;
- replay validation reports;
- transport events only at the concrete maintained transport consumer that produces them.

Diagnostics must not:

- mutate RunenNet membership/lifecycle state;
- infer admission from a presentation flag instead of RunenNet Core;
- recreate a session phase state machine;
- recreate retired lane/delivery semantics for inspection convenience;
- silently recover from protocol errors;
- hide missing baselines;
- become the only place where networking invariants are enforced.

## Inspection Surfaces

Useful current or future views may include:

- lifecycle projection: active RunenNet connection count/bindings plus host reconnect/error policy, without a duplicate admission/phase authority;
- replication view: latest cursor, per-connection baseline, last sent, last ACK, and resync reason;
- prediction view: pending frames, replayed count, corrected count;
- interest view: inclusion/exclusion reason per entity/component;
- delivery view only when backed by an actual RunenNet delivery consumer rather than retired engine lane labels;
- history view: checkpoint tick, hash, and mismatch cause;
- concrete transport view only in an actual transport-owning consumer.

## Invariants

- Diagnostics are observational/projection state, not networking semantic authority.
- Connected/active engine status is derived from accepted RunenNet bindings.
- Host reconnect counters do not redefine RunenNet retention/replacement semantics.
- Per-connection replication diagnostics use RunenNet connection identity.
- Transport/delivery diagnostics remain with real consumers rather than implying a generic engine transport runtime.

## Future Work Constraints

Potential future work includes structured rejection reasons, richer per-connection replication snapshots, interest explanations, queue pressure inspection, and history correlation. Those improvements must follow the owning boundary and must not be used to pre-authorize a later RN8 migration slice.

## Validation Plan

For the current boundary, validate as applicable:

- engine session-projection/diagnostics tests;
- retained replication and prediction diagnostics tests;
- owner-routing and connection-loss projection tests;
- replay validation tests;
- transport diagnostics tests only in the maintained transport consumer;
- repository canonical validation;
- docs validation.
