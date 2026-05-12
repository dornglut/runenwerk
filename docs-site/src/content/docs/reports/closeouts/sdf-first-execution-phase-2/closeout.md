---
title: SDF-First Execution Phase 2 Closeout
description: Completion and drift-check record for query snapshots and strict consumer policy.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../../workspace/sdf-first-execution-roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../domain/product/README.md
  - ../../../domain/scheduler/README.md
  - ../../../domain/ecs/README.md
---

# SDF-First Execution Phase 2 Closeout

## Status

Complete as of 2026-05-13 for runtime query snapshots and strict consumer
policy.

This closeout does not start render product selection producers, GPU residency,
procgen readiness/code, broad AI/physics consumers, worker pools, parallel query
execution, or a global product registry.

## Completion Evidence

- `domain/product/src/consumption.rs` owns strict runtime consumption requests,
  decisions, and diagnostics through `evaluate_product_consumption`.
- `domain/product/src/query_snapshot.rs` keeps query snapshot descriptors and
  adds publication status/report DTOs for published, rejected, preserved, and
  invalidated snapshots.
- `domain/product/src/ratification.rs::ratify_query_snapshot_product` checks
  generation ordering, mirrored descriptor fields, failed-preserved
  diagnostics, and strict-consumption rejection.
- `domain/scheduler/src/plan.rs::ExecutionPlan::from_ordered_nodes` emits
  `ApplyDeferredCommands`, `ProductPublication`, then
  `QuerySnapshotPublication` after every serial execution wave.
- `domain/ecs/src/query/snapshot.rs::query_snapshot_source_generation` computes
  deterministic product-agnostic source generations from component/resource
  change tracking.
- `engine/src/runtime/query_snapshot.rs::QuerySnapshotRuntimeResource` stages
  snapshots, publishes only at `QuerySnapshotPublication`, ratifies strict
  consumption, preserves prior snapshots on rejected updates, invalidates
  deterministically on source generation changes, and records an inspectable
  journal.
- `engine/src/plugins/render/inspect/query_snapshot.rs` exposes read-only
  query snapshot inspection DTOs without backend handles or render-selection
  production.
- `apps/runenwerk_editor/src/runtime/viewport/query_snapshots.rs` publishes
  viewport observation snapshots through the app-owned query snapshot barrier
  handler and records editor journal/console diagnostics.

## Drift Corrections

- The workspace roadmap now records Phase 2 as complete and moves current focus
  to Phase 3 render product selection producers.
- The repository priority checklist now distinguishes completed query
  snapshot/strict-consumer enforcement from remaining render, residency, and
  procgen gates.
- The roadmap index links this closeout evidence and lists Phase 2 among
  finished cross-track baselines.
- Product and ECS docs now mention strict runtime consumption APIs, query
  snapshot publication reports, and product-agnostic ECS source-generation
  helpers.

## Deferred Work

- Phase 3 render product selection producers.
- Phase 4 derived GPU residency.
- Phase 5 procgen readiness and the accepted procgen domain document.
- Broad AI/physics/procgen consumers, parallel query execution, global product
  registry authority, GPU upload, worker pools, and procgen implementation.

## Validation

Focused validation:

- `cargo test -p product`
- `cargo test -p scheduler`
- `cargo test -p ecs query_snapshot`
- `cargo test -p engine query_snapshot`
- `cargo test -p engine render_query_snapshot`
- `cargo test -p runenwerk_editor query_snapshot`

Milestone closeout validation:

- `cargo fmt --all -- --check`
- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 2 - Query Snapshots And Strict Consumer Policy" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
