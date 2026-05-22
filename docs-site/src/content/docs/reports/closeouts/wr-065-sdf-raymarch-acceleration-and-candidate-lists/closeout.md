---
title: WR-065 SDF Raymarch Acceleration And Candidate Lists Closeout
description: Closeout evidence for conservative renderer-owned SDF raymarch acceleration and candidate-list diagnostics.
status: completed
owner: engine
layer: engine-runtime / renderer sdf raymarch acceleration
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
  - ../../implementation-plans/wr-065-sdf-raymarch-acceleration-and-candidate-lists/plan.md
---

# WR-065 SDF Raymarch Acceleration And Candidate Lists Closeout

## Result

`WR-065` is complete at `bounded_contract` quality. The renderer now derives
conservative SDF raymarch acceleration evidence from completed SDF residency
state: distance mip safe-step records, screen-tile/depth-slice candidate lists,
candidate rejection counts, step-budget metadata, and fail-closed diagnostics.

This row does not implement runtime SDF examples, visual proof, benchmarks,
hardware/profile evidence, or production-readiness evidence. Those remain
visible `WR-066` gaps. Final no-gap audit remains
`PT-RENDER-PERFECTION` scope.

## Changed Modules

- `engine/src/plugins/render/features/world/sdf_raymarch.rs`:
  added the owning module for `RenderSdfRaymarchAccelerationConfig`,
  `RenderSdfRaymarchAccelerationResource`,
  `RenderSdfRaymarchAccelerationReport`, `RenderSdfDistanceMipLevel`,
  `RenderSdfRaymarchCandidate`, `RenderSdfRaymarchCandidateList`,
  `RenderSdfRaymarchDiagnostic`, and
  `RenderSdfRaymarchAccelerationResource::derive_from_residency`.
- `engine/src/plugins/render/inspect/sdf_raymarch.rs`:
  added `inspect_render_sdf_raymarch_acceleration` and
  `inspect_last_render_sdf_raymarch_acceleration` as backend-neutral inspection
  entry points.
- `engine/src/plugins/render/features/world/mod.rs`:
  exported the `sdf_raymarch` module.
- `engine/src/plugins/render/inspect/mod.rs`:
  exported the SDF raymarch inspection entry points.
- `engine/src/plugins/render/plugin.rs::RenderPlugin::build`:
  initializes `RenderSdfRaymarchAccelerationResource`.
- `engine/tests/render_sdf_raymarch.rs`:
  added focused tests for bounded candidate/distance-mip evidence, fail-closed
  missing residency, unsafe overstep risk, candidate explosion, and fullscreen
  per-entity multiplication diagnostics.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents the SDF raymarch acceleration public DTOs and inspection contract.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents the normal conservative SDF raymarch acceleration workflow and the
  deferred WR-066 runtime-evidence boundary.

## Architecture Evidence

Architecture governance kickoff:

```text
task ai:architecture-governance -- --task "WR-065 SDF Raymarch Acceleration And Candidate Lists" --scope "PM-RENDER-SDF-003 engine/src/plugins/render conservative SDF raymarch acceleration distance mips empty-space skipping candidate lists diagnostics"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` owns renderer-derived
  SDF acceleration evidence, distance mip summaries, candidate-list DTOs,
  step/candidate budgets, inspection, and diagnostics.
- Source truth owner: `domain/world_sdf` and product publication own SDF payload
  truth, query policy, collision semantics, product fallback legality,
  generation authority, and source freshness.
- Translation boundary: SDF raymarch acceleration consumes
  `RenderSdfResidencyResource` evidence from `WR-064`; it does not read
  product sources directly, mutate SDF products, or become canonical SDF truth.
- ADR decision: no ADR required for this bounded slice because dependency
  direction follows accepted SDF rendering and scale visibility designs. ADR is
  still required before a later change introduces a persisted cross-domain SDF
  acceleration ABI, changes dependency direction, or moves product/collision
  policy into renderer code.

## Evidence

The implementation provides explicit renderer evidence for:

- resident product, page, brick, and clipmap-window counts consumed from SDF
  residency evidence;
- conservative distance mip records with source page/brick counts, minimum
  distance, max safe step, and unsafe-overstep status;
- screen-tile and depth-slice candidate lists with per-list candidate counts,
  rejected-candidate counts, product IDs, cache generations, scale bands, page
  counts, brick counts, and resident-byte estimates;
- step and candidate budgets through `RenderSdfRaymarchAccelerationConfig`;
- fail-closed diagnostics for missing SDF residency, invalid step/candidate
  budgets, unsafe overstep risk, fullscreen raymarching multiplied per entity,
  candidate-list overflow, and residency pressure.

## Validation

Focused validation passed:

```text
cargo fmt
cargo test -p engine render_sdf
cargo test -p engine render_runtime_inspect
cargo test -p engine render_scale
```

The `render_sdf` filter ran the new `engine/tests/render_sdf_raymarch.rs`
coverage:

- `render_sdf_raymarch_reports_bounded_candidates_and_distance_mips`
- `render_sdf_raymarch_fails_closed_without_residency`
- `render_sdf_raymarch_reports_overstep_candidate_explosion_and_fullscreen_multiplication`

Final planning validation after roadmap and production metadata updates:

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

- Runtime SDF examples, visual proof, benchmarks, hardware/profile evidence,
  and production readiness remain `WR-066` scope.
- This row derives deterministic renderer acceleration and candidate-list
  inspection DTOs; it does not submit a full SDF raymarch frame or claim
  `runtime_proven`.
- `perfectionist_verified` remains blocked until `PT-RENDER-PERFECTION`
  completes the final no-gap renderer audit.

These are expected production-track sequencing gaps, not hidden defects in the
WR-065 bounded contract.
