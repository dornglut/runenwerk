# Render Plugin Architecture

## Primary Architecture Docs

- Target architecture summary:
  - `engine/docs/reference/plugins/render/render-target-architecture.md`
- Internal roadmap/backlog:
  - `engine/src/plugins/render/docs/roadmap.md`

## Current Public Surface

- `RenderFlow` v2 builder surface
- pass builders: `compute_pass`, `fullscreen_pass`, `builtin_ui_composite_pass`
- typed handles: storage arrays, uniforms, ping-pong handles
- GPU params derives: `GpuUniform`, `GpuStorage`, `GpuParams`, `ToGpuValue`
- ECS-first state projection with `Resource`-bound state APIs

## Runtime Boundary

- Owns: render runtime resources, flow registry integration, and render prepare/submit scheduling.
- Non-ownership: app input mapping ownership and scene lifecycle ownership.

## Inspection Boundary

Inspection APIs live under `engine::plugins::render::inspect` and expose graph/resource/texture/timing diagnostics without leaking backend internals into common-path flow authoring.
