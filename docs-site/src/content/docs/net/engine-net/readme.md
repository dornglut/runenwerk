---
title: "engine_net"
description: "Documentation for engine_net."
---

# engine_net

`engine_net` is the transport-agnostic networking contract crate.

## Canonical Import Surface

Use `engine_net::prelude::*` for macro + runtime contracts in one place.

It re-exports:

- protocol message types
- session/runtime commands and events
- replication contracts (driver traits, model/interest/profile)
- transport identities (`ConnectionId`, lanes, semantics)
- simulation types/macros (`#[net_component]`, `#[net_entity]`)

## Multi-Client Runtime Contracts

`SessionRuntimeCommand` is explicit:

- `Client(ClientMessage)`
- `ServerToConnection { connection_id, message }`
- `ServerBroadcast(ServerMessage)`
- `SetDrainMode`
- `DisconnectConnection`
- `Shutdown`

This replaces the old broadcast-only server command shape.

## Replication Contracts

Driver traits are defined in `src/replication/driver.rs`:

- `ReplicationDriver`
- `SnapshotApplyDriver`
- `InputDriver`

`InputDriver::receive_remote_input` is sender-aware:

```rust
fn receive_remote_input(
    world: &mut World,
    connection_id: ConnectionId,
    tick: SimulationTick,
    input: Vec<Self::Input>,
) -> Result<(), Self::Error>;
```

## Ownership

`engine_net` defines contracts only. Concrete transport/runtime I/O lives
in adapter crates such as `engine_net_quic`.
