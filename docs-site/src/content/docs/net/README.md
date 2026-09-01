---
title: "net"
description: "Documentation for the remaining Runenwerk networking migration subtree."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# net

`net/` is the remaining Runenwerk simulation/history/network-authoring workspace subtree during the RN8 cutover to standalone RunenNet.

Standalone RunenNet owns reusable realtime networking semantics: connection identity, protocol/schema compatibility, sessions/participants, delivery semantics, recovery contracts, and the reusable replication/prediction model as later cuts adopt it. Concrete QUIC realization belongs to `runen-net-quic` where a maintained consumer exists.

Runenwerk owns engine scheduling, ECS/game/world integration, product/session metadata, host deployment/reconnect policy, presentation, and diagnostics.

## Remaining Crates

- `engine_net/`
  - Temporary replication/prediction/protocol-payload migration evidence for maintained engine consumers.
  - Uses RunenNet `ConnectionHandle` for connection-scoped retained state.
  - Must not contain connection/session/admission/transport-runtime authority.
  - README: [engine_net/README.md](engine-net/README.md)

- `engine_net_macros/`
  - Declarative replication metadata macros pending later RN8 disposition.
  - Must not define networking lifecycle semantics.

- `engine_sim/`
  - Simulation identity, tick, codec/profile, and deterministic vocabulary.

- `engine_history/` (crate name: `engine_replay`)
  - Replay/history/archive/controller/validation substrate.

## Current N2 Boundary

Engine connection/session integration consumes standalone RunenNet Core directly:

- `NegotiationManager` owns compatibility negotiation;
- `Session` owns participant membership, binding, loss, retention, replacement, expiry, and closure;
- `ConnectionHandle` is the connection identity used by engine routing and retained replication state;
- `engine/src/plugins/net` owns only Core placement/invocation, read-only ECS projections, owner routing, host policy, product metadata, and diagnostics.

The engine does not translate RunenNet lifecycle state into another semantic state machine.

`engine_net` now retains only evidence-backed replication/prediction concerns:

- `protocol/`: ACK, input, snapshot/delta, and typed payload envelopes;
- `replication/`: driver/model/profile/interest/prediction/mapping contracts;
- `runtime/`: replication-runtime evidence only;
- `transport/`: retained lane/delivery vocabulary, not connection identity or transport realization;
- `simulation/`: retained simulation-facing networking vocabulary.

The old `engine_net::session`, Hello/Join lifecycle, `ConnectionId`, `SessionPhase`, session runtime bridge, and client/server connection runtimes are not part of the active architecture.

## Dependency Rules

- Runenwerk integration may depend on public standalone RunenNet contracts.
- Retained `engine_net` may depend on RunenNet identity needed by migration consumers.
- RunenNet must not depend on Runenwerk ECS, scheduler, world, gameplay, or product policy.
- `engine_net` must not become a forwarding facade around RunenNet.
- Do not reintroduce `engine_net_quic` or another replacement adapter without a real maintained consumer.
- Do not add compatibility aliases for retired networking authority.

## Migration Direction

1. Consume RunenNet directly at the owning engine/product integration boundary.
2. Preserve Runenwerk-owned ECS, scheduling, gameplay, world, history, and presentation policy.
3. Remove duplicate `engine_net` semantics as their maintained consumers migrate.
4. Keep retained replication/prediction behavior stable during identity/plumbing cuts.
5. Delete migration residue rather than preserving it through forwarding APIs.

## Architecture

- Current architecture: [net-architecture.md](net-architecture.md)
- Direction and ownership rules: [goals.md](goals.md)
- `engine_net` retained surface: [engine-net/README.md](engine-net/README.md)
- Replication pipeline: [engine-net/replication-pipeline.md](engine-net/replication-pipeline.md)

Active design documents remain authoritative only within their stated owner/scope and must be interpreted consistently with the current RunenNet cutover boundary.
