---
title: Engine Render Macros
description: Documentation for the engine_render_macros crate and its GPU parameter derive macros.
status: active
owner: engine
layer: engine/runtime
canonical: true
last_reviewed: 2026-04-28
---

# Engine Render Macros

`engine_render_macros` provides derive macros for renderer-facing GPU parameter contracts.

The crate is a proc-macro crate used by the engine render layer. Its public surface is intentionally small:

- `#[derive(GpuUniform)]`
- `#[derive(GpuStorage)]`

These derives support the normal render-parameter path exposed through the engine render module.

## Ownership

`engine_render_macros` belongs to the engine/runtime layer.

It may generate glue for engine render contracts, but it must not own:

- render graph execution;
- GPU backend policy;
- editor viewport behavior;
- domain UI surface semantics;
- domain invariants.

## Current Derives

### `GpuUniform`

`#[derive(GpuUniform)]` derives uniform-buffer parameter support for a non-generic struct.

The macro generates a raw GPU representation named:

```text
<TypeName>GpuRaw
```

The generated raw type is intended to satisfy the render parameter contract through `GpuParams::Raw`.

### `GpuStorage`

`#[derive(GpuStorage)]` derives storage-buffer parameter support for a non-generic struct.

It follows the same generated raw-type convention:

```text
<TypeName>GpuRaw
```

## Current Constraints

The derives currently reject generic type parameters.

Allowed:

```rust
#[derive(GpuUniform)]
pub struct CameraParams {
    pub view_projection: [[f32; 4]; 4],
}
```

Not currently supported:

```rust
#[derive(GpuUniform)]
pub struct Params<T> {
    pub value: T,
}
```

## Integration Path

The render module re-exports the derives through the normal engine render API.

The generated implementations target these render parameter traits:

- `GpuParams`
- `GpuUniform`
- `GpuStorage`

## Non-scope

This crate does not own runtime resource allocation, bind group creation, render graph execution, shader compilation, or backend-specific rendering behavior.

## Validation

Run:

```text
cargo test -p engine_render_macros
cargo check --workspace
```
