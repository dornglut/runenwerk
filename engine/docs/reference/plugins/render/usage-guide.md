# Render Plugin Usage Guide

## Purpose

Owns render runtime wiring: resources, builtin executor IDs, and render schedule systems.

## Entry Points

- Module: engine/src/plugins/render/plugin.rs
- Entry: RenderPlugin
- Local README: engine/src/plugins/render/README.md

## Happy Path Setup

```rust
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};

app.add_plugins(default_plugins());
app.add_plugin(ScenePlugin);
app.add_plugin(RenderPlugin);
```

## Builtin Executor IDs

`RenderPlugin` registers these builtin executor IDs for graph usage:

- `builtin_compute`
- `builtin_compose`
- `builtin_mesh_overlay`
- `builtin_ui_composite`

Use these IDs directly in feature graph pass `executor` fields when you want builtin behavior.

## Do Not Duplicate Plugin Work

After adding `RenderPlugin`, do not reinitialize these from example/game code:

- `RenderFrameResourceBindings`
- `ShaderRegistryResource`
- `RenderGraphRegistryResource`
- `RenderPassExecutorRegistryResource`
- `PipelineCacheResource`
- `TextureResourceRegistry`
- `BufferResourceRegistry`
- `TransientResourceTracker`
- Debug render resources under `render/debug/*`

Also do not re-add render submit systems. `RenderPlugin` already wires:

- `frame_render_prepare_system` to `RenderPrepare`
- `ui_render_submit_system` to `RenderSubmit`

## Feature Extension Pattern

Feature code should only register feature-owned data:

1. Register feature frame resources into `RenderFrameResourceBindings`.
2. Register feature graph specs into `RenderGraphRegistryResource`.
3. Register only truly custom executors into `RenderPassExecutorRegistryResource`.
4. Prefer builtin executor IDs when behavior already exists.

## Runtime Contract

- Schedule placement: RenderPrepare, RenderSubmit
- Ownership: Render runtime resource and schedule wiring.
- Non-ownership: Scene lifecycle ownership.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../../src/plugins/README.md)
