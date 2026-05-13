---
title: SDF-First Execution Phase 6B Closeout
description: Completion and drift-check record for the visible procgen overlay proof.
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
---

# SDF-First Execution Phase 6B Closeout

## Status

Complete as of 2026-05-13 for the visible procgen overlay proof.

This phase adds editor/runtime proof only. It does not add concrete terrain or
material generator algorithms, field-product payload bytes, bake commands,
persistence, runtime preview reload, worker pools, renderer rebuilds, or GPU
upload.

## Completion Evidence

- `apps/runenwerk_editor/src/runtime/procgen/mod.rs` owns app-local procgen
  proof state, builds a bounded default `ProcgenDocument`, ratifies and lowers
  it through `domain/procgen`, and exposes changed-region/reservation status
  lines.
- The editor stages procgen product publication outcomes only at
  `ProductPublication` barriers and stages procgen query snapshots only at
  `QuerySnapshotPublication` barriers.
- Published procgen operation-window and field-candidate snapshots use strict
  renderer consumption policy and feed the existing viewport render selection
  and derived GPU residency path as selected overlay products.
- `ProcgenGraphCanvasProvider` and `ProcgenPreviewProvider` resolve before the
  M6 fallback and show deterministic procgen graph, ratification, lowering,
  substrate, changed-region, and reservation summaries.
- The viewport shell can project generic overlay status lines, which Phase 6B
  uses for bounded procgen changed-region and reservation visibility.
- Invalid or unbounded procgen documents remain fail-closed: no product
  publication, no query snapshot, and no selected strict overlay product.

## Drift Corrections

- The SDF-first roadmap now records Phase 6B as complete and names Phase 6C as
  the next procgen slice.
- The repo priority checklist now tracks Phase 6C first concrete
  terrain/material generation rather than Phase 6B provider/runtime proof.
- The roadmap index, procgen domain README, editor roadmap, and procedural
  workflow plan now reflect concrete procgen providers and overlay proof while
  keeping algorithms, field bytes, bake, persistence, and reload work deferred.

## Deferred Work

- Phase 6C / M6.2B: deterministic value-noise height surface and material rule
  generation, CPU field-preview payload formation, and formed field-product
  candidate publication.
- Phase 6D / M6.2C: bake-to-`world_ops`, bake-to-field-product, rollback,
  procgen source/cache persistence, and runtime preview reload classification.
- Caves, stamps, scatter, biome systems, worker pools, SDF renderer rebuild,
  material/texture GPU upload, product-family GPU upload, and procgen UI
  editing commands.
- Gameplay graph, particles, physics, animation, and world-process domains.

## Validation

Validation passed on 2026-05-13:

- `cargo test -p procgen`
- `cargo test -p editor_shell viewport_status_overlay_projects_generic_overlay_status_lines`
- `cargo test -p runenwerk_editor procgen`
- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 6B - Visible Procgen Overlay Proof" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
