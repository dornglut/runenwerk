---
title: "net"
description: "Runenwerk realtime networking integration boundaries around standalone RunenNet."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
---

# net

Runenwerk consumes standalone RunenNet for reusable realtime networking semantics. Reusable
simulation and replay/history semantics are separate Runenwerk domains:

- `domain/simulation` (crate `engine_sim`) — simulation identity, tick, profile, deterministic RNG,
  hash, and snapshot-codec vocabulary;
- `domain/replay` (crate `engine_replay`) — replay/archive/controller/policy/validation
  infrastructure.

The local `net/` source subtree contains networking architecture diagrams only; it contains no
workspace package or reusable networking semantic authority. Runtime networking integration lives
in `engine/src/plugins/net/`.

## Ownership

Standalone RunenNet owns connection/session identity and lifecycle, compatibility negotiation,
delivery and recovery semantics, authority/client replication consistency and retained history,
input admission, and prediction/reconciliation.

Runenwerk engine integration owns schedule placement, ECS/game/world mapping, explicit product
policy, encoded gameplay snapshot/input adaptation, host delivery integration, bounded staging,
diagnostics, presentation projections, and the minimal wire/driver contracts required by that
integration.

Concrete transport realization remains a product/adapter concern. Engine inbox/outbox queues and
`NetworkInboundQueue` / `NetworkOutboundQueue` are staging only; queue admission is not RunenNet
`DeliveryAcceptance`.

## RN8 Result

RN8 moved reusable networking authority to standalone RunenNet and deleted the predecessor shell
rather than forwarding it. The former `net/engine_net` crate, local networking runtime/session
authority, component-registration authoring surface, client/server replication consistency state,
and prediction state are not compatibility surfaces.

The retained engine-owned Net protocol surface is deliberately narrow: snapshot/delta/ACK/input
envelopes, `SnapshotCursor`, and gameplay driver traits used to adapt Runenwerk state to
RunenNet-backed integration.

## Dependency Direction

```text
gameplay / world
      |
      v
Runenwerk engine Net integration
      |
      +--> standalone RunenNet
      +--> domain/simulation (engine_sim)

domain/replay (engine_replay) --> domain/simulation (engine_sim)
runen-net-quic --> standalone RunenNet
```

RunenNet must not depend on Runenwerk ECS, scheduling, world, gameplay, simulation-domain, replay,
or product policy.

## Architecture

- [Networking architecture](net-architecture.md)
- [Direction and ownership rules](goals.md)
- [Engine Net plugin](../engine/plugins/net/README.md)
- [Engine Net integration design](../design/active/net-plugin-runtime-bridge.md)
- [Delivery/transport boundary](../design/active/net-transport-lanes-delivery.md)
