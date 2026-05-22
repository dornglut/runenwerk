---
title: WR-056 Renderer GPU Pass Timing And Workload Evidence Closeout
description: Closeout evidence for capability-gated renderer GPU pass timing and workload inspection.
status: completed
owner: engine
layer: engine-runtime / render diagnostics
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-archive.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-056 Renderer GPU Pass Timing And Workload Evidence Closeout

## Outcome

`WR-056` is complete. Renderer inspection now separates CPU encode/submit pass
samples from GPU timestamp-query pass evidence, gates timestamp support through
WGPU backend capabilities, reports unsupported/unavailable states explicitly,
and feeds GPU timing evidence into readiness budgets without exposing backend
handles or product-policy decisions.

## Implementation Evidence

Changed modules:

- `engine/src/plugins/render/backend/device.rs`:
  `RenderBackendTimingCapabilities` and timestamp-query feature request when
  the adapter supports `wgpu::Features::TIMESTAMP_QUERY`.
- `engine/src/plugins/render/backend/wgpu_ctx.rs`: backend timing capability
  storage on `WgpuCtx`.
- `engine/src/plugins/render/inspect/timings.rs`: CPU/GPU timing source DTOs,
  GPU timing capability and diagnostic DTOs, GPU pass timing evidence, GPU
  timing summaries, and `RenderDebugTimingsState` GPU fields.
- `engine/src/plugins/render/inspect/budgets.rs`: GPU pass total and GPU timing
  diagnostic budget measurements.
- `engine/src/plugins/render/inspect/readiness.rs`: readiness diagnostics and
  source summaries for GPU timing diagnostics.
- `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs`: renderer
  runtime query-set ownership, timestamp resolve/readback, and measured GPU
  millisecond projection.
- `engine/src/plugins/render/renderer/render_flow/execute.rs`: per-frame GPU
  timing setup, pass reservation, resolve before submit, readback after submit,
  and explicit diagnostics for pass kinds without timestamp writes.
- `engine/src/plugins/render/renderer/render_flow/execute_passes.rs` and
  `engine/src/plugins/render/renderer/setup.rs`: timestamp writes on compute,
  fullscreen, graphics, and builtin UI render passes.
- `engine/src/plugins/render/runtime/frame_submit.rs`: observed GPU pass timing
  evidence into `RenderDebugTimingsState`.
- `engine/tests/render_runtime_inspect.rs` and
  `engine/tests/render_gpu_timing.rs`: deterministic inspection, readiness, and
  budget coverage.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`
  and `docs-site/src/content/docs/engine/reference/plugins/render/usage-guide.md`:
  public inspection documentation.

## Runtime Evidence

Local runtime command:

```text
cargo test -p engine render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported -- --ignored --nocapture
```

Observed local output:

```text
runtime GPU timing evidence: [RenderPassTimingEvidence { frame_index: Some(1), render_surface_id: Some(1), flow_id: "runtime.gpu", pass_id: "timestamp.empty_compute", pass_kind: "compute", source: GpuTimestampQuery, gpu_capability: Supported, millis: Some(0.0), diagnostics: [] }]
```

The local adapter supports timestamp queries. The runtime evidence test uses a
real WGPU adapter/device, pass timestamp writes, query resolve, readback mapping,
and the renderer-owned GPU timing projection path.

## Validation

Focused validation:

```text
cargo test -p engine render_runtime_inspect
cargo test -p engine render_gpu_timing
cargo test -p engine render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported -- --ignored --nocapture
task docs:validate
task planning:validate
```

Workflow validation after metadata updates:

```text
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
```

## Completion Quality

Completion quality: `runtime_proven`.

Known quality gaps:

- Copy and present flow steps are command-encoder copy operations rather than
  WGPU compute/render passes. They report explicit unavailable GPU timing
  diagnostics unless a later accepted design enables encoder-level timestamp
  writes.
- The local runtime evidence proves the timestamp query path on a minimal
  compute pass. Long-duration scene smoothness and boids production evidence
  remain in later GPU/procedural milestones.

These gaps remain visible because they are outside `WR-056`'s bounded timing
foundation and are carried by later renderer GPU/procedural roadmap rows.
