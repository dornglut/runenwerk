---
title: WR-090 Indirect Draw Contract Hardening
description: Closeout evidence for typed, indexed-aware, fail-closed indirect draw validation.
status: completed
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
related_reports:
  - ../../implementation-plans/wr-090-indirect-draw-contract-hardening/plan.md
  - ../wr-089-renderer-procedural-population-hardening-doctrine-and-track-activation/closeout.md
---

# WR-090 Indirect Draw Contract Hardening

## Result

`WR-090` is completed as the indirect draw hardening slice for
`PM-RENDER-POP-HARDEN-002`.

Render-flow draw sources now carry typed indirect argument metadata through
authoring, graph validation, execution-plan compilation, procedural lowering,
and runtime submission. Indexed and non-indexed indirect draw paths are distinct
and fail closed before WGPU submission when argument type, byte offset, or
CPU-side offset contracts are invalid.

## What Changed

- `engine/src/plugins/render/graph/pass_graph.rs`:
  `RenderDrawSource`, `RenderIndirectDrawArgsKind`,
  `IndirectDrawArgsBuffer`, `RenderDrawDescriptor::indirect`, and
  `RenderDrawDescriptor::indirect_with_offsets` now preserve argument kind,
  element count, element byte size, and byte offset.
- `engine/src/plugins/render/api/passes.rs`:
  `GraphicsPassBuilder::draw_indirect`,
  `GraphicsPassBuilder::draw_indexed_indirect`,
  `GraphicsPassBuilder::draw_indirect_with_offsets`, and
  `GraphicsPassBuilder::draw_indirect_resource` now attach typed argument
  metadata from `StorageArrayHandle<T>`.
- `engine/src/plugins/render/graph/validation.rs`:
  `validate_graphics_draw_source` now rejects indexed/non-indexed argument
  mismatches, out-of-bounds indirect byte offsets, unaligned byte offsets, and
  CPU-side indirect draw offsets.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  `CompiledDrawSource::Indirect` preserves the typed argument metadata.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  `Renderer::encode_graphics_pass` selects `draw_indirect` versus
  `draw_indexed_indirect` from the compiled argument kind and bails on mismatched
  runtime state.
- `engine/src/plugins/render/procedural/authoring.rs` and
  `engine/src/plugins/render/procedural/lowering.rs`:
  procedural indirect draw authoring preserves argument kind and buffer bounds.
- `engine/tests/procedural_instance.rs`:
  tests cover indexed indirect compilation, wrong indexed args without an index
  buffer, out-of-bounds byte offsets, and rejected CPU-side indirect offsets.

## Validation

- `cargo fmt --all -- --check` passed.
- `cargo test -p engine render_flow` passed.
- `cargo test -p engine procedural` passed.
- `cargo test -p engine --test procedural_instance` passed: 11 tests.
- `task ai:closeout -- --task "WR-090 Indirect Draw Contract Hardening" --roadmap "docs-site/src/content/docs/workspace/roadmap-items.yaml"`
  produced the required phase drift-check prompt.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- Reusable primitive shader dispatch remains `WR-091`.
- Graph catch-up scheduling remains `WR-092`.
- Procedural camera and view projection remains `WR-101`.
- Evidence, benchmarks, docs, and track closeout remain `WR-093`.
- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Closeout Decision

`WR-090` may be marked completed at `bounded_contract`, and
`PM-RENDER-POP-HARDEN-002` may be completed with the known gaps above.

The next legal hardening slice is `PM-RENDER-POP-HARDEN-003` / `WR-091`,
subject to production planning, promotion gates, and dependency validation.
