---
title: "Net Authoritative Replication Protocol Design"
description: "Current Runenwerk boundary for retained authoritative snapshot, delta, ACK, baseline, and resync contracts during the RN8 RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Authoritative Replication Protocol Design

## Purpose

This design records the current Runenwerk boundary for retained authoritative replication contracts while RN8 migrates reusable multiplayer semantics to standalone RunenNet.

It does not authorize the next RN8 implementation slice. After each accepted RN8 cut, current repository and upstream authority must be re-established before another boundary is derived.

## Scope

In scope:

- retained full-snapshot and delta payload envelopes;
- snapshot cursors and simulation ticks;
- retained client ACK payloads;
- engine per-connection baseline/checkpoint state;
- deterministic full-resync fallback when a retained baseline cannot be used;
- current driver-based snapshot/delta extraction and application.

Out of scope:

- connection/session identity or lifecycle;
- protocol compatibility negotiation;
- gameplay-specific snapshot contents;
- ECS mutation policy beyond the current driver boundary;
- smoothing and presentation correction;
- concrete transport realization;
- defining a future RunenNet replication migration before its RN8 boundary is authorized.

## Architectural Position

Standalone RunenNet owns reusable networking identity and lifecycle semantics. In particular, RunenNet `ConnectionHandle`, compatibility negotiation, and `Session` are authoritative for the connection/session boundary.

Runenwerk currently retains replication migration contracts that still have maintained consumers:

- `engine_net` contains snapshot, delta, ACK, input, profile, interest, mapping, and prediction-related migration evidence;
- `engine/src/plugins/net` owns engine scheduling, driver invocation, per-connection replication checkpoints, retained snapshot histories, diagnostics, and outbound staging;
- retained connection-scoped state is keyed directly by RunenNet `ConnectionHandle`;
- gameplay/app modules own payload extraction meaning, application meaning, and presentation policy.

`engine_net` is not the long-term reusable networking authority and must not regain session, admission, connection-allocation, or transport-runtime semantics.

## Implemented Substrate

Implemented now:

- retained `Snapshot`, `DeltaSnapshot`, `Ack`, and `SnapshotCursor` contracts;
- engine `ServerSnapshotReplicationState` and `ConnectionBaselineCheckpoint` keyed by RunenNet `ConnectionHandle`;
- sent-cursor and retained-baseline validation for ACK acceptance;
- per-connection full-snapshot fallback when a usable ACK baseline is unavailable;
- driver-based snapshot capture, delta construction, decode, and application;
- client-side cursor/baseline checks in the engine integration;
- deterministic per-connection snapshot/delta emission from admitted RunenNet connections;
- focused tests for stale/future/unsent/pruned ACK handling, independent connection baselines, snapshot/delta application, and full-resync fallback.

The former `AuthoritativeServerRuntime`, `ClientReplicationRuntime`, session runtime bridge, and engine-owned connection/session authority are not part of the current architecture.

## Partial Contracts

Partial now:

- normal gameplay replication still relies on low-level driver integration rather than a complete standard ECS extraction/apply path;
- component/resource schema identity and standard payload authoring remain incomplete at the Runenwerk integration layer;
- retained replication/prediction contracts still live in `engine_net` pending later dependency-ordered RN8 disposition;
- richer per-connection diagnostics and relevancy explanations remain future work.

## Invariants

- Authoritative replicated state originates from the authoritative simulation, not clients.
- Connection identity used by retained replication comes from RunenNet.
- Snapshot cursors advance monotonically within the retained authoritative timeline.
- ACKs cannot advance a baseline unless the cursor was sent and its required retained state is available.
- A delta must reference a valid baseline for that connection.
- Missing, mismatched, malformed, or pruned retained baselines recover through deterministic full-snapshot fallback for the affected connection.
- Replication fallback is per connection, not global.
- Transport does not decide replication or gameplay visibility policy.
- Retained `engine_net` contracts must not become a compatibility facade around RunenNet.

## Migration Constraints

Later replication work must:

- preserve RunenNet lifecycle/identity authority;
- preserve Runenwerk ECS, scheduler, gameplay, world, and presentation ownership;
- follow the available RunenECS boundary rather than freezing Replicated View early;
- migrate/delete retained replication contracts only in an explicitly authorized RN8 slice;
- avoid compatibility aliases, forwarding APIs, or parallel semantic authorities.

This document records the current replication boundary; it does not select the next RN8 slice.

## Validation Plan

For changes to the current retained replication boundary, validate as applicable:

- focused `engine_net` replication tests;
- focused engine networking/Core lifecycle tests;
- independent per-connection baseline and ACK rejection tests;
- snapshot/delta application and fallback tests;
- repository canonical validation at the exact reviewed head;
- documentation validation.

Concrete transport tests belong to an actual maintained transport consumer and are not a prerequisite invented by this engine lifecycle cut.
