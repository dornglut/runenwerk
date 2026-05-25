---
title: WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout Implementation Contract
description: Implementation contract for runtime-proven hardening closeout evidence, benchmarks, documentation, and track closeout.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../closeouts/wr-090-indirect-draw-contract-hardening/closeout.md
  - ../../closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md
  - ../../closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md
  - ../../closeouts/wr-101-procedural-camera-and-view-projection/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-006` / `WR-093` as the evidence, benchmark,
documentation, and track closeout slice for
`PT-RENDER-PROCEDURAL-POPULATION-HARDENING`.

The track may close at `runtime_proven` only when indirect draw hardening,
primitive shader dispatch, graph catch-up scheduling, and procedural camera
projection are all proven by current runtime evidence, public docs, benchmarks,
and completed WR closeouts.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/closeouts/wr-090-indirect-draw-contract-hardening/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-101-procedural-camera-and-view-projection/closeout.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`

## Readiness

`WR-093` depends on completed `WR-101`. That dependency is complete and
archived with procedural camera projection closeout evidence. The preceding
runtime proof closeouts for `WR-090`, `WR-091`, and `WR-092` are also complete
and must remain valid because this slice closes the full hardening evidence
chain.

Current preflight:

```text
task production:plan -- --milestone "PM-RENDER-POP-HARDEN-006" --roadmap "WR-093"
```

The first preflight reported `Status: promotable` and `Next action:
write_promotion_contract`; `WR-093` was promoted to `current_candidate` with
the full dependency closeout evidence chain. The current preflight reports
`Next action: write_implementation_contract`.

This document is the decision-complete implementation contract. This contract
update is planning work only and must not include product code changes.

Accepted promotion evidence:

- `docs-site/src/content/docs/reports/implementation-plans/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/plan.md`
- `docs-site/src/content/docs/reports/closeouts/wr-090-indirect-draw-contract-hardening/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`
- `docs-site/src/content/docs/reports/closeouts/wr-101-procedural-camera-and-view-projection/closeout.md`

Do not implement this slice until this implementation contract validates and
`task ai:goal -- --track PT-RENDER-PROCEDURAL-POPULATION-HARDENING` still
reports `PM-RENDER-POP-HARDEN-006` as the implementation action.

## Gates And Dependencies

- `WR-090`, `WR-091`, `WR-092`, and `WR-101` must remain completed with valid
  closeout evidence.
- `renderer-procedural-population-hardening-platform-design.md` must remain
  active.
- Public docs must match implemented APIs and examples.
- Benchmarks and evidence commands must be reproducible from the repository.
- No ADR is required for evidence/docs closeout unless this slice changes
  source-truth ownership, dependency direction, fallback legality, renderer ABI
  semantics, or product/runtime authority.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/benches/render_flow_planning.rs`
  - Ensure benchmark coverage includes indirect validation, primitive dispatch
    planning/execution, fixed-step catch-up planning/execution, and procedural
    camera evidence paths where measurable.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
  - Make the normal authoring path clear for direct draw, explicit indirect
    draw, renderer-owned GPU primitives, fixed-step graph scheduling, and
    procedural camera projection.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  - Ensure public API entries match the final contracts.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`
  - Document boids evidence for graph-submitted catch-up, input/redraw-rate
    invariance, and aspect-correct procedural camera projection.
- `docs-site/src/content/docs/reports/closeouts/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/closeout.md`
  - Record WR-093 closeout evidence.
- `docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-hardening-runtime-proven/closeout.md`
  - Record track-level closeout evidence.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
  - Mark PM-006 and the track complete only after evidence passes.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
  - Remove WR-093 from active items after closeout.
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
  - Archive WR-093 with completion evidence.

## Evidence Inventory

The implementation must audit and cite the current closeouts:

- `WR-090`: fail-closed typed indirect draw validation.
- `WR-091`: renderer-owned GPU primitive dispatch and hierarchical scan proof.
- `WR-092`: graph-owned fixed-step catch-up scheduling and iteration uniform
  proof.
- `WR-101`: reusable procedural camera projection and boids aspect evidence.

It must also rerun current commands:

- `cargo test -p engine render_flow`
- `cargo test -p engine gpu_primitives`
- `cargo test -p engine procedural`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- `cargo bench -p engine --bench render_flow_planning`

## Acceptance Criteria

- Indirect draw validation closeout proves wrong args type, misaligned byte
  offset, and out-of-range byte offset fail before submit.
- Primitive dispatch closeout proves renderer-owned kernels execute outside the
  boids example.
- Prefix scan benchmark and tests cover hierarchical multi-block behavior.
- Fixed-step graph catch-up closeout proves deterministic `0..N` bounded
  submitted substeps, runtime fixed-time source reuse, input/redraw-rate
  invariance, iteration-scoped uniform projection, and resource sequencing
  across substeps.
- Procedural camera projection closeout proves equal world x/y scale after
  projection, fill-viewport behavior without letterbox or non-uniform stretch,
  and no `PreparedViewFrame` camera source-truth ownership.
- Public docs teach normal direct draw first, then explicit indirect and
  scheduled advanced paths, then procedural camera projection.
- Track closeout lists remaining quality gaps honestly and does not claim
  `perfectionist_verified`.

## Non-Goals

- Do not add new runtime features in the closeout slice unless required to fix
  evidence gaps found by validation.
- Do not fold spatial hash or chunked unbounded populations into the closeout.
- Do not fold behavior authoring or richer boids split/merge dynamics into the
  closeout.
- Do not mark `PT-RENDER-PERFECTION` complete.
- Do not claim final no-gap renderer verification.

## Stop Conditions

- Stop if any earlier WR closeout is missing or only descriptor-level evidence.
- Stop if benchmark output is stale or not reproducible by the listed command.
- Stop if docs describe APIs or behavior not present in code.
- Stop if camera evidence does not prove equal projected world scale across
  landscape, portrait, square, and extreme aspect surfaces.
- Stop if production validation rejects the completion-quality claim.
- Stop if implementation requires product, gameplay, world, or camera source
  truth to move into renderer-owned contracts.

## Validation

Required focused and workflow validation:

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

## Closeout Requirements

WR closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/closeout.md`

Track closeout must live under:

`docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-hardening-runtime-proven/closeout.md`

Completion quality target: `runtime_proven`.

The closeout must update:

- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- generated roadmap and production docs

Known gaps that must remain visible:

- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Perfectionist Closeout Audit

This slice should close the hardening track at `runtime_proven`, not
`perfectionist_verified`.

The closeout audit must explicitly prove:

- all four implementation closeouts are completed and dependency-legal;
- evidence commands were rerun after the final code/docs state;
- public docs match implemented contracts;
- benchmarks run and the command is captured;
- the track-level closeout names remaining non-goals and quality gaps;
- roadmap, production, docs, and planning checks pass after metadata updates.

## Critical Review

The closeout must not promote a checklist that proves only planning artifacts.
Runtime proof requires executed primitive kernels, failed invalid indirect draw
validation cases, graph scheduling evidence, and procedural camera projection
evidence. If any of those are missing, the correct closeout is blocked, not
`runtime_proven`.
