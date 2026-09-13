---
title: "Net Plugin Contract"
description: "Documentation for Net Plugin Contract."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
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

- `prediction_step_system` belongs to `NetFixedSet::Prediction`.
- `replication_step_system` belongs to `NetFixedSet::Replication`.
- `Prediction -> Replication` is the intrinsic required Net relation.
- When a same-`FixedUpdate` `CoreSet::Simulation` owner is present, explicit
  optional-presence references place relevant Net work after it.

The Simulation relation is conditional because valid network assemblies may not
install a Simulation owner. When that owner is present, the explicit
same-schedule relation avoids registration-order coupling.

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
