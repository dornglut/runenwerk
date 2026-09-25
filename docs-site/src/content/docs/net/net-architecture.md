---
title: "Networking Architecture"
description: "Current Runenwerk integration with standalone RunenNet."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# Runenwerk Networking Architecture

Runenwerk consumes standalone RunenNet for reusable realtime networking semantics and keeps only
engine/game/world integration locally.

## Ownership

### Standalone RunenNet

RunenNet owns connection/session identity and lifecycle, compatibility negotiation,
delivery/recovery/resource-pressure semantics, authority and client replication
consistency/history, authority input admission, and client prediction/reconciliation.

### Runenwerk engine Net integration

`engine/src/plugins/net/` owns:

- schedule placement and lifecycle composition around public RunenNet owners;
- `RunenNetSessionProjection` as a read-only routing/diagnostic projection of already-authorized bindings;
- explicit finite authority/client replication and prediction policy;
- gameplay snapshot/input codec adaptation and tick-aware execution;
- host delivery submission integration and truthful `DeliveryAcceptance` feedback;
- bounded inbox/outbox staging and engine-visible queue projection;
- world/streaming integration, diagnostics, and presentation.

It also owns the minimal integration protocol/driver contracts: `InputFrame`, `Ack`, `Snapshot`,
`DeltaSnapshot`, `ClientMessage`, `ServerMessage`, `SnapshotCursor`, `ReplicationDriver`,
`SnapshotApplyDriver`, and `InputDriver`.

These are Runenwerk integration types, not a replacement RunenNet semantic layer.

## Authority Replication

For each authorized participant, RunenNet `AuthorityReplicationSession` is the sole owner of
emitted cursor state, confirmed baselines, retained snapshot history, recovery state, delivery
evidence, and ACK classification.

Runenwerk captures/encodes the gameplay snapshot or delta and exposes a prepared `ServerMessage`.
Merely preparing or staging it is not emission. The host feeds the real RunenNet
`DeliveryAcceptance` result back into the integration; only `Accepted` makes the candidate
emitted and ACK-eligible.

## Client Replication and Prediction

RunenNet `ClientReplicationSet` owns client consistency/history/recovery. Runenwerk retains only
the active complete encoded product needed for downstream gameplay realization.

RunenNet `PredictionLineage` owns prediction eligibility, pending batch identity/accounting,
reconciliation order, and invalidation. Runenwerk owns input codec and tick-aware gameplay
execution/restoration.

## Authority Input

RunenNet `AuthorityInputSession` authorizes and bounds opaque participant/tick batches. Only
accepted batches enter Runenwerk host execution staging.

## Transport Boundary

Concrete transport is outside engine integration. Engine inbox/outbox queues and
`NetworkInboundQueue` / `NetworkOutboundQueue` are bounded staging surfaces only; frame-end
flush is not transport I/O and queue admission is not delivery acceptance.

## Simulation and Replay

`domain/simulation` (crate `engine_sim`) owns simulation identity/tick/profile/RNG/hash/codec
vocabulary. `domain/replay` (crate `engine_replay`) owns replay/archive/controller/validation
infrastructure. Neither owns RunenNet session, delivery, replication-consistency, or prediction
semantics.

## Dependency Direction

```text
gameplay/world
     |
     v
engine Net integration
     |
     +--> standalone RunenNet
     +--> domain/simulation (engine_sim)

domain/replay (engine_replay) --> domain/simulation (engine_sim)
runen-net-quic --> standalone RunenNet
```

See also:

- [goals.md](goals.md)
- [Engine Net Integration Design](../design/active/net-plugin-runtime-bridge.md)
- [Net Transport and Delivery Boundary](../design/active/net-transport-lanes-delivery.md)
- [Net Plugin](../engine/plugins/net/README.md)
