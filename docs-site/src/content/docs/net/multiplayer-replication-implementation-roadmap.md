---
title: "Multiplayer Replication Implementation Roadmap"
description: "Current constraints and retained replication work under the RN8 RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
related_designs:
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-prediction-reconciliation-boundary.md
  - ../design/active/net-plugin-runtime-bridge.md
  - ../design/active/ecs-net-replication-boundary.md
  - ../design/active/net-interest-streaming-design.md
  - ../design/active/net-reconnect-history-recovery.md
  - ../design/active/net-declarative-replication-authoring.md
  - ../design/active/net-transport-lanes-delivery.md
  - ../design/active/net-diagnostics-inspection.md
related_reports:
  - ../reports/closeouts/wr-007-multiplayer-replication-phase-1-3/closeout.md
---

# Multiplayer Replication Implementation Roadmap

## Purpose

This document records retained Runenwerk replication work and its constraints during the RN8 cutover to standalone RunenNet.

It does **not** define the next RN8 boundary. RN8 remains dependency-ordered: after each accepted slice, current repository and upstream authority must be re-established before exactly one next boundary is derived.

Current networking ownership and architecture are defined by:

- [net goals](goals.md);
- [networking architecture](net-architecture.md);
- [engine integration design](../design/active/net-plugin-runtime-bridge.md).

## Current Baseline After RN8 N2

Connection/session authority has moved out of retained `engine_net` and into standalone RunenNet Core.

Current boundary:

- RunenNet `ConnectionHandle` is the connection identity used by engine routing and retained replication state;
- RunenNet compatibility negotiation and `Session` own admission/binding/loss/retention/replacement/expiry/closure semantics;
- `engine/src/plugins/net` owns engine scheduling, read-only projections, owner routing, host reconnect policy, diagnostics, and retained replication/prediction integration;
- retained `engine_net` contains only evidence-backed replication/prediction/protocol-payload/macro migration contracts;
- the old `engine_net` session state machines, `ConnectionId`, `ProtocolVersion`, Hello/Join admission, session runtime bridge, and client/server connection runtimes are removed;
- no engine `runen-net-quic` or replacement transport runtime is introduced because the engine currently has no maintained concrete transport consumer;
- `engine_sim` and `engine_history` remain independent Runenwerk simulation/history owners.

Retained replication substrate includes:

- snapshot, delta, ACK, input-frame, and typed-payload envelopes;
- per-connection baseline/checkpoint state keyed by RunenNet `ConnectionHandle`;
- snapshot/delta construction and apply driver contracts;
- prediction/input integration;
- interest/streaming state and diagnostics;
- declarative replication metadata/macros pending later disposition.

## Completed Pre-RN8 Replication Hardening

The WR-007 Phase 1-3 work remains accepted historical implementation evidence. See the [WR-007 closeout](../reports/closeouts/wr-007-multiplayer-replication-phase-1-3/closeout.md).

It established retained replication invariants that RN8 lifecycle cuts must preserve:

- ACK validation rejects stale, future, unsent, and pruned cursors;
- invalid ACKs cannot poison per-connection baselines;
- delta entity lifecycle handling is deterministic;
- retained engine replication checkpoints and ACK outcomes are tested together;
- per-connection fallback remains local to the affected connection.

These completed phases do not authorize restoring their former `engine_net` session/runtime placement.

## Remaining Replication Work

The following concerns remain real, but their owning RN8 slice must be derived from current authority when prerequisites permit it:

- standard ECS component/resource extraction and apply;
- declarative replication authoring beyond low-level driver escape hatches;
- prediction/reconciliation and input-ack clarity;
- richer interest/relevancy resolution and explanation;
- recovery/history integration where it is not already standardized by RunenNet;
- per-connection diagnostics and desync inspection;
- eventual removal of retained `engine_net` replication/prediction residue after its maintained consumers migrate.

This list is a capability inventory, not an implementation sequence.

## Dependency Constraints

Future work must preserve these boundaries:

- RunenNet owns reusable networking semantics; Runenwerk does not recreate them in `engine_net` or engine ECS resources.
- Runenwerk owns ECS/scheduler/gameplay/world/product policy and presentation.
- Replication work must follow the available RunenECS boundary rather than freezing a future Replicated View contract early.
- Transport realization is added only for a proven maintained consumer.
- Host reconnect scheduling remains separate from RunenNet session retention/recovery semantics.
- `engine_net` is migration residue, not the destination for new reusable networking semantics.
- Clean migration/deletion is preferred over aliases, forwarding APIs, compatibility runtimes, or parallel authority.

## Validation Invariants

Any later replication migration must continue to prove, as applicable:

- deterministic snapshot/delta behavior;
- adversarial ACK/baseline handling;
- independent per-connection baselines;
- authorized connection identity sourced from RunenNet;
- prediction/reconciliation behavior preserved unless that boundary explicitly owns its redesign;
- world/gameplay relevancy policy remains outside reusable networking authority;
- repository canonical validation and focused owning-package tests are green on the exact reviewed head.

## Next Work Rule

Do not use this roadmap to infer or pre-publish the next RN8 implementation issue.

After N2 is merged and accepted-main validation is green, re-establish current Runenwerk, RunenNet, RunenECS, and active architecture authority. Then derive exactly one next RN8 boundary from the remaining consumer graph and prerequisite state.
