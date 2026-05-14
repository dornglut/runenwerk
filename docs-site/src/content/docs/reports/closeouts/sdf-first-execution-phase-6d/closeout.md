---
title: SDF-First Execution Phase 6D Closeout
description: Completion and drift-check record for procgen bake, rollback, persistence, and runtime reload classification.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-14
related_designs:
  - ../../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/active/editor-procedural-content-and-simulation-workflow-plan.md
related_roadmaps:
  - ../../../workspace/sdf-first-execution-roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../domain/procgen/README.md
  - ../../../domain/product/README.md
---

# SDF-First Execution Phase 6D Closeout

## Status

Complete as of 2026-05-14 for procgen bake, rollback, app-owned persistence,
and runtime preview reload classification.

This phase does not add procgen worker pools, renderer rebuilds, GPU upload,
caves, stamps, scatter, biome systems, package sidecars, or a new product-job
API.

## Completion Evidence

- `domain/procgen/src/bake.rs` forms offline bake outcomes from accepted
  procgen documents using the existing ratification, lowering, field-preview,
  and product-contract helpers.
- Accepted bake outcomes carry `world_ops::OperationRecord` windows, formed
  `world_sdf::FieldPreviewProduct` payloads, product descriptors, changed
  regions, explanations, diagnostics, a product job descriptor, and rollback
  evidence.
- `apps/runenwerk_editor/src/runtime/procgen/mod.rs` publishes accepted bakes
  through `ProductPublicationRuntimeResource` at product-publication barriers
  and keeps query snapshots behind the existing query barrier.
- `apps/runenwerk_editor/src/runtime/procgen/mod.rs` restores the last accepted
  bake as last-good rollback state without treating invalid current procgen
  documents as authoritative.
- `apps/runenwerk_editor/src/persistence/procgen.rs` persists app-owned procgen
  bake archives as RON using product descriptors, operation records, formed
  preview products, changed regions, explanations, and diagnostics.
- `apps/runenwerk_editor/src/asset_pipeline/catalog_runtime.rs` classifies
  `AssetKind::ProcgenGraph` as live-reloadable and exposes a typed
  `RuntimeProductKind::ProcgenPreview` product reference.

## Drift Corrections

- The SDF-first roadmap now records Phase 6D as complete and moves procgen
  worker pools, renderer rebuilds, GPU upload, caves, stamps, scatter, and
  sidecar policy to later owning phases.
- The procgen domain README now distinguishes domain-owned deterministic bake
  outcomes from app-owned command, archive, and visibility policy.
- The workspace roadmap index and repository priority checklist now link Phase
  6D closeout evidence instead of listing Phase 6D as current work.
- The editor roadmap now treats M6.2C bake/rollback/persistence/reload
  classification as complete and leaves later procgen expansion gated by the
  product substrate.

## Deferred Work

- Procgen worker pools or broader runtime job execution for procgen.
- Renderer rebuilds, GPU upload, GPU residency upload payloads, and SDF
  renderer visual integration for procgen products.
- Caves, stamps, scatter, biome systems, package-level persistent cache
  sidecars, and package pruning policy.
- Gameplay graph, particles, physics, animation, and world-process domains.

## Validation

Focused validation passed on 2026-05-14:

- `cargo test -p procgen bake`
- `cargo test -p runenwerk_editor procgen_bake --lib`
- `cargo test -p runenwerk_editor procgen_graph_reload --lib`

Closeout validation:

- `cargo fmt --all -- --check`
- `cargo test -p procgen`
- `cargo test -p runenwerk_editor procgen --lib`
- `cargo check --workspace`
- `python tools/docs/validate_docs.py`
- `cargo clippy --workspace --all-targets --all-features --message-format=short -- -D warnings`
- `cargo test --workspace --all-features --quiet`
- `./quiet_full_gate.sh`
