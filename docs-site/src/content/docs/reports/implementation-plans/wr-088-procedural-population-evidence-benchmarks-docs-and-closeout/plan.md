---
title: WR-088 Procedural Population Evidence Benchmarks Docs And Closeout Implementation Contract
description: Runtime-proven closeout contract for procedural population evidence, benchmarks, public docs, and production-track completion.
status: active
owner: engine
layer: engine-runtime / renderer evidence
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-088 Procedural Population Evidence Benchmarks Docs And Closeout Implementation Contract

## Goal

Implement `PM-RENDER-POP-006` / `WR-088` as the final hardening and closeout
slice for `PT-RENDER-PROCEDURAL-POPULATION`.

This slice must prove the full bounded procedural population chain at
`runtime_proven`:

- procedural authoring and typed draw-source contracts are documented;
- reusable scan, compaction, indirect-args, and bounded-grid contracts have
  benchmark coverage;
- the boids production example documents fixed-step limits, resize/aspect
  evidence, bounded submitted work, and unsupported diagnostics;
- closeout evidence proves no production O(n^2) neighbor loop, no render-stage
  storage loop over all boids, no silent grid overflow, no aspect skew on
  resize, explicit unsupported diagnostics, bounded submitted work, and stable
  production evidence.

This slice does not claim `perfectionist_verified`; final no-gap renderer proof
remains `PT-RENDER-PERFECTION`.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  active doctrine and acceptance evidence for the track.
- `docs-site/src/content/docs/reports/closeouts/wr-087-boids-render-flow-production-upgrade/closeout.md`:
  completed boids runtime proof.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PM-RENDER-POP-006` is the final hardening milestone for the track.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-088` is the current candidate and depends on archived completed `WR-087`.

## Readiness

`task production:plan -- --milestone "PM-RENDER-POP-006" --roadmap "WR-088"`
reported:

- production milestone state: `designing`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-087:completed`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-088-procedural-population-evidence-benchmarks-docs-and-closeout/plan.md`.

The active `WR-088` write scope must include
`docs-site/src/content/docs/engine/reference/plugins/render`, not the obsolete
`docs-site/src/content/docs/engine/plugins/render` path.

No ADR is required while this slice only documents and benchmarks the accepted
renderer-owned population chain, updates closeout evidence, and does not change
dependency direction, product truth, fallback policy, or graph scheduling
semantics.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/benches/render_flow_planning.rs::bench_render_flow_planning`.
- `engine/benches/render_flow_planning.rs::build_procedural_boids_flow`.
- `engine/benches/render_flow_planning.rs` helper functions for scan, grid,
  boids production evidence, and evidence reporting benchmarks.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`.
- `docs-site/src/content/docs/reports/closeouts/wr-088-procedural-population-evidence-benchmarks-docs-and-closeout/closeout.md`.
- `docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-runtime-proven/closeout.md`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`.
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`.

This slice may add benchmark-only helper functions in
`engine/benches/render_flow_planning.rs`; it must not introduce product runtime
fallbacks or new implementation scope outside benchmark/docs/evidence closeout.

## Required Decisions

- The closeout proof is evidence-backed, not prose-only. It must cite the boids
  evidence command, benchmark command, and validators.
- Benchmarks cover four procedural population concerns:
  - scan primitive planning;
  - bounded-grid build planning;
  - boids production-shape flow planning/preflight;
  - boids production evidence report formatting/checking.
- Benchmark code must use the reusable renderer contracts from `WR-085` and
  `WR-086`; it must not duplicate an ad hoc grid model for the canonical proof.
- Docs must name fixed-step limitations explicitly. Multi-step catch-up remains
  later graph scheduling work.
- Unsupported GPU timing, readback, storage compaction, or indirect capability
  states remain diagnostics. They must not be described as silent fallback.
- `PT-RENDER-PROCEDURAL-POPULATION` closes at `runtime_proven` only. Known gaps
  stay visible in production-track and closeout metadata.

## Non-Goals

WR-088 does not implement:

- reusable GPU primitive shader dispatch;
- spatial hash or chunked unbounded population support;
- multi-step fixed-update catch-up scheduling;
- new graph scheduling semantics;
- product-owned population truth or gameplay semantics;
- final no-gap renderer audit.

## Implementation Steps

1. Fix the active `WR-088` write scope to point at the render reference docs.
2. Add Criterion benchmark cases for scan primitive planning, bounded-grid
   build planning, procedural-boids production shape/preflight, and boids
   production evidence reporting.
3. Update boids example docs with the canonical grid stage order, fixed-step
   limitation, resize/aspect evidence, and validation commands.
4. Update the render-flow usage guide with the procedural builder, typed
   indirect draw-source, bounded-grid population, and evidence workflow.
5. Update the public API reference with procedural builder, draw-source, GPU
   primitive, bounded-grid population, and boids evidence API surfaces.
6. Write the WR-088 closeout with validation output and honest known gaps.
7. Write a track-level runtime-proven closeout for
   `PT-RENDER-PROCEDURAL-POPULATION`.
8. Move `WR-088` to the roadmap archive with `completion_quality:
   runtime_proven` and a completion audit path.
9. Mark `PM-RENDER-POP-006` and `PT-RENDER-PROCEDURAL-POPULATION` completed at
   `runtime_proven`, with known quality gaps that do not claim perfectionist
   proof.
10. Render and validate roadmap, production, docs, planning, format, and
    benchmark gates.

## Acceptance Criteria

- Benchmark names include scan, grid build, boids production flow, and boids
  production evidence reporting cases.
- Docs identify `RenderFlow::procedural_pass_builder(...)` as procedural-owned
  lowering, not a `GraphicsPassBuilder` leak.
- Docs identify `.draw(...)` as the direct path and `draw_indirect(...)` as the
  typed indirect draw-source path.
- Docs identify bounded-grid stage order:
  clear counts, count cells, scan counts, reset cursors, scatter sorted
  indices, neighbor simulation, publish/draw.
- Docs state that bounded-grid buffers are total-count-sized and that silent
  fixed bucket overflow is forbidden.
- Docs state spatial hash/chunked unbounded populations are deferred.
- Closeout cites evidence for no render-stage storage loop, no production
  O(n^2) neighbor loop, no aspect skew on resize, no silent grid overflow,
  explicit unsupported diagnostics, bounded submitted work, and stable
  production evidence.
- Production track closes at `runtime_proven`, not `perfectionist_verified`.

## Validation

Run:

- `cargo fmt --all -- --check`
- `cargo bench -p engine --bench render_flow_planning`
- `task docs:validate`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task planning:validate`

If benchmark runtime is excessive, first keep the benchmark implementation
correct and inspect Criterion output before reducing sample sizes through the
repository's benchmark configuration conventions.

## Stop Conditions

Stop before closing WR-088 if:

- benchmark coverage is descriptor-only and does not exercise reusable scan,
  grid, boids production flow, and evidence-report paths;
- docs describe hidden fallback, silent overflow, or multi-step catch-up that
  the runtime does not implement;
- closeout tries to claim `perfectionist_verified`;
- production metadata hides remaining gaps for primitive shader dispatch,
  unbounded population support, or final renderer audit;
- validators cannot prove the generated roadmap/production/docs state is clean.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/wr-088-procedural-population-evidence-benchmarks-docs-and-closeout/closeout.md`
with `status: completed`.

Create
`docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-runtime-proven/closeout.md`
with `status: completed`.

Move completed `WR-088` to
`docs-site/src/content/docs/workspace/roadmap-archive.yaml` with:

- `planning_state: completed`;
- `completion_quality: runtime_proven`;
- `known_quality_gaps` listing primitive shader dispatch, spatial
  hash/chunked unbounded populations, multi-step catch-up scheduling, and
  `PT-RENDER-PERFECTION`;
- `completion_audit` pointing at the WR-088 closeout.

Update `PM-RENDER-POP-006` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` with completed
state, evidence gate, completion audit, and honest known gaps.

Update `PT-RENDER-PROCEDURAL-POPULATION` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` to completed at
`runtime_proven`, with the track closeout as evidence.

Run the phase completion drift-check routine after the slice is complete.

## Perfectionist Closeout Audit

WR-088 and `PT-RENDER-PROCEDURAL-POPULATION` target `runtime_proven`.

Known quality gaps that must remain visible at closeout:

- reusable GPU primitive shader dispatch remains deferred unless separately
  implemented in an accepted later slice;
- spatial hash and chunked unbounded population support remain later
  milestones;
- multi-step fixed-update catch-up scheduling remains future graph scheduling
  work;
- final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

No gap list may be empty, and no metadata may claim `perfectionist_verified`.
