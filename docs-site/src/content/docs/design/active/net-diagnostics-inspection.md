---
title: "Net Diagnostics and Inspection Design"
description: "Current diagnostics boundary for RunenNet-backed networking projections and Runenwerk host/integration policy."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Diagnostics and Inspection Design

## Purpose

This design defines diagnostics and inspection surfaces for the current RunenNet/Runenwerk
integration without allowing diagnostics, ECS resources, queues, or presentation views to become a
second source of networking truth.

## Current Authority

Standalone RunenNet owns session, replication, delivery, recovery, input-admission, and prediction
truth. Engine diagnostics may project accepted RunenNet outcomes; they do not authorize them.

Runenwerk host/application policy owns reconnect attempts, deployment, presentation, and
integration-specific observation. Those diagnostics remain distinct from RunenNet semantic state.

## Implemented Substrate

Implemented now:

- `NetworkDiagnostics` for Engine-facing connection/processing/flush counters;
- `ReplicationDiagnostics` for Engine integration observations around snapshot/input/ACK work;
- `PredictionDiagnostics` for fixed-step/replay/correction integration observations;
- `NetworkSessionStatus` and `ConnectionHealth` as Engine status/host-policy projections;
- `RoundTripMetrics` and `NetDiagnosticsView` as presentation-oriented projections;
- `RunenNetSessionProjection` as the read-only binding source used to synchronize connected/count
  status and owner routing;
- current-frame `NetworkInboundQueue` / `NetworkOutboundQueue` projections plus bounded pending
  inbox/outbox lengths;
- transport-specific diagnostics only in maintained applications/adapters that actually own the
  transport.

The deleted Engine session phase/admission state machine, replication-runtime events, synthetic
lane-route diagnostics, and obsolete debug-dump/trace types are not current inspection authority.

## Partial Contracts

Partial now:

- several rejection/failure classes are still aggregated rather than exposed as structured
  per-connection explanations;
- richer per-participant replication/desync state is not yet a unified inspection product;
- interest/relevancy decisions are not generally explainable at the ordinary authoring level;
- queue pressure remains primarily operational warning/counter evidence;
- the Engine gameplay Net plugin has no concrete transport-owning consumer from which generic
  transport inspection should be inferred.

## Ownership Rules

Diagnostics may observe:

- RunenNet-derived active connection/participant projections;
- host-owned reconnect attempts and errors;
- RunenNet-backed replication/prediction outcomes and Engine integration counters;
- pending work queues and current-frame projections;
- streaming/owner-routing integration state;
- transport events only at the concrete maintained transport consumer that produces them.

Diagnostics must not:

- mutate or authorize RunenNet state;
- infer admission from a presentation flag instead of the owning RunenNet contract;
- recreate a session/lifecycle state machine;
- recreate retired lane/delivery vocabulary for inspection convenience;
- treat queue admission/current-frame projection as transport emission;
- silently recover from protocol failures;
- become the only place where networking invariants are enforced.

## Useful Inspection Directions

Future evidence may justify views such as:

- lifecycle projection: active accepted bindings plus host reconnect/error policy;
- replication view: participant/connection cursor, recovery, delivery, ACK, and resync explanation;
- prediction view: pending/replayed/corrected lineage and host realization state;
- relevancy view: inclusion/exclusion reason where a concrete authoring contract exists;
- queue/projection view: bounded pending pressure and current-frame directional traffic;
- transport view only in an actual transport-owning consumer.

These are observation products. They do not create semantic ownership.

## Invariants

- Diagnostics are observational/projection state, not networking authority.
- Connected/active Engine status is derived from accepted RunenNet bindings.
- Host reconnect counters do not redefine RunenNet retention/replacement semantics.
- Replication/prediction diagnostics do not replace RunenNet lineage state.
- Current-frame queue projections do not become delivery/transport evidence.
- Transport diagnostics remain with real consumers rather than implying a generic Engine transport
  runtime.

## Future Work Constraints

Structured rejection reasons, richer per-connection/participant replication inspection, relevancy
explanations, and queue-pressure inspection require their owning current consumer. They must not be
used to pre-authorize final #322 authoring syntax or a generic Engine transport runtime.

## Validation Plan

For this boundary, validate as applicable:

- Engine session-projection/diagnostics tests;
- replication/prediction/authority-input integration diagnostics tests;
- current-frame queue/projection and Host-composition tests;
- owner-routing and connection-loss projection tests;
- transport diagnostics tests only in the maintained transport consumer;
- repository canonical validation;
- documentation validation.
