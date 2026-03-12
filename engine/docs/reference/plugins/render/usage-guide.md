# Render Plugin Usage Guide

## Purpose

Initializes render registries/resources and wires render schedules.

## Entry Points

- Module: engine/src/plugins/render/plugin.rs
- Entry: RenderPlugin
- Local README: engine/src/plugins/render/README.md

## Minimal Setup

```rust
use engine::plugins::render::RenderPlugin;

app.add_plugin(RenderPlugin);
```

## Runtime Contract

- Schedule placement: RenderPrepare, RenderSubmit
- Ownership: Render runtime resource and schedule wiring.
- Non-ownership: Scene lifecycle ownership.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../../src/plugins/README.md)
