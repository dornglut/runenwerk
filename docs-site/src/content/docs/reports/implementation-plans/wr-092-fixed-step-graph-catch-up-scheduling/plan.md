---
title: WR-092 Fixed Step Graph Catch Up Scheduling Implementation Contract
description: Bounded implementation contract for graph-level repeated pass execution, runtime fixed-time source reuse, iteration-scoped uniform projection, and fixed-step catch-up evidence.
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
- `docs-site/src/content/docs/reports/implementation-plans/wr-091-reusable-gpu-primitive-shader-dispatch/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- `engine/src/runtime/fixed_time.rs`

## Readiness

This slice depends on completed `WR-091`. Catch-up scheduling must preserve the
primitive and population resource sequencing that WR-091 makes executable.

The existing runtime resources `FixedTimeConfig`, `FixedTimeState`, and
`CatchupBudget` are the timing source truth. Renderer graph execution owns the
derived repeated GPU pass submission for render flows; it does not introduce a
second accumulator in boids or any other example.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/api/flow.rs`:
  public render-flow scheduling authoring API if needed.
- `engine/src/plugins/render/api/passes.rs`:
  pass grouping or scheduling descriptors only if graph authoring belongs near
  pass builders.
- `engine/src/plugins/render/api/bindings.rs`:
  frame-scoped and iteration-scoped uniform projection contracts.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  graph scheduling descriptors for bounded repeated pass execution.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  compiled repeated-pass schedule, iteration scope metadata, and submitted
  substep metadata.
- `engine/src/plugins/render/graph/validation.rs`:
  validation for repeat bounds, pass existence, acyclic repeated groups, and
  resource sequencing invariants.
- `engine/src/plugins/render/graph/prepared_validation.rs`:
  prepared invocation validation for repeated pass schedules if prepared state
  carries projected dispatch or draw information per substep.
- `engine/src/plugins/render/runtime/frame_prepare.rs`:
  prepared uniform projection plumbing for iteration-scoped uniform bytes.
- `engine/src/plugins/render/renderer/render_flow/execute.rs`:
  submitted substep count selection from runtime fixed-time state and invocation
  scoping for repeated regions.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  execution of bounded repeated pass groups.
- `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` and
  `engine/src/plugins/render/renderer/render_flow/runtime_resources/*.rs`:
  runtime uniform upload scoping for frame and iteration projections.
- `engine/examples/boids_render_flow/rendering/state.rs`:
  evidence consumption only, not local catch-up authority.
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

## Critical Review

The shortcut to avoid is adding an accumulator to
`engine/examples/boids_render_flow/rendering/state.rs::BoidsRenderState` and
calling that catch-up. That would solve one example while leaving render-flow
GPU work tied to redraw cadence. This slice must make repeated execution a
graph contract, consume the existing runtime fixed-time source, add
iteration-scoped uniform projection for repeated regions, and prove that input,
redraw, and resize activity cannot accelerate simulation submission.

