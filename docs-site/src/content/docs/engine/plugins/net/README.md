---
title: "Net Plugin"
description: "Current Runenwerk engine integration with standalone RunenNet and retained networking migration contracts."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Net Plugin

## Purpose

`engine/src/plugins/net` integrates standalone RunenNet lifecycle/session authority and the
remaining Runenwerk replication/input migration contracts with engine resources and schedules.

The game-facing entry point is:

```rust
use engine::net::prelude::*;

app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

`NetRole::Server` and `NetRole::Host` select the corresponding retained integration roles.

`NetPlugin` does **not** install a replacement networking lifecycle runtime or transport. There
is no `NetworkRuntimeHandle` boundary. Application/host lifecycle code places and invokes the
accepted RunenNet negotiation/session owners, and successful bindings are exposed to scheduled
engine integration through the derived `RunenNetSessionProjection`.

## Ownership Boundary

Standalone RunenNet owns reusable networking semantics, including connection identity,
compatibility negotiation, participant/session lifecycle, delivery/recovery contracts,
replication consistency/recovery, and participant-input prediction/reconciliation.

Runenwerk engine integration owns:

- schedule placement for retained receive, streaming, prediction, replication, flush, and
  diagnostics work;
- the read-only `RunenNetSessionProjection` used for engine routing and diagnostics;
- product/session metadata and host reconnect/deployment policy;
- bounded inbox/outbox staging for retained replication/application payloads;
- retained `engine_net` replication/input integration while RN8 migration continues.

The projection is derived state. It never authorizes admission, loss, retention, replacement,
expiry, removal, or closure.

## Multi-Connection Replication

Retained server replication is keyed by RunenNet `ConnectionHandle` and maintains independent
per-connection baseline state.

- `ConnectionBaselineCheckpoint` tracks sent/acknowledged snapshot cursors and full-resync state.
- `ServerSnapshotReplicationState<TSnapshot>` stores checkpoints and snapshot history per
  `ConnectionHandle`.
- `ClientSnapshotReplicationState<TSnapshot>` stores the client's retained applied-snapshot
  state.
- `OutboundServerMessage::ToConnection { connection, message }` stages targeted output;
  `OutboundServerMessage::Broadcast(message)` stages broadcast output.

ACK/input processing that requires connection identity is accepted only for a `ConnectionHandle`
that remains authorized by `RunenNetSessionProjection`.

## Staging, Not Transport

`NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, and `NetworkServerOutbox` are
bounded Runenwerk work queues for retained payload integration. `NetworkInboundQueue` and
`NetworkOutboundQueue` expose engine-visible staged work.

Queue admission and frame-end flush are **not** RunenNet delivery acceptance and are not concrete
transport realization. A concrete transport adapter is selected only by a maintained product
consumer; the engine plugin does not recreate the retired runtime/transport facade.

## Schedule Ownership

- `PreUpdate` / `NetPreUpdateSet::Receive`
  - `client_receive_system`
  - `server_receive_system`
- `FixedUpdate`
  - `sync_connection_streaming_state_system` after an optional `CoreSet::Simulation` owner and
    before prediction;
  - `prediction_step_system` in `NetFixedSet::Prediction`;
  - `replication_step_system` in `NetFixedSet::Replication`, after prediction;
  - when a same-`FixedUpdate` `CoreSet::Simulation` owner is installed, explicit
    optional-presence ordering places relevant Net work after it.
- `FrameEnd` / `CoreSet::FrameEnd`
  - role-appropriate `client_flush_system` / `server_flush_system`;
  - `sync_net_diagnostics_view_system`.

The Net plugin remains valid in assemblies without a `CoreSet::Simulation` owner. Simulation is
conditional composition, not an unconditional intrinsic Net dependency.

## Related Docs

- [Network integration flow](network-runtime-flow.md)
- [Networking usage guide](networking-usage-guide.md)
- [Engine Net integration design](../../../design/active/net-plugin-runtime-bridge.md)
- [Runenwerk networking architecture](../../../net/net-architecture.md)
- [engine_net replication pipeline](../../../net/engine-net/replication-pipeline.md)

## Guides

- Usage: [Net Plugin Usage Guide](../../reference/plugins/net/usage-guide.md)
- Advanced: [Net Plugin Advanced Guide](../../reference/plugins/net/advanced-guide.md)
- Architecture: [Net Plugin Architecture](../../reference/plugins/net/architecture.md)
