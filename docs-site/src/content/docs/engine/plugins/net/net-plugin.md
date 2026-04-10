---
title: "Net Plugin Contract"
description: "Documentation for Net Plugin Contract."
---

# Net Plugin Contract

This document defines the engine plugin contract for networking.

## Public Entry Point

Use:

```rust
use engine::net::prelude::*;

app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Server));
```

`NetPlugin::client/server/host()` remain as convenience aliases.

## Runtime Command Contract

Server flush now emits explicit command intent:

- `SessionRuntimeCommand::ServerToConnection { connection_id, message }`
- `SessionRuntimeCommand::ServerBroadcast(message)`

Client flush emits:

- `SessionRuntimeCommand::Client(message)`

## Replication State Contract

Server state is per connection:

- `ServerSnapshotReplicationState<TSnapshot>`
- `ConnectionBaselineCheckpoint`

Client state is isolated:

- `ClientSnapshotReplicationState<TSnapshot>`

## Ordering Contract

Fixed-step execution is explicit:

- `prediction_step_system` in `NetFixedSet::Prediction` (after `CoreSet::Simulation`)
- `replication_step_system` in `NetFixedSet::Replication`
- `Replication` runs after `Simulation` and prediction

This avoids registration-order coupling.

## Resource Contract

Core net resources:

- `NetworkClientInbox` / `NetworkServerInbox`
- `NetworkClientOutbox` / `NetworkServerOutbox`
- `NetworkInboundQueue` / `NetworkOutboundQueue`
- `NetworkSessionStatus`
- `NetworkAdmissionState`
- `ConnectionHealth`
- `RoundTripMetrics`
- `NetworkDiagnostics`
- `ReplicationDiagnostics`
- `PredictionDiagnostics`
