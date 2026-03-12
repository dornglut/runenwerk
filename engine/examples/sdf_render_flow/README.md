# SDF Render Flow Example

Declarative SDF/raymarch-oriented `RenderFlow` composition using builtin compiled pass kinds only:

- compute field update pass
- fullscreen compose/raymarch pass
- explicit copy pass to `surface.color`
- explicit present pass

This example validates flow composition and prints execution order without custom executors.

Run:

```bash
cargo run -p engine --example sdf_render_flow
```
