---
title: WR-084 Procedural Builder And First Class Draw Sources Closeout
description: Closeout evidence for procedural-owned pass authoring and first-class direct versus indirect draw-source semantics.
status: completed
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../implementation-plans/wr-084-procedural-builder-and-first-class-draw-sources/plan.md
---

# WR-084 Procedural Builder And First Class Draw Sources Closeout

## Result

`WR-084` is completed as a bounded renderer API and graph-contract slice.
Procedural authors now have a procedural-owned builder for advanced pass
authoring, and render-flow graphics passes carry explicit direct versus
indirect draw-source semantics through validation, execution-plan compilation,
and runtime encoding.

This closeout does not claim reusable GPU primitive execution, bounded-grid
population support, boids runtime proof, benchmarks, docs closeout, or final
track evidence. Those remain `WR-085` through `WR-088`.

## What Changed

- Added `engine/src/plugins/render/procedural/authoring.rs::ProceduralPassBuilder`.
- Moved procedural pass lowering into
  `engine/src/plugins/render/procedural/lowering.rs::lower_procedural_pass`.
- Added `engine/src/plugins/render/api/flow.rs::RenderFlow::procedural_pass_builder`
  while preserving `RenderFlow::procedural_pass`.
- Added `engine/src/plugins/render/graph/pass_graph.rs::RenderDrawSource`
  and renderer-owned typed indirect draw args ABI.
- Added typed indirect authoring through
  `engine/src/plugins/render/api/passes.rs::GraphicsPassBuilder::draw_indirect`
  and offset variants.
- Compiled draw-source semantics through
  `engine/src/plugins/render/graph/execution_plan.rs`.
- Added fail-closed graph validation in
  `engine/src/plugins/render/graph/validation.rs`, including rejection for
  direct draws that declare indirect buffer sidecars.
- Updated
  `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::encode_graphics_pass`
  so runtime draw submission is selected only from the compiled draw source.
- Added focused render-flow and procedural tests, including a procedural-builder
  indirect draw test in `engine/tests/procedural_instance.rs`.

## Evidence

- `task production:plan -- --milestone "PM-RENDER-POP-002" --roadmap "WR-084"`
  classified the row as current-candidate eligible and requested this
  implementation contract.
- The implementation contract is recorded at
  `docs-site/src/content/docs/reports/implementation-plans/wr-084-procedural-builder-and-first-class-draw-sources/plan.md`.
- The old ambiguous sidecar behavior is removed: an indirect buffer declared
  with a direct draw source is now a validation error, not a hidden indirect
  submission path.

## Validation

- `cargo fmt --all -- --check` passed.
- `cargo test -p engine procedural` passed.
- `cargo test -p engine render_flow` passed.
- `task docs:validate` passed.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Reusable GPU primitive contracts and runtime execution remain `WR-085`.
- Bounded uniform-grid population support remains `WR-086`.
- Boids runtime proof remains `WR-087`.
- Evidence, benchmarks, docs, and track closeout remain `WR-088`.
- Final no-gap verification remains `PT-RENDER-PERFECTION`.

## Next Slice

The next legal implementation slice is `WR-085`: reusable GPU prefix scan,
compaction, counter reset, and indirect args primitives.
