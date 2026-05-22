---
title: "Render Flow Planning Benchmarks"
description: "Documentation for Render Flow Planning Benchmarks."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-22
---

# Render Flow Planning Benchmarks

## Scope

Bench target: `engine/benches/render_flow_planning.rs`

Measured scenarios:

- `render_flow/simple_fullscreen`
- `render_flow/boids_ping_pong`
- `render_flow/boids_preflight_cold`
- `render_flow/boids_preflight_cached`
- `render_flow/multi_pass_compute_compose`
- `render_flow/sdf_compute_compose`
- `render_flow/mixed_ui_chain`
- `render_scale/production_evidence_report_4096`
- `render_sdf/runtime_evidence_report_4096`

## What is measured

For each scenario, the benchmark runs:

1. flow validation report generation (`RenderFlow::validation_report`)
2. transient window discovery
3. alias candidate discovery
4. transient alias slot assignment planning

The boids preflight scenarios separately measure prepared-frame preflight cost:

- `boids_preflight_cold` runs full fail-fast prepared-frame validation.
- `boids_preflight_cached` runs the steady-state cache key plus runtime guard path and reuses the cached successful report.

The production evidence scenarios measure renderer-owned report aggregation:

- `render_scale/production_evidence_report_4096` covers scale residency,
  visibility, timing, hardware-profile, benchmark-command, and artifact-path
  evidence.
- `render_sdf/runtime_evidence_report_4096` covers SDF residency, raymarch
  candidate, visual proof, timing, benchmark-command, and artifact-path
  evidence.

## Run

```bash
cargo bench -p engine --bench render_flow_planning
```

## Notes

- Treat this benchmark as CPU planning/preflight overhead, not GPU runtime throughput.
- Store raw benchmark output under criterion artifacts; keep this file human-readable.
