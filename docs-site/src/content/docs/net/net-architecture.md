---
title: "Networking Architecture"
description: "Current Runenwerk integration with standalone RunenNet."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# Runenwerk Networking Architecture

Runenwerk consumes standalone RunenNet for reusable realtime networking semantics. Runenwerk itself owns engine scheduling, ECS/game/world integration, product metadata, host policy, and presentation.

The current RN8 architecture is intentionally transitional: connection/session authority has moved to RunenNet, while some replication/prediction contracts still remain in `engine_net` until later dependency-ordered cuts.

## Ownership

### RunenNet Core

Owns:

- `ConnectionHandle` and reusable network identity;
- compatibility negotiation;
- session/participant membership and connection binding;
- connection loss, retention, replacement, expiry, removal, and closure;
- reusable delivery and recovery semantics.

Runenwerk does not mirror these semantics in another session state machine.

### Runenwerk engine integration

`engine/src/plugins/net/` owns:

- invoking RunenNet Core from engine/application lifecycle code;
- `RunenNetSessionProjection`, an iterable read-only projection of already-authorized bindings;
- owner/routing projection into ECS state;
- engine scheduling for receive, prediction, replication, streaming, and flush stages;
- product/session metadata;
- reconnect attempt/timing/deployment policy;
- diagnostics and presentation views.

The projection is derived state. It never authorizes admission, loss, retention, replacement, or closure.

### Retained `engine_net`

After RN8 N2, `engine_net` is only migration evidence for maintained replication/prediction consumers:

- snapshot/delta/ACK/input and typed-payload envelopes;
- replication drivers, profiles, interest/mapping/prediction contracts;
- replication-runtime command/event evidence;
- lane/delivery vocabulary still required by retained replication code;
- networking metadata/macros pending later migration.

It does not own sessions, compatibility negotiation, connection allocation, reconnect policy, or transport realization.

Connection-scoped retained replication state uses RunenNet `ConnectionHandle` directly.

### Transport

Concrete transport is outside the N2 engine lifecycle boundary.

The already-migrated preview channel may consume `runen-net-quic`. The engine must not invent a replacement QUIC/runtime adapter when it has no maintained concrete transport consumer.

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
retained replication integration
```

A connection becomes eligible for engine replication only after RunenNet session admission has produced an active projected binding.

On connection loss, RunenNet decides session membership behavior. The engine projection then removes the lost binding; owner routing and streaming state are reconciled from that projection.

## Retained Replication Flow

For each fixed server tick:

1. Read active RunenNet-authorized `ConnectionHandle`s from the projection.
2. Capture the authoritative snapshot for each connection.
3. Select full or delta payload using retained per-connection replication checkpoints.
4. Stage `ServerMessage::Snapshot` / `DeltaSnapshot` to the corresponding connection.
5. Process ACKs and input only when the source connection remains authorized by the RunenNet projection.
6. Update retained replication/streaming diagnostics.

This preserves existing replication behavior without making the retained pipeline responsible for connection/session authority.

## Prediction

Prediction/reconciliation remains a separate retained integration concern in N2. The lifecycle cut may migrate connection identity mechanically, but it must not redesign prediction, rollback, reconciliation, or replicated-view semantics.

## Interest and World Policy

Runenwerk owns concrete world/spatial/team/gameplay policy inputs. Networking layers may expose reusable vocabulary or consume already-derived relevance information, but they do not own the world model.

## History

`engine_history` remains Runenwerk replay/archive/validation infrastructure. RunenNet retention/recovery contracts must not be confused with host reconnect scheduling or Runenwerk history policy.

## Dependency Direction

```text
gameplay/world
     |
     v
Runenwerk engine integration
     |
     +--> RunenNet Core
     |
     +--> temporary engine_net replication residue

runen-net-quic --> RunenNet Core
```

No lower reusable networking layer may depend on Runenwerk ECS/game/world policy.

## Migration Rule

RN8 removes duplicate authority one owner at a time. A migrated semantic is deleted from `engine_net`; it is not retained through aliases, forwarding modules, compatibility runtimes, or parallel state machines.

See also:

- [goals.md](goals.md)
- [engine-net/README.md](engine-net/README.md)
- [engine-net/replication-pipeline.md](engine-net/replication-pipeline.md)
- [../design/active/net-plugin-runtime-bridge.md](../design/active/net-plugin-runtime-bridge.md)
