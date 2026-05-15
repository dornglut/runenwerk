---
title: WR-007 Multiplayer Replication Phase 1-3 Closeout
description: Completion and drift-check record for ACK/baseline hardening, delta lifecycle rules, and engine bridge baseline convergence.
status: completed
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ../../../design/active/net-authoritative-replication-protocol.md
  - ../../../design/active/net-plugin-runtime-bridge.md
  - ../../../design/active/ecs-net-replication-boundary.md
related_roadmaps:
  - ../../../net/multiplayer-replication-implementation-roadmap.md
  - ../../../workspace/roadmap-index.md
  - ../../../workspace/repo-execution-priority-checklist.md
related_reports:
  - ../../batches/2026-05-15-next-current-candidate-roadmap-batch-wr-/batch.md
---

# WR-007 Multiplayer Replication Phase 1-3 Closeout

## Status

Complete as of 2026-05-15 for ACK/baseline hardening, delta lifecycle
normalization, and engine bridge baseline convergence.

This closeout does not implement standard ECS component extraction, declarative
gameplay replication authoring, richer interest resolvers, reconnect history
recovery, transport delivery hardening, or the public usage path.

## Completion Evidence

- `net/engine_net/src/runtime/server.rs` rejects stale, future, unsent, and
  pruned snapshot ACKs before mutating the per-connection baseline and records
  accepted/rejected ACK diagnostics.
- `net/engine_net/src/replication/diagnostics.rs` exposes
  `SnapshotAckOutcome` and `SnapshotAckRejection` as the shared ACK validation
  vocabulary.
- `net/engine_net/src/replication/timeline.rs` normalizes delta lifecycle
  actions so despawn wins over same-delta or late stale upsert/remove actions.
- `net/engine_net/src/runtime/client.rs` emits normalized incoming delta apply
  actions after lifecycle normalization.
- `engine/src/plugins/net/resources.rs` owns
  `ConnectionBaselineCheckpoint::mark_snapshot_acknowledged` and
  `ConnectionBaselineCheckpoint::mark_snapshot_sent` for engine plugin
  checkpoint state, using the `engine_net` ACK outcome vocabulary.
- `engine/src/plugins/net/runtime_io.rs` routes client snapshot ACK messages
  through the checkpoint validation path and updates streaming ACK state only
  after accepted ACKs.
- `engine/src/plugins/net/prediction.rs` updates sent snapshot cursor metadata
  through the checkpoint API when full or delta snapshots are emitted.
- `engine/tests/network_plugins/delta_and_reconnect.rs` covers rejected future
  ACKs so forged cursors cannot become delta baselines in the engine plugin.

## Drift Corrections

- The net roadmap now records Phase 1 through Phase 3 as completed evidence and
  removes the stale partial-baseline claim that ACK validation did not reject
  unsent future cursors everywhere.
- The workspace roadmap index and repository execution checklist no longer list
  WR-007 as current work.
- The generated roadmap register and triage already classify WR-007 as
  completed; this closeout links the evidence from the workspace index.
- The initial parallel batch report remains historical for the worker batch but
  now points to this closeout for the later Phase 3 bridge convergence.

## Deferred Work

- Phase 4: standard ECS component/resource extraction and apply.
- Phase 5: prediction/input ACK clarity and richer prediction diagnostics.
- Phase 6: interest resolver contracts and streaming diagnostics.
- Phase 7: reconnect history recovery.
- Phase 8: transport delivery hardening.
- Phase 9: desync diagnostics and inspection package.
- Phase 10: public multiplayer usage path and examples.

## Validation

Focused implementation validation passed on 2026-05-15:

- `cargo fmt --all -- --check`
- `uv run pytest tools/workflow/test_workflow.py -q`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task puml:validate`
- `cargo test -p engine_net -p engine_sim`
- `cargo check -p runenwerk_editor`
- `cargo test -p runenwerk_editor viewport`
- `cargo test -p ecs -p scheduler`
- `cargo test -p engine --test network_plugins`
- `cargo test -p engine checkpoint_`

Closeout validation:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task puml:validate`
- `./quiet_full_gate.sh`
