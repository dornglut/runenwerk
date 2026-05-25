---
title: WR-101 Procedural Camera And View Projection Implementation Contract
description: Promotion and implementation-readiness contract for reusable procedural 2D camera projection and aspect-correct population rendering.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../wr-092-fixed-step-graph-catch-up-scheduling/plan.md
  - ../../closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-101 Procedural Camera And View Projection Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-005` / `WR-101` as reusable procedural 2D
camera projection and sprite sizing support.

The production outcome is a renderer procedural contract that lets procedural
population examples fill the target without letterbox and without non-uniform
stretch, while keeping camera intent in producer or example state. Renderer
code owns only derived projection math, projection uniform bytes, sprite sizing
contracts, validation, and evidence.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`
- `docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness And Promotion

`WR-101` depends on completed `WR-092`. That dependency is complete and archived
with closeout evidence proving graph-level fixed-step catch-up scheduling.

Current preflight:

```text
task production:plan -- --milestone "PM-RENDER-POP-HARDEN-005" --roadmap "WR-101"
```

The preflight reports `Status: promotable` and `Next action:
write_promotion_contract`. This document is the promotion contract. It is
planning work only and must not include product code changes.

Promotion command after this contract validates:

```text
task roadmap:promote -- --id WR-101 --state current_candidate --evidence "docs-site/src/content/docs/reports/implementation-plans/wr-101-procedural-camera-and-view-projection/plan.md; docs-site/src/content/docs/reports/closeouts/wr-092-fixed-step-graph-catch-up-scheduling/closeout.md"
```

Do not implement product code until `task ai:goal -- --track
PT-RENDER-PROCEDURAL-POPULATION-HARDENING` advances PM-005 from
`prepare_wr_promotion_contract` to the implementation action after promotion.

## Gates And Dependencies

- `WR-092` must remain completed with valid closeout evidence.
- `renderer-procedural-population-hardening-platform-design.md` must remain
  active.
- The WR row must remain dependency-legal and write-scope-clean.
- No ADR is required while camera projection remains renderer-owned derived data
  and producer/example state owns camera intent. Stop for ADR/design review
  before moving camera source truth into renderer, `PreparedViewFrame`, product
  selection, or runtime global state.

## Implementation Scope

Owning domain: `engine`.

Owning crate: `engine`.

Owning subsystem: renderer procedural infrastructure under
`engine/src/plugins/render/procedural`.

Expected modules and exact responsibilities:

- `engine/src/plugins/render/procedural/camera.rs`
  - Add `ProceduralCamera2d`.
  - Add `ProceduralCamera2dAspectPolicy`.
  - Add `ProceduralCamera2dUniform`.
  - Add `ProceduralSpriteSizing`.
  - Add projection and pixel/world sizing helpers with explicit numeric
    invariants for fill-viewport behavior.
- `engine/src/plugins/render/procedural/mod.rs`
  - Export the common camera contracts from the procedural API.
- `engine/src/plugins/render/api/flow.rs`
  - Add a discoverable render-flow authoring helper only if the camera uniform
    can be wired generically without hiding producer-owned camera intent.
- `engine/examples/boids_render_flow/rendering/state.rs`
  - Keep boids camera intent and world bounds in example state.
  - Replace viewport-derived simulation assumptions with stable world bounds.
  - Build draw/camera uniform data from `ProceduralCamera2d` and
    `ProceduralSpriteSizing`.
- `engine/examples/boids_render_flow/rendering/graph.rs`
  - Wire boids render-flow uniforms through the public procedural camera
    contract.
- `engine/examples/boids_render_flow/rendering/evidence.rs`
  - Report landscape, portrait, square, and extreme-aspect camera evidence.
  - Prove equal projected world x/y scale and sprite aspect stability.
- `assets/shaders/boids_compute.wgsl`
  - Preserve stable simulation world bounds and do not derive simulation truth
    from viewport dimensions.
- `assets/shaders/boids_compose.wgsl`
  - Project world positions through `ProceduralCamera2dUniform`.
  - Consume sprite sizing without hardcoded viewport-only draw fixes.
- `docs-site/src/content/docs/engine/examples/boids-render-flow/README.md`
  - Document the example's fixed-step and procedural camera behavior.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`
  - Document the normal procedural camera authoring flow.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  - Add public API reference entries for the camera contracts.

## Required Public Contracts

The procedural API must expose:

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
camera-governed. `Pixels` remains available for marker, selection, and debug
examples where screen-space size is intentional.

## Source-To-Runtime Data Flow

1. Producer/example code owns camera intent as `ProceduralCamera2d`.
2. Renderer procedural code derives a `ProceduralCamera2dUniform` from camera
   intent and the current target surface size.
3. Render-flow uniform projection uploads the derived uniform as frame data.
4. Boids compose shader consumes the uniform to map world positions into clip
   space with equal projected world x/y scale.
5. Boids evidence reconstructs pixel and world scale for multiple aspect ratios
   and reports whether world stretch or sprite stretch occurred.

`PreparedViewFrame` remains target size, view identity, history, and render
packet metadata. It must not store camera intent, gameplay camera state, or
product camera authority.

## Diagnostics And Failure Policy

- Invalid surface dimensions fail with a typed procedural camera diagnostic or
  explicit `Result` error; do not silently substitute `1x1`.
- Invalid world extents or zero fixed-axis span fail closed.
- Unsupported sprite sizing combinations must be explicit diagnostics, not
  shader constants.
- Evidence must distinguish projection correctness from fixed-step timing
  correctness. Resize/redraw events must not be counted as simulation progress
  proof.
- No hidden CPU fallback may be used as runtime proof for projection behavior.

## Implementation Steps

1. Add the procedural camera module with typed camera, aspect-policy, uniform,
   and sprite-sizing contracts.
2. Add focused unit tests for landscape, portrait, square, and extreme aspect
   surfaces, including equal projected world x/y scale.
3. Export the contracts through `engine/src/plugins/render/procedural/mod.rs`
   and the renderer public API path.
4. Update boids render state to own camera intent and stable world bounds.
5. Wire boids graph uniform projection through the reusable camera contract.
6. Update boids compose shader to consume `ProceduralCamera2dUniform` for world
   to clip projection.
7. Extend boids evidence to prove no letterbox, no world stretch, no sprite
   aspect stretch, and no input/redraw-rate speedup.
8. Update public docs and example docs so the procedural camera API is
   discoverable from normal renderer usage paths.
9. Run the full WR validation set and write the closeout before archiving
   `WR-101`.

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
- Public usage docs and API reference describe the normal authoring path and do
  not require internal shortcuts.

## Non-Goals

- Do not implement product camera ownership or prepared-view camera truth.
- Do not add letterbox-preserving fit modes unless a later design requires
  them.
- Do not implement richer flock split/merge behavior.
- Do not implement spatial hash or chunked unbounded populations.
- Do not change graph catch-up scheduling semantics from `WR-092`.
- Do not claim `perfectionist_verified`; this track targets
  `runtime_proven`.

## Stop Conditions

- Stop if aspect correctness can only be achieved with a boids-only shader
  patch rather than a reusable procedural camera contract.
- Stop if camera intent needs to be stored in `PreparedViewFrame`.
- Stop if the evidence cannot prove equal world x/y scale after projection.
- Stop if resize correctness depends on viewport-dependent simulation bounds.
- Stop if implementation requires ownership of product, gameplay, or world
  camera truth by the renderer.
- Stop if source files changed enough that `task production:plan` or
  `task ai:goal` must be rerun before continuing.

## Validation

Required focused and workflow validation:

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

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-101-procedural-camera-and-view-projection/closeout.md`

Completion quality target: `runtime_proven` for procedural camera projection.

Closeout must update:

- `docs-site/src/content/docs/workspace/roadmap-items.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- generated roadmap and production docs

Known gaps that must remain visible after this slice:

- Evidence, public docs, benchmark reporting, and track closeout remain
  `WR-093`.
- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Perfectionist Closeout Audit

This slice should close at `runtime_proven`, not `perfectionist_verified`.

The closeout audit must explicitly prove:

- the procedural camera contract is public and reusable, not boids-only;
- projection math is tested independently of the boids example;
- boids runtime evidence consumes the public camera uniform path;
- resize/aspect evidence covers landscape, portrait, square, and extreme aspect
  surfaces;
- `PreparedViewFrame` does not become camera source truth;
- fixed-step timing evidence from `WR-092` remains intact after camera changes;
- remaining production gaps are listed in the WR archive and production track.

## Critical Review

The shortcut to avoid is fixing boids by pushing more surface math into its
`DrawParams` and shader only. That would leave aspect-correct population
rendering undiscoverable and unowned. The durable solution is a renderer
procedural camera contract that derives projection uniforms from producer-owned
camera intent and target size.

The second shortcut to avoid is making `PreparedViewFrame` the camera owner.
Prepared views carry render-target packet metadata, not product or gameplay
camera truth.
