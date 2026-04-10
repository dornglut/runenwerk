---
title: "Net Plugin"
description: "Documentation for Net Plugin."
---

# Net Plugin

## Purpose

`engine/src/plugins/net` bridges runtime networking contracts from
`engine_net` into ECS resources and schedules.

The game-facing entry point is now:

```rust
use engine::net::prelude::*;

app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

No manual runtime orchestration is required in gameplay code beyond
inserting a `NetworkRuntimeHandle` at startup.

## Multi-Client Model

Server replication is keyed by `ConnectionId` and maintains explicit
per-connection baseline checkpoints:

- `ConnectionBaselineCheckpoint`
  - `last_ack_cursor`
  - `last_sent_cursor`
  - `last_full_snapshot_cursor`
  - `last_full_snapshot_tick`
  - `needs_full_resync`
- `ServerSnapshotReplicationState<TSnapshot>`
  - `checkpoints: BTreeMap<ConnectionId, ConnectionBaselineCheckpoint>`
  - `snapshot_history`
  - `latest_snapshot`
- `ClientSnapshotReplicationState<TSnapshot>`
  - last applied cursor/tick snapshot state on clients

Server outbox delivery is explicit:

- `OutboundServerMessage::ToConnection { connection_id, message }`
- `OutboundServerMessage::Broadcast(message)`

## Schedule Ownership

- `PreUpdate` / `NetPreUpdateSet::Receive`
  - `network_runtime_receive_system`
  - `client_receive_system`
  - `server_receive_system`
- `FixedUpdate`
  - `prediction_step_system` in `NetFixedSet::Prediction` (after `CoreSet::Simulation`)
  - `replication_step_system` in `NetFixedSet::Replication` (after `CoreSet::Simulation` and prediction)
  - explicit ordering: `Simulation -> Prediction -> Replication`
- `FrameEnd` / `CoreSet::FrameEnd`
  - `client_flush_system`
  - `server_flush_system`

## Related Docs

- [NETWORK_RUNTIME_FLOW.md](network-runtime-flow.md)
- [NET_PLUGIN.md](net-plugin.md)
- [NETWORKING_USAGE_GUIDE.md](networking-usage-guide.md)
- [engine_net REPLICATION_PIPELINE](../../../net/engine-net/replication-pipeline.md)

## Guides

- Usage: [../../../docs/reference/plugins/net/usage-guide.md](../../reference/plugins/net/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/net/advanced-guide.md](../../reference/plugins/net/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/net/architecture.md](../../reference/plugins/net/architecture.md)

