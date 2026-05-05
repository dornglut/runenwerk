---
title: "Engine Net Plugin Runtime Bridge Design"
description: "Design for bridging engine schedules, ECS resources, and network runtime handles without owning gameplay policy."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Engine Net Plugin Runtime Bridge Design

## Purpose

This design defines how `engine/src/plugins/net` connects engine schedules
and ECS resources to transport runtimes while remaining gameplay-agnostic.

## Scope

In scope:

- `NetPlugin<TDriver>`;
- `NetRole`;
- runtime handle resources;
- inbound/outbound work queues;
- schedule placement for receive, prediction, replication, and flush;
- diagnostics view resources;
- driver trait integration.

Out of scope:

- concrete QUIC connection behavior;
- gameplay replication mapping;
- gameplay correction/smoothing.

## Implemented Substrate

Implemented now:

- `NetPlugin<TDriver>` configures client, server, or host roles.
- `NetworkRuntimeHandle` bridges `SessionRuntimeCommand` and
  `SessionRuntimeEvent` channels.
- `NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, and
  `NetworkServerOutbox` use ECS `WorkQueue` primitives.
- `client_receive_system` applies authoritative snapshots/deltas through
  `SnapshotApplyDriver`.
- `server_receive_system` consumes ACKs and remote input frames.
- `replication_step_system` emits per-connection snapshots or deltas.
- `prediction_step_system` moves local input into tick buffers and sends
  input frames.
- `NetDiagnosticsView` summarizes session, replication, prediction, and
  connection health.

## Partial Contracts

Partial now:

- The bridge still requires a custom `TDriver` for normal gameplay.
- Server ACK validation in the engine plugin is not yet hardened against
  unknown future cursors.
- Runtime bridge backpressure reports warnings and counters, but richer
  policy choices remain future work.
- Engine plugin docs describe the normal path, but examples are still
  thinner than the intended declarative workflow.

## Boundary Rules

- The plugin owns engine schedule/resource wiring only.
- The plugin may call `engine_net` contracts and selected transport
  adapter handles.
- The plugin must not own game-specific replication semantics.
- The plugin must not make ECS events the multiplayer truth path.
- The plugin must not make clients authoritative over replicated server
  state.

## Schedule Model

Current order:

1. `PreUpdate`: receive runtime events into ECS work queues.
2. `PreUpdate`: process client/server inboxes.
3. `FixedUpdate`: synchronize streaming state.
4. `FixedUpdate`: prediction after simulation.
5. `FixedUpdate`: replication after simulation and prediction.
6. `FrameEnd`: flush client/server outboxes to runtime handles.
7. `FrameEnd`: synchronize diagnostics view.

## Future Work

Future work:

1. Harden ACK validation against unsent cursors.
2. Provide a higher-level declarative replication bridge for common ECS
   components.
3. Add richer bridge diagnostics for queue pressure and dropped runtime
   commands.
4. Add integration tests that exercise plugin plus QUIC adapter.
5. Separate documentation for low-level driver escape hatch and normal
   declarative gameplay authoring.

## Validation Plan

Required validation:

- engine plugin tests for role setup, queue flow, ACK flow, and
  diagnostics;
- `cargo check --workspace` after public bridge API changes;
- docs validation.
