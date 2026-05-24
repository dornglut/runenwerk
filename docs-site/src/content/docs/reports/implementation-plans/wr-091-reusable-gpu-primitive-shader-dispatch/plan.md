---
title: WR-091 Reusable GPU Primitive Shader Dispatch Implementation Contract
description: Bounded implementation contract for renderer-owned primitive kernels and hierarchical prefix scan dispatch.
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

# WR-091 Reusable GPU Primitive Shader Dispatch Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-003` / `WR-091` as reusable renderer-owned GPU
primitive dispatch.

The outcome is not another descriptor layer. Primitive execution plans must
lower into normal render-flow compute passes and execute renderer-owned WGSL
kernels for counter reset, u32 prefix scan, scatter/compaction, and indirect
draw argument generation.

## Source Of Truth

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-090-indirect-draw-contract-hardening/plan.md`
- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

## Readiness

This slice depends on completed `WR-090`. Indirect args generation must not be
runtime-proven before indirect draw validation is typed and fail-closed.

## Implementation Scope

Owned files and exact modules/functions:

- `engine/src/plugins/render/gpu_primitives/mod.rs`:
  public primitive dispatch module surface.
- `engine/src/plugins/render/gpu_primitives/plan.rs`:
  primitive execution plan lowering contract.
- `engine/src/plugins/render/gpu_primitives/scan.rs`:
  u32 prefix scan descriptor and hierarchical planning.
- `engine/src/plugins/render/gpu_primitives/counters.rs`:
  counter reset descriptor and dispatch lowering.
- `engine/src/plugins/render/gpu_primitives/compaction.rs`:
  scatter/compaction descriptor and dispatch lowering.
- `engine/src/plugins/render/gpu_primitives/draw_args.rs`:
  indirect draw args generation dispatch.
- `engine/src/plugins/render/api/flow.rs`:
  renderer-facing helper only if primitive plans need first-class render-flow
  authoring.
- `engine/src/plugins/render/graph/pass_graph.rs`:
  compute pass descriptors only if primitive-lowered passes need additional
  stable labels or resource access metadata.
- `engine/src/plugins/render/graph/execution_plan.rs`:
  compiled primitive-lowered compute pass evidence if needed.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs`:
  compute dispatch execution support only through normal render-flow paths.
- `assets/shaders/` or renderer shader asset owner:
  reusable WGSL kernels for counter reset, scan block, scan block sums,
  offset propagation, scatter/compaction, and indirect args generation.
- `engine/benches/render_flow_planning.rs`:
  primitive planning and execution benchmarks.

## Required Decisions

- Prefix scan must support arbitrary total counts through hierarchical block
  scan, block-sum scan, and block-offset propagation.
- Single-workgroup scan is allowed only as one case inside the hierarchical
  path, not as the whole production implementation.
- No boids-only shaders are acceptable for primitive proof.
- Primitive execution must emit stable labels and typed diagnostics.
- Unsupported backend capability must fail closed for runtime proof; hidden CPU
  fallback is not a runtime-proven path.
- Capacity validation must use real `StorageArrayHandle<T>` lengths and typed
  element sizes.

## Acceptance Criteria

- Non-boids tests execute renderer-owned primitive dispatch through the render
  flow runtime.
- Prefix scan tests cover counts smaller than one block, exactly one block,
  multiple blocks, and non-power-of-two counts.
- Scatter/compaction consumes prefix output and writes total-count-sized output
  without silent capacity drift.
- Indirect args generation writes typed args consumed by the WR-090 hardened
  draw contract.
- Diagnostics distinguish unsupported capability, invalid capacity, aliasing,
  and invariant mismatch.

## Non-Goals

- Do not implement fixed-step catch-up scheduling.
- Do not implement spatial hash or chunked unbounded populations.
- Do not change boids behavior except where a minimal canonical primitive proof
  needs a non-boids test fixture.

## Stop Conditions

- Stop if primitive execution can only be proven through descriptor inspection.
- Stop if hierarchical scan cannot be expressed with bounded render-flow
  compute passes.
- Stop if dispatch labels or resource accesses are not inspectable in the
  execution plan.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-091-reusable-gpu-primitive-shader-dispatch/closeout.md`

Completion quality target: `runtime_proven` for primitive dispatch.

Validation:

- `cargo fmt --all -- --check`
- `cargo test -p engine gpu_primitives`
- `cargo test -p engine render_flow`
- `cargo test -p engine procedural`
- `cargo bench -p engine --bench render_flow_planning`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task docs:validate`
- `task planning:validate`

## Critical Review

The tempting shortcut is to make scan work only for boids-sized data or one
workgroup. That does not close the gap from the previous track. The long-term
solution is a reusable hierarchical scan pipeline, even if the initial proof
uses modest element counts. The second shortcut is to call descriptors
"execution"; this contract requires renderer-owned kernels to run.

