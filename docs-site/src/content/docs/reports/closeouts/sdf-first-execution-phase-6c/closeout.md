---
title: SDF-First Execution Phase 6C Closeout
description: Completion and drift-check record for the first concrete procgen terrain/material CPU preview.
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

# SDF-First Execution Phase 6C Closeout

## Status

Complete as of 2026-05-13 for the first concrete deterministic
terrain/material CPU preview.

This phase adds deterministic preview payload formation and formed preview
descriptor publication. It does not add bake commands, persistence, rollback,
runtime preview reload classification, worker pools, renderer rebuilds, GPU
upload, caves, stamps, scatter, or biome systems.

## Completion Evidence

- `domain/procgen/src/field_preview.rs` forms deterministic
  `world_sdf::FieldPreviewProduct` payloads for scalar distance and material
  channels from bounded procgen documents.
- Field-preview formation fails closed for ratification failures, missing
  density/material inputs, missing height/material nodes, missing material
  channels, invalid bounds, oversized bounds, and rejected preview products.
- `domain/procgen/src/products.rs` keeps the Phase 6A metadata builder and adds
  formed-preview product contracts that publish the operation-window descriptor
  plus concrete scalar/material preview descriptors.
- `runenwerk_editor` forms previews only on the procgen product-publication
  path, snapshots only descriptors that published, and uses formed preview
  product ids for app-owned viewport overlays.
- Procgen Preview and Field Product Viewer surfaces expose preview product ids,
  payload kinds, grid dimensions, sample counts, distance ranges, material
  masks, publication/query status, and diagnostics.

## Drift Corrections

- The SDF-first roadmap now records Phase 6C as complete and names Phase 6D as
  the next procgen slice.
- The repo priority checklist now tracks Phase 6D bake/rollback/persistence
  work rather than concrete generator formation.
- The roadmap index, procgen domain README, and editor roadmap now reflect
  formed CPU field-preview products while keeping render, GPU, bake,
  persistence, and worker-pool work deferred.

## Deferred Work

- Phase 6D / M6.2C: bake-to-`world_ops`, bake-to-field-product, rollback,
  procgen source/cache persistence, and runtime preview reload classification.
- Caves, stamps, scatter, biome systems, worker pools, SDF renderer rebuild,
  material/texture GPU upload, product-family GPU upload, and procgen UI
  editing commands.
- Gameplay graph, particles, physics, animation, and world-process domains.

## Validation

Validation passed on 2026-05-13:

- `cargo test -p procgen field_preview`
- `cargo test -p procgen`
- `cargo test -p runenwerk_editor procgen`

Closeout validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace`
- `python3 tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 6C - First Concrete Terrain/Material Generator" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
