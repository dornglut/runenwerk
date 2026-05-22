---
title: WR-064 Sparse SDF Brick Page And Clipmap Residency Closeout
description: Closeout evidence for renderer-owned sparse SDF brick, page, and clipmap residency.
status: completed
owner: engine
layer: engine-runtime / renderer sdf residency
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-064-sparse-sdf-brick-page-and-clipmap-residency/plan.md
---

# WR-064 Sparse SDF Brick Page And Clipmap Residency Closeout

## Result

`WR-064` is complete at `bounded_contract` quality. The renderer now has a
derived sparse SDF residency contract for product-owned SDF payloads: selected
SDF products and residency requests are paired with domain-owned
`SdfChunkPayload` sources, then reported as renderer-owned brick atlas,
page-table, clipmap-window, byte, upload, invalidation, budget-pressure, and
inspection evidence.

This row does not implement raymarch acceleration, candidate lists, runtime SDF
visual proof, benchmarks, or production-readiness evidence. Those remain
visible `WR-065` and `WR-066` gaps.

## Changed Modules

- `engine/src/plugins/render/features/world/sdf_residency.rs`:
  added the owning module for `RenderSdfResidencySourceResource`,
  `RenderSdfResidencyResource`, `RenderSdfResidencyBudgetResource`,
  `RenderSdfResidencySummary`, `RenderSdfChunkResidencyEntry`,
  `RenderSdfPageResidencyRecord`, `RenderSdfBrickAtlasRecord`,
  `RenderSdfClipmapWindowRecord`, and
  `RenderSdfResidencyResource::derive_from_sources`.
- `engine/src/plugins/render/inspect/sdf_residency.rs`:
  added `inspect_render_sdf_residency` and backend-neutral inspection DTOs for
  SDF product, page, brick, clipmap, generation, invalidation, byte, upload,
  budget, and diagnostic evidence.
- `engine/src/plugins/render/features/world/mod.rs`:
  exported the `sdf_residency` module.
- `engine/src/plugins/render/inspect/mod.rs`:
  exported the SDF residency inspection DTOs.
- `engine/src/plugins/render/plugin.rs::RenderPlugin::build`:
  initializes `RenderSdfResidencySourceResource`,
  `RenderSdfResidencyResource`, and `RenderSdfResidencyBudgetResource`.
- `engine/tests/render_sdf_residency.rs`:
  added focused tests for successful page/brick/clipmap evidence,
  fail-closed missing/stale input diagnostics, budget pressure, and
  invalidation.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the new public SDF residency DTOs and fail-closed inspection
  contract.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents the normal sparse SDF residency workflow and the deferred WR-065
  and WR-066 boundaries.

## Architecture Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "WR-064 Sparse SDF Brick Page And Clipmap Residency" --scope "PM-RENDER-SDF-002 engine/src/plugins/render sparse SDF derived GPU residency brick atlas page table clipmap inspection"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer-derived
  SDF cache state, GPU residency evidence, inspection DTOs, budgets, and
  diagnostics.
- Source truth owner: `domain/world_sdf` and product publication own
  `SdfChunkPayload`, page tables, brick payloads, chunk revisions, payload
  checksums, freshness, query policy, fallback legality, and strict consumer
  semantics.
- Translation boundary: renderer SDF residency consumes `domain/product`
  selections and source `SdfChunkPayload` records; it does not own product truth
  or product fallback/query decisions.
- ADR decision: no ADR required for this bounded slice because dependency
  direction follows accepted SDF product, renderer SDF, and scale residency
  designs. ADR is still required before a later change introduces a persisted
  cross-domain SDF residency ABI, changes dependency direction, or moves SDF
  product policy into renderer code.

## Evidence

The implementation provides explicit renderer evidence for:

- selected and requested SDF product counts;
- resident product, page, brick, and clipmap-window counts;
- page-table records keyed by `SdfPageCoord3`;
- brick-atlas records keyed by page and brick coordinates;
- product generation, chunk revision, chunk generation, payload checksum, and
  renderer cache generation;
- resident bytes, upload bytes, resident-page pressure, resident-brick
  pressure, resident-byte pressure, upload-byte pressure, and clipmap-page
  pressure;
- fail-closed diagnostics for missing selected products, missing residency
  requests, missing payload sources, generation mismatch, stale products,
  nonresident products, unsupported query policy, and budget pressure.

## Validation

Focused validation passed:

```text
cargo fmt
cargo test -p engine render_sdf
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
```

The `render_sdf` filter ran the new `engine/tests/render_sdf_residency.rs`
coverage:

- `render_sdf_residency_reports_pages_bricks_clipmaps_and_budget_evidence`
- `render_sdf_residency_fails_closed_for_missing_payload_and_stale_product`
- `render_sdf_residency_reports_clipmap_budget_pressure_and_invalidation`

Final planning validation after roadmap and production metadata updates:

```text
task roadmap:render
task production:render
task planning:validate
```

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Raymarch acceleration, distance mips, empty-space skipping, step counts, and
  candidate lists remain `WR-065` scope.
- Runtime SDF examples, visual proof, benchmarks, hardware/profile evidence,
  and production readiness remain `WR-066` scope.
- This row derives renderer inspection DTOs and deterministic residency
  evidence; it does not submit a full SDF raymarch frame or claim
  `runtime_proven`.
- `perfectionist_verified` remains blocked until `PT-RENDER-PERFECTION`
  completes the final no-gap renderer audit.

These are expected production-track sequencing gaps, not hidden defects in the
WR-064 bounded contract.
