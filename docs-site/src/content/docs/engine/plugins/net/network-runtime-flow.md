---
title: "Network Integration Flow"
description: "Current Runenwerk engine scheduling and staging flow around standalone RunenNet lifecycle authority."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Network Integration Flow

## Layers

- Standalone RunenNet: connection identity, compatibility negotiation, session lifecycle,
  reusable delivery/recovery, replication-consistency, and prediction/reconciliation semantics.
- Application/host lifecycle: places and invokes RunenNet owners through the Runenwerk
  integration boundary.
- ECS integration: `engine/src/plugins/net/*`, including derived session/routing projections,
  schedule placement, diagnostics, and retained replication/input staging.
- Retained migration contracts: `net/engine_net`, limited to the evidence-backed payload,
  replication/input, metadata, and authoring surface that still has maintained consumers.
- Concrete transport: separate adapter/product concern. The engine has no generic replacement
  transport runtime; the already-migrated preview channel may consume `runen-net-quic` directly.

## Lifecycle and Projection Boundary

RunenNet lifecycle operations are not driven by a `NetworkRuntimeHandle` or an engine-owned
session event loop.

Application/host lifecycle code invokes the accepted RunenNet `NegotiationManager` / `Session`
owners through `RunenNetSessionCore`. Only successful bindings are recorded in
`RunenNetSessionProjection`.

`RunenNetSessionProjection` is read-only derived engine state used by routing, streaming,
diagnostics, and retained replication integration. It does not authorize admission, loss,
retention, replacement, expiry, removal, or closure.

## Receive Path (`PreUpdate`, `NetPreUpdateSet::Receive`)

1. `client_receive_system`
   - drains `NetworkClientInbox`;
   - applies retained authoritative snapshots/deltas;
   - validates delta base state before apply;
   - stages ACKs in `NetworkClientOutbox`.
2. `server_receive_system`
   - drains `NetworkServerInbox`;
   - requires a projected RunenNet `ConnectionHandle` for ACK/input processing;
   - rejects ACK/input from a connection that is not currently authorized by the projection;
   - updates retained per-connection baseline and input-staging state.

`NetworkInboundQueue` records the corresponding engine-visible staged messages. It is not a
second lifecycle or transport owner.

## Fixed Step (`FixedUpdate`)

The maintained ordering is explicit:

1. `sync_connection_streaming_state_system`
   - reconciles connection-scoped streaming state from the current RunenNet projection;
   - runs after an optional same-schedule `CoreSet::Simulation` owner and before prediction.
2. `prediction_step_system` (`NetFixedSet::Prediction`)
3. `replication_step_system` (`NetFixedSet::Replication`, after prediction)

Relevant Net work is ordered after `CoreSet::Simulation` only when that owner exists. Assemblies
without a Simulation owner remain valid.

Retained server replication:

- reads authorized RunenNet `ConnectionHandle`s from the projection;
- captures/selects full or delta snapshot state per connection;
- maintains independent baseline checkpoints;
- stages targeted output as `OutboundServerMessage::ToConnection { connection, message }` or
  broadcast output as `OutboundServerMessage::Broadcast(message)`.

## Flush Path (`FrameEnd`, `CoreSet::FrameEnd`)

- `client_flush_system`
  - drains `NetworkClientOutbox`;
  - copies staged client messages into `NetworkOutboundQueue`.
- `server_flush_system`
  - drains `NetworkServerOutbox`;
  - copies targeted/broadcast server staging into `NetworkOutboundQueue`.
- `sync_net_diagnostics_view_system`
  - reconciles engine-facing status and diagnostics from the current projection and retained
    integration counters.

Frame-end flush does **not** emit `SessionRuntimeCommand`, perform transport I/O, or establish
RunenNet delivery acceptance. `NetworkOutboundQueue` is Runenwerk staging only.

## Connection and Host Diagnostics

Standalone RunenNet owns connection/session lifecycle decisions. Runenwerk derives
`NetworkSessionStatus`, `ConnectionHealth`, and `NetDiagnosticsView` from accepted projected
bindings and retained integration observations.

Host reconnect-attempt/timing/deployment policy and host-facing error diagnostics remain
Runenwerk-owned policy. They must not be confused with RunenNet membership retention/recovery
semantics.

## Current Authority

- [Net Plugin](README.md)
- [Networking Usage Guide](networking-usage-guide.md)
- [Engine Net Integration Design](../../../design/active/net-plugin-runtime-bridge.md)
- [Runenwerk Networking Architecture](../../../net/net-architecture.md)
- [Net Transport and Delivery Boundary](../../../design/active/net-transport-lanes-delivery.md)
