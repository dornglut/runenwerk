---
title: "Net Authoritative Replication Protocol Design"
description: "Current Runenwerk boundary for retained authoritative snapshot, delta, ACK, baseline, and resync contracts during the RN8 RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
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
- RunenNet-owned authority replication lineage, retained baseline, recovery, emission, and ACK semantics;
- transport-independent host delivery-acceptance feedback;
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

- `engine_net` contains retained snapshot/delta/ACK/input wire envelopes, driver escape hatches, mapping, timeline, and diagnostics helpers;
- `engine/src/plugins/net` owns engine scheduling, driver invocation, encoded candidate formation, host delivery feedback integration, diagnostics, and projections;
- RunenNet `AuthorityReplicationSession` owns authority cursor/baseline/history/recovery/emission/ACK state per authorized participant;
- gameplay/app modules own payload extraction meaning, application meaning, and presentation policy.

`engine_net` is not the long-term reusable networking authority and must not regain session, admission, connection-allocation, or transport-runtime semantics.

## Implemented Substrate

Implemented now:

- retained `Snapshot`, `DeltaSnapshot`, `Ack`, and `SnapshotCursor` wire contracts;
- RunenNet-owned authority cursor, retained encoded baselines, recovery state, pending candidates,
  emission evidence, and ACK classification through `AuthorityReplicationSession<Vec<u8>, Vec<u8>>`;
- explicit finite `AuthorityReplicationPolicy` with exact encoded snapshot/delta byte accounting;
- driver-based connection-specific snapshot capture, delta construction, decode, and application;
- prepared authority submissions that do not become emitted merely through engine projection or queue admission;
- explicit host feedback using actual RunenNet `DeliveryAcceptance`, with `Accepted` alone making
  a cursor emitted/ACK-eligible, `NotAccepted` preserving the exact pending candidate for retry,
  and explicit validated cancellation;
- RunenNet-owned client cursor/baseline/recovery checks through `ClientReplicationSet`;
- lifecycle composition that cancels pending work on retained loss, forces full recovery on
  replacement, removes terminal lineages, and clears authority state on session close;
- focused tests for delivery acceptance, stale feedback, authorization, independent participant
  baselines, snapshot/delta application, recovery, and exact resource accounting.

The former `AuthoritativeServerRuntime`, `ClientReplicationRuntime`, session runtime bridge, and engine-owned connection/session authority are not part of the current architecture.

## Partial Contracts

Partial now:

- normal gameplay replication still relies on low-level driver integration rather than a complete standard ECS extraction/apply path;
- component/resource schema identity and standard payload authoring remain incomplete at the Runenwerk integration layer;
- retained wire/driver/timeline contracts still live in `engine_net` pending later dependency-ordered RN8 disposition;
- richer per-connection diagnostics and relevancy explanations remain future work.

## Invariants

- Authoritative replicated state originates from the authoritative simulation, not clients.
- Connection identity used by retained replication comes from RunenNet.
- Authority snapshot cursors advance only when RunenNet records accepted delivery; candidate preparation alone is not emission.
- ACKs cannot advance a baseline unless RunenNet has emission evidence for that cursor and the live session authorizes the connection.
- A delta must reference the exact retained RunenNet-confirmed baseline for that participant.
- Missing or evicted confirmed baselines recover through RunenNet full-snapshot recovery for the affected participant lineage.
- Replication recovery is participant-scoped, not process-global.
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
- independent participant-lineage baseline, ACK authorization/rejection, and stale-delivery-feedback tests;
- snapshot/delta application, delivery-acceptance, resource-limit, and recovery tests;
- repository canonical validation at the exact reviewed head;
- documentation validation.

Concrete transport tests belong to an actual maintained transport consumer and are not a prerequisite invented by this engine lifecycle cut.
