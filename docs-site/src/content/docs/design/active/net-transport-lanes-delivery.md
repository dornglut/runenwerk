---
title: "Net Transport and Delivery Boundary"
description: "Current boundary between Runenwerk replication staging and standalone RunenNet delivery/transport authority."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Transport and Delivery Boundary

## Purpose

This design records the current boundary between Runenwerk replication/application staging and standalone RunenNet delivery/transport authority during RN8.

It does not authorize a new engine transport adapter or define a later transport migration slice.

## Current Boundary

Standalone RunenNet owns reusable delivery modes, flow identity, resource pressure, custody/exposure semantics, and transport abstraction. Concrete adapters such as `runen-net-quic` own realization for their maintained consumers.

Runenwerk engine integration currently owns only host-side staging and scheduling of retained replication/application payloads. Its outbound work queues are not RunenNet delivery flows and queue admission is not `DeliveryAcceptance`.

The engine has no maintained concrete transport consumer. The runtime-preview control channel is a separate N1 consumer of standalone RunenNet QUIC and must not be generalized into an engine networking runtime.

## Removed Migration Scaffolding

RN8 N4 removes the unconsumed synthetic delivery vocabulary that remained after N2:

- `TransportLane`;
- `DeliveryGuarantee` / `LaneSemantics`;
- profile-to-lane mapping;
- `ReplicationProfile::default_lane`;
- lane-route diagnostics.

These labels were not a concrete transport realization and had no maintained engine runtime consumer. They are deleted rather than redirected through RunenNet aliases.

The retained declarative replication profile/descriptor surface is not otherwise redesigned in N4. Its eventual disposition remains coupled to the later authoring/#322 boundary.

## Ownership Rules

Standalone RunenNet owns reusable networking delivery semantics and transport abstraction.

Concrete transport adapters own:

- endpoint/connection realization for their actual consumers;
- framing and byte transport;
- adapter-specific send/receive mechanics;
- adapter-specific diagnostics.

Runenwerk engine/gameplay integration owns:

- which retained replication/application payload should be staged;
- simulation/gameplay relevancy and replication policy;
- ECS scheduling and input buffering;
- presentation and host policy.

## Invariants

- Engine work-queue admission is not RunenNet delivery acceptance.
- Transport does not decide gameplay visibility or authoritative replication policy.
- No engine delivery/runtime facade is created to replace the deleted lane vocabulary.
- No engine transport adapter is added without a proven maintained consumer.
- The separate preview transport consumer does not authorize a generic engine QUIC dependency.
- Future authoritative replication integration must record actual RunenNet delivery acceptance before treating a snapshot as emitted/ACK-eligible.

## Future Work Constraints

A later RN8 slice may integrate a concrete RunenNet delivery flow only when a maintained consumer and policy owner are proven. Declarative delivery/reliability authoring must be derived with the accepted Replicated View/RunenECS boundary rather than reconstructed from the retired lane presets.

This document does not pre-authorize that work or select the next RN8 boundary.

## Validation Plan

For this boundary, validate as applicable:

- engine outbound routing/staging tests;
- exact connection routing through RunenNet `ConnectionHandle`;
- transport-specific tests only in repositories/apps that actually consume that adapter;
- repository canonical validation;
- docs validation.
