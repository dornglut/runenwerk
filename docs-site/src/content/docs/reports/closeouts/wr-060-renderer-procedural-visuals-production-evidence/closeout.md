---
title: WR-060 Renderer Procedural Visuals Production Evidence Closeout
description: Runtime, benchmark, documentation, and governance evidence for the renderer procedural visuals production-readiness milestone.
status: completed
owner: engine
layer: engine-runtime / renderer evidence
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/renderer-gpu-evidence-and-procedural-visuals-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-060 Renderer Procedural Visuals Production Evidence Closeout

## Outcome

`WR-060` is complete at `runtime_proven` quality. The completed GPU timing,
pass-shape guard, procedural API, and canonical boids rows are now tied together
by a boids production-evidence command, a measured runtime GPU timestamp-query
probe, procedural-boids planning benchmarks, public docs, and this closeout.

The row does not claim `perfectionist_verified`. The final no-gap audit remains
owned by `PT-RENDER-PERFECTION`, and the remaining gaps below stay visible.

## Implementation Evidence

Changed modules:

- `engine/examples/boids_render_flow/rendering/evidence.rs::production_evidence_report`:
  builds the canonical boids evidence report from `build_render_flow`,
  `compile_flow_plan_checked`, compiled flow inspection, typed
  `RenderPassTimingEvidence`, and prepared-frame preflight timing.
- `engine/examples/boids_render_flow/rendering/evidence.rs::measure_cpu_timing_evidence`:
  records a measured `preflight_ms` sample for the canonical boids prepared
  frame and marks submit/present CPU timing unavailable for the non-windowed
  evidence command.
- `engine/examples/boids_render_flow/rendering/evidence.rs::prepared_frame_for_flow`:
  builds a renderer-owned prepared-frame evidence packet without backend
  handles or product truth.
- `engine/examples/boids_render_flow/main.rs::main`: adds
  `--evidence` mode for finite command-line production evidence.
- `engine/benches/render_flow_planning.rs::build_procedural_boids_flow`:
  adds the procedural-boids production shape to the canonical render-flow
  planning benchmark.
- `engine/benches/render_flow_planning.rs::bench_render_flow_planning`:
  adds `render_flow/procedural_boids_production_shape` and
  `render_flow/procedural_boids_preflight_cold`.
- `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs::render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported`:
  records adapter backend/name in the ignored runtime proof and uses a stable
  headless Vulkan backend by default on Windows while preserving `WGPU_BACKEND`
  override for backend-specific investigation.
- `docs-site/src/content/docs/engine/reference/plugins/render/render-flow-usage-guide.md`:
  documents procedural visual evidence, canonical boids evidence mode, and the
  planning benchmark command.
- `docs-site/src/content/docs/engine/reference/plugins/render/public-api-reference.md`:
  documents GPU timing evidence DTOs, pass-shape guard diagnostics,
  procedural APIs, and the boids production-evidence command.

## Runtime Evidence

Boids production evidence command:

```text
cargo run -p engine --example boids_render_flow -- --evidence
```

Observed evidence:

```text
boids_production_evidence flow=boids_render_flow scene=1600x900 boids=384 passes=4
benchmark_command=cargo bench -p engine --bench render_flow_planning
cpu_timing_fields=preflight_ms,flow_encode_ms,encode_submit_ms,present_ms
pass label=boids.simulate kind=compute order=0 gpu_timestamp_expected=true dispatch_available=true local_instance_geometry=false vertex_count=n/a instance_count=n/a
pass label=boids.publish_instances kind=compute order=1 gpu_timestamp_expected=true dispatch_available=true local_instance_geometry=false vertex_count=n/a instance_count=n/a
pass label=boids.draw kind=graphics order=2 gpu_timestamp_expected=true dispatch_available=false local_instance_geometry=true vertex_count=6 instance_count=384
pass label=boids.present kind=present order=3 gpu_timestamp_expected=false dispatch_available=false local_instance_geometry=false vertex_count=n/a instance_count=n/a
gpu_timing flow=boids_render_flow pass=boids.simulate kind=compute capability=unsupported diagnostics=1
gpu_timing flow=boids_render_flow pass=boids.publish_instances kind=compute capability=unsupported diagnostics=1
gpu_timing flow=boids_render_flow pass=boids.draw kind=graphics capability=unsupported diagnostics=1
cpu_timing source=prepared_frame_preflight preflight_ms=0.0538 flow_encode_ms=unavailable encode_submit_ms=unavailable present_ms=unavailable unavailable_reason=windowed_submit_not_run_by_evidence_command
```

Runtime GPU timing command:

```text
cargo test -p engine render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported -- --ignored --nocapture
```

Observed local capability evidence:

```text
runtime GPU timing evidence: backend=Vulkan adapter=AMD Radeon RX 7900 XTX
runtime GPU timing evidence: [RenderPassTimingEvidence { frame_index: Some(1), render_surface_id: Some(1), flow_id: "runtime.gpu", pass_id: "timestamp.empty_compute", pass_kind: "compute", source: GpuTimestampQuery, gpu_capability: Supported, millis: Some(0.0), diagnostics: [] }]
test plugins::render::renderer::render_flow::gpu_timing::tests::render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported ... ok
```

The boids command uses typed unsupported GPU timing diagnostics because it does
not create a backend device. The ignored runtime probe proves the timestamp
query readback path on the local Vulkan adapter.

## Benchmark Evidence

Benchmark command:

```text
cargo bench -p engine --bench render_flow_planning
```

Raw Criterion artifacts:

```text
target/criterion/render_flow_procedural_boids_production_shape
target/criterion/render_flow_procedural_boids_preflight_cold
target/criterion/report
```

Observed procedural-boids benchmark summary:

```text
render_flow/procedural_boids_production_shape
                        time:   [971.55 ns 979.58 ns 989.15 ns]

render_flow/procedural_boids_preflight_cold
                        time:   [1.7905 us 1.8191 us 1.8547 us]
```

Criterion also reported relative regressions against older local baselines in
several pre-existing benchmark cases. The command exited successfully and the
absolute medians remain sub-2 us for the focused procedural-boids cases, but the
regression notices are retained as known quality gaps because this row does not
define automated pass/fail performance thresholds.

## Validation

Focused validation:

```text
cargo fmt
cargo test -p engine --example boids_render_flow
cargo test -p engine render_runtime_inspect
cargo test -p engine render_flow
cargo test -p engine render_gpu_timing
cargo test -p engine render_gpu_timing_runtime_query_readback_reports_measured_or_unsupported -- --ignored --nocapture
cargo bench -p engine --bench render_flow_planning
```

The boids example command executed twelve tests covering pass declaration,
history-copy removal, procedural local instance geometry, pass ordering, uniform
projection, shader-loop removal, state publish parameters, dispatch coverage,
and stable production evidence formatting.

Workflow validation after metadata updates:

```text
task docs:validate
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

- The non-windowed boids evidence command measures prepared-frame preflight and
  records pass shape, but it does not run a finite swapchain submit/present frame
  for boids.
- Criterion reported relative regressions against older local baselines in
  several pre-existing render-flow planning cases. WR-060 records the evidence,
  but automated benchmark thresholds remain future hardening work.
- `perfectionist_verified` remains blocked until `PT-RENDER-PERFECTION`
  performs the final no-gap renderer audit.

These gaps do not invalidate `runtime_proven`; they define the remaining audit
surface for the renderer perfection track.
