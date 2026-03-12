# Render Plugin Architecture

## Primary Architecture Docs

- Hard cutover roadmap (canonical source of truth):
  - `engine/src/plugins/render/docs/roadmap.md`
- Reference summary of the cutover direction:
  - `engine/docs/reference/plugins/render/render-target-architecture.md`

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
