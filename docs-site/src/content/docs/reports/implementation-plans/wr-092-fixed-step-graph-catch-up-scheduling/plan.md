---
title: WR-092 Fixed Step Graph Catch Up Scheduling Implementation Contract
description: Bounded implementation contract for graph-level repeated pass execution, runtime fixed-time source reuse, iteration-scoped uniform projection, and fixed-step catch-up evidence.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-092 Fixed Step Graph Catch Up Scheduling Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-004` / `WR-092` as graph-level fixed-step
catch-up scheduling.

The outcome is bounded repeated pass execution in the render-flow graph, with
fixed dt, accumulated time, submitted substep count, max substeps, and explicit
dropped or deferred time diagnostics. This must reuse the runtime fixed-time
resources and must not be boids-local timing logic.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-091-reusable-gpu-primitive-shader-dispatch/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- `engine/src/runtime/fixed_time.rs`
- `engine/src/runtime/fixed_step_executor.rs`

## Readiness

This slice depends on completed `WR-091`. Catch-up scheduling must preserve the
primitive and population resource sequencing that WR-091 makes executable.

The existing runtime resources `FixedTimeConfig`, `FixedTimeState`, and
`CatchupBudget` are the timing source truth. Renderer graph execution owns the
derived repeated GPU pass submission for render flows; it does not introduce a
second accumulator in boids or any other example.

Promotion preflight on 2026-05-25 reports `WR-092` as promotable after
`WR-091:completed`. This contract is the only allowed planning change before
promotion. Product code may start only after `task roadmap:promote -- --id
WR-092 --state current_candidate --evidence "<accepted evidence>"` succeeds and
the next `task ai:goal -- --track PT-RENDER-PROCEDURAL-POPULATION-HARDENING`
run confirms implementation is the legal action.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/api/flow.rs`:
  add `RenderFlow` authoring methods for declaring a fixed-step repeated region
  by stable pass labels; preserve `RenderFlow::new`,
  `RenderFlow::validation_report`, `RenderFlow::pass_order`,
  `RenderFlow::gpu_primitive_plan`, and `RenderFlow::graph` behavior.
- `engine/src/plugins/render/api/passes.rs`:
  add builder conveniences only if they delegate into the `RenderFlow` region
  descriptor; keep existing `uniform_from_state`,
  `uniform_from_state_with_surface`, `uniform_from_state_to`, and
  `uniform_from_state_with_surface_to` frame-scoped and source-compatible.
- `engine/src/plugins/render/api/bindings.rs`:
  add an explicit iteration-scoped uniform projection contract alongside
  `PassParamBinding`, `ParamProjection`, and `project_uniform_bindings_for_pass`
  without changing frame-scoped projection semantics.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  add graph-owned repeated-region descriptors in the `PassGraph` module, such as
  a fixed-step repeat region id, region label, ordered pass ids, timing source,
  and max-substep policy. Do not add hidden timing state to `RenderPassNode`
  without a typed region descriptor.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  compile repeated-region descriptors into `CompiledFlowExecutionPlan` and
  `CompiledPassExecutionPlan` ordering metadata so execution can submit
  `0..=max_substeps` bounded passes deterministically.
- `engine/src/plugins/render/graph/validation.rs`:
  extend `RenderFlowValidationIssue`, `validate_render_flow_graph`, and pass
  dependency checks to reject unknown repeat pass labels, duplicate region
  membership, empty regions, zero or excessive max-substep bounds, dependency
  cycles, and resource sequencing that cannot be proven across repeated passes.
- `engine/src/plugins/render/graph/prepared_validation.rs`:
  extend prepared invocation validation when iteration-scoped projected uniforms
  or dispatch metadata are carried per substep.
- `engine/src/plugins/render/runtime/frame_prepare.rs`:
  preserve `frame_render_prepare_system`, `build_prepared_flow_invocations`,
  `apply_invocation_uniform_overrides`, `build_prepared_flow_inputs`, and
  `project_dispatch_for_pass` frame-preparation behavior while adding any
  required per-invocation fixed-step schedule inputs.
- `engine/src/plugins/render/renderer/render_flow/execute.rs`:
  update `Renderer::render_packet` so each flow invocation derives submitted
  substeps from `FixedTimeConfig`, `FixedTimeState`, and `CatchupBudget`, then
  executes repeated regions by substep without moving timing source truth into
  renderer-owned state.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  keep `Renderer::encode_compiled_pass`, `encode_compute_pass`,
  `encode_graphics_pass`, and copy/present execution reusable; repeated regions
  should call the same encoded pass path rather than introducing duplicate pass
  encoders.
- `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` and
  `engine/src/plugins/render/renderer/render_flow/runtime_resources/*.rs`:
  add iteration uniform buffer scoping to `FlowRuntimeResources` only if needed;
  it must coexist with invocation uniform scopes and preserve
  `realize_for_frame`, `retain_invocation_uniform_scopes`,
  `set_active_invocation_uniform_scope`, and resource resolution behavior.
- `engine/examples/boids_render_flow/rendering/state.rs`:
  consume graph-submitted fixed-step evidence only; do not add a boids-local
  accumulator or private catch-up loop.
- `engine/examples/boids_render_flow/rendering/evidence.rs`:
  submitted-step, input/redraw-rate, and resize evidence if boids is used as
  the canonical visible example.

## Required Decisions

- The graph owns repeated pass scheduling. Individual examples may provide time
  input and evidence but must not implement private catch-up loops.
- The graph scheduling contract consumes existing runtime fixed-time resources:
  `FixedTimeConfig::step_seconds`, `FixedTimeState::steps_ran_last_frame`,
  `FixedTimeState::accumulator_seconds`, `FixedTimeState::saturated_frames`,
  and `CatchupBudget::max_steps_per_frame`.
- The scheduling contract must support submitted substeps in the closed range
  `0..=max_substeps`.
- Accumulated time, fixed dt, submitted substeps, max substeps, and dropped or
  deferred time diagnostics must be inspectable.
- Passes inside a repeated region need iteration-scoped uniform projection so a
  pass can receive the current substep index and fixed dt without rewriting
  frame-scoped state.
- Existing `uniform_from_state` and `uniform_from_state_with_surface` must
  remain frame-scoped and source-compatible.
- Ping-pong resources, counters, primitive passes, and draw publication must
  preserve order across substeps.
- Catch-up must be bounded. Unbounded simulation catch-up is not acceptable in
  render submission.

## Implementation Steps

1. Add graph descriptors in
   `engine/src/plugins/render/graph/pass_graph.rs::PassGraph` for fixed-step
   repeated regions, including typed ids, labels, ordered pass membership,
   max-substep bounds, and a fixed-time timing source.
2. Add authoring in `engine/src/plugins/render/api/flow.rs::RenderFlow` that
   declares a fixed-step region by pass labels after normal pass construction.
   The API must fail clearly for unknown labels during validation and must not
   make pass order depend on builder call order outside existing graph rules.
3. Compile the region into
   `engine/src/plugins/render/graph/execution_plan.rs::CompiledFlowExecutionPlan`
   so the execution plan exposes the repeated group, normal pass order, and
   per-region pass order for tests and diagnostics.
4. Extend
   `engine/src/plugins/render/graph/validation.rs::validate_render_flow_graph`
   with typed repeat-region validation. The validation must prove that each
   repeated region has existing passes, no duplicate membership, valid
   max-substeps, no impossible dependencies, and explicit resource sequencing.
5. Add iteration-scoped uniform projection in
   `engine/src/plugins/render/api/bindings.rs::PassParamBinding` and
   `engine/src/plugins/render/runtime/frame_prepare.rs::build_prepared_flow_inputs`
   only for repeated-region iteration values. Existing frame-scoped projection
   APIs stay unchanged.
6. Update
   `engine/src/plugins/render/renderer/render_flow/execute.rs::Renderer::render_packet`
   to derive the submitted substep count from the runtime fixed-time resources
   and execute repeated regions by calling
   `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::Renderer::encode_compiled_pass`
   for each pass in each submitted substep.
7. Add focused render-flow tests for `0`, `1`, and `N` submitted substeps,
   invalid repeat regions, iteration uniforms, preserved ping-pong/resource
   order, and diagnostics for bounded saturation.
8. Update boids evidence in
   `engine/examples/boids_render_flow/rendering/evidence.rs` to report graph
   submitted substeps and redraw/input/resize invariance without introducing
   boids-local timing authority.

## Runtime Data Flow

The source truth is `engine/src/runtime/fixed_step_executor.rs::run_fixed_update_frame`
and the runtime resources in `engine/src/runtime/fixed_time.rs`. Renderer code
may read those resources to derive a render submission product, but it must not
mutate the accumulator, advance `SimulationTick`, or own fixed-step scheduling
truth.

The intended flow is:

1. Runtime updates `FixedTimeState::steps_ran_last_frame`,
   `FixedTimeState::accumulator_seconds`, and
   `FixedTimeState::saturated_frames`.
2. `Renderer::render_packet` reads the prepared frame plus fixed-time runtime
   resources and derives a bounded submitted-substep count per flow invocation.
3. Repeated-region passes execute through the existing compiled pass encoder
   with iteration-scoped uniform values.
4. Evidence records fixed dt, max substeps, submitted substeps, accumulator
   status, and saturation/dropped-time diagnostics.

## Diagnostics And Failure Policy

- Missing fixed-time resources should use the same explicit default resources
  installed by runtime bootstrap paths, not a hidden renderer accumulator.
- Invalid fixed dt, zero max substeps, unknown repeated pass labels, duplicate
  repeat membership, and unsatisfied resource order are validation failures.
- Runtime saturation must remain observable through
  `FixedTimeState::saturated_frames` and evidence; the renderer must not mask it
  as successful unlimited catch-up.
- GPU or adapter absence must not be converted into CPU simulation proof for
  this slice. The completion claim depends on renderer execution evidence.

## Acceptance Criteria

- A test flow submits 0, 1, and N bounded substeps deterministically.
- Resource ordering across substeps is validated and execution-plan visible.
- Invalid repeated groups fail validation before submit.
- Boids or a minimal canonical example reports graph-submitted substep evidence
  without boids-local scheduling logic.
- `max_substeps` and dropped/deferred time behavior are explicit diagnostics.
- Cursor movement, mouse motion, redraw bursts, and resize events do not
  increase submitted simulation steps per real second.
- Iteration-scoped uniform projection is covered by render-flow tests and
  visible evidence.

## Non-Goals

- Do not implement gameplay simulation ownership.
- Do not implement unbounded catch-up.
- Do not implement spatial hash or chunked unbounded populations.
- Do not change primitive dispatch semantics from WR-091.
- Do not implement procedural camera projection; that is `WR-101`.
- Do not tune richer boids split/merge behavior; that is separate behavior
  authoring work.

## Stop Conditions

- Stop if repeated scheduling would require renderer ownership of gameplay
  source truth.
- Stop if resource sequencing cannot be proven for ping-pong buffers.
- Stop if catch-up can only be observed in boids-local state.
- Stop if the implementation introduces a second fixed-time accumulator outside
  the runtime fixed-time resources.
- Stop if iteration-scoped uniform projection breaks existing frame-scoped
  uniform projection behavior.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`

Completion quality target: `runtime_proven` for graph scheduling.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine render_flow`
- `cargo test -p engine procedural`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task docs:validate`
- `task planning:validate`

## Perfectionist Closeout Audit

Completion quality target: `runtime_proven` for graph scheduling. This slice
must not claim `perfectionist_verified` because procedural camera projection,
public docs and benchmark closeout, spatial hash/chunked unbounded populations,
and richer behavior authoring remain outside WR-092.

The WR-092 closeout must name the exact completed audit path:

`docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`

Known gaps that must remain visible after closeout:

- Procedural camera and view projection remains `WR-101`.
- Evidence, public docs, benchmark reporting, and track closeout remain
  `WR-093`.
- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

Anti-drift guards required before closeout:

- A render-flow test proves repeated execution through the graph/execution plan,
  not a descriptor-only schedule.
- A runtime or renderer evidence path proves submitted substeps are consumed by
  pass execution, not only status text.
- Tests prove frame-scoped uniform projection remains source-compatible.
- Tests prove repeated pass execution reuses existing pass encoders and does
  not introduce fallback-only or boids-only scheduling.

## Critical Review

The shortcut to avoid is adding an accumulator to
`engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState` and
calling that catch-up. That would solve one example while leaving render-flow
GPU work tied to redraw cadence. This slice must make repeated execution a
graph contract, consume the existing runtime fixed-time source, add
iteration-scoped uniform projection for repeated regions, and prove that input,
redraw, and resize activity cannot accelerate simulation submission.

