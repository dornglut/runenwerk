---
title: "Boids Render Flow Example"
description: "Documentation for Boids Render Flow Example."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-07
---

# Boids Render Flow Example

Windowed boids simulation sample built on the public `RenderFlow` API.

What it demonstrates:

- ECS simulation state (`BoidsRenderState`) projected into flow uniforms.
- ping-pong storage simulation (`boids.instances`) on a compute pass.
- graphics draw pass that binds the boid storage as an instance draw buffer through the public `instance_buffer(...)` API.
- flow-owned color target, history copy, and explicit terminal present pass.
- pure builtin compiled runtime path (no custom executor hooks).

Flow chain:

- `boids.simulate` (compute, `assets/shaders/boids_compute.wgsl`)
- `boids.draw` (graphics, `assets/shaders/boids_compose.wgsl`)
- `boids.history` (copy, `boids.color` -> `boids.history`)
- `boids.present` (present, `boids.color` -> surface)

Run:

```bash
cargo run -p engine --example boids_render_flow
```

Tests:

```bash
cargo test -p engine --example boids_render_flow
```
