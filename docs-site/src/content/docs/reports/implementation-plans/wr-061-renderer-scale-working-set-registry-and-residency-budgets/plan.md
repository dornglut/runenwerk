---
title: WR-061 Renderer Scale Working Set Registry And Residency Budgets Implementation Contract
description: Design-first contract for renderer-owned finite working-set registries, residency budgets, and inspection evidence.
status: active
owner: engine
layer: engine-runtime / renderer scale
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-061 Renderer Scale Working Set Registry And Residency Budgets Implementation Contract

## Goal

Prepare the bounded implementation slice for `PM-RENDER-SCALE-002` and
`WR-061`. This row introduces renderer-owned scale working-set registry DTOs and
residency budget diagnostics that explain finite renderer execution state
without moving product truth, product fallback, or product residency intent into
the renderer.

This is the active implementation contract after WR-061 roadmap application and
current-candidate promotion. It authorizes only the bounded renderer inspection
and residency-budget slice selected by the stack coordinator; later scale
culling, indirect submission, production examples, and hardware evidence remain
outside this row.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`:
  renderer scale is finite selected, resident, visible, submitted, and measured
  execution evidence.
- `docs-site/src/content/docs/design/accepted/render-product-graph-platform-design.md`:
  product domains and Product Jobs own product truth; renderer consumes prepared
  render selections and feature contributions.
- `docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md`:
  inspection DTOs, budgets, and closeout evidence must fail closed and avoid
  backend-handle leaks.
- `docs-site/src/content/docs/reports/closeouts/pm-render-scale-001-scale-residency-and-visibility-doctrine/closeout.md`:
  the scale doctrine milestone is complete and gates WR-061 design work.

## Readiness

Initial design-first readiness from
`task production:plan -- --milestone PM-RENDER-SCALE-002 --roadmap WR-061`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-061-renderer-scale-working-set-registry-and-residency-budgets/plan.md`.

Architecture governance was run for the accepted scale doctrine before this
contract:

```text
task ai:architecture-governance -- --task "Accept renderer scale residency and GPU-driven visibility doctrine for PM-RENDER-SCALE-001" --scope "docs-site/src/content/docs/design/active/renderer-scale-residency-and-gpu-driven-visibility-design.md docs-site/src/content/docs/workspace/production-tracks.yaml docs-site/src/content/docs/workspace/roadmap-deferred.yaml docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-working-set-registry-and- docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-gpu-driven-culling-lod-an docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-evidence-and-production-r"
```

No ADR is required for WR-061 if implementation only adds renderer-owned
derived registry and budget evidence. Stop for ADR or design update before
implementation if the slice introduces persisted cross-domain ABI, moves product
fallback or residency authority into renderer code, or makes renderer
working-set records canonical product truth.

Current implementation readiness now reports:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-060` is completed;
- next stack action: `execute_next_wr_implementation_contract`;
- WR plan action classification: `write_implementation_contract`;
- batch eligibility: current-candidate eligible after policy-deferred metadata
  was cleared from the active roadmap row.

## Promotion Readiness

After the design-first contract and intake proposal were applied,
`task production:plan -- --milestone PM-RENDER-SCALE-002 --roadmap WR-061`
reported:

- production milestone state: `active`;
- roadmap state: `ready_next`;
- dependency evidence: `WR-060` is completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

Accepted promotion evidence:

- `PM-RENDER-SCALE-001` accepted scale doctrine closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-scale-001-scale-residency-and-visibility-doctrine/closeout.md`.
- Accepted scale doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md`.
- Completed renderer GPU/procedural production evidence:
  `docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md`.
- This WR-061 design-first implementation contract.

Promotion command:

```text
task roadmap:promote -- --id WR-061 --state current_candidate --evidence "Accepted scale doctrine closeout, completed WR-060 renderer GPU/procedural production evidence, and WR-061 design-first registry/residency budget implementation contract."
```

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Implementation Scope

Allowed write scopes after roadmap promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-scale-working-set-registry-and-
docs-site/src/content/docs/reports/implementation-plans/wr-061-renderer-scale-working-set-registry-and-residency-budgets/plan.md
docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/residency/resource.rs`:
  `RenderGpuResidencyBudgetResource`, `RenderGpuResidencySummary`,
  `RenderGpuResidencyResource::derive_from_selections`, and helper diagnostics
  are the owning residency execution state. Extend these contracts for bounded
  resident entry, byte, and upload-pressure evidence only.
- `engine/src/plugins/render/inspect/gpu_residency.rs`:
  `RenderGpuResidencyInspection`, `RenderGpuResidencyInspectionEntry`, and
  `inspect_render_gpu_residency` are the public DTO/export boundary for current
  residency evidence. Extend this file for working-set records and budget
  pressure instead of leaking backend handles.
- `engine/src/plugins/render/inspect/budgets.rs`:
  `RenderReadinessBudgetKind`, `RenderReadinessBudgetSnapshot`, and
  `evaluate_render_readiness_budgets` are the existing aggregate budget
  diagnostic pattern. Add scale budget aggregation here only if the new
  residency DTOs need readiness-budget integration.
- `engine/src/plugins/render/inspect/mod.rs` and
  `engine/src/plugins/render/residency/mod.rs`: module-boundary exports only.
- `engine/tests/render_scale_working_set.rs`: integration tests for public
  DTOs, budget diagnostics, invalid count/byte pressure, and product-boundary
  failures. Unit tests may remain beside private helpers in
  `residency/resource.rs` when that is the smallest owner.

## Required Contracts

WR-061 must introduce explicit renderer evidence for:

- addressable count: descriptive product-lineage count supplied by prepared
  selections or feature contributions;
- selected count: records handed to renderer execution for this frame or view;
- resident count and bytes: derived GPU-backed records, texture/buffer ranges,
  or page/cluster ranges currently available;
- upload pressure: bytes scheduled or rejected for renderer-owned upload work;
- memory pressure: budget class, used bytes, budget bytes, over-budget state,
  and degraded-mode diagnostic;
- lineage: stable debug identity that can point back to product owners without
  making renderer code authoritative for product truth.

The API must fail closed:

- missing lineage, missing budget class, negative or overflowing counts, and
  resident counts larger than selected/addressable counts must be diagnostics;
- over-budget or unsupported states must be explicit DTOs;
- no broad fallback may silently hide pressure, eviction, or missing capability.

## Non-Goals

WR-061 does not implement:

- GPU-driven culling, LOD selection, visible-list compaction, or indirect
  command generation; those belong to WR-062.
- Production scale examples, hardware profile reports, or final scale benchmark
  evidence; those belong to WR-063.
- Product streaming, fallback, freshness, semantic LOD, material/model/SDF
  truth, editor asset workflows, or gameplay entity semantics.
- Backend-specific memory allocator rewrites unless required to expose existing
  renderer-owned derived evidence.

## Implementation Steps

1. Extend `RenderGpuResidencyBudgetResource` with explicit renderer-owned
   resident-entry, resident-byte, and upload-byte budgets. Keep defaults finite
   and deterministic.
2. Extend `RenderGpuResidencySummary` and `RenderGpuResidencyResource` so the
   last summary distinguishes selected, accepted residency requests, resident,
   allocated, preserved, evicted, rejected, resident bytes, upload bytes, and
   pressure diagnostics. Do not add product fallback or streaming decisions.
3. Add typed pressure classification and diagnostics for over-entry,
   over-memory, over-upload, and hard-pinned-over-budget states. Diagnostics
   must be visible in both the residency resource and inspection DTOs.
4. Extend `inspect_render_gpu_residency` with working-set records that include
   product lineage, source state, cache id string, resident byte/upload byte
   estimates, and budget-pressure status without exposing WGPU handles or
   backend allocator internals.
5. Add `engine/tests/render_scale_working_set.rs` so `cargo test -p engine
   render_scale` proves valid residency evidence, over-budget diagnostics,
   missing/invalid product requests fail closed, and inspection DTOs are public
   and descriptive.
6. Update the renderer reference docs to explain addressable, selected,
   accepted, resident, submitted-later, budget pressure, and degraded-mode
   evidence, with WR-062/WR-063 scope called out as deferred.
7. Close the row only after focused tests, docs validation, roadmap validation,
   production validation, and planning validation pass.

## Required Validation

Implementation validation must include:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

If no `render_scale` test target exists yet, create focused tests in
`engine/tests/render_scale_working_set.rs` and run them through the same
`cargo test -p engine render_scale` filter.

## Stop Conditions

Stop before implementation if:

- WR-061 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- a proposed registry type would own product truth, fallback, freshness,
  authority, or residency intent;
- a budget diagnostic would silently choose product fallback or streaming
  behavior;
- implementation needs a new persisted cross-domain ABI without ADR/design
  acceptance;
- validation cannot prove no backend handles or product source objects leak
  through public inspection DTOs.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-061-renderer-scale-working-set-registry-and-residency-budgets/closeout.md
```

The closeout must record:

- exact changed modules and functions;
- registry DTO and budget diagnostic evidence;
- focused tests and validation output;
- docs updates;
- completion quality and known gaps.

Expected completion quality is `bounded_contract` unless runtime evidence and
benchmarks exceed this contract. `runtime_proven` remains a later
`PT-RENDER-SCALE` claim after WR-062 and WR-063 close.
