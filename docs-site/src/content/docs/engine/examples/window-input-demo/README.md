---
title: "Window Input Demo"
description: "Documentation for Window Input Demo."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-24
---

# Window Input Demo

Small windowed example for the `engine::App` path.

It demonstrates:

- real `winit` window creation through `engine::App::run()`
- plugins on top of `ecs`
- selected default plugins providing `Time`, physical `InputState`, and product `ActionState`
- action-mapped movement with `W`, `A`, `S`, `D` through `ActionState`
- close-on-`Escape` by approving close on the primary `NativeWindowRecord` in `WindowStateRegistryResource`
- dynamic native title intent through the same primary native record

Run it with:

```bash
cargo run -p engine --example window_input_demo
```
