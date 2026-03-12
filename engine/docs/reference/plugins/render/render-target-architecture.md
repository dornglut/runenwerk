# Render Target Architecture

This reference captures the current target direction for render authoring in this repository.

## Public Direction

- `RenderFlow`
- `RenderFlowContribution`
- `GpuUniform`
- `GpuStorage`
- `ToGpuValue`
- `GpuParams`
- ECS-first state projection (`ecs_resource`, `uniform_state`, `uniform_state_with_surface`)
- pass/resource-oriented render authoring with namespaced IDs

## Ownership Rules

- Input bindings stay in the app/input layer (`App::add_input_bindings`), not in render-flow authoring.
- Low-level graph executor/registry plumbing is internal/advanced and not the primary user API.
- Macro scope stays narrow to GPU layout/conversion (`GpuUniform`, `GpuStorage`), not broad state DSL extraction.

## Canonical Specs

- Target architecture spec:
  - `engine/src/plugins/render/docs/target-architecture.md`
- Implementation roadmap:
  - `engine/src/plugins/render/docs/target-architecture-roadmap.md`
