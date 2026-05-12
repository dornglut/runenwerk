---
title: SDF-First Execution Phase 3 Closeout
description: Completion and drift-check record for render product selection producers.
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
  - ../../../engine/reference/architecture.md
---

# SDF-First Execution Phase 3 Closeout

## Status

Complete as of 2026-05-13 for render product selection producers.

This closeout does not start derived GPU residency, GPU upload, SDF terrain
renderer rebuilds, material/texture upload, procgen readiness/code, worker
pools, broad runtime consumers, or a global product registry.

## Completion Evidence

- `domain/product/src/render_selection.rs` now carries typed freshness,
  residency, authority, and query-policy state on `RenderSelectedProduct` and
  provides builders for selected products, required targets, residency requests,
  and diagnostics.
- `domain/product/src/ratification.rs::ratify_render_product_selection`
  rejects empty views, empty/duplicate selected products, invalid/duplicate
  targets, invalid residency requests, and selections whose typed product state
  cannot satisfy their query policy.
- `engine/src/plugins/render/frame/product_selection.rs` stores
  producer-scoped `RenderProductSelection` contributions keyed by
  `RenderFrameProducerId`, ratifies contributions, records diagnostics, and
  snapshots them deterministically for render prepare.
- `engine/src/plugins/render/runtime/mod.rs::RenderRuntimeSet::FramePrepare`
  gives app producers a stable ordering target before prepared-frame
  publication.
- `engine/src/plugins/render/inspect/prepared_frame.rs` exposes read-only
  product-selection inspection entries for selected products, required targets,
  residency requests, and diagnostic counts without backend handles.
- `apps/runenwerk_editor/src/runtime/viewport/render_product_selection.rs`
  produces editor viewport render selections from accepted query snapshots,
  viewport presentation state, viewport product targets, and render jobs during
  `RenderPrepare`.
- `apps/runenwerk_editor/src/editor_app/state.rs` records concise editor
  viewport render-selection journal entries and suppresses repeated console
  summaries when the aggregate state has not changed.

## Drift Corrections

- The workspace roadmap now records Phase 3 as complete and moves current focus
  to Phase 4 derived GPU residency.
- The repository priority checklist now distinguishes completed render product
  selection producers from remaining GPU residency and procgen gates.
- The roadmap index links this closeout evidence and lists Phase 3 among
  finished cross-track baselines.
- Product, engine architecture, and render roadmap docs now describe typed
  render selections, producer-scoped prepared selection contributions, and
  prepared-frame inspection.

## Deferred Work

- Phase 4 derived GPU residency.
- Phase 5 procgen readiness and the accepted procgen domain document.
- SDF terrain renderer rebuilds, material/texture upload, GPU upload,
  renderer-owned product authority, broad AI/physics/procgen consumers, worker
  pools, parallel product dispatch, and global product registry authority.

## Validation

All listed validation passed on 2026-05-13.

Focused validation:

- `cargo test -p product render_selection`
- `cargo test -p engine render_product_selection`
- `cargo test -p runenwerk_editor render_product_selection`

Milestone closeout validation:

- `cargo fmt --all -- --check`
- `cargo test -p product`
- `cargo test -p engine`
- `cargo test -p runenwerk_editor`
- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 3 - Render Product Selection Producers" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
