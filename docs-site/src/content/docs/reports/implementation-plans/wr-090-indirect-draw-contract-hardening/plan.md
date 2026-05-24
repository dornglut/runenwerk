---
title: WR-090 Indirect Draw Contract Hardening Implementation Contract
description: Bounded implementation contract for typed, indexed-aware, bounds-checked indirect draw submission.
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

# WR-090 Indirect Draw Contract Hardening Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-002` / `WR-090` as the renderer draw-source
hardening slice.

The outcome is fail-closed indirect drawing: invalid indexed versus non-indexed
argument buffers, missing indirect declarations, byte-offset misalignment, and
out-of-range byte offsets must fail graph validation before WGPU submission.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-089-renderer-procedural-population-hardening-doctrine-and-track-activation/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness

This slice depends on completed `WR-089`. It must not start until
`task production:plan -- --milestone "PM-RENDER-POP-HARDEN-002" --roadmap "WR-090"`
reports either `write_promotion_contract` or `write_implementation_contract`,
and any required promotion has passed.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/api/passes.rs`:
  `GraphicsPassBuilder::draw_indirect`,
  `GraphicsPassBuilder::draw_indexed_indirect`,
  `GraphicsPassBuilder::draw_with_offsets`,
  removal or redesign of misleading indirect CPU offset authoring.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  `RenderDrawSource`, `RenderDrawDescriptor`, `DrawIndirectArgs`,
  `DrawIndexedIndirectArgs`, and any typed indirect argument ABI.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  `CompiledDrawSource` and draw-source compilation.
- `engine/src/plugins/render/graph/validation.rs`:
  `validate_graphics_draw_source` and indirect draw validation issue variants.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  `encode_graphics_pass` submission branch for direct, indexed direct,
  indirect, and indexed indirect draws.
- `engine/src/plugins/render/api/handles.rs`:
  `StorageArrayHandle<T>` metadata only if byte-size or element-size inspection
  is needed to validate indirect argument bounds.
- `engine/src/plugins/render/gpu_primitives/draw_args.rs`:
  typed draw-args compatibility updates only; generation dispatch remains
  `WR-091`.
- `engine/tests/procedural_instance.rs` or focused render-flow tests:
  API and validation coverage for valid and invalid indirect draw sources.

## Required Decisions

- `.draw(...)` and `.draw_with_offsets(...)` remain direct-only APIs.
- Indirect methods must be explicit:
  `draw_indirect(StorageArrayHandle<DrawIndirectArgs>, byte_offset)` and
  `draw_indexed_indirect(StorageArrayHandle<DrawIndexedIndirectArgs>, byte_offset)`.
- Indexed indirect must not accept `DrawIndirectArgs`.
- Non-indexed indirect must not accept `DrawIndexedIndirectArgs`.
- Byte offsets must be 4-byte aligned and within the typed argument buffer byte
  length.
- Validation must know the typed argument element size before submission.
- CPU-side vertex, base-vertex, or instance offsets must not be accepted on
  indirect APIs unless they are stored inside the indirect argument buffer.

## Acceptance Criteria

- Invalid indexed/non-indexed indirect argument combinations fail graph
  validation.
- Out-of-range indirect byte offsets fail graph validation.
- Direct draw authoring remains source-compatible for `.draw(...)`.
- The execution plan carries enough typed draw-source information for
  `execute_passes.rs::encode_graphics_pass` to choose the correct WGPU call
  without inference from sidecar buffers.
- Tests cover direct, indexed direct, indirect, indexed indirect, missing args
  declaration, wrong args type, misaligned offset, and out-of-bounds offset.

## Non-Goals

- Do not implement primitive shader dispatch.
- Do not implement hierarchical prefix scan.
- Do not implement fixed-step catch-up.
- Do not add spatial hash or unbounded population support.

## Stop Conditions

- Stop if WGPU cannot express a requested draw-source contract directly.
- Stop if validation would need runtime-only buffer metadata that the graph does
  not record yet; repair the typed metadata contract first.
- Stop if public API compatibility for existing `.draw(...)` users would break.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-090-indirect-draw-contract-hardening/closeout.md`

Completion quality target: `bounded_contract`.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine render_flow`
- `cargo test -p engine procedural`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task docs:validate`
- `task planning:validate`

## Critical Review

The highest-risk shortcut is keeping a generic indirect argument trait that
becomes untyped before validation. That would still allow wrong indexed versus
non-indexed semantics to reach execution. The second shortcut is checking only
4-byte alignment. Runtime proof requires byte-size and element-count bounds.
This slice is a blocker for primitive dispatch because generated indirect args
are not useful unless the renderer can validate and submit them correctly.

