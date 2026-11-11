---
title: "Runtime Minimal"
description: "Documentation for Runtime Minimal."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-11-11
publication: primary
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

## Maintained Rust source

This code is read directly from
`engine/examples/runtime_minimal/main.rs` when Astro builds the site.
There is no separately maintained copy of the example in the documentation.

```rust runenwerk-source=engine/examples/runtime_minimal/main.rs
```
