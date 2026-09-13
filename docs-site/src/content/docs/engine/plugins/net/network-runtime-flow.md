---
title: "Network Runtime Flow"
description: "Documentation for Network Runtime Flow."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Network Runtime Flow

## Layers

- ECS bridge: `engine/src/plugins/net/*`
- Transport-agnostic contracts: `net/engine_net`
- QUIC adapter: `net/engine_net_quic`

## Receive Path (`PreUpdate`, `NetPreUpdateSet::Receive`)

1. `network_runtime_receive_system`
   - drains `NetworkRuntimeHandle`
   - preserves `connection_id` for server ingress
   - updates status/health/admission resources
2. `client_receive_system`
   - applies authoritative snapshots/deltas
   - validates delta base cursor before apply
   - emits ack to `NetworkClientOutbox`
3. `server_receive_system`
   - applies ack/input per connection
   - updates per-connection baseline checkpoint state
   - emits targeted server responses

## Fixed Step (`FixedUpdate`)

Net's intrinsic order is explicit and not registration-order dependent:

1. `prediction_step_system` (`NetFixedSet::Prediction`)
2. `replication_step_system` (`NetFixedSet::Replication`, after prediction)

When a same-`FixedUpdate` `CoreSet::Simulation` owner is installed, the Net
systems' explicit optional-presence references place relevant work after that
owner. Assemblies without a Simulation owner remain valid; Simulation is not an
unconditional part of the intrinsic Net chain.

Server replication behavior:

- capture authoritative snapshot
- evaluate each active `ConnectionId`
- send `Snapshot` or `DeltaSnapshot` per connection baseline checkpoint
- route output as `OutboundServerMessage::ToConnection`

## Flush Path (`FrameEnd`, `CoreSet::FrameEnd`)

- `client_flush_system`
  - drains client outbox
  - sends `SessionRuntimeCommand::Client(...)`
- `server_flush_system`
  - drains server outbox
  - sends:
    - `SessionRuntimeCommand::ServerToConnection { ... }`
    - `SessionRuntimeCommand::ServerBroadcast(...)`

## Connection Lifecycle

Runtime events mapped into ECS:

- `Connected`
- `Phase`
- `JoinAccepted` / `JoinRejected`
- `Reconnecting`
- `ConnectionClosed`
- `Error`
- `RttUpdated`

These drive `NetworkSessionStatus`, `ConnectionHealth`,
`RoundTripMetrics`, and session admission state.
