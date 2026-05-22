---
title: WR-055 Runtime Frame Pacing, Shader Reload Throttle, And Diagnostics Tiers Closeout
description: Closeout evidence for the runtime pacing and steady-state render diagnostics performance repair.
status: completed
owner: engine
layer: engine
canonical: true
last_reviewed: 2026-05-22
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../engine/reference/usage-guide.md
  - ../../../engine/reference/plugins/render/usage-guide.md
  - ../../../engine/reference/plugins/render/public-api-reference.md
  - ../../../engine/reference/plugins/render/render-flow-usage-guide.md
  - ../../../engine/reference/plugins/debug-metrics/usage-guide.md
  - ../../../engine/benchmarks/render-flow-planning.md
---

# WR-055 Runtime Frame Pacing, Shader Reload Throttle, And Diagnostics Tiers Closeout

## Result

WR-055 is complete as a bounded runtime/render performance contract repair.
Windowed runtime pacing is now policy-driven instead of unbounded redraw-driven,
shader live reload remains available but no longer polls every frame in steady
state, and render diagnostics keep cheap timing/cache/pacing evidence every
frame while avoiding full debug report construction unless a diagnostic trigger
requires it.

The repair does not lower boid count, weaken prepared-frame preflight, remove
diagnostics capability, change boids shader behavior, or move product truth,
freshness, authority, fallback legality, rebuild policy, or residency policy
into renderer runtime code.

## Evidence

- `engine/src/runtime/frame_pacing.rs` defines `FramePacingPolicyResource`,
  `FramePacingMode::ContinuousCapped { target_fps: 60 }`, on-demand mode, and
  deterministic next-frame deadline decisions.
- `engine/src/runtime/winit_runner.rs` uses `ControlFlow::WaitUntil` for capped
  continuous pacing and `ControlFlow::Wait` for on-demand mode. It no longer
  calls `request_redraw()` unconditionally from steady-state window effects.
- `engine/src/app/domain/app.rs` exposes `App::with_frame_pacing(...)` for apps
  that need an explicit runtime pacing policy.
- `engine/src/plugins/render/shader/registry.rs` makes the first shader poll
  immediate, throttles normal watch polling to 500 ms by default, and lets
  explicit reload requests bypass the throttle.
- `engine/src/plugins/render/inspect/config.rs` adds
  `RenderFrameDiagnosticsPolicyResource` with tiered diagnostics by default.
- `engine/src/plugins/render/runtime/frame_submit.rs` skips full
  `RenderDebugFrameReport` construction in healthy steady state and forces it
  for capture, provenance, readback, probes, diffs, export, slow frames,
  explicit request, or full-every-frame mode.
- `RenderDebugTimingsState` and the debug metrics overlay expose preflight,
  flow encode, shader poll, diagnostics report, frame pacing, and preflight
  cache source/status evidence without backend handles.
- `engine/examples/boids_render_flow/runtime/app.rs` installs
  `DebugMetricsPlugin`, so the boids runtime can show the F10 overlay fields
  while retaining the public render-flow example path.

## Validation

- `cargo test -p engine frame_pacing` passed.
- `cargo test -p engine shader` passed.
- `cargo test -p engine render_runtime_inspect` passed.
- `cargo test -p engine --test render_cutoff_guard` passed.
- `cargo test -p engine --example boids_render_flow` passed.
- `cargo bench -p engine --bench render_flow_planning` completed. The
  boids-shaped preflight scenarios measured about `1.04-1.06us` cold and
  `0.81-0.83us` cached in this run.
- `task docs:validate` passed.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed.
- `task planning:validate` passed.
- Windowed smoke: `target/debug/examples/boids_render_flow.exe` was launched
  after `cargo build -p engine --example boids_render_flow`, stayed alive for
  an 8 second bounded run, then was stopped by the smoke script.

## Completion Quality

Completion quality: `runtime_proven`.

Known quality gaps:

- The smoke run proves the windowed path starts and remains alive under the new
  pacing/diagnostics contracts, but it is not a long-duration subjective human
  smoothness certification.
- The F10 debug overlay is available in the boids example for local inspection;
  no screenshot artifact was captured in this closeout.
