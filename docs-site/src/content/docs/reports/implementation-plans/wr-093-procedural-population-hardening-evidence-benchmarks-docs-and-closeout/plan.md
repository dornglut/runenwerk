---
title: WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout Implementation Contract
description: Bounded implementation contract for runtime-proven hardening closeout evidence, benchmarks, and documentation.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-005` / `WR-093` as the evidence, benchmark,
documentation, and track closeout slice for
`PT-RENDER-PROCEDURAL-POPULATION-HARDENING`.

The track may close at `runtime_proven` only when indirect draw hardening,
primitive shader dispatch, and graph catch-up scheduling are all proven by
runtime evidence and documented public contracts.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-090-indirect-draw-contract-hardening/plan.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-091-reusable-gpu-primitive-shader-dispatch/plan.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-092-fixed-step-graph-catch-up-scheduling/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness

This slice depends on completed `WR-092`. It cannot close the track if any
runtime proof remains descriptor-only, boids-only, or documentation-only.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/benches/render_flow_planning.rs`:
  benchmark cases for indirect validation, primitive dispatch planning and
  execution, and fixed-step catch-up planning and execution.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  user-facing authoring docs for hardened indirect draws, primitive dispatch,
  and graph catch-up scheduling.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public API reference for new or changed renderer contracts.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`:
  boids evidence updates only if boids reports graph-submitted catch-up
  evidence.
- `docs-site/src/content/docs/reports/closeouts/wr-090-indirect-draw-contract-hardening/closeout.md`:
  evidence dependency.
- `docs-site/src/content/docs/reports/closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md`:
  evidence dependency.
- `docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`:
  evidence dependency.
- `docs-site/src/content/docs/reports/closeouts/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/closeout.md`:
  slice closeout.
- `docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-hardening-runtime-proven/closeout.md`:
  track closeout.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  milestone and track evidence update.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` and
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml`:
  WR completion evidence update.

## Acceptance Criteria

- Indirect draw validation closeout proves wrong args type, misaligned byte
  offset, and out-of-range byte offset fail before submit.
- Primitive dispatch closeout proves renderer-owned kernels execute outside the
  boids example.
- Prefix scan benchmark and tests cover hierarchical multi-block behavior.
- Fixed-step graph catch-up closeout proves deterministic `0..N` bounded
  submitted substeps and resource sequencing across substeps.
- Public docs teach normal direct draw first, then explicit indirect and
  scheduled advanced paths.
- The track closeout lists remaining quality gaps honestly and does not claim
  `perfectionist_verified`.

## Non-Goals

- Do not add new runtime features in the closeout slice unless required to fix
  evidence gaps found by validation.
- Do not fold spatial hash or chunked unbounded populations into the closeout.
- Do not mark `PT-RENDER-PERFECTION` complete.

## Stop Conditions

- Stop if any earlier WR closeout is missing or only descriptor-level evidence.
- Stop if benchmark output is stale or not reproducible by the listed command.
- Stop if docs describe APIs or behavior not present in code.
- Stop if production validation rejects the completion-quality claim.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/closeout.md`

Track closeout must live under:

`docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-hardening-runtime-proven/closeout.md`

Completion quality target: `runtime_proven`.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine render_flow`
- `cargo test -p engine gpu_primitives`
- `cargo test -p engine procedural`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- `cargo bench -p engine --bench render_flow_planning`
- `task docs:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task planning:validate`

## Critical Review

The closeout must not promote a checklist that proves only planning artifacts.
Runtime proof requires executed primitive kernels, failed invalid indirect draw
validation cases, and graph scheduling evidence. If any of those are missing,
the correct closeout is blocked, not `runtime_proven`.

