# Profiling And Tracing

## Purpose
Define a repeatable profiling workflow for `engine_v2` with two layers:
- structured hot-path logs for quick diagnosis
- Tracy timeline capture for deep frame analysis

## Current State
- Frame timing breakdown logs are emitted from `engine_v2/src/systems/render.rs`.
- Mesh hot-path breakdown logs are emitted when mesh prepare is slow and include:
  - sub-step timings (`collect`, `merge`, `upload`, `camera update`)
  - workload counters (mesh counts, vertex/index counts, upload bytes)
- Hot-path spans are emitted from `engine_v2/src/render/renderer.rs` and `engine_v2/src/systems/render.rs`.
- Tracy integration is feature-gated in `engine_v2` and enabled at runtime with an env var.

## Tracy Setup
1. Build/run with Tracy support:
   - `cargo run -p engine_v2 --features tracy`
2. Enable Tracy layer at runtime:
   - `ENGINE_V2_TRACY=1 cargo run -p engine_v2 --features tracy`
3. Open Tracy Profiler and connect to the running process.

If `ENGINE_V2_TRACY` is unset (or `0`), standard logging remains active and Tracy layer is disabled.

## Log Signals
- Slow frame log: `"frame render timing breakdown"`
- Slow mesh prep log: `"mesh prepare hot path breakdown"`

These logs should be used first to find the expensive stage before capturing a Tracy timeline.

## Constraints
- Logging and spans must remain lightweight enough for development builds.
- Tracy stays optional and must not break normal builds without the `tracy` feature.
- Profiling fields should remain stable so comparisons across commits are meaningful.

## Next Actions
- Add rolling percentile summaries (`p50/p95/p99`) for mesh hot-path metrics.
- Add GPU timestamp query instrumentation for world compute, mesh pass, and UI pass.
- Add a command to toggle high-detail performance logging at runtime.
