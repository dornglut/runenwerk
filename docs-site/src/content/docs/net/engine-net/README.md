---
title: "engine_net"
description: "Remaining engine_net replication, input, protocol-payload, and authoring migration evidence."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# engine_net

`engine_net` is temporary Runenwerk migration evidence for replication, input/prediction integration, protocol payloads, and network-authoring metadata that still have maintained engine consumers.

It is **not** a connection, protocol-negotiation, session, admission, reconnect, delivery, or transport-runtime authority. Those reusable realtime networking semantics belong to standalone RunenNet.

## Current Public Surface

`engine_net::prelude::*` exposes the remaining migration contracts:

- snapshot, delta, ACK, input-frame, and typed-payload envelopes;
- replication driver, model, profile, interest, mapping, timeline, and diagnostics contracts;
- simulation-facing networking metadata and macros.

Connection-scoped retained state uses RunenNet `ConnectionHandle` directly. There is no `engine_net::ConnectionId` compatibility identity.

## Explicitly Removed

RN8 N2 removed connection/session lifecycle authority, including:

- `ProtocolVersion` compatibility authority;
- `ConnectionId`;
- `SessionPhase`;
- client/server session state machines;
- Hello/Join admission messages;
- `SessionRuntimeCommand` / `SessionRuntimeEvent` lifecycle bridges;
- client/server connection runtime implementations;
- transport connection identity or transport realization.

RN8 N4 additionally removes dead post-N2 scaffolding that had no maintained engine runtime consumer:

- `ReplicationRuntimeCommand` / `ReplicationRuntimeEvent`;
- `TransportLane`, `DeliveryGuarantee`, and synthetic profile-to-lane mappings;
- lane-only route diagnostics;
- the standalone snapshot-payload `PredictionState` / `ReconciliationResult` helper.

Do not restore these through aliases, forwarding modules, or wrapper state machines.

## Replication Boundary

The retained driver traits are:

- `ReplicationDriver`;
- `SnapshotApplyDriver`;
- `InputDriver`.

Engine scheduling, owner routing, session projections, host snapshot/install/replay callbacks, and product metadata live in `engine/src/plugins/net`. RunenNet owns whether a connection/participant is admitted and remains live. Retained `engine_net` replication code may consume accepted RunenNet identity but must not recreate lifecycle, delivery, replication-consistency, or prediction authority as later RN8 cuts migrate those live consumers.

## Ownership

Standalone RunenNet is the reusable realtime networking semantic authority. `runen-net-quic` is the concrete QUIC adapter where a maintained consumer requires it.

`engine_net` remains only until later RN8 cuts migrate its evidence-backed live replication/input/authoring consumers. New reusable networking semantics must not be added here.

Design details:

- [../../design/active/net-authoritative-replication-protocol.md](../../design/active/net-authoritative-replication-protocol.md)
- [../../design/active/ecs-net-replication-boundary.md](../../design/active/ecs-net-replication-boundary.md)
- [../../design/active/net-declarative-replication-authoring.md](../../design/active/net-declarative-replication-authoring.md)
