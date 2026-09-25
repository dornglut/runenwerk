---
title: "Network Integration Flow"
description: "Current Runenwerk Engine scheduling and staging flow around standalone RunenNet networking authority."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# Network Integration Flow

## Layers

- Standalone RunenNet: connection/session identity and lifecycle, compatibility negotiation,
  participant-input admission, replication consistency/history/recovery, prediction/reconciliation,
  delivery evidence, and reusable delivery/resource-pressure semantics.
- Application/host lifecycle: places and invokes RunenNet owners through the Runenwerk integration
  boundary and owns deployment/reconnect/concrete-transport policy.
- Engine integration: `engine/src/plugins/net/*`, including derived session/routing projections,
  schedule placement, bounded pending work, current-frame projections, gameplay driver adaptation,
  complete-product realization, diagnostics, and host execution staging.
- Concrete transport: separate adapter/product concern. The Engine gameplay Net plugin has no
  generic transport runtime; the Editor ↔ Runtime Preview channel consumes `runen-net-quic`
  directly in its owning applications.

## Lifecycle and Projection Boundary

RunenNet lifecycle operations are not driven by a `NetworkRuntimeHandle` or an Engine-owned
session event loop.

Application/host lifecycle code invokes the accepted RunenNet `NegotiationManager` / `Session`
owners through `RunenNetSessionCore`. Only successful bindings are recorded in
`RunenNetSessionProjection`.

`RunenNetSessionProjection` is read-only derived Engine state used by routing, streaming, and
diagnostics. It does not authorize admission, loss, retention, replacement, expiry, removal,
closure, or remote participant input.

## Receive Path (`PreUpdate`, `NetPreUpdateSet::Receive`)

1. `client_receive_system`
   - drains `NetworkClientInbox`;
   - delegates snapshot/delta consistency, retained-history, and recovery classification to
     RunenNet `ClientReplicationSet`;
   - atomically activates the accepted complete replicated product before downstream gameplay/ECS
     realization;
   - stages ACKs in `NetworkClientOutbox` only from accepted RunenNet outcomes.
2. `server_receive_system`
   - drains `NetworkServerInbox`;
   - routes ACK classification through the RunenNet-backed authority replication integration;
   - for remote input, requires a source `ConnectionHandle`, resolves the participant through the
     live RunenNet session, and submits the opaque participant/tick batch to RunenNet
     `AuthorityInputSession` under explicit finite policy;
   - stages only accepted remote-input batches for host execution at their target fixed tick.

`NetworkInboundQueue` is a current-frame Engine projection of processed traffic. Client/server
directions replace only their own side, including replacement with an empty projection on an empty
frame. It is not lifecycle, protocol, delivery, or transport authority.

## Fixed Step (`FixedUpdate`)

The maintained ordering is explicit:

1. `sync_connection_streaming_state_system`
   - reconciles connection-scoped streaming state from the current RunenNet projection;
   - runs after an optional same-schedule `CoreSet::Simulation` owner and before prediction.
2. `prediction_step_system` (`NetFixedSet::Prediction`)
   - drains already-accepted remote authority-input batches for the current tick before local input;
   - stages local input by tick;
   - for client authority, admits the encoded batch to RunenNet `PredictionLineage` against the
     live `ClientReplicationSet` before local predicted application;
   - keeps outbound queue admission orthogonal to prediction admission, so queue backpressure does
     not roll back an already-admitted prediction batch;
   - for server/peer authority, applies the legal combined authority/local batch at the target tick.
3. `replication_step_system` (`NetFixedSet::Replication`, after prediction)
   - reads authorized active RunenNet connections;
   - skips a participant that already has a pending authority candidate;
   - captures the connection-specific snapshot through the gameplay driver;
   - prepares a full/delta candidate through RunenNet-backed
     `AuthorityReplicationSession` integration.

Relevant Net work is ordered after `CoreSet::Simulation` only when that owner exists. Assemblies
without a Simulation owner remain valid.

Candidate preparation does not emit a snapshot. RunenNet owns per-participant emitted cursor,
confirmed baseline/history, recovery state, pending candidate, delivery evidence, and ACK
classification. The host consumes `authority_replication_submissions` and records the actual
delivery result through `record_authority_replication_delivery_acceptance`; only accepted delivery
creates emission evidence and ACK eligibility.

## Flush Path (`FrameEnd`, `CoreSet::FrameEnd`)

- `client_flush_system`
  - drains `NetworkClientOutbox`;
  - replaces only the client direction of `NetworkOutboundQueue` for the current frame.
- `server_flush_system`
  - drains `NetworkServerOutbox`;
  - replaces only the server direction of `NetworkOutboundQueue` for the current frame.
- `sync_net_diagnostics_view_system`
  - reconciles Engine-facing status and diagnostics from current projections and integration
    counters.

An empty role direction clears that direction's current-frame projection and corresponding
`*_last_frame` diagnostic count. Cumulative flush count advances only for a non-empty flush.

Frame-end flush does **not** perform transport I/O or establish RunenNet delivery acceptance.
`NetworkOutboundQueue` is a current-frame Engine observation/integration projection only.

## Connection and Host Diagnostics

Standalone RunenNet owns connection/session and replication/prediction semantic decisions.
Runenwerk derives `NetworkSessionStatus`, `ConnectionHealth`, `ReplicationDiagnostics`,
`PredictionDiagnostics`, and `NetDiagnosticsView` from accepted RunenNet outcomes and
Engine-owned integration observations.

Host reconnect-attempt/timing/deployment policy and host-facing error diagnostics remain
Runenwerk-owned policy. They do not redefine RunenNet membership retention/recovery semantics.

## Current Authority

- [Net Plugin](README.md)
- [Networking Usage Guide](networking-usage-guide.md)
- [Engine Net Integration Design](../../../design/active/net-plugin-runtime-bridge.md)
- [Runenwerk Networking Architecture](../../../net/net-architecture.md)
- [Net Transport and Delivery Boundary](../../../design/active/net-transport-lanes-delivery.md)
