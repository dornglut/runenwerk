---
title: WR-084 Procedural Builder And First Class Draw Sources Implementation Contract
description: Bounded implementation contract for procedural-owned pass authoring and typed direct versus indirect draw-source semantics.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-084 Procedural Builder And First Class Draw Sources Implementation Contract

## Goal

Implement `PM-RENDER-POP-002` / `WR-084` as the renderer API slice that makes
advanced procedural pass authoring possible without leaking
`GraphicsPassBuilder`, and makes graphics draw submission explicitly typed as
direct or indirect.

This slice is not the GPU primitive platform, population grid, boids runtime
proof, benchmark pass, or final production closeout. Those remain `WR-085`
through `WR-088`.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  procedural population doctrine and the prohibition on exposing graphics
  builder internals through the procedural API.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PM-RENDER-POP-002` is active and links to `WR-084`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-084` is the current implementation row and depends on completed archived
  `WR-083`.
- `docs-site/src/content/docs/reports/closeouts/wr-083-renderer-procedural-population-doctrine-and-track-activation/closeout.md`:
  doctrine and track activation are complete at `bounded_contract`.

## Readiness

`task production:plan -- --milestone "PM-RENDER-POP-002" --roadmap "WR-084"`
reported:

- production milestone state: `active`;
- roadmap state: `current_candidate`;
- dependency evidence: `WR-083:completed`;
- next action: `write_implementation_contract`;
- contract target:
  `docs-site/src/content/docs/reports/implementation-plans/wr-084-procedural-builder-and-first-class-draw-sources/plan.md`.

No ADR is required while the work remains renderer-owned and does not move
product source truth, fallback policy, world state, or residency authority into
the renderer.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/procedural/authoring.rs`:
  `ProceduralPassBuilder`.
- `engine/src/plugins/render/procedural/lowering.rs`:
  `lower_procedural_pass` and default simple-path lowering.
- `engine/src/plugins/render/procedural/mod.rs`:
  procedural module exports and removal of the old builder module.
- `engine/src/plugins/render/api/flow.rs`:
  `RenderFlow::procedural_pass_builder` plus focused tests.
- `engine/src/plugins/render/api/passes.rs`:
  `GraphicsPassBuilder::draw_indirect` and offset variants.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  `RenderDrawSource`, draw descriptors, and typed indirect draw-args ABI.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  compiled direct versus indirect draw source.
- `engine/src/plugins/render/graph/validation.rs`:
  fail-closed validation for indirect draw source declarations and offsets.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  `encode_graphics_pass` uses the compiled draw source instead of sidecar
  inference.

The implementation may touch `engine/src/plugins/render/gpu_primitives/draw_args.rs`
only to consume or re-export the draw-args ABI owned by `WR-084`; reusable
generation descriptors remain `WR-085`.

## Required Decisions

- `RenderFlow::procedural_pass(...)` remains the simple direct path.
- `RenderFlow::procedural_pass_builder(...)` returns a procedural-owned
  `ProceduralPassBuilder`; public procedural users never receive a
  `GraphicsPassBuilder`.
- `ProceduralPassBuilder` supports:
  `uniform_from_state`, `uniform_from_state_with_surface`,
  explicit `*_to` variants for preallocated handles, and typed indirect draw
  authoring.
- `RenderDrawSource` is the source of truth for submission mode.
  `Direct` must submit direct draw calls.
  `Indirect` must submit indirect draw calls only after validation proves the
  args buffer was declared and the byte offset is aligned.
- A declared indirect buffer sidecar must not silently convert a direct
  `.draw(...)` into indirect submission. That would preserve the old ambiguity
  and violate this slice.

## Non-Goals

WR-084 does not implement:

- prefix scan, scatter, compaction, counter reset, or indirect args generation
  primitives;
- bounded uniform-grid population support;
- boids shader/runtime upgrade;
- visual resize evidence, benchmarks, usage-guide updates, or final track
  closeout;
- multi-step fixed-update graph scheduling.

## Implementation Steps

1. Keep `ProceduralPassBuilder` in
   `engine/src/plugins/render/procedural/authoring.rs` and ensure it stores only
   procedural descriptors, uniform bindings, and a procedural draw-source
   choice.
2. Keep lowering in
   `engine/src/plugins/render/procedural/lowering.rs::lower_procedural_pass`.
   The lowering function may create a `GraphicsPassBuilder` internally, but that
   builder must not become procedural public API.
3. Add or keep `RenderFlow::procedural_pass_builder` while preserving
   `RenderFlow::procedural_pass`.
4. Define first-class draw-source descriptors in
   `engine/src/plugins/render/graph/pass_graph.rs`.
5. Add typed indirect draw APIs to
   `engine/src/plugins/render/api/passes.rs::GraphicsPassBuilder`.
6. Compile draw-source semantics through
   `engine/src/plugins/render/graph/execution_plan.rs`.
7. Validate fail-closed in
   `engine/src/plugins/render/graph/validation.rs`, including missing args
   buffer declarations and invalid offsets.
8. Update
   `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::encode_graphics_pass`
   so direct and indirect submission are selected from the compiled draw
   source, not inferred from `indirect_buffer(...)`.
9. Add focused tests proving direct-path preservation, indirect compilation,
   missing declaration rejection, and unaligned offset rejection.

## Acceptance Criteria

- The procedural API does not expose `GraphicsPassBuilder`.
- Existing `.draw(...)` stays the simple direct path.
- Indirect draw authoring uses typed args buffers.
- Indirect draw source is visible in graph and compiled execution-plan data.
- Validation rejects indirect draw sources with undeclared args buffers or
  unaligned offsets.
- Runtime encoding uses the compiled draw source and has no hidden
  indirect-sidecar fallback.

## Validation

Run:

- `cargo fmt --all -- --check`
- `cargo test -p engine procedural`
- `cargo test -p engine render_flow`
- `task docs:validate`

Closeout metadata must then pass:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task planning:validate`

## Stop Conditions

Stop before closing WR-084 if:

- any procedural API requires callers to receive or manipulate
  `GraphicsPassBuilder`;
- indirect submission is still inferred from the presence of an indirect buffer
  sidecar;
- validation allows an indirect draw source without a declared args buffer;
- the slice requires reusable GPU primitive execution, population grids, or
  boids evidence to pass.

## Closeout Requirements

Create
`docs-site/src/content/docs/reports/closeouts/wr-084-procedural-builder-and-first-class-draw-sources/closeout.md`
with `status: completed`.

Update `WR-084` in `docs-site/src/content/docs/workspace/roadmap-items.yaml`
or, after completion, move it to
`docs-site/src/content/docs/workspace/roadmap-archive.yaml`. The completed row
must set:

- `planning_state: completed`;
- `completion_quality: bounded_contract`;
- `known_quality_gaps` listing `WR-085` through `WR-088` and
  `PT-RENDER-PERFECTION`;
- `completion_audit` pointing at the WR-084 closeout.

Update `PM-RENDER-POP-002` in
`docs-site/src/content/docs/workspace/production-tracks.yaml` with completed
state, evidence gate, completion audit, and honest quality gaps.

Run the phase completion drift-check routine before starting `WR-085`.

## Perfectionist Closeout Audit

WR-084 targets `bounded_contract`, not `runtime_proven` or
`perfectionist_verified`.

Known quality gaps at closeout:

- reusable GPU primitive contracts and runtime execution remain `WR-085`;
- bounded-grid population support remains `WR-086`;
- boids runtime proof remains `WR-087`;
- evidence, benchmarks, docs, and track closeout remain `WR-088`;
- final no-gap verification remains `PT-RENDER-PERFECTION`.
