---
title: "ECS Net Replication Boundary Design"
description: "Design for separating ECS events, tick-buffered input, replicated state, and network protocol contracts."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# ECS Net Replication Boundary Design

## Purpose

This design defines the boundary between ECS runtime primitives and
network replication. It prevents net code from duplicating ECS runtime
behavior and prevents ECS events from becoming the network truth path.

## Core Split

Multiplayer uses separate flows:

- replicated state: authoritative server state sent by snapshots/deltas;
- input streams: tick-buffered client intent sent to authority;
- ECS events: local fan-out notifications and optional runtime signals;
- runtime bridge queues: transport/session messages between engine and
  runtime adapters.

These flows may interact, but they must not collapse into one generic
event system.

## Implemented Substrate

Implemented now:

- ECS `Broadcast*`, `WorkQueue*`, and `TickBuffer*` primitives.
- Engine net plugin work queues for inbound/outbound network messages.
- Tick-buffer registration for driver input type.
- `ReplicationExtractionFilter` over ECS structural deltas.
- `ReplicationRegistry` and component/entity/resource descriptors.
- `SnapshotApplyDriver`, `InputDriver`, and `ReplicationDriver` escape
  hatches for custom integration.
- ECS ownership and controller routing helpers used by the engine net
  plugin.

## Partial Contracts

Partial now:

- Standardized component payload extraction is not yet the normal
  gameplay-facing path.
- Resource snapshot extraction remains partial.
- Component metadata exists, but runtime extraction/application still
  depends on custom drivers.
- Generic interest and ownership resolvers are available as contracts,
  but not yet a complete declarative ECS replication pipeline.

## Ownership Rules

ECS/domain crates own:

- world state;
- component/resource storage;
- structural change logs;
- event, queue, and tick-buffer primitives;
- ownership target state.

`engine_net` owns:

- protocol contracts;
- replication metadata vocabulary;
- interest/profile vocabulary;
- transport-agnostic runtime contracts.

`engine/src/plugins/net` owns:

- schedule/resource bridge;
- driver invocation;
- input buffering and replay integration.

Gameplay/app modules own:

- component semantics;
- input meaning;
- ownership policies beyond generic connection routing;
- state correction and smoothing.

## Negative Doctrine

- Do not serialize raw ECS entity IDs over the network.
- Do not use ECS events as the primary source of replicated truth.
- Do not copy ECS work queues or tick buffers into `engine_net`.
- Do not put game-specific component semantics in `engine_net`.
- Do not make transport own extraction or interest policy.

## Future Work

Future work:

1. Define a standard component extraction/apply bridge using existing ECS
   structural deltas.
2. Add resource snapshot and delta contracts where justified.
3. Add metadata lookup APIs that are ergonomic for extractors and tools.
4. Add tests proving ECS events, input streams, and replicated state stay
   separate.
5. Add examples for common component replication without custom drivers.

## Validation Plan

Required validation:

- ECS structural extraction tests;
- engine net plugin input stream tests;
- replication metadata registry tests;
- docs validation after boundary changes.
