---
title: "Net Diagnostics and Inspection Design"
description: "Design for networking diagnostics, replication inspection, and desync triage surfaces."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Diagnostics and Inspection Design

## Purpose

This design defines diagnostics and inspection surfaces for networking,
replication, prediction, transport, and recovery without making
diagnostics a second source of truth.

## Implemented Substrate

Implemented now:

- `SnapshotDebugDump` and `DeltaDebugDump`.
- `LaneRouteTrace`.
- `EntityMapTrace`.
- `ReplicationStats` in `engine_net`.
- Engine plugin `NetworkDiagnostics`, `ReplicationDiagnostics`,
  `PredictionDiagnostics`, `ConnectionHealth`, `RoundTripMetrics`, and
  `NetDiagnosticsView`.
- ECS messaging diagnostics for work queues and tick buffers.
- Replay validation mismatch reports in `engine_history`.
- QUIC runtime event and error surfaces.

## Partial Contracts

Partial now:

- Rejection reasons are counted coarsely in some layers.
- Per-connection replication health is split between engine plugin
  checkpoint resources and aggregate diagnostics.
- Interest decisions are not yet explainable per entity/component.
- Queue and lane backpressure is warning-heavy and not yet exposed as a
  complete inspection model.
- Replay validation does not yet include all network cursor and queue
  state.

## Ownership Rules

Diagnostics may observe:

- runtime queues;
- snapshot/delta payload shape;
- lane routing;
- ACK, resync, and correction counters;
- session and transport events;
- replay validation reports.

Diagnostics must not:

- mutate authoritative gameplay state;
- silently recover from protocol errors;
- hide missing baselines;
- become the only place where protocol invariants are enforced.

## Inspection Surfaces

Recommended long-term inspection views:

- session view: phase, connection, admission, reconnect status;
- replication view: latest cursor, per-connection baseline, last sent,
  last ACK, resync reason;
- prediction view: pending frames, replayed count, corrected count;
- interest view: inclusion/exclusion reason per entity/component;
- transport view: lane, delivery guarantee, dropped/backpressure counts;
- history view: checkpoint tick, hash, mismatch cause.

## Future Work

Future work:

1. Add structured rejection reason enums.
2. Add per-connection replication diagnostics snapshots.
3. Add interest explanation traces.
4. Add queue/lane pressure inspection APIs.
5. Include stream cursors and ownership state in replay mismatch reports.
6. Add docs for standard desync triage workflow.

## Validation Plan

Required validation:

- unit tests for counters and debug dumps;
- engine plugin diagnostics view tests;
- replay validation tests;
- docs validation.
