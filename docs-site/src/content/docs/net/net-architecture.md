---
title: "Networking Architecture"
description: "Current Runenwerk integration with standalone RunenNet."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-24
---

# Runenwerk Networking Architecture

Runenwerk consumes standalone RunenNet for reusable realtime networking semantics. Runenwerk owns engine scheduling, ECS/game/world integration, product metadata, host policy, presentation, diagnostics, and the remaining migration consumers that have not yet been cut over.

The current RN8 architecture is intentionally transitional through N5: connection/session and remote authority-input admission are in RunenNet, dead post-N2 runtime/delivery scaffolding has been removed, and retained replication, client-prediction, envelope, and authoring integration still remains in `engine_net` until later dependency-ordered cuts.

## Ownership

### Standalone RunenNet

Owns reusable networking semantics, including:

- `ConnectionHandle` and reusable network identity;
- compatibility negotiation;
- session/participant membership and connection binding;
- connection loss, retention, replacement, expiry, removal, and closure;
- delivery flows, resource pressure, custody/exposure, and recovery semantics;
- authoritative replication consistency and full-snapshot recovery;
- participant-input admission, bounded input policy semantics, prediction, and authoritative reconciliation.

Runenwerk does not mirror these semantics in another session, delivery, replication-consistency, or prediction authority.

### Runenwerk engine integration

`engine/src/plugins/net/` owns:

- placing/invoking public RunenNet owners from application/host lifecycle code;
- `RunenNetSessionProjection`, an iterable read-only projection of already-authorized bindings;
- owner/routing projection into ECS state;
- engine scheduling for retained receive, streaming, prediction, replication, flush, and diagnostics stages;
- bounded inbox/outbox and engine-visible staging for retained replication/application payloads;
- product/session metadata;
- reconnect attempt/timing/deployment policy;
- diagnostics and presentation views;
- explicit finite authority-input policy selection and host execution staging for batches already accepted by RunenNet.

The projection is derived state. It never authorizes admission, loss, retention, replacement, expiry, removal, or closure.

### Retained `engine_net`

After RN8 N5, `engine_net` is only migration evidence for maintained replication/envelope/local-prediction/authoring consumers:

- snapshot/delta/ACK/input-frame and typed-payload envelopes;
- replication drivers, models, profiles, interest/mapping/timeline/diagnostics contracts;
- simulation-facing networking metadata and macros.

Connection-scoped retained replication state uses RunenNet `ConnectionHandle` directly.

`engine_net` does not own sessions, compatibility negotiation, connection allocation, reconnect policy, reusable delivery, transport realization, replication consistency/recovery, or reusable prediction/reconciliation semantics.

RN8 N4 removed the dead post-N2 surfaces that no longer had a maintained engine runtime consumer:

- `ReplicationRuntimeCommand` / `ReplicationRuntimeEvent`;
- `TransportLane`, `DeliveryGuarantee`, synthetic profile-to-lane mappings, and lane-only route diagnostics;
- the standalone snapshot-payload `PredictionState` / `ReconciliationResult` helper.

They are not compatibility contracts and must not return through aliases, forwarding modules, or a replacement engine runtime facade.

### Transport and Delivery

Concrete transport is outside the engine integration boundary. The already-migrated preview channel may consume `runen-net-quic` directly; the engine must not invent a generic QUIC/runtime adapter without a maintained concrete consumer.

Engine inbox/outbox resources and `NetworkInboundQueue` / `NetworkOutboundQueue` are bounded staging/projection surfaces only. Queue admission is not RunenNet delivery acceptance, and frame-end flush is not transport I/O.

## Current Lifecycle Flow

```text
host/application lifecycle
          |
          v
RunenNet NegotiationManager
          |
          v
RunenNet Session
          |
          v
RunenNetSessionProjection
          |
    +-----+----------------+
    |                      |
owner/routing state   status/diagnostics
    |
retained replication/input integration
```

A connection becomes eligible for retained engine replication routing only after RunenNet session admission has produced an active projected binding. Remote input admission does not use the projection as authority: it resolves the current participant through the actual RunenNet `Session` and is classified by RunenNet `AuthorityInputSession`.

On connection loss, RunenNet decides session membership behavior. The engine projection then removes the lost binding; owner routing and connection-scoped streaming/diagnostic state are reconciled from that projection.

## Retained Replication Flow

For each fixed server tick:

1. Read active RunenNet-authorized `ConnectionHandle`s from the projection.
2. Capture/select retained authoritative snapshot state per connection.
3. Select full or delta payload using retained per-connection baseline checkpoints.
4. Stage `ServerMessage::Snapshot` / `DeltaSnapshot` through `OutboundServerMessage::ToConnection` for the corresponding connection.
5. Process retained ACKs only for projected active connections.
6. Submit remote input against actual RunenNet session authorization and `AuthorityInputSession`; only accepted batches enter host execution staging.
7. Update retained replication/streaming/input diagnostics.

This preserves existing integration behavior without making the retained pipeline responsible for connection/session, transport/delivery, or reusable replication semantics.

## Retained Prediction/Input Integration

Remote authority-input admission is now RunenNet-owned. Runenwerk retains only explicit policy selection, encoded host execution staging, decoding/application at the target fixed tick, and the still-local client prediction/replay integration.

Reusable participant-input prediction and authoritative reconciliation semantics also belong to standalone RunenNet, but the live client prediction path remains a later cut that depends on `ClientReplicationSet`. This boundary does not define future ordinary replicated-view authoring syntax.

## Interest and World Policy

Runenwerk owns concrete world/spatial/team/gameplay policy inputs. Networking layers may expose reusable vocabulary or consume already-derived relevance information, but they do not own the Runenwerk world model.

## History

`engine_history` remains Runenwerk replay/archive/validation infrastructure. RunenNet retention/recovery contracts must not be confused with host reconnect scheduling or Runenwerk history policy.

## Dependency Direction

```text
gameplay/world
     |
     v
Runenwerk engine integration
     |
     +--> standalone RunenNet
     |
     +--> temporary engine_net migration residue

runen-net-quic --> standalone RunenNet
```

No lower reusable networking layer may depend on Runenwerk ECS/game/world policy.

## Migration Rule

RN8 removes duplicate authority one owner at a time. A migrated semantic is deleted from `engine_net`; it is not retained through aliases, forwarding modules, compatibility runtimes, or parallel state machines.

This architecture records the N5 authority boundary. It does not pre-authorize the next RN8 implementation child or select a future common-path authoring API.

See also:

- [goals.md](goals.md)
- [engine-net/README.md](engine-net/README.md)
- [engine-net/replication-pipeline.md](engine-net/replication-pipeline.md)
- [Engine Net Integration Design](../design/active/net-plugin-runtime-bridge.md)
- [Net Transport and Delivery Boundary](../design/active/net-transport-lanes-delivery.md)
- [Net Plugin](../engine/plugins/net/README.md)
