# Render Plugin Architecture

## Primary Architecture Docs

- Target architecture:
  - `engine/src/plugins/render/docs/target-architecture.md`
- Implementation roadmap:
  - `engine/src/plugins/render/docs/target-architecture-roadmap.md`

## Current Public Surface

- `RenderFlow`
- `RenderFlowContribution`
- `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuParams`
- ECS-first uniform projection (`ecs_resource`, `uniform_state`, `uniform_state_with_surface`)
- pass/resource-oriented flow authoring with namespaced IDs

## Runtime Boundary

- Owns: render runtime resources, graph/flow integration, and render prepare/submit scheduling.
- Non-ownership: app input mapping ownership and scene lifecycle ownership.

## Inspection Boundary

- inspection APIs live under `engine::plugins::render::inspect`
- these tools expose graph/resource/texture/timing diagnostics without leaking low-level backend internals into normal flow authoring
