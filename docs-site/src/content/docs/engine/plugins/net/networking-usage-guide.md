---
title: "Networking Usage Guide"
description: "Current low-level expert guide for Runenwerk Engine networking integration around standalone RunenNet."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Networking Usage Guide

This guide documents the **current low-level expert networking path** for Runenwerk Engine integration around standalone RunenNet. It is not authority for the future ordinary multiplayer authoring syntax. Current ownership and future-work constraints are defined by the Runenwerk networking architecture and multiplayer replication roadmap.

## 1) Import the Current Engine Surface

```rust
use engine::net::prelude::*;
```

This provides the current engine-facing integration surface, including:

- engine-owned replication/input wire contracts and gameplay driver traits;
- `NetPlugin`;
- `NetRole`.

Connection/session lifecycle and reusable replication/input semantics are owned by standalone RunenNet.

## 2) Implement the Low-Level Driver Boundary

Current maintained consumers use:

- `ReplicationDriver`;
- `SnapshotApplyDriver`;
- `InputDriver`.

`InputDriver::receive_remote_input` receives RunenNet `ConnectionHandle`, so authoritative gameplay/integration code can preserve the already-authorized connection lineage.

These driver traits are the maintained low-level expert path and remain useful for specialized representations. The future common-path authoring API is intentionally not defined by this guide.

## 3) Install the Net Plugin

```rust
app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

or `NetRole::Server` / `NetRole::Host`.

`NetPlugin` owns Engine schedule placement and RunenNet-backed replication/prediction integration. It does not become the owner of reusable RunenNet session, delivery, recovery, or prediction semantics.

## 4) Session and Runtime Boundary

There is no `NetworkRuntimeHandle` session bridge in the current architecture.

Standalone RunenNet Core owns compatibility negotiation, session membership, connection binding/loss/retention/replacement/expiry, and closure. Runenwerk projects already-authorized bindings into Engine integration and uses Engine inbox/outbox work queues for replication/application payload staging.

Those engine work queues are staging, not a replacement transport or delivery-acceptance runtime. Concrete transport realization is added only where a maintained product consumer requires it.

## 5) Multi-Connection Authority Replication

Server/host replication state is participant-scoped in RunenNet, not process-global in Runenwerk.
Configure explicit finite `AuthorityReplicationPolicy` on `RunenNetSessionCore` before active
authority replication work.

A fixed replication step prepares a complete snapshot/delta submission for each eligible authorized
connection. The host then:

1. reads `authority_replication_submissions`;
2. submits the provided complete `ServerMessage` through its real delivery implementation;
3. feeds the resulting RunenNet `DeliveryAcceptance` back through
   `record_authority_replication_delivery_acceptance`.

Reading/projecting a prepared submission is not emission. `NotAccepted` keeps the same pending
candidate for explicit retry, while `Accepted` alone advances RunenNet emission evidence and makes
the cursor ACK-eligible. Explicit cancellation is available when the host abandons a pending
attempt. Full-recovery versus delta preparation is selected from RunenNet authority state and the
exact retained encoded baseline; there is no periodic full-snapshot timer.

## Current Authority

- [Runenwerk networking architecture](../../../net/net-architecture.md)
- [Engine net integration design](../../../design/active/net-plugin-runtime-bridge.md)
- [ECS/net replication boundary](../../../design/active/ecs-net-replication-boundary.md)
- [Multiplayer replication implementation roadmap](../../../net/multiplayer-replication-implementation-roadmap.md)

The former component-registration authoring target and pre-RunenNet prediction/interest target designs are archived historical evidence and must not be used as current implementation or future-authoring authority.
