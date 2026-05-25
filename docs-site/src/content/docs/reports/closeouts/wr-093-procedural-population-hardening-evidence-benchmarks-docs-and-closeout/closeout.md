---
title: WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout
description: Closeout evidence for procedural population hardening documentation, benchmarks, runtime proof, and production-track completion.
status: completed
owner: engine
layer: engine-runtime / renderer evidence
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../implementation-plans/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/plan.md
  - ../wr-090-indirect-draw-contract-hardening/closeout.md
  - ../wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md
  - ../wr-092-fixed-step-graph-catch-up-scheduling/closeout.md
  - ../wr-101-procedural-camera-and-view-projection/closeout.md
  - ../pt-render-procedural-population-hardening-runtime-proven/closeout.md
---

# WR-093 Procedural Population Hardening Evidence Benchmarks Docs And Closeout

## Result

`WR-093` is completed as the evidence, benchmark, documentation, and closeout
slice for `PT-RENDER-PROCEDURAL-POPULATION-HARDENING`.

The renderer procedural population hardening track now has current runtime
evidence for fail-closed indirect draw validation, reusable renderer-owned GPU
primitive dispatch, graph-owned fixed-step catch-up scheduling, and reusable
procedural camera projection. Public render-flow docs, API reference entries,
boids example docs, benchmark coverage, WR closeouts, and the track-level
runtime-proven closeout agree with the implemented contracts.

No product code changes were required in this closeout slice after validation.

## Evidence Chain

The closeout relies on completed dependency closeouts:

- `WR-090`: indirect draw validation fails wrong argument type, missing indirect
  declaration, misaligned byte offset, and out-of-range byte offset before
  submit.
- `WR-091`: renderer-owned GPU primitive kernels execute outside the boids
  example, and prefix scan supports hierarchical multi-block counts through
  block scan, block-sum scan, and block-offset propagation.
- `WR-092`: graph-owned fixed-step catch-up scheduling submits deterministic
  bounded `0..N` substeps from runtime fixed-time resources with iteration
  uniforms and resource sequencing across substeps.
- `WR-101`: procedural camera projection preserves equal projected world x/y
  scale across landscape, portrait, square, and extreme-aspect surfaces while
  keeping camera intent producer-owned.

## Runtime Evidence

`cargo run -p engine --example boids_render_flow -- --evidence` passed and
reported:

- `graph_fixed_step_region label=boids.fixed_step max_substeps=4 submitted_substeps=2 pass_count=8`;
- `aspect_correct_impostors=true`;
- fixed-step simulation with `fixed_dt_seconds=0.016667` and `submitted_steps=2`;
- canonical compute pass order from `boids.seed_or_hold` through
  `boids.grid.publish_draw`, then `boids.draw` and `boids.present`;
- draw evidence with local instance geometry, `vertex_count=6`, and
  `instance_count=384`;
- typed unsupported GPU timing diagnostics for compute and graphics passes;
- CPU preflight timing evidence from prepared frame preflight;
- resize pixel evidence for `1600x900`, `900x1600`, `1024x1024`, and
  `3200x360` with `aspect_error_px=0.00000`;
- procedural camera projection evidence for:
  - `1600x900` with `world_scale_error=0.000000`;
  - `900x1600` with `world_scale_error=0.000000`;
  - `1024x1024` with `world_scale_error=0.000000`;
  - `3200x360` with `world_scale_error=0.000031`.

## Benchmark Evidence

`cargo bench -p engine --bench render_flow_planning` passed.

WR-093 procedural population benchmark cases from the 2026-05-25 run:

- `render_population/prefix_scan_plan_4096`: `13.655 us` to `14.956 us`;
  Criterion reported no statistically significant performance change.
- `render_population/scan_compaction_indirect_args_plan_4096`: `20.998 us`
  to `21.618 us`; Criterion reported no statistically significant performance
  change.
- `render_population/bounded_grid_build_plan_4096`: `2.8733 us` to
  `2.9442 us`; Criterion reported improvement.
- `render_population/boids_production_flow_planning`: `2.4578 us` to
  `2.4974 us`; Criterion reported no statistically significant performance
  change.
- `render_population/boids_production_preflight_cold`: `4.5197 us` to
  `4.6597 us`; Criterion reported improvement.
- `render_population/boids_production_evidence_report`: `18.815 us` to
  `19.360 us`; Criterion reported improvement.

The same run reported local Criterion regressions in several generic
render-flow and temporal benchmark baselines:

- `render_flow/simple_fullscreen`: `+25.337%`;
- `render_flow/boids_ping_pong`: `+8.4647%`;
- `render_flow/multi_pass_compute_compose`: `+12.355%`;
- `render_flow/sdf_compute_compose`: `+17.795%`;
- `render_temporal/production_evidence_report`: `+5.7311%`.

Those deltas did not fail the benchmark command. They should be treated as
local Criterion baseline signals to re-profile if repeated, not as portable FPS
claims. The WR-093 population and evidence cases required for this closeout
were stable or improved.

## Documentation Audit

The public docs match the implemented contracts:

- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
  documents direct draw first, then explicit indirect draw, renderer-owned GPU
  primitives, fixed-step graph scheduling, procedural camera projection, the
  evidence command, and the benchmark command.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  lists the procedural camera contracts, explicit indirect draw APIs,
  renderer-owned primitive contracts, boids fixed-step/camera evidence, and the
  render-flow benchmark command.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`
  documents the boids runtime evidence, graph-level fixed-step contract,
  resize/camera evidence, and benchmark coverage.

## Validation

- `cargo fmt --all -- --check` passed.
- `cargo test -p engine render_flow` passed with 24 library tests, 1 ignored
  adapter-dependent GPU timing test, and the filtered render-flow integration
  tests.
- `cargo test -p engine gpu_primitives` passed with 13 GPU primitive tests,
  including runtime scan/scatter/draw-args dispatch on the available adapter.
- `cargo test -p engine procedural` passed with 11 procedural library tests and
  8 `procedural_instance` tests.
- `cargo test -p engine --example boids_render_flow` passed with 19 tests.
- `cargo run -p engine --example boids_render_flow -- --evidence` passed.
- `cargo bench -p engine --bench render_flow_planning` passed.
- `task roadmap:render` passed after closeout metadata updates.
- `task roadmap:validate` passed after closeout metadata updates.
- `task roadmap:check` passed after closeout metadata updates.
- `task production:render` passed after closeout metadata updates.
- `task production:validate` passed after closeout metadata updates.
- `task production:check` passed after closeout metadata updates.
- `task docs:validate` passed after closeout metadata updates.
- `task planning:validate` passed after closeout metadata updates.
- `task ai:closeout -- --task "WR-093 procedural population hardening evidence benchmarks docs and closeout" --roadmap "docs-site/src/content/docs/workspace/roadmap-items.yaml"`
  produced the phase completion drift-check prompt after closeout.
- `git diff --check` passed before closeout with line-ending warnings only.

## Completion Quality

Completion quality: `runtime_proven`.

This claim is scoped to the procedural population hardening track: indirect
draw validation, renderer-owned GPU primitive dispatch, graph fixed-step
catch-up scheduling, procedural camera projection, public docs, benchmark
coverage, and closeout evidence.

Known quality gaps:

- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- The generic Criterion regressions listed above need re-profiling if repeated
  in future benchmark runs, but they did not fail this closeout command.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Closeout Decision

`WR-093` may be archived as completed and `PM-RENDER-POP-HARDEN-006` may be
completed. `PT-RENDER-PROCEDURAL-POPULATION-HARDENING` may close at
`runtime_proven`, with the known gaps above remaining visible.

This closeout does not claim `perfectionist_verified`.
