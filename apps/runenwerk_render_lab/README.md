# Runenwerk Render Lab — native performance report (NP1)

The native Render Lab's existing RL2 GPU measurement continues to write schema-v2 JSON.
An **opt-in**, additional schema-v1 performance report reads the **same** bounded
Render-owned frame-history snapshot at the normal measurement close boundary.

## Run

~~~sh
cargo run --release -p runenwerk_render_lab --bin runenwerk-render-lab -- \
  --rl2-measure artifacts/gpu-v2.json \
  --performance-report artifacts/performance-v1.json \
  --submitted-frames 240 \
  --window-size-px 1600x900 \
  --radiance-size-px 1280x720
~~~

The output files must differ. Omitting `--performance-report` preserves the
existing RL2 schema-v2 artifact and default behavior. Omitting
`--submitted-frames` preserves interactive close; an explicit positive limit
must not exceed the Render Lab's retained-history capacity (4096). This is a
snapshot report; it performs no new GPU readbacks or wait-for-GPU drain.

## Interpretation

- A row is keyed by `(frame_index, render_surface_id)` and carries
  `prepare_epoch` and explicit output/internal extents. Retention and
  correlation are supplied by the Render owner, not reconstructed from clocks.
- CPU stages have their **renderer-defined names** and units of milliseconds.
  The three `encode_submit_ms.*` children are nested within their parent,
  so **do not sum them**. CPU pass samples are separate observations.
  These values do not represent whole-application CPU wall time.
- GPU composed timing is one timestamp interval, **not** a sum of GPU pass
  timings. It excludes RunenGPU initial-content setup, terminal queue completion,
  readback tail, Present and physical scanout. Each GPU pass separately records
  its source, capability, state, optional duration and diagnostics.
- `unsupported`, `readback_pending`, `unavailable_this_frame`, and
  `invalid_or_missing_sample` are not zero-length GPU durations. The report
  declares eligible, missing and observed counts for every duration distribution.
- Median is the central sorted eligible value (or mean of middle two); p95 uses
  nearest rank `ceil(0.95 × eligible_count)`. Grouping is by surface ID,
  output extent and internal extent. The report also records all four Render
  history drop counters. Groups do not establish device-generation equality.
- Present submission is **not** display scanout. Actual Host frame wall time,
  display FPS, 1%-lows, queue completion timing, physical adapter/backend,
  actual Present mode and source fixed-step timing are **unknown** unless a
  qualified owner supplies them. A pacing target is a configuration, not FPS.

## Qualification and overhead

The report is generated only when requested, once at close, and can contain up
to 4096 retained frame rows plus existing per-pass evidence. Historical GPU
readbacks may still be pending when the report is written; closure never waits
unboundedly to manufacture completeness.

For controlled performance claims, run the same *release* workload repeatedly
with identical adapter/backend, host pacing, Present mode, extents, warm-up,
submission limit and system load, first with the existing RL2 measurement only,
then with NP1 enabled. Measure application/CPU overhead separately and record
the machine and driver. Do not infer true display FPS from either artifact.
Hosted software Vulkan and ordinary CI tests do not substitute for qualification
on the target physical GPU.
