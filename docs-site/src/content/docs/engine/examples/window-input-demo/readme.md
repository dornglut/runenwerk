---
title: "Window Input Demo"
description: "Documentation for Window Input Demo."
---

# Window Input Demo

Small windowed example for the `engine::App` path.

It demonstrates:

- real `winit` window creation through `engine::App::run()`
- plugins on top of `ecs`
- default runtime resources: `WindowState`, `Time`, `InputState`
- action-mapped movement with `W`, `A`, `S`, `D`
- close-on-`Escape` using the runtime API

Run it with:

```bash
cargo run -p engine --example window_input_demo
```
