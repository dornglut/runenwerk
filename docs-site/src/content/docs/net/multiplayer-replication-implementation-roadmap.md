---
title: "Multiplayer Replication Implementation Roadmap"
description: "Implementation order for Runenwerk networking and replication convergence work."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
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
---

# Multiplayer Replication Implementation Roadmap

## Purpose

This document is an implementation roadmap. It does not define
networking architecture.

Architecture lives in the active design package:

- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)
- [Prediction and reconciliation boundary](../design/active/net-prediction-reconciliation-boundary.md)
- [Engine net plugin runtime bridge](../design/active/net-plugin-runtime-bridge.md)
- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Interest and streaming](../design/active/net-interest-streaming-design.md)
- [Reconnect and history recovery](../design/active/net-reconnect-history-recovery.md)
- [Declarative replication authoring](../design/active/net-declarative-replication-authoring.md)
- [Transport lanes and delivery](../design/active/net-transport-lanes-delivery.md)
- [Diagnostics and inspection](../design/active/net-diagnostics-inspection.md)

Use this roadmap to decide what to implement next, in what order, and
which validation gate proves a phase is complete.

## Current Baseline

Implemented substrate:

- `engine_net` protocol/session/replication/runtime contracts;
- snapshot, delta, ACK, cursor, profile, interest, and lane vocabulary;
- `SnapshotTimeline` full/delta construction and pruning;
- client-side cursor and baseline validation in `ClientReplicationRuntime`;
- server-side per-connection full/delta choice in
  `AuthoritativeServerRuntime`;
- engine net plugin work queues, tick-buffer input flow, prediction
  replay, replication checkpoints, and diagnostics resources;
- `engine_net_quic` QUIC runtime adapter;
- `engine_history` replay/checkpoint/validation substrate;
- declarative metadata macros for component/entity descriptors.

Partial contracts:

- ACK validation does not yet reject unsent future cursors everywhere.
- Declarative metadata does not yet remove the need for low-level drivers
  in normal gameplay.
- Standard component/resource extraction and apply are partial.
- Interest predicates exist, but rich resolvers and explanation traces
  are future work.
- Reconnect currently relies primarily on resync; history-backed recovery
  is not yet the default path.
- Diagnostics are useful but not yet per-connection enough for desync
  triage.

## Non-Goals

This roadmap must not:

- move gameplay-specific semantics into `engine_net`;
- move QUIC or transport behavior into `engine_net` core;
- make transport own replication policy;
- make clients authoritative over replicated server state;
- use ECS events as the primary network truth path;
- duplicate ECS runtime primitives inside net crates;
- put smoothing/correction presentation policy in engine-generic code.

## Phase 0: Stabilize Current Branch Truth

Goal: keep docs and tests honest before adding new runtime behavior.

Tasks:

- classify current code as implemented, partial, or future in docs;
- keep old proposal/model pages marked as historical or superseded where
  they are no longer current;
- ensure docs link to the design package rather than restating
  architecture.

Completion criteria:

- docs validation passes;
- roadmap links to active designs;
- no doc claims generated snapshot/delta/apply code exists unless code
  proves it.

## Phase 1: ACK and Baseline Hardening

Goal: make ACK handling durable and adversarial.

Tasks:

- track sent cursors per connection where ACKs are accepted;
- reject or ignore ACKs for unknown, unsent, future, or pruned cursors;
- ensure bogus future ACKs cannot poison a connection baseline;
- emit structured diagnostics for ACK rejection and resync decisions.

Completion criteria:

- tests cover stale ACK, future ACK, unknown ACK, pruned ACK, and recovery
  after a later valid ACK;
- per-connection fallback remains local to the affected connection;
- `cargo test -p engine_net -p engine_sim` passes.

Linked design:

- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)

## Phase 2: Delta Lifecycle Contract

Goal: define and enforce entity lifecycle edge cases in deltas.

Tasks:

- decide whether same-delta spawn/despawn for one `NetEntityId` is
  rejected or despawn-wins;
- document the rule in protocol docs;
- add client and timeline tests for same-delta spawn/despawn;
- keep late upserts/removes for despawned entities from reintroducing
  stale state.

Completion criteria:

- protocol docs state the lifecycle rule;
- tests cover despawn plus late upsert, despawn plus late remove, and
  spawn plus despawn for the same ID;
- operation plans emit only authoritative incoming delta actions.

Linked design:

- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)

## Phase 3: Engine Bridge Baseline Convergence

Goal: align lower-level `engine_net` runtime contracts and engine plugin
checkpoint behavior.

Tasks:

- reconcile `AuthoritativeServerRuntime` state with engine plugin
  `ConnectionBaselineCheckpoint` fields;
- avoid duplicate or divergent baseline logic;
- decide which layer owns last-sent cursor tracking;
- update docs after code evidence exists.

Completion criteria:

- one authoritative contract for per-connection baseline fields;
- tests cover engine plugin and `engine_net` runtime behavior;
- docs no longer describe fields absent from the current owning layer.

Linked design:

- [Engine net plugin runtime bridge](../design/active/net-plugin-runtime-bridge.md)

## Phase 4: Standard ECS Component Extraction and Apply

Goal: make common ECS component replication possible without custom
drivers for every game.

Tasks:

- define standard extraction over ECS structural deltas;
- use replication metadata registry to select components/resources;
- define schema/version policy for component payloads;
- define standard apply behavior for spawn, despawn, upsert, and remove;
- preserve low-level driver escape hatches.

Completion criteria:

- examples replicate at least one normal component without custom
  transport glue;
- tests prove ECS events remain separate from input streams and
  replicated state;
- `engine_net` remains gameplay-agnostic.

Linked designs:

- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Declarative replication authoring](../design/active/net-declarative-replication-authoring.md)

## Phase 5: Prediction and Input ACK Clarity

Goal: make prediction replay correctness inspectable and independent of
ambiguous ACK semantics.

Tasks:

- decide whether input ACKs need a dedicated channel separate from
  snapshot ACKs;
- add diagnostics for pending predicted frames by tick;
- add tests for replay order after authoritative correction;
- keep smoothing policy in gameplay/app code.

Completion criteria:

- predicted input replay tests cover correction, pruning, and reconnect;
- docs distinguish correction/replay from presentation smoothing;
- no gameplay movement semantics enter net core.

Linked design:

- [Prediction and reconciliation boundary](../design/active/net-prediction-reconciliation-boundary.md)

## Phase 6: Interest Resolver and Streaming Diagnostics

Goal: make per-connection relevancy explainable without moving gameplay
visibility into net core.

Tasks:

- define an integration-facing interest resolver contract;
- add explanation traces for include/exclude decisions;
- harden streaming ACK gap and full-resync tests;
- document how chunk/world streaming and entity replication coexist.

Completion criteria:

- owner-only, distance, team, and spatial transitions are tested;
- streaming gaps force per-connection full resync;
- diagnostics identify why state was or was not sent.

Linked design:

- [Interest and streaming](../design/active/net-interest-streaming-design.md)

## Phase 7: Reconnect and History Recovery

Goal: connect replay/history substrate to reconnect recovery where useful.

Tasks:

- define generic ECS checkpoint capture/restore hooks;
- decide when reconnect uses checkpoint restore versus full snapshot
  resync;
- include replication cursors, queues, and ownership state in recovery
  diagnostics;
- add reconnect tests with pruned baselines and history gaps.

Completion criteria:

- reconnect recovery preserves authority semantics;
- tests cover full-resync recovery and history-backed recovery where
  implemented;
- `engine_history` remains a substrate, not gameplay policy.

Linked design:

- [Reconnect and history recovery](../design/active/net-reconnect-history-recovery.md)

## Phase 8: Transport Delivery Hardening

Goal: validate delivery behavior across lane mappings and QUIC runtime.

Tasks:

- add per-lane diagnostics for drops/backpressure;
- test unreliable sequenced delta reorder/loss behavior;
- verify profile-to-lane mapping in `engine_net` and `engine_net_quic`;
- define adapter requirements for future non-QUIC transports.

Completion criteria:

- QUIC integration tests cover snapshot, delta, ACK, input, reconnect,
  and dropped/reordered delivery;
- transport remains gameplay-agnostic;
- lane docs match code.

Linked design:

- [Transport lanes and delivery](../design/active/net-transport-lanes-delivery.md)

## Phase 9: Diagnostics and Inspection Package

Goal: make desync triage practical.

Tasks:

- add structured rejection reasons;
- expose per-connection replication diagnostics snapshots;
- add interest and streaming explanation traces;
- include queue, cursor, and ownership state in replay mismatch reports;
- document a standard desync triage workflow.

Completion criteria:

- diagnostics tests cover counters and reason codes;
- inspector-facing DTOs are read-only;
- docs explain what each counter means and which layer owns it.

Linked design:

- [Diagnostics and inspection](../design/active/net-diagnostics-inspection.md)

## Phase 10: Public Usage Path

Goal: make normal multiplayer integration discoverable and low
boilerplate.

Tasks:

- update the engine networking usage guide after declarative extraction
  exists;
- keep low-level driver examples as advanced escape hatches;
- add a small end-to-end example with one input stream and one replicated
  authoritative component;
- verify imports from `lib.rs`, `prelude.rs`, crate README pages, and the
  usage guide.

Completion criteria:

- normal users can find what to import and what to implement;
- examples do not rely on private shortcuts;
- docs and examples match current public APIs.

Linked designs:

- [Declarative replication authoring](../design/active/net-declarative-replication-authoring.md)
- [Engine net plugin runtime bridge](../design/active/net-plugin-runtime-bridge.md)

## Validation Gates

Documentation-only changes:

```text
python3 tools/docs/validate_docs.py
```

Net contract changes:

```text
cargo fmt --all -- --check
cargo test -p engine_net -p engine_sim
```

Public API or engine bridge changes:

```text
cargo check --workspace
```

Full gate when appropriate:

```text
./quiet_full_gate.sh
```
