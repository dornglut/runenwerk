# Boids Render Flow Example

Declarative boids-oriented `RenderFlow` composition example that uses the builtin compiled path only:

- compute simulation pass
- graphics draw pass
- explicit copy pass to `surface.color`
- explicit present pass

This example validates and prints execution order. It intentionally avoids custom executors and low-level registry mutation.

Run:

```bash
cargo run -p engine --example boids_render_flow
```
