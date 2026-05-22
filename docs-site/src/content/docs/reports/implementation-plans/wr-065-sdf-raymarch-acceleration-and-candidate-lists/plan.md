---
title: WR-065 SDF Raymarch Acceleration And Candidate Lists Implementation Contract
description: Design-first contract for conservative renderer-owned SDF raymarch acceleration and candidate-list evidence.
status: active
owner: engine
layer: engine-runtime / renderer sdf raymarch acceleration
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md
  - ../../../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../../../design/accepted/renderer-scale-residency-and-gpu-driven-visibility-design.md
related_roadmaps:
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-065 SDF Raymarch Acceleration And Candidate Lists Implementation Contract

## Goal

Prepare the bounded implementation slice for `PM-RENDER-SDF-003` and
`WR-065`. This row adds conservative renderer-owned SDF raymarch acceleration
evidence: distance-mip descriptors, empty-space skipping bounds, screen-tile and
depth-slice candidate lists, candidate explosion diagnostics, overstep-risk
diagnostics, cache-pressure evidence, and ray step budget reporting.

This is a design-first contract. It clears deferred intake questions and
prepares WR-065 for roadmap application and promotion. It does not authorize
product code changes until the stack coordinator selects WR-065 for
implementation after roadmap gates are satisfied.

## Source Of Truth

- `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`:
  raymarch acceleration must be conservative, inspectable, and bounded per view;
  empty-space hierarchies, distance mips, and macro steps must never
  overestimate safe travel distance.
- `docs-site/src/content/docs/design/accepted/sdf-product-renderer-and-gpu-residency-design.md`:
  renderer acceleration is derived execution state over prepared products, not
  SDF product truth, physics truth, fallback policy, or query authority.
- `docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md`:
  WR-064 completed renderer-owned sparse SDF brick, page, clipmap, generation,
  invalidation, and budget evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`:
  WR-062 completed bounded visibility, LOD, compaction, and submitted-work
  evidence that candidate-list work must not bypass.
- `engine/src/plugins/render/features/world/sdf_residency.rs`:
  `RenderSdfResidencyResource::derive_from_sources` and
  `RenderSdfChunkResidencyEntry` are the input residency evidence for raymarch
  acceleration.
- `engine/src/plugins/render/inspect/sdf_residency.rs`:
  `inspect_render_sdf_residency` is the existing public SDF residency
  inspection boundary.
- `engine/src/plugins/render/inspect/scale_visibility.rs`:
  `inspect_render_scale_visibility` demonstrates bounded candidate,
  compaction, unsupported-diagnostic, and submitted-work reporting.

## Readiness

`task production:plan -- --milestone PM-RENDER-SDF-003 --roadmap WR-065`
reported:

- production milestone state: `designing`;
- roadmap state: `blocked_deferred`;
- roadmap blocker: `B5`;
- next action: `design_first`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-065-sdf-raymarch-acceleration-and-candidate-lists/plan.md`.

Architecture governance kickoff was run with:

```text
task ai:architecture-governance -- --task "WR-065 SDF Raymarch Acceleration And Candidate Lists" --scope "PM-RENDER-SDF-003 engine/src/plugins/render conservative SDF raymarch acceleration distance mips empty-space skipping candidate lists diagnostics"
```

Governance decision:

- DDD bounded context owner: `engine/src/plugins/render` for renderer-derived
  acceleration records, candidate-list DTOs, raymarch step evidence, and
  diagnostics.
- Source truth owner: `domain/world_sdf` and product publication for authored
  SDF fields, payload lineage, page/brick contents, collision conservatism,
  query policy, and fallback legality.
- Translation boundary: WR-065 consumes WR-064 SDF residency evidence and
  produces renderer execution acceleration evidence; it does not modify SDF
  products or decide product fallback.
- ADR requirement: no ADR is required if implementation only adds derived
  renderer acceleration DTOs, diagnostics, and tests. Stop for ADR or accepted
  design update before implementation if a persisted acceleration ABI,
  dependency-direction change, renderer-owned SDF product truth, or renderer
  collision/query authority is introduced.
- Team Topologies ownership: complicated-subsystem renderer platform work with
  stream-aligned SDF/domain product producers.

## Promotion Readiness

After the design-first contract and intake proposal were applied,
`task production:plan -- --milestone PM-RENDER-SDF-003 --roadmap WR-065`
reported:

- production milestone state: `designing`;
- roadmap state: `ready_next`;
- dependency evidence: `WR-064` and `WR-062` are completed;
- next action: `write_promotion_contract`;
- promotion preflight status: `promotable`.

Accepted promotion evidence:

- Completed WR-064 sparse SDF brick/page/clipmap residency closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-064-sparse-sdf-brick-page-and-clipmap-residency/closeout.md`.
- Completed WR-062 bounded visibility, LOD, compaction, and submitted-work
  closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-062-renderer-scale-gpu-driven-culling-lod-and-indirect-submission/closeout.md`.
- Accepted SDF world rendering and raymarch acceleration doctrine:
  `docs-site/src/content/docs/design/accepted/sdf-world-rendering-and-raymarch-acceleration-design.md`.
- This WR-065 design-first conservative raymarch acceleration and
  candidate-list implementation contract.

Promotion command:

```text
task roadmap:promote -- --id WR-065 --state current_candidate --evidence "Completed WR-064 SDF residency closeout, completed WR-062 bounded visibility and submitted-work closeout, accepted SDF raymarch doctrine, and WR-065 design-first conservative raymarch acceleration implementation contract."
```

Promotion does not authorize code by itself. After promotion, rerun the stack
and single-track coordinators and follow the selected implementation action.

## Implementation Scope

Allowed write scopes after roadmap application and promotion:

```text
engine/src/plugins/render
engine/tests
docs-site/src/content/docs/engine/reference/plugins/render
docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-sdf-raymarch-acceleration-and-candidate-
docs-site/src/content/docs/reports/implementation-plans/wr-065-sdf-raymarch-acceleration-and-candidate-lists/plan.md
docs-site/src/content/docs/reports/closeouts/wr-065-sdf-raymarch-acceleration-and-candidate-lists/closeout.md
docs-site/src/content/docs/workspace/roadmap-items.yaml
docs-site/src/content/docs/workspace/roadmap-deferred.yaml
docs-site/src/content/docs/workspace/roadmap-archive.yaml
docs-site/src/content/docs/workspace/production-tracks.yaml
```

Exact implementation owners:

- `engine/src/plugins/render/features/world/sdf_raymarch.rs`:
  preferred new module for `RenderSdfRaymarchAccelerationResource`,
  distance-mip descriptors, empty-space step bounds, screen-tile/depth-slice
  candidate lists, step-budget summaries, and overstep/candidate diagnostics.
- `engine/src/plugins/render/features/world/sdf_residency.rs`:
  input owner for resident pages, bricks, clipmap windows, generation keys, and
  cache-pressure evidence. Extend only if acceleration needs a narrow field on
  existing residency entries.
- `engine/src/plugins/render/inspect/sdf_raymarch.rs`:
  preferred public inspection DTO owner for raymarch acceleration reports.
- `engine/src/plugins/render/inspect/sdf_residency.rs` and
  `engine/src/plugins/render/inspect/scale_visibility.rs`: reuse inspection
  vocabulary for resident inputs and bounded candidate reporting.
- `engine/src/plugins/render/features/world/mod.rs` and
  `engine/src/plugins/render/inspect/mod.rs`: module-boundary exports only.
- `engine/tests/render_sdf_raymarch.rs`: focused integration tests for
  conservative distance mips, candidate-list bounds, overstep diagnostics,
  candidate explosion diagnostics, and no fullscreen-per-entity multiplication.

## Required Contracts

WR-065 must introduce explicit renderer evidence for:

- source residency digest: resident SDF product count, page count, brick count,
  clipmap window count, cache generation, and pressure state consumed from
  WR-064;
- distance-mip levels with conservative minimum safe step distances and
  diagnostics when a mip could overestimate safe travel;
- empty-space step bounds and per-view step-budget configuration;
- screen-tile and depth-slice candidate lists with candidate counts, rejected
  counts, and explosion diagnostics;
- ray step budget estimates for near, mid, far, and summary scale bands;
- missed-surface risk, unsafe-overstep risk, cache-pressure risk, and fallback
  or unsupported diagnostics;
- explicit statement that fullscreen raymarching is one bounded view pass over
  prepared resident products and must not multiply by entity count.

The API must fail closed:

- missing SDF residency evidence, missing clipmap coverage, invalid distance
  mip bounds, overestimated safe travel, unbounded candidate list growth, or
  per-entity fullscreen multiplication are diagnostics;
- unsupported acceleration data must report degraded state instead of falling
  back to unbounded scanning;
- candidate lists must reference renderer-resident products and must not become
  product truth or visibility truth.

## Non-Goals

WR-065 does not implement:

- SDF brick/page/clipmap residency; that is completed WR-064 scope.
- Runtime examples, visual proof, hardware evidence, benchmarks, or
  production-readiness closeout for the complete SDF renderer path; those belong
  to WR-066.
- SDF authoring, collision truth, physics conservatism, gameplay interaction
  fields, product fallback policy, query authority, or product rebuild policy.
- Hardware ray queries, hybrid tracing, temporal upscaling, mesh/material
  lighting, product visual producers, or final renderer perfectionist audit.

## Implementation Steps

1. Inspect WR-064 SDF residency resource and inspection DTOs, WR-062 scale
   visibility/candidate reporting, and the accepted SDF raymarch design before
   adding acceleration types.
2. Add a narrow `sdf_raymarch` renderer module with typed acceleration inputs,
   distance-mip descriptors, candidate-list descriptors, and diagnostics.
3. Derive candidate lists from resident SDF chunk/page/clipmap evidence and
   renderer view/tile/depth-slice configuration; do not scan all products per
   ray step.
4. Add conservative distance-mip checks that flag any unsafe overstep risk
   before an acceleration report can claim readiness.
5. Add inspection DTOs through `inspect_render_sdf_raymarch_acceleration(...)`.
6. Add focused `render_sdf` tests for valid bounded candidates, missing
   residency evidence, unsafe overstep risk, candidate explosion, and
   fullscreen-per-entity rejection.
7. Update renderer reference docs and close WR-065 only after focused tests,
   docs validation, roadmap validation, production validation, and planning
   validation pass.

## Required Validation

Implementation validation must include:

```text
cargo fmt
cargo test -p engine render_sdf
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

## Acceptance Criteria

- Distance-mip and empty-space step evidence is conservative and diagnostics
  expose unsafe overstep risk.
- Screen-tile and depth-slice candidate lists bound raymarch work by resident
  SDF products and expose candidate explosion diagnostics.
- Fullscreen raymarching remains a bounded per-view pass and is never
  multiplied by per-entity instance counts.
- Renderer acceleration records do not own SDF product truth, product fallback,
  collision truth, or query policy.

## Stop Conditions

Stop before implementation if:

- WR-065 has not been applied to active roadmap state and promoted through the
  required roadmap workflow;
- implementation would move SDF product truth, fallback legality, collision
  conservatism, query authority, or rebuild policy into renderer code;
- acceleration requires a persisted cross-domain ABI without ADR/design
  acceptance;
- conservative distance bounds cannot be proven with focused tests;
- candidate-list bounds cannot prevent unbounded per-ray or per-entity scanning.

## Closeout Requirements

Closeout must create:

```text
docs-site/src/content/docs/reports/closeouts/wr-065-sdf-raymarch-acceleration-and-candidate-lists/closeout.md
```

The closeout must record exact changed modules/functions, acceleration DTOs,
candidate-list evidence, diagnostics, focused validation output, docs updates,
roadmap/production metadata updates, completion quality, and known gaps.

Expected completion quality is `bounded_contract`. `runtime_proven` remains
blocked until WR-066 proves the complete SDF runtime path through examples,
visual proof, benchmarks, and production evidence. `perfectionist_verified`
remains blocked until `PT-RENDER-PERFECTION` completes the final no-gap
renderer audit.

## Perfectionist Closeout Audit

WR-065 must preserve visible gaps for later tracks instead of claiming runtime
perfection:

- WR-066 owns runtime SDF examples, visual proof, benchmarks, and
  runtime-proven evidence.
- PT-RENDER-PERFECTION owns final no-gap verification.

Anti-drift guards must prevent descriptor-only acceleration, unconsumed
candidate lists, unsafe overstep success, fallback-only reports, unbounded
product scans, and per-entity fullscreen raymarch multiplication.
