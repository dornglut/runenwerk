---
title: "Networking Usage Guide"
description: "Documentation for Networking Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-05
---

# Networking Usage Guide

## 1) Import the Canonical API

```rust
use engine::net::prelude::*;
```

This provides:

- `#[net_component]`
- `#[net_entity]`
- protocol/session/runtime contracts
- `NetPlugin`
- `NetRole`

## 2) Declare Replicated Types

```rust
use engine::net::prelude::*;

#[net_entity]
pub struct Player;

#[net_component(
    authority = Server,
    profile = PredictedMovement,
    owner_prediction = true,
    interest = Spatial
)]
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerState {
    pub x: f32,
    pub y: f32,
}
```

Current status: declarations generate replication metadata. They do not
yet generate the complete snapshot extraction, delta generation, or ECS
apply path.

## 3) Implement a Driver

Implement:

- `ReplicationDriver`
- `SnapshotApplyDriver`
- `InputDriver`

`InputDriver::receive_remote_input` receives `ConnectionId`, so
authoritative gameplay can map input to sender identity.

This is currently required for real gameplay integration. The long-term
design keeps driver traits as an escape hatch while adding a lower
boilerplate declarative path later.

## 4) Install a Single Net Plugin

```rust
app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

or `NetRole::Server` / `NetRole::Host`.

## 5) Runtime Handle Wiring

Insert `NetworkRuntimeHandle` once startup networking runtime is
available. The plugin handles:

- runtime event intake
- inbox/outbox processing
- targeted server dispatch
- per-connection baseline replication
- client ack + prediction replay

## 6) Multi-Client Semantics

Server replication is computed per connection, not globally:

- independent ack/baseline cursors per `ConnectionId`
- targeted snapshot/delta delivery
- delta fallback to full resync only for the affected connection

Related designs:

- [../../../design/active/net-plugin-runtime-bridge.md](../../../design/active/net-plugin-runtime-bridge.md)
- [../../../design/active/net-declarative-replication-authoring.md](../../../design/active/net-declarative-replication-authoring.md)
