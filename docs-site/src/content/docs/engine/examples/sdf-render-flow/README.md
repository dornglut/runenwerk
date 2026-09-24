---
title: "3D SDF Render Flow Example"
description: "Documentation for 3D SDF Render Flow Example."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# 3D SDF Render Flow Example

Windowed public `RenderFlow` example that renders a raymarched 3D SDF scene with compute preparation, a fullscreen compose pass, flow-owned history copy, and an explicit terminal present pass.

## Controls

- `Tab`: cycle view mode (`lit` -> `depth` -> `normals` -> `steps`)

## Structure

- `main.rs`
  - entry point
- `rendering/graph.rs`
  - flow declaration (`with_state`, `with_surface_color`, flow-owned color/history targets, compute preparation pass, fullscreen compose pass, copy pass, present pass)
- `rendering/state.rs`
  - ECS-owned render state, compute preparation DTOs, and projected compose uniforms
- `runtime/app.rs`
  - app/plugin wiring and per-frame state advance
- shader:
  - `assets/shaders/sdf_render_flow_3d_compose.wgsl`

Flow chain:

- `sdf.prepare` (compute, public storage/uniform bindings)
- `sdf.compose` (fullscreen, `assets/shaders/sdf_render_flow_3d_compose.wgsl`)
- `sdf.history` (copy, `sdf.color` -> `sdf.history`)
- `sdf.present` (present, `sdf.color` -> surface)

## Run

```bash
cargo run -p engine --example sdf_render_flow
```
