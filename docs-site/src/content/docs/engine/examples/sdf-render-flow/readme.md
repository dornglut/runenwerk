---
title: "3D SDF Render Flow Example"
description: "Documentation for 3D SDF Render Flow Example."
---

# 3D SDF Render Flow Example

Windowed public `RenderFlow` example that renders a raymarched 3D SDF scene with a single fullscreen pass.

## Controls

- `Tab`: cycle view mode (`lit` -> `depth` -> `normals` -> `steps`)

## Structure

- `main.rs`
  - entry point
- `rendering/graph.rs`
  - flow declaration (`with_state`, `with_surface_color`, fullscreen compose pass)
- `rendering/state.rs`
  - ECS-owned render state and projected uniforms
- `runtime/app.rs`
  - app/plugin wiring and per-frame state advance
- shader:
  - `assets/shaders/sdf_render_flow_3d_compose.wgsl`

## Run

```bash
cargo run -p engine --example sdf_render_flow
```
