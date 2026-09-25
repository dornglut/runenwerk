---
title: "Net Plugin"
description: "Current Runenwerk engine integration with standalone RunenNet and retained networking migration contracts."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Net Plugin

## Purpose

`engine/src/plugins/net` integrates standalone RunenNet lifecycle/session, authority-input, and
client-replication consistency and prediction semantics with the remaining Runenwerk server-replication integration, engine resources, and schedules.

The game-facing entry point is:

```rust
use engine::net::prelude::*;

app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

`NetRole::Server` and `NetRole::Host` select the corresponding retained integration roles.

`NetPlugin` does **not** install a replacement networking lifecycle runtime or transport. There
is no `NetworkRuntimeHandle` boundary. Application/host lifecycle code places and invokes the
accepted RunenNet negotiation/session owners, and successful bindings are exposed to scheduled
engine integration through the derived `RunenNetSessionProjection`.

## Ownership Boundary

Standalone RunenNet owns reusable networking semantics, including connection identity,
compatibility negotiation, participant/session lifecycle, delivery/recovery contracts,
replication consistency/recovery, and participant-input prediction/reconciliation.

Runenwerk engine integration owns:

- schedule placement for retained receive, streaming, prediction, replication, flush, and
  diagnostics work;
- the read-only `RunenNetSessionProjection` used for engine routing and diagnostics;
- product/session metadata and host reconnect/deployment policy;
- bounded inbox/outbox staging for retained replication/application payloads;
- retained `engine_net` server-replication/envelope integration while RN8 migration continues;
- explicit finite authority-input policy selection and host execution staging after RunenNet admission;
- explicit finite client-replication policy, complete encoded-product activation, and downstream host realization around RunenNet `ClientReplicationSet`.

The projection is derived state. It never authorizes admission, loss, retention, replacement,
expiry, removal, or closure.

## Multi-Connection Replication

Retained server replication is keyed by RunenNet `ConnectionHandle` and maintains independent
per-connection baseline state.

- `ConnectionBaselineCheckpoint` tracks sent/acknowledged snapshot cursors and full-resync state.
- `ServerSnapshotReplicationState<TSnapshot>` stores checkpoints and snapshot history per
  `ConnectionHandle`.
- client consistency/history/recovery state is owned by RunenNet `ClientReplicationSet`; Runenwerk retains only the active complete encoded product used for downstream realization.
- `OutboundServerMessage::ToConnection { connection, message }` stages targeted output;
  `OutboundServerMessage::Broadcast(message)` stages broadcast output.

Server-side ACK processing still uses projected active `ConnectionHandle`s for the retained authority baseline state. Client-side ACK cursor/tick authority comes from the committed RunenNet client lineage, not from incoming message fields or a Runenwerk client history mirror. Remote participant input uses `RunenNetSessionCore` to resolve the actual session participant for the source connection and delegates stale/future/duplicate/conflict/resource and authorization semantics to RunenNet `AuthorityInputSession`.

## Client Replication Consistency

Client snapshot/delta consistency requires an explicit finite `ClientReplicationPolicy`. The policy supplies one accepted `ReplicationLineageKey`, `ClientAggregateLimits`, and per-lineage `ReplicationRetentionLimits`; Runenwerk defines no hidden numeric defaults.

RunenNet `ClientReplicationSet` owns cursor progression, retained complete products, base selection, recovery classification, and the acknowledgement cursor. Runenwerk reconstructs deltas into one complete encoded product and uses its exact encoded byte length for accounting. The RunenNet host-commit callback atomically replaces one active derived-product resource; only after protocol commit does the retained `SnapshotApplyDriver::apply_snapshot` escape hatch realize that complete product into ECS/game state. Tracked client prediction is separately owned by RunenNet `PredictionLineage`, which observes this live `ClientReplicationSet`.

Downstream realization failure does not roll back the RunenNet protocol commit and does not emit an ACK. A duplicate-current resend may retry realization and re-ACK the existing committed cursor without a second protocol commit.

## Authority Input Admission

Server/host remote input requires explicit finite policy via
`RunenNetSessionCore::with_authority_input_policy(AuthorityInputPolicy)`. The policy wraps public
RunenNet participant and aggregate limit types; Runenwerk supplies the values and defines no
implicit networking defaults.

Each received `InputFrame` payload is submitted as one opaque participant/tick batch to RunenNet.
Only `InputAccepted` batches enter participant-aware host execution staging and are decoded/applied
at their target fixed tick before local input. Stale, future-window, duplicate, conflicting,
resource-rejected, or unauthorized batches are not executed. Retained membership preserves accepted
evidence across connection replacement; terminal membership removal, recovery expiry, and session
closure purge pending host execution.

## Staging, Not Transport

`NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, and `NetworkServerOutbox` are
bounded Runenwerk work queues for retained payload integration. `NetworkInboundQueue` and
`NetworkOutboundQueue` expose engine-visible staged work.

Queue admission and frame-end flush are **not** RunenNet delivery acceptance and are not concrete
transport realization. A concrete transport adapter is selected only by a maintained product
consumer; the engine plugin does not recreate the retired runtime/transport facade.

## Schedule Ownership

- `PreUpdate` / `NetPreUpdateSet::Receive`
  - `client_receive_system`
  - `server_receive_system`
- `FixedUpdate`
  - `sync_connection_streaming_state_system` after an optional `CoreSet::Simulation` owner and
    before prediction;
  - `prediction_step_system` in `NetFixedSet::Prediction`;
  - `replication_step_system` in `NetFixedSet::Replication`, after prediction;
  - when a same-`FixedUpdate` `CoreSet::Simulation` owner is installed, explicit
    optional-presence ordering places relevant Net work after it.
- `FrameEnd` / `CoreSet::FrameEnd`
  - role-appropriate `client_flush_system` / `server_flush_system`;
  - `sync_net_diagnostics_view_system`.

The Net plugin remains valid in assemblies without a `CoreSet::Simulation` owner. Simulation is
conditional composition, not an unconditional intrinsic Net dependency.

## Related Docs

- [Network integration flow](network-runtime-flow.md)
- [Networking usage guide](networking-usage-guide.md)
- [Engine Net integration design](../../../design/active/net-plugin-runtime-bridge.md)
- [Runenwerk networking architecture](../../../net/net-architecture.md)
- [engine_net replication pipeline](../../../net/engine-net/replication-pipeline.md)

## Guides

- Usage: [Net Plugin Usage Guide](../../reference/plugins/net/usage-guide.md)
- Advanced: [Net Plugin Advanced Guide](../../reference/plugins/net/advanced-guide.md)
- Architecture: [Net Plugin Architecture](../../reference/plugins/net/architecture.md)


## Client Prediction

Tracked client prediction requires explicit `ClientPredictionPolicy` plus `ClientReplicationPolicy`. The prediction policy supplies finite RunenNet `PredictionLimits`; lineage identity is reused from client replication and is not independently configurable.

RunenNet `PredictionLineage<Vec<u8>>` owns prediction eligibility/frontier, exact encoded per-tick pending batches, duplicate/conflict/resource classification, retirement, recovery invalidation, and replay order. Runenwerk retains input codec, tick-aware gameplay execution, bounded queue staging, authoritative product realization/restoration, scheduling, and diagnostics.

Outbound queue admission is orthogonal to prediction admission. Backpressure does not roll back an already admitted prediction batch. A client cannot mutate gameplay through tracked prediction before RunenNet admits that batch, and replay uses the target tick supplied by RunenNet.
