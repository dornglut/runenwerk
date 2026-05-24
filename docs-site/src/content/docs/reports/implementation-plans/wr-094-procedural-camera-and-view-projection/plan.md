---
title: WR-094 Procedural Camera And View Projection Implementation Contract
description: Bounded implementation contract for reusable procedural 2D camera projection and aspect-correct population rendering.
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

# WR-094 Procedural Camera And View Projection Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-005` / `WR-094` as reusable procedural 2D
camera projection and sprite sizing support.

The outcome is a renderer procedural contract that lets procedural population
examples fill the target without letterbox and without non-uniform stretch,
while keeping camera intent in producer/example state and making renderer
projection uniforms derived execution data.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-092-fixed-step-graph-catch-up-scheduling/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness

This slice depends on completed `WR-092`. Camera evidence must be gathered
after graph catch-up evidence so resize and redraw events cannot be mistaken for
simulation timing correctness.

Do not start this slice until `task production:plan -- --milestone
"PM-RENDER-POP-HARDEN-005" --roadmap "WR-094"` confirms the milestone and row
are promotable.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/procedural/camera.rs`:
  reusable procedural camera descriptors, aspect policy, projection uniform, and
  sprite sizing contracts.
- `engine/src/plugins/render/procedural/mod.rs`:
  public renderer procedural exports for the camera contracts.
- `engine/src/plugins/render/api/flow.rs`:
  render-flow API integration only if procedural camera projection needs a
  discoverable authoring helper.
- `engine/examples/boids_render_flow/rendering/state.rs`:
  boids producer-owned camera intent, stable world bounds, sprite sizing choice,
  and evidence counters.
- `engine/examples/boids_render_flow/rendering/graph.rs`:
  boids render-flow wiring for procedural camera uniforms.
- `engine/examples/boids_render_flow/rendering/evidence.rs`:
  landscape, portrait, square, and extreme-aspect projection evidence.
- `assets/shaders/boids_compute.wgsl`:
  ensure simulation uses stable world bounds and not viewport-derived truth.
- `assets/shaders/boids_compose.wgsl`:
  project world positions through `ProceduralCamera2dUniform`.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`:
  example evidence and user-facing behavior notes.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  normal procedural camera authoring flow.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  public API reference for procedural camera contracts.

## Required Contracts

Public renderer procedural contracts:

- `ProceduralCamera2d`
- `ProceduralCamera2dAspectPolicy`
- `ProceduralCamera2dUniform`
- `ProceduralSpriteSizing`

The canonical boids aspect policy is:

```text
FillViewport { fixed_axis: Vertical }
```

Fill viewport means:

- no letterbox;
- no non-uniform stretch;
- stable vertical world scale;
- horizontal visible world span follows viewport aspect.

The canonical boids sprite sizing is `WorldUnits` by default, so sprite scale is
camera-governed. `Pixels` remains available for marker and debug-style
examples.

## Acceptance Criteria

- `engine/src/plugins/render/procedural/camera.rs` exposes reusable camera,
  aspect policy, uniform, and sprite sizing contracts from the procedural API.
- `PreparedViewFrame` remains target/view/history metadata and does not own
  camera intent.
- Boids compute uses stable world bounds rather than viewport-dependent
  simulation truth.
- Boids compose projects world positions through `ProceduralCamera2dUniform`.
- Projection tests prove equal projected world x/y scale for landscape,
  portrait, square, and extreme aspect surfaces.
- Boids evidence proves no world stretch, no sprite aspect stretch, and no
  input/redraw-rate speedup after the camera change.

## Non-Goals

- Do not implement product camera ownership or prepared-view camera truth.
- Do not add letterbox-preserving fit modes unless a later design requires
  them.
- Do not implement richer flock split/merge behavior.
- Do not implement spatial hash or chunked unbounded populations.
- Do not change graph catch-up scheduling semantics from `WR-092`.

## Stop Conditions

- Stop if aspect correctness can only be achieved with a boids-only shader
  patch rather than a reusable procedural camera contract.
- Stop if camera intent needs to be stored in `PreparedViewFrame`.
- Stop if the evidence cannot prove equal world x/y scale after projection.
- Stop if resize correctness depends on viewport-dependent simulation bounds.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-094-procedural-camera-and-view-projection/closeout.md`

Completion quality target: `runtime_proven` for procedural camera projection.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine procedural`
- `cargo test -p engine render_flow`
- `cargo test -p engine --example boids_render_flow`
- `cargo run -p engine --example boids_render_flow -- --evidence`
- camera projection unit tests for landscape, portrait, square, and extreme
  aspect ratios
- shader/evidence tests proving equal projected world scale on x/y
- boids evidence proving no input/redraw-rate speedup, no world stretch, and no
  sprite aspect stretch
- `task docs:validate`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task planning:validate`

## Critical Review

The shortcut to avoid is fixing boids by pushing more surface math into its
`DrawParams` and shader only. That would leave aspect-correct population
rendering undiscoverable and unowned. The durable solution is a renderer
procedural camera contract that derives projection uniforms from producer-owned
camera intent and target size. The second shortcut to avoid is making
`PreparedViewFrame` the camera owner; prepared views carry render-target packet
metadata, not product or gameplay camera truth.
