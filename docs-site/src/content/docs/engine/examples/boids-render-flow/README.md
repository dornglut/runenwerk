---
title: "Boids Render Flow Example"
description: "Documentation for Boids Render Flow Example."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Boids Render Flow Example

Windowed boids simulation sample built on the public `RenderFlow` API.

What it demonstrates:

- ECS simulation state (`BoidsRenderState`) projected into flow uniforms.
- ping-pong storage simulation (`boids.instances`) on a compute pass.
- fullscreen compose pass that draws boids directly from storage.
- pure builtin compiled runtime path (no custom executor hooks).

Flow chain:

- `boids.simulate` (compute, `assets/shaders/boids_compute.wgsl`)
- `boids.compose` (fullscreen, `assets/shaders/boids_compose.wgsl`)

Run:

```bash
cargo run -p engine --example boids_render_flow
```

Tests:

```bash
cargo test -p engine --example boids_render_flow
```
