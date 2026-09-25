---
title: "Net Transport and Delivery Boundary"
description: "Post-RN8 boundary between Runenwerk staging, RunenNet delivery authority, and concrete transport consumers."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Transport and Delivery Boundary

## Purpose

This design records the post-RN8 boundary between Runenwerk replication/application staging,
standalone RunenNet delivery authority, and concrete transport realization.

It does not authorize a generic Engine transport runtime or a new transport adapter without a
maintained gameplay consumer.

## Current Boundary

Standalone RunenNet owns reusable delivery modes, flow identity, resource pressure,
custody/exposure semantics, delivery acceptance, and transport abstraction. Concrete adapters such
as `runen-net-quic` own endpoint and byte-transport realization for their maintained consumers.

Runenwerk Engine networking owns host-side staging, scheduling, gameplay integration, and the host
handoff around RunenNet-owned delivery semantics. Its inbox/outbox work queues and
`NetworkInboundQueue` / `NetworkOutboundQueue` projections are not RunenNet delivery flows.

Authority replication now uses actual RunenNet delivery evidence. Preparing an authority
replication candidate or placing work in an Engine queue does not emit it. The host reports the
real submission result through the RunenNet `DeliveryAcceptance` boundary; only accepted delivery
makes that authority cursor emitted and ACK-eligible.

The Engine gameplay Net plugin has no maintained concrete transport consumer. The existing Editor
↔ Runtime Preview channel is a separate product consumer of standalone `runen-net-quic` and
must not be generalized into an Engine networking runtime.

## Current-Frame Staging and Projection

Engine networking separates pending work from current-frame observation:

- inbox/outbox resources are bounded pending work queues;
- receive/flush systems drain the role-owned pending work;
- `NetworkInboundQueue` and `NetworkOutboundQueue` expose current-frame Engine projections;
- client and server directions replace only their own projection, so Host composition cannot erase
  the opposite role;
- an empty direction becomes an empty current-frame projection rather than retaining stale work.

These resources are integration/diagnostic surfaces. They do not decide session authority,
delivery acceptance, transport custody, or protocol recovery.

## Removed Migration Scaffolding

RN8 deleted synthetic Engine delivery vocabulary that had no concrete transport consumer,
including:

- `TransportLane`;
- `DeliveryGuarantee` / `LaneSemantics`;
- profile-to-lane mapping;
- `ReplicationProfile::default_lane`;
- lane-route diagnostics;
- the final `engine_net` migration shell.

Those concepts must not be restored as aliases around RunenNet.

## Ownership Rules

Standalone RunenNet owns reusable networking delivery semantics and transport abstraction.

Concrete transport adapters own, for their actual consumers:

- endpoint/connection realization;
- framing and byte transport;
- adapter-specific send/receive mechanics;
- adapter-specific diagnostics.

Runenwerk engine/gameplay integration owns:

- which replication/application payload is formed or staged;
- simulation/gameplay relevancy and replication policy;
- ECS scheduling and input execution staging;
- product presentation and host policy;
- reporting actual host delivery outcomes into the RunenNet-owned authority replication contract.

## Invariants

- Engine work-queue admission is not RunenNet delivery acceptance.
- Current-frame Engine projection is not transport emission.
- Authority cursors become emitted/ACK-eligible only through actual RunenNet accepted-delivery
  evidence.
- Transport does not decide gameplay visibility or authoritative replication policy.
- No Engine delivery/runtime facade replaces deleted RN8 vocabulary.
- No Engine transport adapter is added without a proven maintained gameplay consumer.
- The separate Runtime Preview transport consumer does not authorize a generic Engine QUIC
  dependency.

## Future Work Constraints

A future concrete gameplay transport integration requires a maintained gameplay consumer, an
explicit host/product owner, and a demonstrated mapping from Engine integration payloads to the
selected RunenNet transport realization.

Declarative delivery/reliability authoring must follow accepted Replicated View/RunenECS evidence
under #322 rather than reconstruct retired lane presets. This document does not pre-authorize that
authoring syntax or make the Runtime Preview channel a template for gameplay networking.

## Validation Plan

For this boundary, validate as applicable:

- Engine outbound routing/staging and Host-composition tests;
- authority delivery-acceptance and stale-feedback tests;
- exact connection routing through RunenNet `ConnectionHandle`;
- transport-specific tests only in repositories/apps that actually consume that adapter;
- repository canonical validation;
- documentation validation.
