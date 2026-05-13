---
title: SDF-First Execution Phase 6A Closeout
description: Completion and drift-check record for the procgen domain product track.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
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
  - ../../../domain/graph/README.md
  - ../../../domain/world-ops/README.md
---

# SDF-First Execution Phase 6A Closeout

## Status

Complete as of 2026-05-13 for the domain-first procgen product track.

This phase creates `domain/procgen` and keeps runtime/editor execution
deferred. It does not add editor procgen providers, preview execution, bake
commands, concrete generator algorithms, worker pools, renderer changes, GPU
upload, or field-product payload bytes.

## Completion Evidence

- `domain/procgen` is a workspace domain crate with dependencies only on
  `domain/graph`, `domain/product`, `domain/world_ops`, `domain/spatial`, and
  `foundation/ratification`.
- `ProcgenDocument` wraps `graph::GraphDefinition` and carries procgen-owned
  node parameters, seed/version/scope/input/write-target/output, lowering,
  diagnostics, execution, and cache-lineage fields.
- `ProcgenNodeCatalog::first_slice()` admits only the terrain/material node
  families needed by Phase 6A: height/noise, material rule, world-operation
  output, field-product output, and diagnostics.
- `ratify_procgen_document` rejects unsupported or malformed graphs,
  unbounded scopes, invalid deterministic inputs, duplicate or invalid write
  targets, missing output nodes/products, cache-lineage drift, and reservation
  conflicts.
- `lower_procgen_to_world_ops` lowers ratified bounded terrain/material
  documents into deterministic `world_ops::OperationRecord` windows using
  `DensityFieldDeform` and `MaterialFieldEdit` metadata payloads.
- `build_procgen_product_contracts` creates product job descriptors, product
  descriptors, and ready publication outcomes that pass existing
  `domain/product` ratifiers.
- Tests prove identical seed/scope/version/inputs/upstream generations produce
  identical operation records, product contracts, diagnostics, and explanation
  data, while seed or upstream-generation changes alter deterministic ids.

## Drift Corrections

- The SDF-first roadmap now records Phase 6A as complete and names Phase 6B as
  the remaining editor/runtime procgen proof.
- The repo priority checklist now tracks remaining M6.2 work as editor/runtime
  proof on top of `domain/procgen`.
- The roadmap index and domain overview now list `domain/procgen` as an active
  domain crate instead of a future placeholder.
- The editor roadmap and procedural workflow plan now keep providers, preview,
  bake execution, and concrete algorithms deferred while acknowledging the
  accepted domain crate.
- The crate inventory and crate-docs status include `procgen`.

## Deferred Work

- Phase 6B editor/runtime procgen proof through product publication barriers,
  query snapshots, render selection, and derived GPU residency.
- Procgen graph canvas and preview providers.
- Preview and bake commands, rollback, authored-overlay merge/rebake commands,
  and app-owned diagnostics.
- Real field-product payload formation, concrete terrain/material/cave/stamp
  algorithms, scatter/biome systems, worker pools, and product-family GPU
  upload.
- Gameplay graph, particles, physics, animation, and world-process domains.

## Validation

Validation passed on 2026-05-13:

- `cargo test -p procgen`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 6A - Procgen Domain Product Track" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
