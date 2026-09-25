---
title: "net Goals"
description: "Target ownership and dependency rules for Runenwerk realtime networking."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# net Goals

Runenwerk uses standalone RunenNet as its reusable realtime networking semantic layer while retaining concrete engine/game/world integration locally.

## Ownership

### Standalone RunenNet

Owns reusable connection/session identity and lifecycle, compatibility negotiation, delivery/custody/resource pressure, recovery, replication consistency/history, authority input admission, and participant prediction/reconciliation. `runen-net-quic` owns concrete QUIC realization where selected by a maintained consumer.

### Runenwerk engine integration

`engine/src/plugins/net/` owns placement of RunenNet owners in schedules, ECS/game/world adaptation, explicit finite product policy, encoded snapshot/delta/input formation, host delivery feedback integration, bounded staging, diagnostics, and presentation projections.

The engine-owned protocol/driver types are integration mechanics only. They must not grow into a second reusable networking semantic layer.

### Gameplay and world domains

Own gameplay replication mapping, world/spatial relevancy inputs, game/team/ownership rules, correction and presentation policy, and simulation architecture.

### Simulation and history

`engine_sim` and `engine_history` remain independent Runenwerk owners for simulation vocabulary and replay/history infrastructure. They must not duplicate RunenNet networking authority.

## Dependency Direction

```text
gameplay / world
      |
      v
Runenwerk engine integration
      |
      +--> standalone RunenNet
      +--> engine_sim

engine_history --> engine_sim
runen-net-quic --> standalone RunenNet
```

Rules:

- RunenNet never depends on Runenwerk ECS, scheduler, gameplay, world, or product policy.
- Runenwerk may project accepted RunenNet state but must not copy its state machines.
- Concrete transport adapters are selected only for real consumers.
- No compatibility crate or forwarding facade may recreate retired networking authority.
- Clean deletion is preferred when a migration owner is no longer needed.

## Runtime Principles

1. Server-authoritative simulation is the default multiplayer model.
2. Clients send intent/input, not authoritative world state.
3. RunenNet decides connection/session authorization and reusable replication/input semantics.
4. Runenwerk supplies explicit finite policy and gameplay/world integration.
5. Engine queues are staging, not delivery acceptance.
6. Interest/relevancy vocabulary remains separate from gameplay/world policy.
7. Transport details do not define gameplay or session semantics.
8. Replay/history remains Runenwerk-owned unless a reusable networking semantic is explicitly owned by RunenNet.

## End State

The steady-state architecture contains standalone RunenNet, optional RunenNet transport adapters, Runenwerk engine integration, gameplay/world policy, and independent `engine_sim` / `engine_history` owners. The former `engine_net` migration shell is absent rather than wrapped.
