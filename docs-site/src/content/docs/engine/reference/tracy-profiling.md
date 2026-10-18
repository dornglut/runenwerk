---
title: Tracy CPU Profiling
description: Opt-in CPU tracing in native Runenwerk applications without changing GPU execution ownership.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-10-18
publication: primary
---

# Tracy CPU Profiling

Runenwerk uses the optional `tracing-tracy` subscriber layer to send **CPU**
spans to the external [Tracy profiler](https://github.com/wolfpld/tracy).
Tracy does not own the native frame scheduler, Render's frame history, RunenGPU
submissions, GPU shader timings, or on-screen presentation measurements.

## Enable in a native application

Compile with Engine's opt-in feature and keep the debug symbols for source-level
inspection. The Render Lab binary is the first native application using the
`engine::utils::setup_tracing_if_requested()` bootstrap:

```powershell
$env:CARGO_PROFILE_RELEASE_DEBUG = "2"
$env:ENGINE_TRACY = "1"
$env:RUST_LOG = "info,runenwerk_profile=trace"
cargo +stable run --release -p runenwerk_render_lab --features engine/tracy --locked -- --rl2
```

Open the separate Tracy profiler UI and connect to the running application.
Capture during representative, warmed-up interaction. The
`render.g3.graph_build` span encloses Runenwerk's canonical GPU-graph authoring
and preparation. `render.g3.provisional`, `render.g3.final` and
`render.g3.timing_final` mark the actual G3 graph preparations taken on that
path; not every frame requires every preparation. The `occurrences` field
counts resolved work nodes, not GPU shader executions.

The Engine must initialize exactly one process-global `tracing` subscriber.
Application entry points may call `setup_tracing_if_requested()` once, before
starting the runtime, and retain its returned guard for the application lifetime.
This helper performs **no initialization** when the runtime switch is absent.
`ENGINE_TRACY=1` without the `tracy` Cargo feature emits a diagnostic rather
than pretending to record a capture. `setup_tracing()` remains available to
applications such as Runenwerk Draw that intentionally always configure stdout
and file logging. Existing `RUST_LOG` filters still apply to Tracy spans.

To disable profiling in the current shell:

```powershell
Remove-Item Env:ENGINE_TRACY, Env:RUST_LOG, Env:CARGO_PROFILE_RELEASE_DEBUG -ErrorAction SilentlyContinue
```

## Correct interpretation and overhead

Tracy CPU spans reflect host process time, including possible blocking; they
**do not** measure GPU execution, terminal GPU completion or actual display
scanout. Continue to use Render's correlated per-frame/surface observations,
RunenGPU timestamp/completion evidence and platform GPU profiling tools for
those distinct questions. Native host opportunity counts and submitted Present
counts must not be interpreted as identical.

For overhead assessment, compare unchanged source, adapter, backend, extent,
shader state, fixed-step work and warm-up across repeated steady-state captures:

1. `--release` without the `engine/tracy` feature.
2. `--release --features engine/tracy` with `ENGINE_TRACY` unset.
3. The same feature build with `ENGINE_TRACY=1`, the same trace filter, and a
   connected Tracy UI.

Record each configuration and sample window, CPU stage timings, Present
submission interval and simulation continuity independently. Sampling/capture
overhead and run-to-run variance must be reported; no frame-rate improvement
is assumed from enabling Tracy. The initial native Boids capture is tracked by
[Runenwerk #1298](https://github.com/dornglut/runenwerk/issues/1298) and
requires coordination with the active [#1291](https://github.com/dornglut/runenwerk/issues/1291)
writer before changing the Boids example's entry point.
