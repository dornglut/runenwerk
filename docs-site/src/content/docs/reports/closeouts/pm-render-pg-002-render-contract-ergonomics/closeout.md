---
title: PM-RENDER-PG-002 Render Contract Ergonomics Closeout
description: Closeout evidence for the bounded render contract ergonomics implementation slice.
status: completed
owner: engine
layer: engine-runtime / render public API
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/render-contract-ergonomics-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# PM-RENDER-PG-002 Render Contract Ergonomics Closeout

## Result

`PM-RENDER-PG-002` completed as a bounded render contract ergonomics slice.

The implementation adds shared, return-only render product-surface request
helpers and migrates exactly two producers:

- editor viewport render requests;
- material preview render requests.

Both producers still publish explicitly into ECS resources. The helper path does
not publish into ECS, does not expose renderer-private handles, and does not
move product truth, product selection, freshness, fallback legality, authority,
rebuild policy, or residency policy into renderer helper code.

## Implementation Evidence

- `engine/src/plugins/render/frame/product_surface.rs` adds
  `RenderProductSurfaceRequest` and `RenderProductSurfaceRequestBatch`.
- `engine/src/plugins/render/frame/view.rs` adds
  `PreparedViewFrame::with_history_signature(...)`.
- `engine/src/plugins/render/frame/packet.rs` adds
  `PreparedFlowInvocationRequest::new(...)`, alias/history/uniform helpers,
  `PreparedRenderFrameRequestError`, `PreparedRenderFrameRequestDiagnostic`,
  `PreparedRenderFrameRequestKind`, and
  `PreparedRenderFrameRequestResource::diagnostics()`.
- `engine/src/plugins/render/resource/dynamic_target.rs` adds common dynamic
  target descriptor constructors.
- `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs` uses the shared
  product-surface helper path for viewport prepared views and invocations.
- `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` uses the same
  helper path for material preview prepared views, invocations, and its
  provider-owned dynamic surface target.
- Render public API docs and the render flow usage guide document the common
  authoring path and return-only publication boundary.

## Validation

Focused validation passed:

```text
cargo test -p engine render_dynamic_targets
cargo test -p engine render_runtime_inspect
cargo test -p runenwerk_editor viewport::render_jobs
cargo test -p runenwerk_editor material_preview
```

Workflow validation passed before closeout:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-RENDER-PG
```

## Completion Quality

Completion quality is `bounded_contract`.

This is not `runtime_proven`: the slice improves request construction and
diagnostics for existing render product-surface producers, but it does not claim
new GPU product-chain or pixel-proof behavior.

This is not `perfectionist_verified`: later production milestones still own
feature-owned contribution collectors, compiler maturity, broad product-surface
hardening, multi-surface presentation, render fragments, and final production
inspection.

## Known Gaps

- `PM-RENDER-PG-003` still owns feature-owned render contribution collectors.
- `PM-RENDER-PG-004` still owns render execution graph compiler maturity.
- `PM-RENDER-PG-005` still owns broad product-surface hardening beyond viewport
  and material preview request ergonomics.
- `PM-RENDER-PG-006` still owns multi-surface presentation.
- `PM-RENDER-PG-007` still owns render fragments and hot reload.
- `PM-RENDER-PG-008` still owns production readiness, capture/replay, budgets,
  final examples, and final inspection evidence.
