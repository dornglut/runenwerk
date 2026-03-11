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

## 3) Implement a Driver

Implement:

- `ReplicationDriver`
- `SnapshotApplyDriver`
- `InputDriver`

`InputDriver::receive_remote_input` receives `ConnectionId`, so
authoritative gameplay can map input to sender identity.

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
