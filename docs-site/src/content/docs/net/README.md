---
title: "net"
description: "Runenwerk simulation/history ownership and realtime networking integration boundaries."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# net

Runenwerk consumes standalone RunenNet for reusable realtime networking semantics. The local `net/` workspace subtree now contains only Runenwerk-owned simulation and history/replay crates:

- `engine_sim/` — simulation identity, tick, codec/profile, deterministic RNG, command-frame, and hash vocabulary.
- `engine_history/` (crate name `engine_replay`) — replay/history/archive/controller/policy/validation infrastructure.

Realtime networking integration itself lives in `engine/src/plugins/net/`.

## Ownership

Standalone RunenNet owns connection/session identity and lifecycle, compatibility negotiation, delivery and recovery semantics, authority/client replication consistency and retained history, input admission, and prediction/reconciliation.

Runenwerk engine integration owns schedule placement, ECS/game/world mapping, explicit product policy, encoded gameplay snapshot/input adaptation, host delivery integration, bounded staging, diagnostics, presentation projections, and the minimal wire/driver contracts required by that integration.

Concrete transport realization remains a product/adapter concern. Engine inbox/outbox queues and `NetworkInboundQueue` / `NetworkOutboundQueue` are staging only; queue admission is not RunenNet `DeliveryAcceptance`.

## RN8 Result

RN8 progressively moved reusable networking authority to standalone RunenNet and deleted the predecessor shell rather than forwarding it. The former `net/engine_net` crate, local networking runtime/session authority, component-registration authoring surface, client/server replication consistency state, and prediction state are not compatibility surfaces.

The retained engine-owned Net protocol surface is deliberately narrow: snapshot/delta/ACK/input envelopes, `SnapshotCursor`, and the gameplay driver traits used to adapt Runenwerk state to RunenNet-backed integration.

## Dependency Direction

```text
gameplay / world
      |
      v
Runenwerk engine Net integration
      |
      +--> standalone RunenNet
      +--> engine_sim

engine_history --> engine_sim
runen-net-quic --> standalone RunenNet
```

RunenNet must not depend on Runenwerk ECS, scheduling, world, gameplay, or product policy.

## Architecture

- [Networking architecture](net-architecture.md)
- [Direction and ownership rules](goals.md)
- [Engine Net plugin](../engine/plugins/net/README.md)
- [Engine Net integration design](../design/active/net-plugin-runtime-bridge.md)
- [Delivery/transport boundary](../design/active/net-transport-lanes-delivery.md)
