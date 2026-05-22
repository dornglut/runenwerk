---
title: WR-061 Renderer Scale Working Set Registry And Residency Budgets Closeout
description: Closeout evidence for renderer-owned working-set residency counts and budget-pressure inspection.
status: completed
owner: engine
layer: engine-runtime / renderer scale
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-061 Renderer Scale Working Set Registry And Residency Budgets Closeout

## Outcome

`WR-061` is complete at `bounded_contract` quality. The renderer now exposes a
finite GPU residency working-set summary with addressable, selected, requested,
accepted, resident, allocated, preserved, evicted, rejected, resident-byte, and
upload-byte evidence. Residency budget pressure is typed and inspectable for
resident entries, resident bytes, upload bytes, invalid byte estimates, and
hard-pinned over-budget states.

The row does not claim runtime scale proof or `perfectionist_verified`. GPU
driven culling, LOD, visible-list compaction, indirect submission, production
scale examples, benchmarks, and hardware profiles remain later `PT-RENDER-SCALE`
rows.

## Implementation Evidence

Changed modules:

- `engine/src/plugins/render/residency/resource.rs::RenderGpuResidencyBudgetResource`:
  adds finite resident-entry, resident-byte, upload-byte, resident-byte-per-entry,
  and upload-byte-per-allocation budget limits.
- `engine/src/plugins/render/residency/resource.rs::RenderGpuResidencyBudgetStatus`:
  adds explicit `within_budget`, `over_budget`, `invalid_budget`, and
  `not_measured` status vocabulary for renderer scale diagnostics.
- `engine/src/plugins/render/residency/resource.rs::RenderGpuResidencySummary`:
  records addressable, selected, requested, accepted, resident, byte, upload, and
  budget-pressure counts.
- `engine/src/plugins/render/residency/resource.rs::RenderGpuResidencyResource::derive_from_selections`:
  derives the renderer-owned working set from prepared product selections and
  residency requests while keeping product truth and policy outside the
  renderer.
- `engine/src/plugins/render/residency/resource.rs::RenderGpuResidencyResource::evaluate_budget_pressure`:
  classifies resident-entry, resident-byte, upload-byte, hard-pinned, and invalid
  budget pressure into typed diagnostics.
- `engine/src/plugins/render/inspect/gpu_residency.rs::RenderGpuResidencyInspection`:
  exposes working-set counts, byte totals, and budget inspection fields.
- `engine/src/plugins/render/inspect/gpu_residency.rs::RenderGpuResidencyBudgetInspection`:
  exposes renderer budget limits and statuses without backend handles.
- `engine/src/plugins/render/inspect/gpu_residency.rs::inspect_render_gpu_residency`:
  maps runtime residency state into backend-neutral DTOs with cache identity
  strings, product source-state lineage, and diagnostics.
- `engine/tests/render_scale_working_set.rs`: adds focused public tests for
  working-set counts, byte evidence, budget pressure, and missing selected
  products failing closed.
- `engine/tests/render_runtime_inspect.rs::render_runtime_inspect_render_gpu_residency_inspection_exposes_logical_cache_without_backend_handles`:
  extends the existing runtime inspection test with working-set and budget
  assertions.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents scale working-set residency evidence and the product-policy boundary.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the residency budget resource, budget status, inspection DTOs, and
  non-policy contract.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
```

Workflow validation after roadmap and production metadata updates:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- GPU-driven culling, LOD, visible-list compaction, and indirect draw/dispatch
  generation remain WR-062 scope.
- Runtime scale production examples, hardware profiles, benchmarks, and final
  scale-readiness evidence remain WR-063 scope.
- Byte evidence is renderer-owned deterministic budget estimation. It is not a
  backend allocator measurement and does not claim hardware memory residency
  proof.
- `perfectionist_verified` remains blocked until the final renderer perfection
  audit verifies the full production stack with no known quality gaps.

These gaps are sequencing boundaries for the renderer scale track, not hidden
defects in the WR-061 bounded implementation contract.
