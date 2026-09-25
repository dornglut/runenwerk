---
title: "Multiplayer Replication Implementation Roadmap"
description: "Current constraints and retained replication work under the RN8 RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-24
related_designs:
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-plugin-runtime-bridge.md
  - ../design/active/ecs-net-replication-boundary.md
  - ../design/active/net-reconnect-history-recovery.md
  - ../design/active/net-transport-lanes-delivery.md
  - ../design/active/net-diagnostics-inspection.md
---

# Multiplayer Replication Implementation Roadmap

## Purpose

This document records retained Runenwerk replication work and its constraints during the RN8 cutover to standalone RunenNet.

It does **not** define the next RN8 boundary. RN8 remains dependency-ordered: after each accepted slice, current repository and upstream authority must be re-established before exactly one next boundary is derived.

Current networking ownership and architecture are defined by:

- [net goals](goals.md);
- [networking architecture](net-architecture.md);
- [engine integration design](../design/active/net-plugin-runtime-bridge.md).

## Current Baseline Through RN8 N7

Connection/session authority has moved out of retained `engine_net` and into standalone RunenNet Core.

Current boundary:

- RunenNet `ConnectionHandle` is the connection identity used by engine routing and retained replication state;
- RunenNet compatibility negotiation and `Session` own admission/binding/loss/retention/replacement/expiry/closure semantics;
- `engine/src/plugins/net` owns engine scheduling, read-only projections, owner routing, host reconnect policy, diagnostics, explicit integration policy, active complete client product realization, and the remaining server-replication/prediction integration;
- retained `engine_net` contains only evidence-backed live replication/protocol-payload/input/authoring migration contracts;
- old connection/session lifecycle state, replication-runtime command/events, synthetic transport-lane/delivery-guarantee mappings, lane-route diagnostics, and the standalone snapshot-payload prediction helper are removed;
- no engine `runen-net-quic` or replacement transport runtime is introduced because the engine currently has no maintained concrete transport consumer;
- `engine_sim` and `engine_history` remain independent Runenwerk simulation/history owners.

Retained live replication substrate includes:

- snapshot, delta, ACK, input-frame, and typed-payload envelopes;
- server per-connection baseline/checkpoint state keyed by RunenNet `ConnectionHandle`;
- RunenNet-owned client replication consistency/history/recovery through `ClientReplicationSet` with exact encoded complete products;
- snapshot/delta construction plus retained complete-snapshot realization driver contracts;
- live pending-input prediction/replay integration in the engine plugin;
- interest/streaming state and diagnostics;
- declarative replication metadata/macros pending later disposition.

## Completed Pre-RN8 Replication Hardening

The WR-007 Phase 1-3 work remains accepted historical implementation evidence in Git history.

It established retained replication invariants that RN8 lifecycle cuts must preserve:

- ACK validation rejects stale, future, unsent, and pruned cursors;
- invalid ACKs cannot poison per-connection baselines;
- delta entity lifecycle handling is deterministic;
- retained engine replication checkpoints and ACK outcomes are tested together;
- per-connection fallback remains local to the affected connection.

These completed phases do not authorize restoring their former `engine_net` session/runtime placement.

## RN8 Progression Constraints

N3 identified several dependency-ordered ownership gaps. Subsequent accepted work resolved two of them without compatibility mirrors:

- N5 moved authoritative remote participant-input admission to RunenNet `AuthorityInputSession` with explicit finite Runenwerk policy;
- N6 identified atomic client host commit as the client cut blocker;
- Replicated View R0 proved a complete immutable encoded product, exact byte accounting, stable replicated identity, and one owner-level activation operation;
- N7 moves client cursor/history/recovery consistency to RunenNet `ClientReplicationSet`.

Remaining hard constraints are still dependency ordered:

- RunenNet prediction must observe the live `ClientReplicationSet`; no prediction-first mirror is allowed;
- authority replication needs actual RunenNet `DeliveryAcceptance`; engine work-queue admission is not equivalent;
- declarative profile/reliability/interest/authoring disposition remains coupled to #322 evidence rather than old component-registration plumbing.

Do not solve remaining gaps with aliases, guessed defaults, fake delivery acceptance, or interim runtimes.

## Remaining Replication Work

The following concerns remain real, but their owning RN8 slice must be derived from current authority when prerequisites permit it:

- migration of authority replication consistency/delivery state to RunenNet once real `DeliveryAcceptance` exists;
- prediction/reconciliation integration against the live RunenNet client replication authority;
- standard ECS component/resource extraction and apply;
- declarative replication authoring beyond low-level driver escape hatches;
- richer interest/relevancy resolution and explanation;
- recovery/history integration where it is not already standardized by RunenNet;
- per-connection diagnostics and desync inspection;
- eventual removal of retained `engine_net` live residue after its maintained consumers migrate.

This list is a capability inventory, not an implementation sequence.

## Dependency Constraints

Future work must preserve these boundaries:

- RunenNet owns reusable networking semantics; Runenwerk does not recreate them in `engine_net` or engine ECS resources.
- Runenwerk owns ECS/scheduler/gameplay/world/product policy and presentation.
- Replication work must follow the available RunenECS boundary rather than freezing a future Replicated View contract early.
- Transport realization is added only for a proven maintained consumer.
- Engine work queues are staging, not RunenNet delivery acceptance.
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

After each RN8 child is merged and accepted-main validation is green, re-establish current Runenwerk, RunenNet, RunenECS, and active architecture authority. Then derive exactly one next RN8 boundary from the remaining consumer graph and prerequisite state.
