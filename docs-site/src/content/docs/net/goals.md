---
title: "net Goals"
description: "Target ownership and dependency rules for Runenwerk realtime networking."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# net Goals

This document pins the intended Runenwerk networking ownership model.

## Primary Goal

Runenwerk should use standalone RunenNet as its reusable realtime networking semantic layer while keeping engine/game/world integration inside Runenwerk.

The architecture must support authoritative multiplayer without making Runenwerk maintain a second connection/session/protocol runtime.

## Ownership Model

### Standalone RunenNet

Owns reusable networking semantics:

- connection and participant/session identity;
- protocol/schema compatibility negotiation;
- session membership, admission/binding, loss, retention, replacement, expiry, and closure;
- delivery identity and ordering/reliability semantics;
- recovery/resynchronization contracts;
- reusable replication and input/prediction contracts as RN8 migration reaches those boundaries;
- transport abstraction.

`runen-net-quic` owns concrete QUIC realization where a maintained consumer requires it.

### Runenwerk engine integration

`engine/src/plugins/net/` owns:

- placement and progression of RunenNet owners in engine schedules;
- ECS resources/projections derived from accepted RunenNet state;
- mapping connections/participants to Runenwerk owner/routing state;
- product/session metadata not standardized by RunenNet;
- reconnect timing/attempt/deployment policy;
- engine diagnostics and presentation views;
- retained replication/prediction integration while those later RN8 cuts remain incomplete.

These integration resources must not become alternate networking authority.

### Gameplay and world domains

Own:

- gameplay replication mapping and correction policy;
- world/spatial relevancy inputs;
- team/ownership/game rules;
- smoothing/interpolation/presentation policy;
- simulation architecture.

### Retained `engine_net`

`engine_net` is temporary migration evidence only. After RN8 N2 it may retain evidence-backed replication/prediction/protocol-payload/macro support, but it must not own connection identity, compatibility negotiation, sessions, admission, reconnect semantics, or transport runtime.

## Dependency Direction

Required direction:

```text
gameplay / world
      |
      v
Runenwerk engine integration
      |
      +--> standalone RunenNet
      |
      +--> temporary engine_net replication residue

runen-net-quic --> standalone RunenNet
```

Rules:

- RunenNet never depends on Runenwerk ECS, scheduler, gameplay, world, or product policy.
- Runenwerk may adapt public RunenNet state into engine-owned projections, but must not copy its state machines.
- `engine_net` must not forward or alias RunenNet APIs as a compatibility facade.
- Concrete transport adapters are introduced only for real consumers.

## Runtime Principles

1. Server-authoritative simulation is the default multiplayer model.
2. Clients send intent/input, not authoritative world state.
3. Connection/session authorization is decided by RunenNet Core.
4. Engine replication consumes only authorized connection identity.
5. Replication/prediction behavior changes occur only in their owning RN8 boundary, not during lifecycle plumbing cuts.
6. Interest/relevancy vocabulary must remain separate from gameplay/world policy inputs.
7. Transport details must not define gameplay or session semantics.
8. History/replay remains a Runenwerk concern unless a reusable networking contract is explicitly owned by RunenNet.

## Current RN8 State

After N2, the engine connection/session boundary is intended to be:

```text
RunenNet NegotiationManager + Session
               |
               v
RunenNetSessionProjection (read-only engine projection)
               |
        +------+------+
        |             |
 owner routing   status/diagnostics
        |
 retained replication integration
```

The projection is not consulted to authorize lifecycle mutations. RunenNet remains the authority.

## End State

The steady-state architecture contains:

1. standalone RunenNet for reusable realtime networking semantics;
2. `runen-net-quic` or other RunenNet transport adapters only where required;
3. Runenwerk engine integration for ECS/scheduling/product/host policy;
4. gameplay/world domains for game-specific simulation and replication policy;
5. `engine_sim` and `engine_history` as independent Runenwerk simulation/history owners.

`engine_net` and its old networking authority disappear once their maintained migration consumers are removed. Clean deletion is preferred over long-lived compatibility layers.
