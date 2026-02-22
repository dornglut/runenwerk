# Scheduler Design Goals

## Purpose
Define the scheduler crate's role: deterministic, validated system orchestration for game/runtime pipelines.

## Core Goals
- Deterministic execution order.
- Explicit dependency graph semantics.
- Clear, fail-fast validation for invalid graphs.
- Lightweight runtime overhead in hot loops.
- Good diagnostics for debugging stage orchestration.

## Invariants
- No self-dependencies.
- Edges only connect existing nodes.
- Duplicate edges should not create duplicate execution.
- Cycles are rejected with actionable errors.

## API Direction
- Builder APIs should return `Result` on invalid graph configuration.
- Avoid hidden side effects in `build()`.
- Keep graph mutation explicit and safe.
- Scheduler should remain context-generic (`Scheduler<C>`).

## Observability
- Expose optional graph export (DOT) via explicit API call.
- Keep tracing around scheduler run and node execution timings.

## Testing Priorities
- Dependency order correctness.
- Cycle detection behavior.
- Unknown-node and duplicate-name validation failures.
- Edge deduplication behavior.

## Integration Goals
- Works cleanly as the orchestrator for ECS-based game stages.
- Minimal glue for `Scheduler<World>` or `Scheduler<GameContext>` usage.

## UI Pipeline Orchestration Goals
- Scheduler must support deterministic UI stage ordering.
- UI stages should be first-class nodes in the main frame DAG.
- Dependency validation must protect against invalid UI stage wiring.
