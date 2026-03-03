# Window Input Demo

Small typed-runtime windowed example for the new `engine::App` path.

It demonstrates:

- real `winit` window creation through `engine::App::run()`
- typed plugins on top of `ecs_v2`
- default runtime resources: `WindowState`, `Time`, `InputState`
- action-mapped movement with `W`, `A`, `S`, `D`
- close-on-`Escape` using the typed runtime API

Run it with:

```bash
cargo run -p engine --example window_input_demo
```
