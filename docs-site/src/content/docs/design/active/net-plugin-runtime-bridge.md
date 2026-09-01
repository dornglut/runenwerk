---
title: "Engine Net Integration Design"
description: "Design for engine scheduling, RunenNet session projection, and retained replication integration."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-01
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Engine Net Integration Design

## Purpose

`engine/src/plugins/net` integrates standalone RunenNet and the remaining Runenwerk replication/prediction migration contracts with engine schedules and ECS resources.

It owns integration placement and projections, not reusable networking lifecycle semantics.

## Ownership

RunenNet Core owns:

- connection identity;
- compatibility negotiation;
- session membership and binding;
- connection loss, retention, replacement, expiry, removal, and closure.

Runenwerk engine integration owns:

- when Core owners are invoked by application/engine lifecycle code;
- `RunenNetSessionProjection`, an iterable read-only projection of successful Core bindings;
- mapping projected connections to engine owner/routing state;
- product/session metadata;
- reconnect attempt/timing/deployment policy;
- schedule placement, work queues, diagnostics, and presentation views;
- retained replication/prediction integration pending later RN8 cuts.

## Implemented Substrate

- `NetPlugin<TDriver>` configures client, server, or host integration roles.
- `RunenNetSessionCore` places public RunenNet `NegotiationManager` and `Session` owners at the engine boundary without copying their state machines.
- `RunenNetSessionProjection` is updated only after successful RunenNet lifecycle operations.
- Engine owner routing and connection/diagnostic views are reconciled from that projection.
- `NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, and `NetworkServerOutbox` use ECS work queues for retained replication/application payloads.
- `client_receive_system` applies retained snapshots/deltas through `SnapshotApplyDriver`.
- `server_receive_system` accepts ACK/input processing only from projected RunenNet-authorized connections.
- `sync_connection_streaming_state_system` reconciles per-connection streaming state from the RunenNet projection.
- `replication_step_system` emits retained per-connection snapshots/deltas.
- `prediction_step_system` preserves the existing prediction/input integration.
- frame-end flush stages retained outbound messages in engine-visible outbound queues; it is not a transport runtime.
- diagnostics expose engine-owned status/health/replication/prediction projections.

## Removed Lifecycle Bridge

RN8 N2 removes the old engine/runtime lifecycle bridge rather than adapting it:

- no `NetworkRuntimeHandle` session channel;
- no `SessionRuntimeCommand` / `SessionRuntimeEvent` authority;
- no `ConnectionId`, `SessionPhase`, or client/server session state machine;
- no Hello/Join admission protocol in retained engine envelopes;
- no engine-owned connection allocation or transport teardown semantics.

Do not recreate these through compatibility aliases or a generic Runenwerk networking runtime facade.

## Schedule Model

Current engine scheduling remains:

1. `PreUpdate`: process retained client/server replication/application inboxes.
2. `FixedUpdate`: synchronize connection streaming state from the RunenNet projection.
3. `FixedUpdate`: prediction after simulation.
4. `FixedUpdate`: replication after simulation and prediction.
5. `FrameEnd`: flush retained outbound work queues into engine outbound staging.
6. `FrameEnd`: synchronize diagnostics views.

RunenNet connection/session mutations occur from the owning application/host lifecycle integration; the ECS projection then feeds scheduled routing/replication systems.

## Boundary Rules

- Use only public RunenNet APIs.
- Never consult `RunenNetSessionProjection` to authorize lifecycle mutations; it is derived state.
- Do not copy RunenNet lifecycle semantics into ECS resources.
- Host reconnect policy must remain distinct from RunenNet retention/recovery semantics.
- Product lobby/roster/settings metadata remains Runenwerk-owned.
- Retained `engine_net` usage is limited to replication/prediction migration evidence.
- Do not add a concrete transport adapter without a maintained consumer.
- Do not change replication/prediction correctness semantics as part of lifecycle plumbing.

## Validation

The maintained proof must cover:

- established RunenNet negotiation admitted through engine integration;
- projection/status/owner routing derived from accepted bindings;
- terminal and retained connection-loss behavior through RunenNet Core;
- host reconnect diagnostics remaining host-owned;
- multiple RunenNet connection identities preserving independent routing/baselines;
- retained replication/prediction tests remaining behaviorally green;
- no replacement transport runtime introduced.

Before merge, repository authority remains `cargo validate` plus the issue-specific focused engine tests at the exact reviewed head.
