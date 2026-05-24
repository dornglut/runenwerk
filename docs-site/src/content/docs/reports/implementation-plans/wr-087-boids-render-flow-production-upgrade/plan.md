---
title: WR-087 Boids Render Flow Production Upgrade Implementation Contract
description: Runtime-proof contract for the boids render-flow production upgrade using reusable procedural population support.
status: active
owner: engine
layer: engine-runtime / renderer example
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-087 Boids Render Flow Production Upgrade Implementation Contract

## Goal

Implement `PM-RENDER-POP-005` / `WR-087` as the runtime example proof for the
renderer procedural population platform:

- boids consumes reusable bounded uniform-grid support from `WR-086`;
- simulation uses fixed-step state with explicit evidence;
- neighbor lookup uses bounded grid traversal, not production O(n^2);
- draw is surface-aware and does not aspect-skew on resize;
- visual heading is smoothed separately from simulation velocity;
- evidence reports bounded work, unsupported diagnostics, and pixel-space
  resize/aspect proof.

This slice targets `runtime_proven` for the boids example, while final no-gap
proof remains `PT-RENDER-PERFECTION`.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  boids is the runtime proof of the bounded procedural population path.
- `docs-site/src/content/docs/reports/closeouts/wr-086-bounded-uniform-grid-procedural-population-support/closeout.md`:
  reusable bounded-grid support is complete.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PM-RENDER-POP-005` is active and links to `WR-087`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-087` is the current implementation row and depends on archived completed
  `WR-086`.

## Readiness

`task production:plan -- --milestone "PM-RENDER-POP-005" --roadmap "WR-087"`
reported:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-086:completed`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-087-boids-render-flow-production-upgrade/plan.md`.

No ADR is required while this remains a renderer example consuming renderer
platform APIs without changing graph scheduling semantics or moving gameplay
truth into the renderer.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/examples/boids_render_flow/rendering/state.rs::BoidAgent`.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`.
- `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`.
- `engine/examples/boids_render_flow/rendering/evidence.rs::production_evidence_report`.
- `assets/shaders/boids_compute.wgsl::cs_main`.
- `assets/shaders/boids_compose.wgsl::vs_main`.

This slice may consume public renderer population APIs from `WR-086`, but it
must not move boids-only shader policy into the reusable population module.

## Required Decisions

- Fixed-step simulation is explicit. The frame delta is evidence/diagnostic
  input, not multi-step catch-up scheduling.
- Bounded uniform-grid pass order comes from reusable `WR-086` stage metadata.
- The production compute shader must not contain an all-boids neighbor loop.
- The render-stage compose shader must not storage-loop over all boids.
- Resize/aspect correctness needs pixel-space evidence in the evidence command,
  not only shader string checks.
- Any grid overflow/capacity assumption must be visible in evidence; no silent
  dense-cell drop policy is allowed.

## Non-Goals

WR-087 does not implement:

- reusable primitive GPU dispatch beyond what the boids shader uses;
- benchmark updates or public docs closeout;
- multi-step fixed-update catch-up scheduling;
- spatial hash or chunked unbounded population support;
- final track closeout.

## Implementation Steps

1. Use `BoundedUniformGrid2dBuildPlan` in `build_render_flow` to validate grid
   resources and drive canonical grid pass labels/dependencies.
2. Keep `BoidAgent::visual_heading` separate from velocity and ensure compose
   uses the smoothed heading.
3. Keep `BoidsRenderState` fixed-step and evidence-friendly; avoid hiding
   catch-up limitations.
4. Keep `assets/shaders/boids_compute.wgsl::cs_main` grid-based and remove any
   production all-boids neighbor loop.
5. Keep `assets/shaders/boids_compose.wgsl::vs_main` surface-aware by converting
   sprite pixel extents to clip offsets from current surface dimensions.
6. Extend evidence with pixel-space resize/aspect calculations across landscape
   and portrait surfaces.
7. Update tests for reusable stage labels, shader guardrails, fixed-step
   evidence, pixel resize evidence, no render-stage storage loop, and no
   production O(n^2) neighbor loop.

## Acceptance Criteria

- Boids graph consumes `WR-086` bounded-grid support.
- No production O(n^2) neighbor loop exists in the boids compute shader.
- No render-stage storage loop over all boids exists in the compose shader.
- Aspect-correct impostors are proven by evidence across resized surfaces.
- Visual heading smoothing is represented in state, compute shader, compose
  shader, and evidence.
- Evidence command remains stable:
  `cargo run -p engine --example boids_render_flow -- --evidence`.

## Validation

Run:

- `cargo fmt --all -- --check`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- `task docs:validate`

Closeout metadata must then pass:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task planning:validate`

## Stop Conditions

Stop before closing WR-087 if:

- boids graph does not consume reusable bounded-grid support;
- shader tests still allow an all-boids production neighbor loop;
- resize/aspect proof is only a shader-string assertion;
- fixed-step limitations are hidden or presented as catch-up scheduling;
- closeout tries to complete evidence, benchmarks, docs, or final track state
  that belong to `WR-088`.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/wr-087-boids-render-flow-production-upgrade/closeout.md`
with `status: completed`.

Move completed `WR-087` to
`docs-site/src/content/docs/workspace/roadmap-archive.yaml` with:

- `planning_state: completed`;
- `completion_quality: runtime_proven`;
- `known_quality_gaps` listing `WR-088`, reusable primitive shader dispatch if
  still deferred, and `PT-RENDER-PERFECTION`;
- `completion_audit` pointing at the WR-087 closeout.

Update `PM-RENDER-POP-005` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` with completed
state, evidence gate, completion audit, and honest known gaps.

Run the phase completion drift-check routine before starting `WR-088`.

## Perfectionist Closeout Audit

WR-087 targets `runtime_proven` for the boids example only.

Known quality gaps at closeout:

- evidence, benchmarks, docs, and final track closeout remain `WR-088`;
- reusable primitive shader dispatch remains a visible gap unless implemented
  in a later slice;
- final no-gap verification remains `PT-RENDER-PERFECTION`.
