---
title: "net"
description: "Documentation for net."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-08-31
---

# net

`net/` is the legacy Runenwerk networking/simulation/history workspace subtree being reduced during the RN8 RunenNet cutover.

Standalone realtime networking semantics and transport realization are owned by RunenNet. Runenwerk retains product integration and the still-unmigrated `engine_net` surface until later RN8 cuts remove that duplicate authority. `engine_sim` and `engine_history` remain adjacent simulation/history owners rather than transport implementations.

Pinned direction and architecture goals are defined in [goals.md](goals.md).

## Crates

- `engine_net/`
  - Remaining Runenwerk networking contracts and runtime-facing integration pending later RN8 migration.
  - Must not gain new standalone networking authority while the RunenNet cutover is in progress.
  - README: [engine_net/README.md](engine-net/README.md)

- `engine_net_macros/`
  - Declarative replication macros for gameplay/component mapping pending their later RN8 disposition.
  - Owns `#[net_component(...)]` and `#[net_entity]` attribute generation for current `engine_net` metadata traits.
  - README: [engine_net_macros/README.md](engine-net-macros/README.md)

- `engine_sim/`
  - Shared simulation identity and deterministic vocabulary.
  - Owns codec/profile/rng helpers plus simulation-facing identity used by networking/history.
  - README: [engine_sim/README.md](engine-sim/README.md)

- `engine_history/` (crate name: `engine_replay`)
  - Replay/history substrate.
  - Owns archive, recorder, controller, and validation primitives for recovery and deterministic verification.
  - README: [engine_history/README.md](engine-history/README.md)

## Domain Boundaries

- RunenNet
  - Owns standalone sessions/connections, protocol/schema compatibility, delivery semantics, transport realization, and consuming connection teardown.
  - Is consumed by Runenwerk rather than redefined inside `net/`.

- `engine_net`
  - Is retained only for still-unmigrated Runenwerk consumers during RN8.
  - Must not be treated as the long-term standalone networking authority.

- `engine_sim`
  - Supplies simulation-facing identity, deterministic vocabulary, and supporting helpers consumed by networking/history.
  - Remains independent from concrete transport implementation.

- `engine_history`
  - Handles replay, archive, controller, and validation concerns independent of transport implementation.
  - Supports recovery, deterministic verification, and divergence investigation.

## Current Internal Shape

### `engine_net`

`engine_net` remains structured as a contract-first crate while its consumers are migrated:

- `engine_net/src/protocol/`
  - Legacy protocol envelopes, IDs, versioning, control/input/snapshot/ack types
- `engine_net/src/replication/`
  - Legacy replication model, profile vocabulary, timeline, prediction, interest, diagnostics
- `engine_net/src/session/`
  - Legacy admission, handoff, and session identity contracts
- `engine_net/src/simulation/`
  - Frame/tick vocabulary bridging simulation and networking
- `engine_net/src/transport/`
  - Legacy lane semantics and transport-facing vocabulary
- `engine_net/src/runtime/`
  - Remaining runtime-facing client/server integration surfaces

### `engine_net_macros`

- `engine_net_macros/src/lib.rs`
  - Attribute macro generation for current replication metadata
  - Expands gameplay annotations into `engine_net::replication::NetComponentMetadata` and `NetEntity` implementations

### `engine_history`

`engine_history` is structured as a replay/history substrate:

- `engine_history/src/archive/`
- `engine_history/src/recorder/`
- `engine_history/src/controller/`
- `engine_history/src/validation/`
- `engine_history/src/model.rs`
- `engine_history/src/policy.rs`

## Module Structure Rules

Within each remaining `net/*` crate, organize code by subdomain responsibility using explicit module trees.

Follow the repository-wide guidance in:

- `../guidelines/module-structure-guidelines.md`

Preferred approach:

- use explicit subdomain folders with `mod.rs` boundaries when a subsystem grows
- use names that describe ownership and responsibility
- keep public surfaces intentional and narrow

Avoid:

- `include!` module composition
- `_internal` module suffixes
- catch-all files such as `utils.rs`, `helpers.rs`, or `misc.rs` when a more specific name is possible

Do not recreate a generic Runenwerk transport/runtime adapter that duplicates RunenNet during the RN8 cutover.

## Current Migration Direction

1. Consume standalone RunenNet directly at bounded Runenwerk product/integration boundaries.
2. Preserve application/ECS/simulation/history policy in its existing owning Runenwerk domain.
3. Remove duplicate `engine_net` semantics only when the corresponding maintained consumers have migrated.
4. Keep `engine_sim` identities/ticks/hashes/seed vocabulary under simulation ownership.
5. Keep `engine_history` under replay/history ownership.
6. Bridge networking into engine schedules only as Runenwerk integration; do not redefine RunenNet semantics there.

## Architecture Docs

- Networking architecture guide: [net-architecture.md](net-architecture.md)
- Goals and pinned direction: [goals.md](goals.md)
- Implementation roadmap: [multiplayer-replication-implementation-roadmap.md](multiplayer-replication-implementation-roadmap.md)

## Canonical Design Package

Use these active design documents for Runenwerk-side networking integration and migration context:

- Authoritative replication protocol: [../design/active/net-authoritative-replication-protocol.md](../design/active/net-authoritative-replication-protocol.md)
- Prediction and reconciliation boundary: [../design/active/net-prediction-reconciliation-boundary.md](../design/active/net-prediction-reconciliation-boundary.md)
- Engine net plugin runtime bridge: [../design/active/net-plugin-runtime-bridge.md](../design/active/net-plugin-runtime-bridge.md)
- ECS/net replication boundary: [../design/active/ecs-net-replication-boundary.md](../design/active/ecs-net-replication-boundary.md)
- Interest and streaming: [../design/active/net-interest-streaming-design.md](../design/active/net-interest-streaming-design.md)
- Reconnect and history recovery: [../design/active/net-reconnect-history-recovery.md](../design/active/net-reconnect-history-recovery.md)
- Declarative replication authoring: [../design/active/net-declarative-replication-authoring.md](../design/active/net-declarative-replication-authoring.md)
- Transport lanes and delivery: [../design/active/net-transport-lanes-delivery.md](../design/active/net-transport-lanes-delivery.md)
- Diagnostics and inspection: [../design/active/net-diagnostics-inspection.md](../design/active/net-diagnostics-inspection.md)

Historical context only:

- Multiplayer design proposal: [multiplayer-design-proposal.md](multiplayer-design-proposal.md)
- Replication model: [replication-model.md](replication-model.md)

## ECS Runtime Audit Docs

Current repository-grounded ECS/runtime/multiplayer audit and sequencing docs:

- Dataflow and support systems design: [ecs-runtime-dataflow-and-support-systems-design.md](ecs-runtime-dataflow-and-support-systems-design.md)
- Feature inventory: [ecs-runtime-feature-inventory.md](ecs-runtime-feature-inventory.md)
- Capability gap cross-check: [ecs-runtime-gap-summary.md](ecs-runtime-gap-summary.md)
- Prioritized implementation roadmap: [ecs-runtime-prioritized-roadmap.md](ecs-runtime-prioritized-roadmap.md)
