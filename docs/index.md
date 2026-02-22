# Documentation Index

This folder is the single source of truth for project and crate docs.

## Project
- `docs/project/engineering-guidelines.md` - engineering standards, ownership, and workflow rules.
- `docs/project/execution-plan.md` - current implementation status and near-term plan.
- `docs/project/backlog.md` - active and planned backlog tracks with acceptance criteria.
- `docs/project/product-roadmap.md` - priority roadmap across foundation, MVP, and polish.
- `docs/project/scene-architecture.md` - ECS-first scene stack model, transitions, and overlay layering strategy.
- `docs/project/render-graph-architecture.md` - frame graph and pipeline registry model for mixed compute/render rendering.
- `docs/project/profiling-and-tracing.md` - runtime performance profiling setup, hot-path logs, and Tracy workflow.
- `docs/project/ui-architecture.md` - retained ECS SDF/MSDF UI architecture and implementation scope.
- `docs/project/game-design.md` - high-level game design and loop direction.
- `docs/project/gameplay-scene-mvp.md` - concrete ECS gameplay scene vertical-slice spec and definition of done.

## ECS
- `docs/ecs/design-goals.md` - ECS goals, invariants, performance targets, and testing expectations.

## Scheduler
- `docs/scheduler/engineering-guidelines.md` - scheduler contributor rules.
- `docs/scheduler/design-goals.md` - scheduler behavior goals and invariants.

## Naming Conventions
- Documentation files use lowercase kebab-case.
- Each doc should include: purpose, current state, constraints, and next actions.
- Keep implementation details aligned with actual code state; avoid speculative architecture text unless marked as proposed.
