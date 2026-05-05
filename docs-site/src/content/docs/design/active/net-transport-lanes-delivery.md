---
title: "Net Transport Lanes and Delivery Design"
description: "Design for profile-to-lane mapping, delivery semantics, and transport adapter boundaries."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Transport Lanes and Delivery Design

## Purpose

This design defines the boundary between replication delivery intent and
concrete transport implementation.

## Implemented Substrate

Implemented now:

- `TransportLane` vocabulary in `engine_net`.
- Delivery semantics for reliable, unreliable, unreliable sequenced, and
  input stream lanes.
- `ReplicationProfilePreset` to lane mapping.
- `LaneRouteTrace` diagnostics.
- `engine_net_quic` lane mapping and QUIC datagram/stream runtime
  behavior.
- Session runtime commands for targeted server-to-connection delivery and
  broadcast delivery.

## Partial Contracts

Partial now:

- Bandwidth priority exists in profiles, but budgeted packet selection is
  not yet a complete scheduler.
- `InputStream` is a transport lane label, while ECS tick buffers own
  simulation input buffering.
- Per-lane backpressure diagnostics are still thin.
- End-to-end delivery tests across the QUIC adapter are not yet complete
  for all replication failure modes.

## Ownership Rules

`engine_net` owns:

- lane names;
- lane semantics;
- profile-to-lane mapping;
- route diagnostics;
- protocol envelopes.

`engine_net_quic` owns:

- QUIC endpoints;
- stream/datagram framing;
- trust and admission;
- concrete send/receive loops;
- reconnect/backoff mechanics.

Replication/runtime code owns:

- which message should be sent;
- which profile applies;
- whether a full snapshot or delta is appropriate.

Transport does not own gameplay interest, replication policy, or
correction policy.

## Invariants

- Profile routing must be deterministic.
- Transport lanes move bytes; they do not decide game visibility.
- QUIC-specific details must not leak into `engine_net` protocol
  contracts.
- ECS input buffering and transport input lanes must stay distinct.

## Future Work

Future work:

1. Add delivery budget and priority scheduling above transport lanes.
2. Add per-lane dropped/backpressure counters.
3. Add tests for profile/lane mapping across engine_net and
   engine_net_quic.
4. Add integration tests for unreliable sequenced delta reorder/loss.
5. Document adapter requirements for any future non-QUIC transport.

## Validation Plan

Required validation:

- profile-to-lane unit tests;
- QUIC framing tests;
- runtime command routing tests;
- docs validation.
