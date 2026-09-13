---
title: "net"
description: "Documentation for the remaining Runenwerk networking migration subtree."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-13
---

# net

`net/` is the remaining Runenwerk simulation/history/network-authoring workspace subtree during the RN8 cutover to standalone RunenNet.

Standalone RunenNet owns reusable realtime networking semantics: connection identity, protocol/schema compatibility, sessions/participants, delivery and resource-pressure semantics, recovery contracts, replication consistency/recovery, and participant-input prediction/reconciliation. Concrete QUIC realization belongs to `runen-net-quic` where a maintained consumer requires it.

Runenwerk owns engine scheduling, ECS/game/world integration, product/session metadata, host deployment/reconnect policy, presentation, diagnostics, and the remaining migration consumers that have not yet been cut over.

## Remaining Crates

- `engine_net/`
  - Temporary replication/input/protocol-payload/authoring migration evidence for maintained engine consumers.
  - Uses RunenNet `ConnectionHandle` for connection-scoped retained state.
  - Must not contain connection/session/admission/delivery/transport-runtime authority.
  - README: [engine_net/README.md](engine-net/README.md)

- `engine_net_macros/`
  - Declarative replication metadata macros pending later RN8 disposition.
  - Must not define reusable networking lifecycle, delivery, replication, or prediction semantics.

- `engine_sim/`
  - Simulation identity, tick, codec/profile, and deterministic vocabulary.

- `engine_history/` (crate name: `engine_replay`)
  - Replay/history/archive/controller/validation substrate.

## Current RN8 Boundary Through N4

Engine connection/session integration consumes standalone RunenNet Core directly:

- `NegotiationManager` owns compatibility negotiation;
- `Session` owns participant membership, binding, loss, retention, replacement, expiry, and closure;
- `ConnectionHandle` is the connection identity used by engine routing and retained replication state;
- `engine/src/plugins/net` owns Core placement/invocation, read-only ECS projections, owner routing, schedule placement, host policy, product metadata, bounded staging, and diagnostics.

The engine does not translate RunenNet lifecycle state into another semantic state machine. Its `RunenNetSessionProjection` is derived routing/diagnostic state only.

`engine_net` retains only evidence-backed migration contracts that still have maintained consumers:

- snapshot, delta, ACK, input-frame, and typed-payload envelopes;
- replication driver/model/profile/interest/mapping/timeline/diagnostics contracts;
- simulation-facing networking metadata and macros.

Connection-scoped retained state uses RunenNet `ConnectionHandle` directly.

RN8 N2 removed the old `engine_net::session`, Hello/Join lifecycle, `ConnectionId`, `SessionPhase`, session runtime bridge, and client/server connection runtimes.

RN8 N4 additionally removed dead post-N2 scaffolding with no maintained engine runtime consumer:

- `ReplicationRuntimeCommand` / `ReplicationRuntimeEvent`;
- `TransportLane`, `DeliveryGuarantee`, synthetic profile-to-lane mappings, and lane-only route diagnostics;
- the standalone snapshot-payload `PredictionState` / `ReconciliationResult` helper.

These removed concepts are not compatibility surfaces and must not be recreated through aliases or forwarding facades.

## Engine Staging Boundary

The Net plugin's inbox/outbox resources and `NetworkInboundQueue` / `NetworkOutboundQueue` are bounded Runenwerk staging/projection surfaces for retained replication/application messages.

They are not RunenNet delivery flows, queue admission is not RunenNet `DeliveryAcceptance`, and frame-end flush is not transport I/O. No generic engine transport adapter is part of the current boundary.

## Dependency Rules

- Runenwerk integration may depend on public standalone RunenNet contracts.
- Retained `engine_net` may depend on RunenNet identity needed by migration consumers.
- RunenNet must not depend on Runenwerk ECS, scheduler, world, gameplay, or product policy.
- `engine_net` must not become a forwarding facade around RunenNet.
- Do not reintroduce `engine_net_quic` or another replacement engine adapter without a real maintained consumer.
- Do not add compatibility aliases for retired networking authority.

## Migration Direction

1. Consume RunenNet directly at the owning engine/product integration boundary.
2. Preserve Runenwerk-owned ECS, scheduling, gameplay, world, history, host, and presentation policy.
3. Remove duplicate `engine_net` semantics as their maintained consumers migrate.
4. Keep retained integration behavior stable during dependency-ordered cuts without treating it as reusable semantic authority.
5. Delete migration residue rather than preserving it through forwarding APIs.

RN8 is currently parked after N4. This page does not authorize N5, a future ordinary multiplayer authoring syntax, or a new engine networking runtime.

## Architecture

- Current architecture: [net-architecture.md](net-architecture.md)
- Direction and ownership rules: [goals.md](goals.md)
- `engine_net` retained surface: [engine-net/README.md](engine-net/README.md)
- Replication pipeline: [engine-net/replication-pipeline.md](engine-net/replication-pipeline.md)
- Engine integration design: [../design/active/net-plugin-runtime-bridge.md](../design/active/net-plugin-runtime-bridge.md)
- Delivery/transport boundary: [../design/active/net-transport-lanes-delivery.md](../design/active/net-transport-lanes-delivery.md)

Active design documents remain authoritative only within their stated owner/scope and must be interpreted consistently with standalone RunenNet normative ownership and the accepted RN8 cutover boundary.
