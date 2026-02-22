# ECS Design Goals

## Purpose
Define required behavior and constraints for the `ecs` crate as the core data runtime.

## Core Goals
- Deterministic and predictable behavior.
- Data-oriented storage for cache-friendly iteration.
- Safe typed query/mutation ergonomics.
- Support runtime composition without sacrificing correctness.

## Correctness Invariants
- Entity generation/versioning prevents stale-handle misuse.
- Archetype row/column alignment is always maintained.
- Query APIs do not permit unsound aliasing.

## API Direction
- Keep typed spawn/builder/query paths first-class.
- Keep dynamic paths available for advanced runtime content.
- Prefer fallible APIs over panic for expected misuse.

## Performance Direction
- Minimize per-entity indirection in hot loops.
- Keep archetype iteration contiguous.
- Avoid avoidable allocations in frequent query/update paths.

## Integration Goals
- Scheduler-friendly without scheduler coupling.
- Support retained UI data and dirty-flag workflows efficiently.
- Enable extraction-oriented access patterns for rendering systems.

## Testing Priorities
- Query tuple coverage (read + mutable variants).
- Lifecycle and entity recycling behavior.
- Archetype transition correctness during component add/remove.
- Regression tests for any mutation safety edge case.
