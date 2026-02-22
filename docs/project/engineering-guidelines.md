# Project Engineering Guidelines

## Mission
Build Grotto Quest as a modular, testable ECS-driven action RPG with scheduler-driven orchestration and a renderer-integrated UI stack.

## Product Priorities
1. Combat feel and clarity.
2. Replayability through procedural variation and build expression.
3. Party control depth without onboarding overload.
4. Architectural stability and test coverage.

## Ownership Boundaries
- `ecs` crate: data model, archetypes, queries, entity lifecycle.
- `scheduler` crate: ordering, dependency graph validation, node execution orchestration.
- `engine_v2` crate: runtime loop, input, rendering, ECS system wiring, retained UI implementation.
- `game` crate: gameplay systems/content as project scope expands.

## Non-Negotiables
- Deterministic behavior for scheduler and UI stage ordering.
- Renderer-resource logic separated from ECS data representation.
- Favor fallible APIs and typed errors in runtime paths.
- No hidden side effects in builder/setup APIs.

## Testing Expectations
- ECS behavior changes: add or update tests in `ecs/tests`.
- Scheduler behavior changes: add or update tests in `scheduler/tests`.
- `engine_v2` behavior changes: add or update unit tests in `engine_v2/src/**/tests`.
- Before merging substantial changes, run crate-local tests at minimum.

## Delivery Workflow
1. Implement smallest useful slice.
2. Add tests for behavior and edge cases.
3. Run tests and fix regressions.
4. Update docs when behavior or architecture changes.
5. Expand scope only after stabilization.

## Core Reference Docs
- Execution plan: `docs/project/execution-plan.md`
- Roadmap: `docs/project/product-roadmap.md`
- UI architecture: `docs/project/ui-architecture.md`
- ECS design goals: `docs/ecs/design-goals.md`
- Scheduler design goals: `docs/scheduler/design-goals.md`
- Scheduler contributor rules: `docs/scheduler/engineering-guidelines.md`

## UI Priority Directive
UI is currently a high-priority engine track. New UI work must follow retained ECS state + scheduler stages + wgpu submission architecture (see `docs/project/ui-architecture.md`).
