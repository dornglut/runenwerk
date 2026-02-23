# Project Engineering Guidelines

## Product Priorities
4. Architectural stability and test coverage.

## Ownership Boundaries
- `ecs` crate: data model, archetypes, queries, entity lifecycle.
- `scheduler` crate: ordering, dependency graph validation, node execution orchestration.
- `engine` crate: runtime loop, plugin composition, rendering/UI/scene/time/input plugin implementations.
- `game` crate: gameplay systems/content as project scope expands.

## Non-Negotiables

## Testing Expectations
- add and run tests for the crate or plugin

## Delivery Workflow
1. Implement smallest useful slice.
2. Add tests for behavior and edge cases.
3. Run tests and fix regressions.
4. Update docs when behavior or architecture changes.
5. Expand scope only after stabilization.