---
title: "ECS Net Replication Boundary Design"
description: "Design for separating ECS events, tick-buffered input, retained replicated state, and RunenNet-authorized connection identity."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# ECS Net Replication Boundary Design

## Purpose

This design defines the boundary between Runenwerk ECS runtime primitives and retained network replication integration. It prevents networking code from duplicating ECS runtime behavior and prevents ECS events or engine projections from becoming networking lifecycle truth.

## Core Split

Multiplayer integration keeps separate flows:

- replicated state: authoritative simulation state represented by retained snapshot/delta contracts;
- input streams: tick-buffered client intent applied by the authoritative simulation;
- ECS events: local fan-out notifications and runtime signals;
- replication/application work queues: engine-local staging for retained network payloads;
- RunenNet lifecycle state: compatibility, participant membership, connection binding/loss/replacement, and lifecycle identity owned by standalone RunenNet Core.

These flows may interact, but they must not collapse into one generic event system or a second Runenwerk session state machine.

## Implemented Substrate

Implemented now:

- ECS `Broadcast*`, `WorkQueue*`, and `TickBuffer*` primitives;
- engine networking work queues for retained replication/application payload staging;
- tick-buffer registration for driver input types;
- `ReplicationExtractionFilter` over ECS structural deltas;
- `ReplicationRegistry` and component/entity/resource descriptors;
- `SnapshotApplyDriver`, `InputDriver`, and `ReplicationDriver` escape hatches for custom integration;
- ECS ownership and controller routing helpers used by the engine networking integration;
- `RunenNetSessionProjection` as a read-only engine projection of successful RunenNet bindings;
- RunenNet `ConnectionHandle` as the connection identity used by owner routing and retained replication state.

There is no engine session/runtime bridge that owns admission, connection lifetime, or transport teardown semantics.

## Partial Contracts

Partial now:

- standardized component payload extraction is not yet the normal gameplay-facing path;
- resource snapshot extraction remains partial;
- component metadata exists, but runtime extraction/application still depends on custom drivers;
- generic interest and ownership resolvers exist as retained migration contracts, but not yet as a complete declarative ECS replication pipeline;
- the eventual Replicated View boundary remains separately sequenced and must not be frozen by this design.

## Ownership Rules

Standalone RunenNet owns:

- connection identity;
- compatibility negotiation;
- session/participant membership and connection binding/lifecycle;
- reusable networking semantics adopted by later authorized RN8 cuts.

ECS/domain crates own:

- world state;
- component/resource storage;
- structural change logs;
- event, queue, and tick-buffer primitives;
- ownership target state.

Retained `engine_net` currently owns only evidence-backed replication/prediction migration contracts still required by maintained consumers. It must not own connection/session lifecycle or transport runtime semantics.

`engine/src/plugins/net` owns:

- schedule/resource integration;
- read-only projection of accepted RunenNet bindings into owner routing and diagnostics;
- retained driver invocation;
- input buffering and replay integration;
- retained replication work queues and per-connection state.

Gameplay/app modules own:

- component semantics;
- input meaning;
- ownership and relevancy policy beyond generic integration routing;
- state correction and smoothing.

## Negative Doctrine

- Do not serialize raw ECS entity IDs as reusable network identity.
- Do not use ECS events as the primary source of replicated truth.
- Do not copy ECS work queues or tick buffers into reusable networking semantics.
- Do not put game-specific component semantics in RunenNet or retained `engine_net`.
- Do not make transport own extraction or interest policy.
- Do not use `RunenNetSessionProjection` to authorize RunenNet lifecycle mutations; it is derived state only.
- Do not recreate deleted engine session/runtime authority through generic bridge or facade types.

## Future Work Constraints

Potential future replication work includes standard ECS extraction/apply and lower-boilerplate authoring, but its owning RN8 slice must be derived from current authority when prerequisites permit it.

Any later change must preserve:

- RunenNet lifecycle authority;
- ECS/scheduler/gameplay ownership in Runenwerk;
- separation of input, replicated state, ECS events, and diagnostics;
- the independently sequenced RunenECS/Replicated View program.

This design does not authorize or sequence the next RN8 slice.

## Validation Plan

For the current boundary, validate as applicable:

- ECS structural extraction tests;
- engine networking input/replication tests;
- RunenNet admission/projection/owner-routing tests;
- replication metadata registry tests;
- repository canonical validation;
- docs validation after boundary changes.
