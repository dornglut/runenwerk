---
title: "ECS Net Replication Boundary Design"
description: "Current boundary between RunenECS simulation state and RunenNet-backed Runenwerk multiplayer integration."
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

# ECS Net Replication Boundary Design

## Purpose

This design separates RunenECS simulation truth from RunenNet networking authority and from
Runenwerk's concrete ECS/game integration.

The boundary prevents ECS storage/events from becoming the network protocol, prevents Engine
projections from becoming networking authority, and preserves a clean correctness reference for
future Replicated View work.

## Core Split

Multiplayer integration keeps these concerns distinct:

- authoritative simulation state in RunenECS/gameplay owners;
- explicit network-visible replicated state products and payloads;
- participant input batches and their target simulation ticks;
- RunenNet session, replication, delivery, recovery, and prediction authorities;
- Engine pending work queues and current-frame projections;
- downstream ECS/game realization and presentation.

These concerns may compose, but they do not collapse into one generic event system, one ECS storage
layout, or a second Runenwerk networking authority.

## Implemented Substrate

Implemented now:

- RunenNet `ConnectionHandle` as the connection identity used by Engine routing/integration;
- `RunenNetSessionProjection` as a read-only Engine projection of successful RunenNet bindings;
- RunenNet `AuthorityInputSession` for remote participant/tick input admission, with only accepted
  opaque batches retained in Engine host-execution staging until their target tick;
- RunenNet `ClientReplicationSet` for client replication consistency/history/recovery and a
  Runenwerk-owned complete encoded product for atomic downstream realization;
- RunenNet `PredictionLineage` for tracked client prediction/reconciliation;
- RunenNet `AuthorityReplicationSession` for authority cursor/baseline/history/recovery and real
  `DeliveryAcceptance` evidence;
- `ReplicationDriver`, `SnapshotApplyDriver`, and `InputDriver` as maintained low-level expert
  escape hatches;
- bounded Engine inbox/outbox work queues and direction-independent current-frame message
  projections;
- gameplay/application ownership of extraction meaning, relevancy, correction presentation, and
  other domain policy.

There is no Engine session/runtime bridge, `engine_net` compatibility shell, or generic Engine
transport runtime.

## Partial Contracts

Partial now:

- ordinary gameplay replication still requires low-level driver adaptation;
- there is no accepted standard ECS extraction/apply authoring path for the common case;
- final network-visible state/schema authoring and mechanical projection generation remain
  evidence-gated by #322;
- richer relevancy explanation and presentation ergonomics remain product/integration work where a
  maintained consumer proves the gap.

## Ownership Rules

Standalone RunenNet owns:

- connection/session identity and lifecycle;
- compatibility negotiation;
- participant input admission;
- replication consistency, history, recovery, and delivery evidence;
- participant prediction/reconciliation;
- reusable delivery/resource-pressure semantics and transport abstraction.

RunenECS/domain owners own:

- world/simulation state;
- component/resource storage and ECS-local query/change semantics;
- execution semantics and domain invariants.

`engine/src/plugins/net` owns:

- Engine schedule/resource integration;
- derived routing/status/diagnostic projections;
- bounded pending work and current-frame observation surfaces;
- low-level gameplay driver invocation;
- complete replicated-product realization;
- host execution staging for RunenNet-accepted remote input;
- authority candidate formation and host delivery-feedback integration.

Gameplay/app modules own:

- which state is network-visible;
- payload/domain meaning;
- ownership, audience, relevancy, smoothing, and presentation policy unless a separately accepted
  reusable contract owns part of that behavior.

## Negative Doctrine

- Do not serialize raw ECS layout or raw entity identity as the reusable network contract.
- Do not infer network visibility merely because an ECS component/resource exists.
- Do not use ECS events or Engine frame projections as replicated-state authority.
- Do not copy RunenNet lifecycle, replication, delivery, or prediction semantics into ECS/Engine
  resources.
- Do not put game-specific state semantics into standalone RunenNet.
- Do not make transport own extraction, audience, interest, or gameplay policy.
- Do not use `RunenNetSessionProjection` to authorize lifecycle mutations or remote input; it is
  derived state.
- Do not recreate deleted component-registration authoring or `engine_net` compatibility surfaces.

## Future Work Constraints

The ordinary ECS-to-network authoring path remains a real product gap, but #322 is a north star
rather than final syntax authority.

Any later Replicated View evidence slice must preserve:

- explicit network-visible contract distinct from ECS storage/layout;
- RunenNet schema/protocol authority;
- RunenECS ownership of simulation state and execution semantics;
- full/clean projection or resynchronization as the correctness reference;
- inspectable generated/mechanical adaptation where automation is eventually proven safe;
- the low-level driver path as an expert escape hatch only where maintained consumers justify it.

This design does not select macros, attributes, audience vocabulary, adapter packages, or the next
multiplayer implementation issue without current consumer evidence.

## Validation Plan

For changes to this boundary, validate as applicable:

- Engine networking input/replication/Host-composition tests;
- RunenNet admission, replication, prediction, delivery, projection, and owner-routing integration
  tests;
- exact product formation/realization and recovery tests;
- relevant RunenECS integration proofs for any new projection boundary;
- repository canonical validation;
- documentation validation.
