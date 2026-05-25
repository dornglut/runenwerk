---
title: WR-091 Reusable GPU Primitive Shader Dispatch Closeout
description: Closeout evidence for reusable renderer-owned primitive shader dispatch, hierarchical prefix scan lowering, and primitive runtime smoke proof.
status: completed
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../implementation-plans/wr-091-reusable-gpu-primitive-shader-dispatch/plan.md
  - ../wr-090-indirect-draw-contract-hardening/closeout.md
---

# WR-091 Reusable GPU Primitive Shader Dispatch Closeout

## Result

`WR-091` is completed for reusable renderer-owned primitive dispatch.

Primitive execution plans now lower into normal render-flow compute passes with
stable pass labels, renderer-owned WGSL shader assets, typed storage
dependencies, temporary scan buffers, fixed dispatch dimensions, and
descriptor-derived shader override constants. The implementation covers counter
reset, hierarchical u32 prefix scan, u32 scatter/compaction, and typed indirect
draw argument generation.

## What Changed

- Added `engine/src/plugins/render/gpu_primitives/plan.rs::GpuPrimitiveDispatchPlan`,
  `GpuPrimitiveDispatchStage`, dispatch resources, temporary storage records,
  hierarchical prefix-scan lowering, and dispatch metadata for every primitive
  step.
- Added `engine/src/plugins/render/api/flow.rs::RenderFlow::gpu_primitive_plan`
  to append primitive dispatch plans as ordinary compute pass nodes while
  registering primitive-owned temporary storage.
- Added `engine/src/plugins/render/graph/pass_graph.rs::RenderShaderConstant`
  and compiled constant propagation through
  `engine/src/plugins/render/graph/execution_plan.rs::CompiledComputeExecutionPlan`.
- Updated `engine/src/plugins/render/renderer/render_flow/execute_passes.rs::encode_compute_pass`
  to pass shader override constants to WGPU compute pipeline creation and fold
  constants into the pipeline identity.
- Added reusable WGSL kernels in `assets/shaders/` for counter reset, prefix
  scan, prefix offset propagation, scatter, direct indirect args, and indexed
  indirect args.
- Updated `engine/benches/render_flow_planning.rs` so primitive benchmarks
  include dispatch-plan lowering, temporary resources, flow validation, and
  compile-plan construction.

## Evidence

- `cargo test -p engine gpu_primitives` passed with 13 primitive tests. The
  suite includes WGSL parsing, hierarchical dispatch coverage for one-block and
  multi-block counts, render-flow lowering proof, WR-090 indirect draw
  compatibility, and a local WGPU readback smoke test that wrote and verified
  scan offsets, scatter output, and indirect draw args.
- `cargo test -p engine render_flow` passed with 22 tests, 1 ignored existing
  adapter-dependent GPU timing test.
- `cargo test -p engine procedural` passed with 13 tests across procedural
  population and procedural instance coverage.
- `cargo bench -p engine --bench render_flow_planning` passed. The new
  primitive-dispatch benchmark cases reported:
  - `render_population/prefix_scan_plan_4096`: `[13.213 us 13.536 us 13.873 us]`
  - `render_population/scan_compaction_indirect_args_plan_4096`:
    `[20.509 us 21.517 us 22.543 us]`
- `cargo fmt --all -- --check` passed.
- `git diff --check` passed.

The benchmark run also reported one local Criterion comparison regression for
the unrelated `render_flow/mixed_ui_chain` case at `[679.29 ns 694.39 ns
712.78 ns]`; the benchmark command exited successfully and the changed
primitive cases were newly measured rather than compared against older
baselines.

## Completion Quality

Completion quality: `runtime_proven` for the bounded primitive dispatch slice.

This does not claim the full procedural population hardening track is
runtime-proven. It only proves the reusable primitive dispatch path introduced
by `WR-091`.

Known quality gaps:

- Graph catch-up scheduling remains `WR-092`.
- Procedural camera and view projection remains `WR-101`.
- Evidence, public docs, benchmark reporting, and track closeout remain
  `WR-093`.
- Spatial hash and chunked unbounded populations remain separate intake/design
  work.
- Behavior authoring and richer boids dynamics remain separate deferred intake
  work.
- Final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

## Next Slice

The next legal implementation slice is `WR-092`: fixed-step graph catch-up
scheduling.
