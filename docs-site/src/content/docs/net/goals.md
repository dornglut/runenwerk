---
title: "net Goals"
description: "Documentation for net Goals."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
---

# net Goals

This document pins the intended networking direction for the `net/` domain.

## Primary Architecture Goal

- Dedicated authoritative server with connected clients.
- The server is the source of truth for simulation state.
- Clients are predictive/rendering participants, not authorities.

## Pinned Networking Stack

The networking architecture should converge on this explicit 4-part stack:

1. Network Contract Layer
  - `net/engine_net`
  - Defines transport-agnostic protocol, replication, session, simulation-facing network vocabulary, transport-lane semantics, and runtime contracts.

2. Transport Adapter Layer
  - `net/engine_net_quic`
  - Implements QUIC transport/runtime behavior for `engine_net` contracts.
  - Owns connection lifecycle, framing, trust, routing, admission, and transport mapping.

3. Engine Integration Layer
  - `engine/src/plugins/net/`
  - Bridges engine schedules/resources/events to the selected runtime adapter.
  - Owns engine-side commands, events, resources, prediction hooks, runtime I/O, and schedules.
  - Must remain gameplay-agnostic.

4. Gameplay Networking Layer
  - owning gameplay domain/app modules
  - Owns gameplay-specific replication mapping, correction, smoothing, interpolation, tuning, and presentation-side multiplayer behavior.

## Dependency Direction

Pinned direction:

- gameplay domain/app networking modules -> `engine` + `net/engine_net`
- `engine/src/plugins/net/` -> `engine` + `net/engine_net` + selected transport adapter
- `net/engine_net_quic` -> `net/engine_net` + transport dependencies
- `net/engine_net` -> no gameplay-specific dependency
- `engine_sim` / `engine_history` -> supporting net-domain crates without gameplay ownership

Lower layers must not depend on higher-layer gameplay semantics.

In particular:

- `engine_net` must not depend on game-specific logic
- `engine_net_quic` must not depend on engine/game crates
- engine integration must not become a second home for game replication policy

## Replication Model Goals

- Server publishes authoritative snapshots/deltas on a simulation timeline.
- Clients send input commands tagged with simulation tick/frame identity.
- Replication remains transport-agnostic at the `engine_net` contract level.
- Interest management controls what each client receives.
- Client-side prediction and reconciliation are first-class paths.
- Correction/smoothing policies are gameplay-owned (in the owning domain/app module), not transport-owned.

Current design details are split into:

- [../design/active/net-authoritative-replication-protocol.md](../design/active/net-authoritative-replication-protocol.md)
- [../design/active/net-prediction-reconciliation-boundary.md](../design/active/net-prediction-reconciliation-boundary.md)
- [../design/active/ecs-net-replication-boundary.md](../design/active/ecs-net-replication-boundary.md)
- [../design/active/net-interest-streaming-design.md](../design/active/net-interest-streaming-design.md)
- [../design/active/net-transport-lanes-delivery.md](../design/active/net-transport-lanes-delivery.md)

## Session and Transport Goals

- Session lifecycle is explicit: admission, active play, handoff/reconnect, teardown.
- Runtime adapters map concrete transport events to `engine_net` runtime contracts.
- Transport concerns (QUIC handshake, trust, framing, lanes, endpoint policy) stay outside gameplay rules.
- Reconnect should recover from history/checkpoints without changing authority semantics.

## Determinism and History Goals

- Shared identity/tick/hash vocabulary comes from `engine_sim`.
- Replay/checkpoint/validation flows in `engine_history` (`engine_replay`) support:
  - reconnect recovery
  - divergence detection
  - deterministic verification
  - archive/controller/recorder workflows

## Ownership Rules

- `engine_net`
  - Protocol/session/replication/runtime contracts only.
  - The single source of truth for transport-agnostic network semantics.

- `engine_net_quic`
  - Concrete QUIC transport/runtime adapter.
  - Owns QUIC-specific transport/runtime behavior only.

- `engine_sim`
  - Simulation identity and deterministic core vocabulary.

- `engine_history` (`engine_replay`)
  - Replay/checkpoint/archive/controller/validation substrate.

- `engine/src/plugins/net/`
  - Engine integration bridge only.
  - Owns engine-side resources/events/commands/schedules and runtime wiring.
  - Must not own game replication semantics.

- gameplay domain/app networking modules
  - Gameplay replication mapping, correction policy, smoothing/interpolation, and presentation behavior.

## Structural Goals Inside `net/*` Crates

Within `net/*` crates:

- organize by explicit subdomain responsibility
- prefer subdomain folders with `mod.rs` boundaries for larger concerns
- keep public surfaces narrow and intentional

Avoid:

- `include!` module composition
- `_internal` module suffixes
- ambiguous catch-all buckets when a more precise module name is available

Repository-wide guidance lives in:

- `../guidelines/module-structure-guidelines.md`

## Non-Goals

- No gameplay rule ownership in transport/runtime adapter crates.
- No transport-specific protocol semantics leaking into `engine_net` core contracts.
- No client-authoritative state model as the default architecture.
- No correction/smoothing policy ownership in engine-generic or transport crates.

## Practical Steady-State Model

The intended steady-state ownership is:

1. `engine_net`
  - defines the language

2. `engine_net_quic`
  - moves bytes and owns transport/runtime adaptation

3. `engine/src/plugins/net/`
  - bridges runtime adapters into engine scheduling/resources/events

4. gameplay domain/app networking modules
  - defines what multiplayer means for a specific game

This is the model new work in the networking domain should reinforce.

Implementation order is tracked separately in [multiplayer-replication-implementation-roadmap.md](multiplayer-replication-implementation-roadmap.md).
