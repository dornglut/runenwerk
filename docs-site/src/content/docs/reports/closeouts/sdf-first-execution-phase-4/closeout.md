---
title: SDF-First Execution Phase 4 Closeout
description: Completion and drift-check record for derived GPU residency.
status: completed
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../../workspace/sdf-first-execution-roadmap.md
  - ../../../workspace/repo-execution-priority-checklist.md
  - ../../../workspace/roadmap-index.md
related_domains:
  - ../../../engine/reference/architecture.md
  - ../../../engine/plugins/render/docs/roadmap.md
---

# SDF-First Execution Phase 4 Closeout

## Status

Complete as of 2026-05-13 for logical derived GPU residency.

This closeout does not start real `wgpu` product uploads, SDF terrain renderer
rebuilds, material or texture upload, procgen readiness/code, worker pools,
broad runtime consumers, or a global product registry.

## Completion Evidence

- `engine/src/plugins/render/residency/` owns renderer GPU residency resources,
  logical `RenderGpuCacheHandle` ids, deterministic allocation, preservation,
  invalidation, eviction, rejection, budget diagnostics, and residency
  journals.
- `engine/src/plugins/render/plugin.rs` initializes residency resources and
  derives residency during `RenderPrepare` before prepared-frame publication.
- `engine/src/plugins/render/runtime/mod.rs::RenderRuntimeSet::GpuResidency`
  gives producers and diagnostics a stable ordering target before
  `FramePrepare`.
- `engine/src/plugins/render/inspect/gpu_residency.rs` exposes read-only
  residency DTOs with product ids, generations, status, priority, hard-pin
  state, diagnostics, and logical cache ids without backend handles.
- `engine/src/plugins/render/features/world/runtime_cache.rs` now uses typed
  renderer cache handles and invalidates matching chunk cache entries when
  world render-cache invalidations mark chunks stale.
- `apps/runenwerk_editor/src/runtime/viewport/gpu_residency.rs` records
  bounded editor viewport residency summaries and avoids repeated console
  output when the summary is unchanged.

## Drift Corrections

- The workspace roadmap now records Phase 4 as complete and moves current focus
  to Phase 5 procgen readiness.
- The repository priority checklist now gates M6.2 procgen code on procgen
  readiness rather than derived GPU residency.
- The roadmap index lists Phase 4 among finished cross-track baselines.
- Engine architecture and render roadmap docs now describe logical derived GPU
  residency as renderer-owned cache state, not product truth.

## Deferred Work

- Phase 5 procgen readiness and the accepted procgen domain document.
- Real `wgpu` resource materialization for product-family uploads.
- SDF terrain renderer rebuilds, SDF brick/page-table residency, material and
  texture upload, broad AI/physics/procgen consumers, worker pools, parallel
  product dispatch, and global product registry authority.
- Query snapshot console warning cleanup unless it blocks later validation.

## Validation

All listed validation passed on 2026-05-13.

Focused validation:

- `cargo test -p engine render_gpu_residency`
- `cargo test -p engine world_render_cache`
- `cargo test -p runenwerk_editor gpu_residency`

Milestone closeout validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine`
- `cargo test -p runenwerk_editor`
- `python3 tools/docs/validate_docs.py`
- `./quiet_editor_gate.sh`
- `./quiet_full_gate.sh`
- `./workflow closeout --task "SDF-first execution roadmap Phase 4 - Derived GPU Residency" --roadmap "docs-site/src/content/docs/workspace/sdf-first-execution-roadmap.md"`
