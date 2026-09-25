---
title: "Net Authoritative Replication Protocol Design"
description: "Post-RN8 Runenwerk boundary for authoritative snapshot, delta, ACK, baseline, recovery, and delivery integration."
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

This design records the current post-RN8 Runenwerk integration boundary for authoritative
replication around standalone RunenNet.

RN8 is complete. This document therefore describes the accepted boundary; it does not select a
future Replicated View authoring API or restart migration work.

## Scope

In scope:

- Engine-owned full-snapshot and delta payload envelopes;
- snapshot cursors and simulation ticks;
- client ACK payloads;
- RunenNet-owned authority replication lineage, retained baseline, recovery, emission, and ACK
  semantics;
- transport-independent host delivery-acceptance feedback;
- current driver-based snapshot/delta extraction and application.

Out of scope:

- connection/session identity or lifecycle semantics owned by RunenNet;
- protocol compatibility negotiation owned by RunenNet;
- gameplay-specific snapshot contents;
- final ECS Replicated View authoring/schema-generation syntax;
- smoothing and presentation correction policy;
- concrete transport realization.

## Architectural Position

Standalone RunenNet owns reusable networking identity, lifecycle, replication consistency,
recovery, delivery evidence, and ACK classification.

Runenwerk Engine integration retains the application-specific bridge around that authority:

- `engine/src/plugins/net` owns snapshot/delta/ACK/input wire envelopes, gameplay driver
  contracts, scheduling, encoded candidate formation, host delivery-feedback integration,
  diagnostics, and projections;
- RunenNet `AuthorityReplicationSession` owns authority cursor/baseline/history/recovery/emission/
  ACK state per authorized participant;
- gameplay/app modules own payload extraction meaning, application meaning, relevancy, and
  presentation policy.

Runenwerk's Engine wire/driver integration types are not reusable networking authority and must not
acquire session, admission, connection-allocation, replication-consistency, prediction, or
transport-runtime semantics.

## Implemented Substrate

Implemented now:

- `Snapshot`, `DeltaSnapshot`, `Ack`, and `SnapshotCursor` Engine integration contracts;
- RunenNet-owned authority cursor, retained encoded baselines, recovery state, pending candidates,
  emission evidence, and ACK classification through
  `AuthorityReplicationSession<Vec<u8>, Vec<u8>>`;
- explicit finite `AuthorityReplicationPolicy` with exact encoded snapshot/delta byte accounting;
- driver-based connection-specific snapshot capture, delta construction, decode, and application;
- prepared authority submissions that do not become emitted merely through Engine projection or
  queue admission;
- explicit host feedback using actual RunenNet `DeliveryAcceptance`, with `Accepted` alone
  making a cursor emitted/ACK-eligible, `NotAccepted` preserving the exact pending candidate for
  retry, and explicit validated cancellation;
- RunenNet-owned client cursor/baseline/recovery checks through `ClientReplicationSet`;
- RunenNet-owned client prediction/reconciliation through `PredictionLineage`;
- lifecycle composition that cancels pending authority work on retained loss, forces full recovery
  on replacement, removes terminal lineages, and clears authority state on session close;
- focused tests for delivery acceptance, stale feedback, authorization, independent participant
  baselines, snapshot/delta application, recovery, and exact resource accounting.

The former `AuthoritativeServerRuntime`, `ClientReplicationRuntime`, session runtime bridge,
Engine-owned connection/session authority, and `engine_net` migration shell are not part of the
current architecture.

## Partial Contracts

Partial now:

- normal gameplay replication still relies on low-level driver integration rather than a complete
  standard ECS extraction/apply path;
- standard network-visible state/schema authoring remains evidence-gated by #322 rather than
  frozen in the Engine protocol layer;
- richer per-connection diagnostics, desync inspection, relevancy explanation, and presentation
  policy remain future product/integration work where current consumers demonstrate a gap.

## Invariants

- Authoritative replicated state originates from the authoritative simulation, not clients.
- Connection identity used by replication comes from RunenNet.
- Authority snapshot cursors advance only when RunenNet records accepted delivery; candidate
  preparation, queue admission, or Engine frame projection alone is not emission.
- ACKs cannot advance a baseline unless RunenNet has emission evidence for that cursor and the live
  session authorizes the connection.
- A delta references the exact retained RunenNet-confirmed baseline for that participant.
- Missing or evicted confirmed baselines recover through RunenNet full-snapshot recovery for the
  affected participant lineage.
- Replication recovery is participant-scoped, not process-global.
- Transport does not decide replication or gameplay visibility policy.
- Deleted `engine_net` authority or compatibility surfaces must not be recreated around RunenNet.

## Evolution Constraints

Later replication work must:

- preserve RunenNet lifecycle, replication, delivery, recovery, and prediction authority;
- preserve Runenwerk ECS, scheduler, gameplay, world, realization, and presentation ownership;
- follow current RunenECS and #322 evidence rather than freezing Replicated View syntax early;
- retain low-level driver escape hatches only where maintained consumers justify them;
- avoid compatibility aliases, forwarding APIs, or parallel semantic authorities;
- add concrete transport realization only for a proven maintained gameplay consumer.

This document records the current replication boundary. A later feature requires its own current
consumer evidence and owning issue.

## Validation Plan

For changes to the current replication boundary, validate as applicable:

- focused Engine networking and RunenNet-integration tests;
- Core lifecycle and participant authorization tests;
- independent participant-lineage baseline, ACK authorization/rejection, and stale-delivery-
  feedback tests;
- snapshot/delta application, delivery-acceptance, resource-limit, prediction/recovery, and
  Host-composition tests;
- repository canonical validation at the exact reviewed head;
- documentation validation.

Concrete transport tests belong to an actual maintained transport consumer and are not a
prerequisite invented by the Engine integration layer.
