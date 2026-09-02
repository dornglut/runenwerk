---
title: "Net Transport Lanes and Delivery Design"
description: "Current boundary for retained Runenwerk replication delivery vocabulary and standalone RunenNet transport realization."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Transport Lanes and Delivery Design

## Purpose

This design records the current boundary between retained Runenwerk replication delivery vocabulary and concrete transport realization during RN8.

It does not authorize a new engine transport adapter or define a future transport migration slice.

## Current Boundary

Standalone RunenNet owns reusable transport abstraction and networking delivery semantics. Concrete RunenNet transport adapters such as `runen-net-quic` own their realization for maintained consumers.

Runenwerk currently retains a narrower set of replication migration contracts:

- `TransportLane` labels still consumed by retained replication/profile code;
- retained `DeliveryGuarantee` / lane semantics vocabulary;
- profile-to-lane mapping and route diagnostics used by current replication code;
- engine outbound work queues that stage retained replication/application messages but do not constitute a transport runtime.

The engine connection/session cut does not add `runen-net-quic`. The engine has no maintained concrete transport consumer at the N2 boundary.

The runtime-preview control channel is a separate N1 consumer of standalone RunenNet QUIC and must not be generalized into an engine networking runtime.

## Implemented Substrate

Implemented now:

- retained `TransportLane` vocabulary in `engine_net`;
- retained lane semantics for reliable, unreliable, unreliable-sequenced, and input-stream labels;
- retained replication-profile-to-lane mapping;
- retained route diagnostics;
- engine replication/application outbound staging keyed where necessary by RunenNet `ConnectionHandle`;
- standalone RunenNet QUIC realization for the separately maintained preview control-channel consumer.

The former `engine_net_quic`, engine session runtime commands, and Runenwerk-owned connection/transport runtime are not part of the current architecture.

## Partial Contracts

Partial now:

- retained lane/profile vocabulary is migration residue pending later RN8 disposition;
- bandwidth priority exists in retained profiles, but budgeted selection is not a complete scheduler;
- `InputStream` remains a retained delivery label while ECS tick buffers own simulation input buffering;
- generic per-lane diagnostics are incomplete;
- the steady-state relationship between RunenNet delivery contracts and Runenwerk replication authoring must be derived in a later authorized boundary rather than guessed here.

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

Retained `engine_net` lane/profile contracts are temporary migration evidence only. They must not grow into a replacement transport framework or a forwarding facade around RunenNet.

## Invariants

- Transport does not decide gameplay visibility or authoritative replication policy.
- ECS input buffering and transport/delivery labels remain distinct concerns.
- RunenNet lifecycle identity is not recreated in retained lane vocabulary.
- Engine outbound staging is not transport authority.
- No engine transport adapter is added without a proven maintained consumer.
- The separate preview transport consumer does not authorize a generic engine QUIC dependency.

## Future Work Constraints

Possible later work includes delivery budgeting, richer pressure diagnostics, and migration of retained lane/profile vocabulary. Its owning RN8 slice must be derived from live repository and RunenNet authority after prerequisite cuts are accepted.

This document does not pre-authorize that work or select the next RN8 boundary.

## Validation Plan

For changes to the current retained delivery boundary, validate as applicable:

- retained profile/lane mapping tests;
- engine outbound routing/staging tests;
- exact connection routing through RunenNet `ConnectionHandle`;
- transport-specific tests only in repositories/apps that actually consume that adapter;
- repository canonical validation;
- docs validation.
