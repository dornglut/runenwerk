# Documentation Index

This folder is the single source of truth for project and crate docs.

## Project
- `docs/project/AGENTS.md` - short redirect for project contributor guidance.
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
- `docs/scheduler/AGENTS.md` - short redirect for scheduler contributor guidance.
- `docs/scheduler/engineering-guidelines.md` - scheduler contributor rules.
- `docs/scheduler/design-goals.md` - scheduler behavior goals and invariants.

## Asset Docs
- `assets/editor/README.md` - local editor/tooling config expectations.
- `assets/models/README.md` - model import pipeline behavior and commands.

## Documentation Conventions
- Documentation files use lowercase kebab-case.
- Long-form docs should include purpose, current state, constraints, and next actions when practical.
- Keep implementation details aligned with actual code state; mark forward-looking sections as proposed/planned.
