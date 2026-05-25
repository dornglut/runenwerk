---
title: WR-092 Fixed Step Graph Catch Up Scheduling Closeout
description: Closeout evidence for renderer graph-owned fixed-step repeated pass scheduling, runtime fixed-time source reuse, and boids graph-submitted substep evidence.
status: completed
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../implementation-plans/wr-092-fixed-step-graph-catch-up-scheduling/plan.md
  - ../wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md
---

# WR-092 Fixed Step Graph Catch Up Scheduling Closeout

## Result

`WR-092` is completed for graph-level fixed-step catch-up scheduling.

Render-flow graphs can now declare a fixed-step repeated region over stable pass
labels. The compiled execution plan carries typed region metadata, frame
preparation projects runtime `FixedTimeConfig`, `FixedTimeState`, and
`CatchupBudget` resources into fixed-step iteration uniform bytes, and renderer
execution expands the compiled region into `0..=max_substeps` repeated pass
submissions through the existing pass encoder.

## What Changed

- Added fixed-step region descriptors in
  `engine/src/plugins/render/graph/pass_graph.rs::RenderFixedStepRegionId` and
  `RenderFixedStepRegionMembership`.
- Added fixed-step iteration uniform bytes in
  `engine/src/plugins/render/api/bindings.rs::RenderFixedStepIterationUniform`.
- Added public graph authoring in
  `engine/src/plugins/render/api/flow.rs::RenderFlow::fixed_step_region`.
- Compiled region metadata through
  `engine/src/plugins/render/graph/execution_plan.rs::CompiledFixedStepRegion`
  and `compile_execution_plan`.
- Extended validation in
  `engine/src/plugins/render/graph/validation.rs::validate_flow_graph` for
  repeated-region bounds, unsupported pass kinds, descriptor consistency,
  iteration uniforms, and contiguous pass order.
- Projected fixed-time resources in
  `engine/src/plugins/render/runtime/frame_prepare.rs::build_prepared_flow_inputs`.
- Reused renderer pass encoding in
  `engine/src/plugins/render/renderer/render_flow/execute.rs::schedule_invocation_passes`
  and `Renderer::upload_fixed_step_iteration_uniform`.
- Marked the boids compute region in
  `engine/examples/boids_render_flow/rendering/graph.rs::build_render_flow`
  and updated
  `engine/examples/boids_render_flow/rendering/evidence.rs::production_evidence_report`
  to report graph-submitted fixed-step evidence.

## Evidence

- `task ai:architecture-governance -- --task "WR-092 fixed-step graph catch-up scheduling" --scope "engine/src/runtime/fixed_time.rs, engine/src/plugins/render/api, engine/src/plugins/render/graph, engine/src/plugins/render/runtime/frame_prepare.rs, engine/src/plugins/render/renderer/render_flow, engine/examples/boids_render_flow/rendering"` completed the governance checklist. No ADR was required because timing source truth stays in runtime fixed-time resources and renderer owns only derived pass submission.
- `cargo fmt --all -- --check` passed.
- `cargo test -p engine render_flow` passed with 24 passed, 1 ignored adapter-dependent GPU timing test, and 0 failed.
- `cargo test -p engine fixed_step_region` passed with 2 focused graph authoring and validation tests.
- `cargo test -p engine fixed_step_scheduler` passed with 2 focused scheduler expansion tests covering zero, one, many, and clamped substep counts.
- `cargo test -p engine procedural` passed with 13 procedural tests.
- `cargo test -p engine --example boids_render_flow` passed with 17 tests.
- `cargo run -p engine --example boids_render_flow -- --evidence` passed and reported `graph_fixed_step_region label=boids.fixed_step max_substeps=4 submitted_substeps=2 pass_count=8`.

## Completion Quality

Completion quality: `runtime_proven` for the bounded graph scheduling slice.

This claim is scoped to renderer graph scheduling and runtime fixed-time source
consumption. It does not claim the full procedural population hardening track is
complete, and it does not claim final renderer perfection.

Known quality gaps:

- Procedural camera and view projection remains `WR-101`.
- Evidence, public docs, benchmark reporting, and track closeout remain
  `WR-093`.
- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Next Slice

The next legal implementation slice is `WR-101`: procedural camera and view
projection, after `task ai:goal` confirms the dependency-order action.
