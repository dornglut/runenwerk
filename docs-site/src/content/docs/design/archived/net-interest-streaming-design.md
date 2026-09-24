---
title: "Net Interest and Streaming Design"
description: "Historical pre-RN8 engine_net interest-policy target retained as migration and implementation evidence."
status: archived
owner: net
layer: net
canonical: false
last_reviewed: 2026-09-13
related:
  - ../../net/net-architecture.md
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Interest and Streaming Design

## Archive disposition

This document records the former hard-coded `engine_net` interest-policy target. It is
not current reusable networking architecture. The `InterestPolicy` vocabulary described
below still exists in current source as RN8 migration residue; that implementation fact
does not make it the future semantic owner.

Current Runenwerk networking authority keeps concrete world, spatial, team, audience,
and gameplay relevancy policy outside reusable networking semantics. Current integration
ownership is recorded by the [Runenwerk networking architecture](../../net/net-architecture.md)
and the [multiplayer replication roadmap](../../net/multiplayer-replication-implementation-roadmap.md).
Future audience/relevancy authoring remains dependent on accepted ECS↔Net integration
evidence; this archive does not define a replacement enum, policy vocabulary, or API.

The historical body below is retained for provenance.

## Purpose

This design defines how Runenwerk limits replicated state to relevant
clients without moving gameplay visibility or world-streaming policy into
transport or net core.

## Implemented Substrate

Implemented now:

- `InterestPolicy` vocabulary in `engine_net`:
  - `Global`
  - `OwnerOnly`
  - `Spatial`
  - `Team`
  - `Distance`
- `InterestContext` predicate evaluation.
- Server runtime payload filtering hooks.
- Engine plugin `NetStreamingStateResource` for per-connection chunk
  streaming state.
- World streaming integration that derives relevant chunks from runtime
  chunk maps, ownership roles, and region invalidation journals.
- Per-connection full-resync flags for streaming gaps.

## Partial Contracts

Partial now:

- `engine_net` has policy vocabulary, not a spatial index.
- Team membership, distance sources, and AOI data are supplied by
  gameplay/domain/app integration.
- Streaming state is currently engine/world-plugin specific, not a
  generic `engine_net` protocol for all games.
- Interest diagnostics are not yet rich enough to explain why an entity
  was included or excluded.

## Ownership Rules

`engine_net` owns:

- interest policy names;
- predicate inputs and deterministic allow/deny behavior.

Domain/gameplay/app code owns:

- spatial partitioning;
- team membership;
- owner assignment beyond generic IDs;
- distance thresholds and visibility rules.

Engine/world plugins may bridge:

- chunk residency;
- world invalidation journals;
- per-connection streaming state.

Transport adapters do not own interest decisions.

## Invariants

- Interest is evaluated before payload delivery.
- Owner-only state must not be sent to unrelated clients.
- Interest filtering must be per connection.
- Missing streaming history or pruned invalidation journals must fall
  back to full resync for the affected connection.
- Interest decisions must not mutate authoritative gameplay state.

## Future Work

Future work:

1. Add a generic interest resolver trait for engine/app integration.
2. Add explainable interest diagnostics per connection/entity.
3. Add tests for policy transitions and streaming full-resync gaps.
4. Add budgeted replication selection by interest and priority.
5. Define how world streaming payloads coexist with entity/component
   replication payloads.

## Validation Plan

Required validation:

- policy predicate unit tests in `engine_net`;
- engine streaming-state tests for ACK gaps and full resync;
- integration tests for owner-only and spatial transitions;
- docs validation.
