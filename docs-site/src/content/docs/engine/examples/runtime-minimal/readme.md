---
title: "Runtime Minimal"
description: "Documentation for Runtime Minimal."
---

# Runtime Minimal

Headless proof example for the engine runtime.

It demonstrates:

- `engine::App`
- `engine::App::headless()` / `run_for_frames(...)`
- `engine::Plugin`
- `Startup` and `Update` schedules
- `Query`, `ResMut`, and `Commands`
- `ecs`-backed world state with typed resources and queries

Run it with:

```bash
cargo run -p engine --example runtime_minimal
```
