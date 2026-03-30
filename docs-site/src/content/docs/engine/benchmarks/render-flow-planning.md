---
title: "Render Flow Planning Benchmarks"
description: "Documentation for Render Flow Planning Benchmarks."
---

# Render Flow Planning Benchmarks

## Scope

Bench target: `engine/benches/render_flow_planning.rs`

Measured scenarios:

- `render_flow/simple_fullscreen`
- `render_flow/boids_ping_pong`
- `render_flow/multi_pass_compute_compose`
- `render_flow/sdf_compute_compose`
- `render_flow/mixed_ui_chain`

## What is measured

For each scenario, the benchmark runs:

1. flow validation report generation (`RenderFlow::validation_report`)
2. transient window discovery
3. alias candidate discovery
4. transient alias slot assignment planning

## Run

```bash
cargo bench -p engine --bench render_flow_planning
```

## Notes

- Treat this benchmark as planning/validation overhead, not GPU runtime throughput.
- Store raw benchmark output under criterion artifacts; keep this file human-readable.
