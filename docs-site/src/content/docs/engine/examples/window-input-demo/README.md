---
title: "Window Input Demo"
description: "Documentation for Window Input Demo."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Window Input Demo

Small windowed example for the `engine::App` path.

It demonstrates:

- real `winit` window creation through `engine::App::run()`
- plugins on top of `ecs`
- selected default plugins providing `Time`, physical `InputState`, and product `ActionState`, alongside the current `WindowState` runtime resource
- action-mapped movement with `W`, `A`, `S`, `D` through `ActionState`
- close-on-`Escape` through the same product action projection

Run it with:

```bash
cargo run -p engine --example window_input_demo
```
