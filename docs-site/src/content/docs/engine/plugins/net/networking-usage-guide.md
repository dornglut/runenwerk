---
title: "Networking Usage Guide"
description: "Current guide for the retained low-level Runenwerk networking migration surface."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Networking Usage Guide

This guide documents the **retained low-level RN8 migration surface** that still has maintained Runenwerk consumers. It is not authority for the future ordinary multiplayer authoring syntax. Current ownership and migration constraints are defined by the Runenwerk networking architecture and multiplayer replication roadmap.

## 1) Import the Current Engine Surface

```rust
use engine::net::prelude::*;
```

This provides the current engine-facing integration surface, including:

- retained `#[net_component]` and `#[net_entity]` metadata macros;
- retained replication/input/protocol-payload contracts from `engine_net`;
- `NetPlugin`;
- `NetRole`.

Connection/session lifecycle authority is not provided by `engine_net`; standalone RunenNet owns that boundary.

## 2) Retained Replication Metadata

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

These declarations currently generate replication metadata. They do not generate the complete snapshot extraction, delta generation, ECS apply path, or a future Replicated View authoring API.

The macros are migration-surface implementation, not a promise that per-component registration remains the final ordinary authoring model.

## 3) Implement the Retained Driver Boundary

Current maintained consumers use:

- `ReplicationDriver`;
- `SnapshotApplyDriver`;
- `InputDriver`.

`InputDriver::receive_remote_input` receives RunenNet `ConnectionHandle`, so authoritative gameplay/integration code can preserve the already-authorized connection lineage.

These driver traits are the retained low-level path and remain useful for specialized representations. The future common-path authoring API is intentionally not defined by this guide.

## 4) Install the Net Plugin

```rust
app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

or `NetRole::Server` / `NetRole::Host`.

`NetPlugin` owns engine schedule placement and retained replication/prediction integration. It does not become the owner of reusable RunenNet session, delivery, recovery, or prediction semantics.

## 5) Session and Runtime Boundary

There is no `NetworkRuntimeHandle` session bridge in the current architecture.

Standalone RunenNet Core owns compatibility negotiation, session membership, connection binding/loss/retention/replacement/expiry, and closure. Runenwerk projects already-authorized bindings into engine integration and uses engine inbox/outbox work queues for retained replication/application payloads.

Those engine work queues are staging, not a replacement transport or delivery-acceptance runtime. Concrete transport realization is added only where a maintained product consumer requires it.

## 6) Multi-Connection Semantics

Retained server replication is computed per RunenNet `ConnectionHandle`, not globally:

- independent ACK/baseline cursors per authorized connection;
- targeted snapshot/delta staging;
- delta fallback to full resync only for the affected connection.

## Current Authority

- [Runenwerk networking architecture](../../../net/net-architecture.md)
- [Engine net integration design](../../../design/active/net-plugin-runtime-bridge.md)
- [ECS/net replication boundary](../../../design/active/ecs-net-replication-boundary.md)
- [Multiplayer replication implementation roadmap](../../../net/multiplayer-replication-implementation-roadmap.md)

The former component-registration authoring target and pre-RunenNet prediction/interest target designs are archived historical evidence and must not be used as current implementation or future-authoring authority.
