---
title: "Engine Net Integration Design"
description: "Current design for Engine scheduling, RunenNet authority integration, staging/projections, replication, prediction, and diagnostics."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Engine Net Integration Design

## Purpose

`engine/src/plugins/net` integrates standalone RunenNet with Runenwerk engine schedules, ECS/game
realization, bounded staging, projections, and diagnostics.

It owns Runenwerk integration placement and product policy. It does not own reusable networking
lifecycle, replication-consistency, delivery, recovery, prediction, or transport semantics.

## Ownership

Standalone RunenNet owns:

- connection identity and compatibility negotiation;
- session membership, binding, loss, retention, replacement, expiry, removal, and closure;
- remote participant/tick input admission;
- client replication consistency/history/recovery;
- participant prediction/reconciliation lineage;
- authority replication cursor/baseline/history/recovery/emission/ACK semantics;
- reusable delivery/resource-pressure semantics and transport abstraction.

Runenwerk Engine integration owns:

- placement and invocation of RunenNet owners from application/host lifecycle code;
- `RunenNetSessionProjection`, an iterable read-only projection of accepted session bindings;
- mapping projected connections into Engine owner/routing state;
- network schedule placement and Host role composition;
- bounded pending inbox/outbox work queues and current-frame message projections;
- gameplay snapshot/delta/input driver adaptation and complete-product realization;
- host execution staging for remote input already admitted by RunenNet;
- formation of authority replication candidates and reporting actual host delivery outcomes back to
  RunenNet;
- diagnostics projection plus reconnect/deployment/presentation policy.

## Implemented Substrate

- `NetPlugin<TDriver>` configures client, server, or Host integration roles.
- `RunenNetSessionCore` places public RunenNet `NegotiationManager` and `Session` owners at the
  Engine boundary without copying their state machines.
- `RunenNetSessionProjection` is updated only after successful RunenNet lifecycle operations.
- Engine owner routing, streaming state, and connection/diagnostic views are reconciled from that
  projection.
- `NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, and
  `NetworkServerOutbox` are bounded pending work queues.
- `NetworkInboundQueue` and `NetworkOutboundQueue` are current-frame Engine projections; client
  and server directions replace only their own side so Host composition is order-safe.
- `client_receive_system` delegates client cursor/history/recovery consistency to RunenNet
  `ClientReplicationSet`, atomically activates the complete encoded product, and uses
  `SnapshotApplyDriver::apply_snapshot` for downstream ECS/game realization.
- `server_receive_system` resolves remote input through the live RunenNet session and submits the
  opaque participant/tick batch to RunenNet `AuthorityInputSession`; only accepted batches enter
  Engine host-execution staging.
- `prediction_step_system` composes accepted remote input execution with local input while tracked
  client prediction/reconciliation is owned by RunenNet `PredictionLineage`.
- `replication_step_system` prepares per-participant authority snapshot/delta submissions around
  RunenNet `AuthorityReplicationSession`; preparation or queue projection is not emission.
- Host code reports the actual submission result through
  `record_authority_replication_delivery_acceptance`; only RunenNet
  `DeliveryAcceptance::Accepted` creates emission evidence and ACK eligibility.
- Frame-end flush drains pending Engine outboxes into current-frame outbound projections. It does
  not perform transport I/O.
- Diagnostics expose Engine-owned status/health/replication/prediction projections around RunenNet
  authority.

## Removed Predecessors

RN8 removed rather than forwarded the former Engine-owned networking authorities and migration
shells:

- no `NetworkRuntimeHandle` session channel;
- no `SessionRuntimeCommand` / `SessionRuntimeEvent` authority;
- no Engine-owned `ConnectionId`, `SessionPhase`, or client/server lifecycle state machine;
- no Engine-owned Hello/Join admission protocol;
- no synthetic lane/delivery vocabulary pretending to be transport;
- no `net/engine_net` compatibility shell.

Do not recreate those predecessors through aliases, facades, or parallel state.

## Schedule Model

Current Engine scheduling is:

1. `PreUpdate`: drain/process client and server pending inbox work.
2. `FixedUpdate`: synchronize connection streaming state from the RunenNet projection.
3. `FixedUpdate`: prediction/input execution after optional simulation work.
4. `FixedUpdate`: authority replication candidate preparation after prediction.
5. `FrameEnd`: flush pending client/server outboxes into current-frame outbound projections.
6. `FrameEnd`: synchronize diagnostics views.

RunenNet connection/session mutations remain application/host lifecycle operations. The derived
Engine projection then feeds scheduled routing and integration systems.

## Boundary Rules

- Use only public RunenNet APIs.
- Never consult `RunenNetSessionProjection` to authorize lifecycle or participant-input
  admission; it is derived state.
- Do not copy RunenNet lifecycle, replication, delivery, recovery, or prediction semantics into
  Engine resources.
- Engine queue admission and current-frame projection are not RunenNet delivery acceptance.
- Host reconnect/deployment policy remains distinct from RunenNet membership retention/recovery.
- Product lobby/roster/settings metadata remains Runenwerk-owned.
- The former `engine_net` shell remains deleted; Engine-owned integration lives under
  `engine/src/plugins/net`.
- Do not generalize the Editor ↔ Runtime Preview QUIC channel into Engine gameplay networking.
  A concrete gameplay transport requires a maintained gameplay consumer.
- Do not freeze final ordinary Replicated View authoring syntax from this low-level integration;
  #322 remains evidence-gated.

## Validation

The maintained proof should cover, as applicable:

- established RunenNet negotiation/session operations through Engine integration;
- projection/status/owner routing derived from accepted bindings;
- terminal and retained connection-loss behavior through RunenNet;
- authority-input admission and target-tick host execution;
- independent participant authority-replication state, real delivery acceptance, and ACK
  authorization/rejection;
- client complete-product activation and RunenNet-backed prediction/reconciliation behavior;
- Host client/server projection composition, including empty-frame replacement;
- no replacement transport runtime or deleted compatibility authority.

Repository acceptance remains the canonical `cargo validate` baseline plus issue-specific focused
tests on the exact reviewed head.
