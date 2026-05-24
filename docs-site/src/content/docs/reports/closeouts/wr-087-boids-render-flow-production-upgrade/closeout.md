---
title: WR-087 Boids Render Flow Production Upgrade Closeout
description: Closeout evidence for the grid-accelerated, aspect-correct, evidence-backed boids render-flow production proof.
status: completed
owner: engine
layer: engine-runtime / renderer example
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../implementation-plans/wr-087-boids-render-flow-production-upgrade/plan.md
---

# WR-087 Boids Render Flow Production Upgrade Closeout

## Result

`WR-087` is completed as the boids runtime proof slice. The boids render flow
now consumes reusable bounded-grid support, submits a fixed-step grid pipeline,
keeps visual heading separate from simulation velocity, renders aspect-correct
impostors from surface-aware pixel offsets, and emits stable production
evidence including resize pixel checks.

This closeout does not claim final documentation, benchmark, or track closeout
completion. Those remain `WR-088`.

## What Changed

- `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`
  now builds a `BoundedUniformGrid2dBuildPlan` and uses its canonical grid stage
  labels for clear, count, scan, reset, scatter, simulate, and publish/draw.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidAgent` keeps
  `visual_heading` separate from velocity.
- `engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState`
  keeps fixed-step simulation explicit and avoids hidden catch-up scheduling.
- `assets/shaders/boids_compute.wgsl::cs_main` uses bounded-grid neighbor
  traversal and smoothed visual heading.
- `assets/shaders/boids_compose.wgsl::vs_main` uses surface-aware pixel-to-clip
  offsets and instance inputs, with no render-stage storage loop.
- `engine/examples/boids_render_flow/rendering/evidence.rs::production_evidence_report`
  reports fixed-step state, grid capacities, pass order, unsupported GPU timing
  diagnostics, CPU preflight timing, and resize pixel evidence for landscape,
  portrait, and square surfaces.

## Evidence

- `task production:plan -- --milestone "PM-RENDER-POP-005" --roadmap "WR-087"`
  classified the row as current-candidate eligible and requested this
  implementation contract.
- The implementation contract is recorded at
  `docs-site/src/content/docs/reports/implementation-plans/wr-087-boids-render-flow-production-upgrade/plan.md`.
- `cargo run -p engine --example boids_render_flow -- --evidence` reports:
  no silent grid overflow, fixed-step simulation, smoothed visual heading,
  aspect-correct impostors, bounded pass order, unsupported GPU timing
  diagnostics, and resize pixel evidence with `aspect_error_px=0.00000` across
  1600x900, 900x1600, and 1024x1024 surfaces.

## Validation

- `cargo fmt --all -- --check` passed.
- `cargo test -p engine --example boids_render_flow` passed.
- `cargo run -p engine --example boids_render_flow -- --evidence` passed.
- `task docs:validate` passed.

## Completion Quality

Completion quality: `runtime_proven`.

Known quality gaps:

- Evidence, benchmarks, docs, and final track closeout remain `WR-088`.
- Reusable GPU primitive shader dispatch remains deferred; boids proves the
  bounded runtime path with its example shader pipeline.
- Final no-gap verification remains `PT-RENDER-PERFECTION`.

## Next Slice

The next legal implementation slice is `WR-088`: evidence, benchmarks, docs,
and final `PT-RENDER-PROCEDURAL-POPULATION` closeout.
