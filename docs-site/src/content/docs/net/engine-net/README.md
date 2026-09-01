---
title: "engine_net"
description: "Remaining engine_net replication and prediction migration evidence."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# engine_net

`engine_net` is temporary Runenwerk migration evidence for replication, prediction, protocol payloads, and network-authoring metadata that still have maintained engine consumers.

It is **not** a connection, protocol-negotiation, session, admission, reconnect, or transport-runtime authority. Those reusable realtime networking semantics belong to standalone RunenNet.

## Current Public Surface

`engine_net::prelude::*` exposes the remaining migration contracts:

- snapshot, delta, ACK, input-frame, and typed-payload envelopes;
- replication driver, model, profile, interest, mapping, and prediction contracts;
- replication-runtime command/event evidence;
- transport-lane and delivery-semantics vocabulary still consumed by retained replication code;
- simulation-facing networking metadata and macros.

Connection-scoped retained state uses RunenNet `ConnectionHandle` directly. There is no `engine_net::ConnectionId` compatibility identity.

## Explicitly Removed in RN8 N2

The active crate no longer owns or exports:

- `ProtocolVersion` compatibility authority;
- `ConnectionId`;
- `SessionPhase`;
- client/server session state machines;
- Hello/Join admission messages;
- `SessionRuntimeCommand` / `SessionRuntimeEvent` lifecycle bridges;
- client/server connection runtime implementations;
- transport connection identity or transport realization.

Do not restore these through aliases, forwarding modules, or wrapper state machines.

## Replication Boundary

The retained driver traits are:

- `ReplicationDriver`;
- `SnapshotApplyDriver`;
- `InputDriver`.

`InputDriver::receive_remote_input` is connection-aware through RunenNet identity:

```rust
fn receive_remote_input(
    world: &mut World,
    connection: ConnectionHandle,
    tick: SimulationTick,
    input: Vec<Self::Input>,
) -> Result<(), Self::Error>;
```

Engine scheduling, owner routing, session projections, and product metadata live in `engine/src/plugins/net`. RunenNet owns whether a connection/participant is admitted and remains live. Retained `engine_net` replication code may consume that accepted identity but must not recreate lifecycle authority.

## Ownership

Standalone RunenNet is the reusable realtime networking semantic authority. `runen-net-quic` is the concrete QUIC adapter where a maintained consumer requires it.

`engine_net` remains only until later RN8 cuts migrate its evidence-backed replication/prediction consumers. New reusable networking semantics must not be added here.

Design details:

- [../../design/active/net-authoritative-replication-protocol.md](../../design/active/net-authoritative-replication-protocol.md)
- [../../design/active/ecs-net-replication-boundary.md](../../design/active/ecs-net-replication-boundary.md)
- [../../design/active/net-declarative-replication-authoring.md](../../design/active/net-declarative-replication-authoring.md)
