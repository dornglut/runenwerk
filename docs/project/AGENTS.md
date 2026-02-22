# Project AGENTS Guidelines

## Mission
Build Grotto Quest as a modular, testable ECS-driven action RPG with scheduler-driven systems orchestration.

## Product Priorities
- Priority 1: Combat feel and clarity.
- Priority 2: Replayability via procedural dungeons and build expression.
- Priority 3: Party control depth without overwhelming onboarding.
- Priority 4: Stable architecture and tests.

## Architecture Rules
- `ecs` crate owns core data model and queries.
- `game` crate owns gameplay systems and content behavior.
- `scheduler` crate owns execution ordering and dependency orchestration.
- Keep APIs explicit and typed where possible.
- Prefer recoverable errors over panic in runtime paths.

## Testing Rules
- ECS changes must include tests in `ecs/tests`.
- Scheduler changes must include tests in `scheduler/tests`.
- New gameplay loop behavior should have at least one reproducible smoke path.

## Delivery Workflow
- Implement minimal useful behavior.
- Add tests.
- Run crate-local tests and workspace check.
- Refine naming/docs.
- Expand scope only after validation.

## Scoped Technical Docs
- ECS-specific design goals: `ecs/docs/DESIGN_GOALS.md`
- Scheduler-specific design goals: `scheduler/docs/DESIGN_GOALS.md`
- Scheduler-specific contributor rules: `scheduler/docs/AGENTS.md`

## High-Priority UI Directive
- Custom SDF/MSDF UI is a very high-priority engine scope.
- UI must be fully integrated: ECS state + scheduler stages + wgpu renderer.
- Do not build isolated UI runtime outside engine architecture.
- New UI work should follow `docs/project/UI_SDF_SCOPE.md`.
- Active UI target: retained ECS console panel with input field + confirm button, submitted through a single shared submit flow.
