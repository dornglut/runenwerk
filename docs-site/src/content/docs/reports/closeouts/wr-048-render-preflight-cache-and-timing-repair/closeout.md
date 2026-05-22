---
title: WR-048 Render Preflight Cache And Timing Repair Closeout
description: Closeout evidence for the bounded prepared-frame preflight cache and render timing repair.
status: completed
owner: engine
layer: engine
canonical: true
last_reviewed: 2026-05-22
related:
  - ../../../workspace/roadmap-items.yaml
  - ../../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../../engine/reference/plugins/render/public-api-reference.md
  - ../../../engine/reference/plugins/render/render-flow-usage-guide.md
  - ../../../engine/benchmarks/render-flow-planning.md
---

# WR-048 Render Preflight Cache And Timing Repair Closeout

## Result

WR-048 is complete as a bounded render contract repair. The renderer keeps
typed prepared-frame preflight as the submit-adjacent correctness authority,
but steady-state unchanged prepared-frame structures can reuse a cached
successful report after cheap runtime guards pass.

The repair adds public timing visibility for preflight and flow encoding,
inspection-visible cache mode/status/source, strict every-frame override for
debugging and audits, and benchmark coverage for cold and cached boids-shaped
preflight paths.

## Evidence

- `engine/src/plugins/render/graph/prepared_validation.rs` defines the typed
  preflight validation mode, cache key, cache state, and cheap runtime guards.
- `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs` owns the
  renderer-runtime successful-report cache and strict-mode behavior.
- `RendererFrameTimings`, `RenderDebugTimingsState`, readiness budgets, and
  preflight inspection expose `preflight_ms`, `flow_encode_ms`, and cache
  source/status without backend handles.
- The render cutoff guard now requires the renderer submit path to call the
  renderer-owned cached preflight entrypoint instead of directly running full
  validation in `render_packet` every frame.
- `engine/benches/render_flow_planning.rs` includes boids-shaped cold and warm
  cached prepared-frame preflight scenarios.

## Validation

- `cargo check -p ui_theme` passed, clearing the unrelated UI compile
  prerequisite that had blocked engine test execution.
- `cargo test -p engine --test render_cutoff_guard` passed.
- `cargo test -p engine render_dynamic_targets` passed.
- `cargo test -p engine render_runtime_inspect` passed.
- `cargo test -p engine --example boids_render_flow` passed.
- `cargo bench -p engine --bench render_flow_planning` completed before this
  closeout. The corrected boids-shaped runtime path measured warm cached
  preflight at roughly `0.92-0.99us` and cold preflight at roughly
  `1.21-1.29us`.
- `task docs:validate`, `task roadmap:render`, `task roadmap:validate`,
  `task roadmap:check`, and `task planning:validate` passed before closeout
  metadata completion.

## Completion Quality

Completion quality: `bounded_contract`.

Known quality gaps:

- The benchmark measures CPU preflight/cache overhead, not a full windowed GPU
  runtime frame-rate proof.
- Runtime stutter still needs a separate governed repair for frame pacing,
  shader reload polling, and diagnostics tiering.
