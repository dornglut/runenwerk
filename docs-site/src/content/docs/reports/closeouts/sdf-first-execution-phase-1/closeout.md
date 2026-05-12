---
title: SDF-First Execution Phase 1 Closeout
description: Completion and drift-check record for serial product jobs and deterministic publication barriers.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-12
related_designs:
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/accepted/execution-fabric-and-product-jobs-design.md
related_roadmaps:
  - ../../../workspace/sdf-first-execution-roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../domain/product/README.md
  - ../../../domain/scheduler/README.md
  - ../../../domain/ecs/README.md
---

# SDF-First Execution Phase 1 Closeout

## Status

Complete as of 2026-05-12 for the serial product jobs and deterministic
publication barrier boundary.

This closeout does not start worker pools, parallel product dispatch, a global
product registry, query snapshots, render product selection producers, GPU
upload or residency, procgen products, or strict consumer enforcement outside
the existing ratifiers.

## Completion Evidence

- `domain/product/src/publication.rs` owns product publication outcomes, status,
  reports, deterministic stage sequence, and diagnostics.
- `domain/product/src/ratification.rs::ratify_product_publication` rejects
  missing or undeclared outputs, invalid output descriptors, missing
  failed-preserved diagnostics, missing rejected diagnostics, and
  failed-preserved outcomes that do not use the preserve-with-diagnostic failure
  policy.
- `domain/scheduler/src/plan.rs::ExecutionPlan::from_ordered_nodes` emits
  `ApplyDeferredCommands` and then `ProductPublication` after every serial
  execution wave.
- `domain/ecs/src/system/runtime.rs::Runtime::add_barrier_handler` registers
  product-agnostic barrier handlers by `BarrierKind`; ECS still does not depend
  on `domain/product`.
- `engine/src/runtime/product_publication.rs::ProductPublicationRuntimeResource`
  stages publication outcomes, publishes them only from product-publication
  barrier handling, and keeps an inspectable deterministic journal.
- `engine/src/app/domain/app.rs::App::add_barrier_handler` exposes barrier
  handler installation for engine and app plugins.
- `engine/src/app/runtime/bootstrap.rs::App::install_builtin_resources` installs
  the default product publication resource and barrier handler.
- `apps/runenwerk_editor/src/asset_pipeline/product_publication.rs` stages
  field-product publication outcomes with the app-owned candidate and asset
  artifact, then publishes them at the editor product-publication barrier into
  `AssetCatalogRuntime`.
- `apps/runenwerk_editor/src/asset_pipeline/field_product_jobs.rs::run_field_product_job`
  converts current field-product job outcomes into publication outcomes while
  preserving serial execution and app ownership.

## Drift Corrections

- The workspace roadmap now records Phase 1 as complete and moves current focus
  to Phase 2 query snapshots and strict consumer policy.
- The repository priority checklist now distinguishes completed product
  publication barriers from remaining query, render, residency, and procgen
  gates.
- The roadmap index now links this closeout evidence and lists Phase 1 among
  finished cross-track baselines.
- Product, scheduler, ECS, and engine docs now mention the new publication
  contracts, deterministic barrier order, generic barrier hooks, and engine
  runtime staging resource.

## Deferred Work

- Phase 2 query snapshot production and strict runtime consumer enforcement.
- Phase 3 render product selection producers.
- Phase 4 derived GPU residency.
- Phase 5 procgen readiness and the accepted procgen domain document.
- Worker pools, parallel product dispatch, global product registry authority,
  GPU upload, and procgen implementation.

## Validation

Focused validation:

- `cargo fmt --all -- --check`
- `cargo test -p product`
- `cargo test -p scheduler`
- `cargo test -p ecs --test runtime_phase3`
- `cargo test -p engine product_publication`
- `cargo test -p runenwerk_editor field_product_publication`

Milestone closeout validation:

- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 1 - Serial Product Jobs And Publication Barriers" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
