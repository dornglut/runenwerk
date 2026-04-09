---
title: "net"
description: "Documentation for net."
---

# net

`net/` is the networking domain workspace subtree.

It owns the transport-agnostic multiplayer contract crate, the QUIC runtime adapter crate, simulation-facing identity and deterministic vocabulary shared with networking, and replay/history primitives used for recovery, recording, and validation.

Pinned direction and architecture goals are defined in [GOALS.md](goals.md).

## Crates

- `engine_net/`
  - Transport-agnostic multiplayer contracts.
  - Owns protocol, replication, session, transport-lane semantics, and runtime-facing client/server contracts.
  - README: [engine_net/README.md](engine-net/readme.md)

- `engine_net_macros/`
  - Declarative replication macros for gameplay/component mapping.
  - Owns `#[net_component(...)]` and `#[net_entity]` attribute generation for `engine_net` metadata traits.
  - README: [engine_net_macros/README.md](engine-net-macros/readme.md)

- `engine_net_quic/`
  - Quinn-based QUIC runtime adapter for `engine_net`.
  - Owns QUIC transport/runtime wiring, connection lifecycle, routing, framing, trust, and admission over QUIC.
  - README: [engine_net_quic/README.md](engine-net-quic/readme.md)

- `engine_sim/`
  - Shared simulation identity and deterministic vocabulary.
  - Owns codec/profile/rng helpers plus simulation-facing identity used by networking/history.
  - README: [engine_sim/README.md](engine-sim/readme.md)

- `engine_history/` (crate name: `engine_replay`)
  - Replay/history substrate.
  - Owns archive, recorder, controller, and validation primitives for recovery and deterministic verification.
  - README: [engine_history/README.md](engine-history/readme.md)

## Domain Boundaries

- `engine_net`
  - Defines shared contracts and model vocabulary.
  - Does not perform concrete transport I/O.
  - Is the single source of truth for transport-agnostic protocol/session/replication/runtime contracts.

- `engine_net_quic`
  - Implements concrete transport/runtime behavior over QUIC.
  - Maps QUIC events and connection behavior to `engine_net` contracts.
  - Must not own gameplay replication semantics.

- `engine_sim`
  - Supplies simulation-facing identity, deterministic vocabulary, and supporting helpers consumed by networking/history.
  - Remains independent from concrete transport implementation.

- `engine_history`
  - Handles replay, archive, controller, and validation concerns independent of transport implementation.
  - Supports recovery, deterministic verification, and divergence investigation.

## Current Internal Shape

The `net/` subtree is organized around explicit subdomain modules.

### `engine_net`

`engine_net` is structured as a contract-first crate:

- `engine_net/src/protocol/`
  - Protocol envelopes, IDs, versioning, control/input/snapshot/ack types
- `engine_net/src/replication/`
  - Replication model, profile vocabulary, timeline, prediction, interest, diagnostics
- `engine_net/src/session/`
  - Admission, handoff, and session identity contracts
- `engine_net/src/simulation/`
  - Frame/tick vocabulary that bridges simulation and networking
- `engine_net/src/transport/`
  - Lane semantics and transport-facing contract vocabulary
- `engine_net/src/runtime/`
  - Runtime-facing client/server contract surfaces and events

### `engine_net_macros`

- `engine_net_macros/src/lib.rs`
  - Attribute macro generation for replication metadata
  - Expands gameplay annotations into `engine_net::replication::NetComponentMetadata` and `NetEntity` implementations

### `engine_net_quic`

`engine_net_quic` is structured as a runtime adapter crate:

- `engine_net_quic/src/client/`
  - Client bootstrap, policy, and runtime
- `engine_net_quic/src/server/`
  - Server accept/admission/peer/policy/runtime concerns
- `engine_net_quic/src/runtime/`
  - Command/event buses, connection lifecycle, reconnect, routing, handles
  - Transitional helper modules currently present: `helpers.rs`, `utils.rs`
- `engine_net_quic/src/transport/`
  - QUIC framing, certificates, trust, lane mapping, endpoint creation
- `engine_net_quic/src/driver/`
  - Driver loop / runtime execution entrypoints
- `engine_net_quic/src/config/`
  - Client/server/transport configuration

### `engine_history`

`engine_history` is structured as a replay/history substrate:

- `engine_history/src/archive/`
- `engine_history/src/recorder/`
- `engine_history/src/controller/`
- `engine_history/src/validation/`
- `engine_history/src/model.rs`
- `engine_history/src/policy.rs`

## Module Structure Rules

Within each `net/*` crate, organize code by subdomain responsibility using explicit module trees.

Follow the repository-wide guidance in:

- `docs/guidelines/module-structure-guidelines.md`

Preferred approach:

- use explicit subdomain folders with `mod.rs` boundaries when a subsystem grows
- use names that describe ownership and responsibility
- keep public surfaces intentional and narrow

Avoid:

- `include!` module composition
- `_internal` module suffixes
- catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs` when a more specific name is possible

Note: `engine_net_quic/src/runtime/helpers.rs` and
`engine_net_quic/src/runtime/utils.rs` currently exist as transitional
modules and should be treated as refactor targets toward explicit
ownership-oriented submodules.

## Typical Flow

1. Define protocol/session/replication/runtime contracts in `engine_net`.
2. Implement concrete transport/runtime behavior in `engine_net_quic`.
3. Use `engine_sim` identities/ticks/hashes/seed vocabulary for deterministic interoperability.
4. Record, restore, and validate sessions with `engine_history`.
5. Bridge the selected runtime into engine schedules through `engine/src/plugins/net/`.
6. Keep gameplay replication mapping, correction, smoothing, and tuning in `games/*/src/net/`.

## Architecture Docs

- Current architecture sketch: [architecture.puml](architecture.puml)
- Target architecture sketch: [architecture-target.puml](architecture-target.puml)
- Goals and pinned direction: [GOALS.md](goals.md)

## ECS Runtime Audit Docs

Current repository-grounded ECS/runtime/multiplayer audit and sequencing docs:

- Dataflow and support systems design: [ecs-runtime-dataflow-and-support-systems-design.md](ecs-runtime-dataflow-and-support-systems-design.md)
- Feature inventory: [ecs-runtime-feature-inventory.md](ecs-runtime-feature-inventory.md)
- Capability gap cross-check: [ecs-runtime-gap-summary.md](ecs-runtime-gap-summary.md)
- Prioritized implementation roadmap: [ecs-runtime-prioritized-roadmap.md](ecs-runtime-prioritized-roadmap.md)
