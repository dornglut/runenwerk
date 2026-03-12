# Render Plugin Usage Guide

This page covers plugin/runtime wiring. For `RenderFlow` authoring, use:

- `engine/docs/reference/plugins/render/render-flow-usage-guide.md`
- `engine/docs/reference/plugins/render/gpu-params-guide.md`
- `engine/docs/reference/plugins/render/render-flow-contributions.md`

## Purpose

`RenderPlugin` owns runtime render wiring: resources, builtin executor registrations, and render schedules.

## Setup

```rust
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};

app.add_plugins(default_plugins());
app.add_plugin(ScenePlugin);
app.add_plugin(RenderPlugin);
```

## Runtime Ownership

`RenderPlugin` initializes render runtime resources and systems, including:

- graph/flow registries
- pass executor registry
- pipeline/resource registries
- render prepare/submit systems

Feature modules should register feature-owned graph/executor data only; they should not duplicate plugin bootstrap wiring.
