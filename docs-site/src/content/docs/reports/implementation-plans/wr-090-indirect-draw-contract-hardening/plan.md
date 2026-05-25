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

Current promotion evidence:

- `docs-site/src/content/docs/reports/closeouts/wr-089-renderer-procedural-population-hardening-doctrine-and-track-activation/closeout.md`
  completes the doctrine and track-activation dependency at
  `bounded_contract`.
- `task production:plan -- --milestone "PM-RENDER-POP-HARDEN-002" --roadmap "WR-090"`
  reports `Next action: write_promotion_contract` and promotion preflight
  status `promotable`.

Promotion to `current_candidate` is legal only after this contract remains
decision-complete and roadmap, production, docs, and planning validation pass.

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

## Implementation Steps

1. Promote `WR-090` to `current_candidate` only after this contract and the
   `WR-089` closeout evidence validate.
2. Inspect the existing draw-source API and graph compilation path before
   editing `engine/src/plugins/render/api/passes.rs`,
   `engine/src/plugins/render/graph/pass_graph.rs`,
   `engine/src/plugins/render/graph/execution_plan.rs`, and
   `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`.
3. Split direct, indexed direct, indirect, and indexed indirect draw-source
   semantics into typed graph and execution-plan contracts.
4. Carry typed indirect argument element size and byte length metadata far
   enough for graph validation to reject wrong argument type, missing
   declaration, misaligned byte offset, and out-of-bounds byte offset before
   WGPU submission.
5. Remove or redesign any indirect public API that accepts CPU-side offsets
   that WGPU indirect submission cannot consume.
6. Add focused render-flow/procedural tests for valid and invalid direct,
   indexed direct, indirect, and indexed indirect draw-source paths.
7. Update public docs only if the implementation changes public authoring
   behavior or discoverable renderer APIs.

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

## Perfectionist Closeout Audit

`WR-090` should close as `bounded_contract`. The slice proves fail-closed
indirect draw validation and typed submission semantics, but it does not prove
the complete hardening track. It must not claim `runtime_proven` for the track
or `perfectionist_verified`.

The closeout audit must keep these gaps visible:

- reusable primitive shader dispatch remains `WR-091`;
- graph catch-up scheduling remains `WR-092`;
- procedural camera and view projection remains `WR-101`;
- evidence, benchmarks, docs, and track closeout remain `WR-093`;
- spatial hash, chunked unbounded populations, and richer behavior authoring
  remain separate intake/design work;
- final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Critical Review

The highest-risk shortcut is keeping a generic indirect argument trait that
becomes untyped before validation. That would still allow wrong indexed versus
non-indexed semantics to reach execution. The second shortcut is checking only
4-byte alignment. Runtime proof requires byte-size and element-count bounds.
This slice is a blocker for primitive dispatch because generated indirect args
are not useful unless the renderer can validate and submit them correctly.

