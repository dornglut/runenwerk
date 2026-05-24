---
title: WR-092 Fixed Step Graph Catch Up Scheduling Implementation Contract
description: Bounded implementation contract for graph-level repeated pass execution and fixed-step catch-up evidence.
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
dropped or deferred time diagnostics. This must not be boids-local timing logic.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-091-reusable-gpu-primitive-shader-dispatch/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness

This slice depends on completed `WR-091`. Catch-up scheduling must preserve the
primitive and population resource sequencing that WR-091 makes executable.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/api/flow.rs`:
  public render-flow scheduling authoring API if needed.
- `engine/src/plugins/render/api/passes.rs`:
  pass grouping or scheduling descriptors only if graph authoring belongs near
  pass builders.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  graph scheduling descriptors for bounded repeated pass execution.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  compiled repeated-pass schedule and submitted substep metadata.
- `engine/src/plugins/render/graph/validation.rs`:
  validation for repeat bounds, pass existence, acyclic repeated groups, and
  resource sequencing invariants.
- `engine/src/plugins/render/graph/prepared_validation.rs`:
  prepared invocation validation for repeated pass schedules if prepared state
  carries projected dispatch or draw information per substep.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  execution of bounded repeated pass groups.
- `engine/examples/boids_render_flow/rendering/state.rs`:
  evidence consumption only, not local catch-up authority.
- `engine/examples/boids_render_flow/rendering/evidence.rs`:
  submitted-step evidence if boids is used as the canonical visible example.

## Required Decisions

- The graph owns repeated pass scheduling. Individual examples may provide time
  input and evidence but must not implement private catch-up loops.
- The scheduling contract must support submitted substeps in the closed range
  `0..=max_substeps`.
- Accumulated time, fixed dt, submitted substeps, max substeps, and dropped or
  deferred time diagnostics must be inspectable.
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

## Non-Goals

- Do not implement gameplay simulation ownership.
- Do not implement unbounded catch-up.
- Do not implement spatial hash or chunked unbounded populations.
- Do not change primitive dispatch semantics from WR-091.

## Stop Conditions

- Stop if repeated scheduling would require renderer ownership of gameplay
  source truth.
- Stop if resource sequencing cannot be proven for ping-pong buffers.
- Stop if catch-up can only be observed in boids-local state.

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
calling that catch-up. That would solve one example but leave the render graph
unable to schedule repeated GPU work generally. This slice must make repeated
execution a graph contract and prove it independently of boids-local logic.

