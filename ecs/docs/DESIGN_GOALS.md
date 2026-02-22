# ECS Design Goals

## Purpose
Define what the `ecs` crate should optimize for as the foundational runtime for Grotto Quest.

## Core Goals
- Deterministic and predictable behavior.
- Data-oriented archetype storage for cache-friendly iteration.
- Clear typed APIs for common gameplay usage.
- Safe mutation patterns (e.g. query-based mutation without aliasing issues).
- Modularity to support skill/entity composition at runtime.

## API Direction
- Prefer typed spawn and builder flows.
- Auto-register components in typed paths.
- Keep raw dynamic (`Any`) flows available for advanced/runtime-generated content.
- Expand query ergonomics (`query`, `query_mut`) while preserving safety.

## Performance Goals
- Minimize per-entity indirection in hot loops.
- Keep archetype iteration contiguous.
- Avoid unnecessary allocations in common query/update paths.

## Correctness Goals
- Entity generation safety for stale handle prevention.
- Archetype row and column alignment invariants always preserved.
- Explicit tests for every query tuple/mutation path.

## Integration Goals
- ECS remains scheduler-agnostic but scheduler-friendly.
- Game systems should be able to run as scheduler nodes using ECS APIs without extra glue.

## Near-Term TODOs
- Add richer mutable query forms.
- Improve query tuple ergonomics and type clarity.
- Replace panic-based expected failures with typed errors where appropriate.

## UI Integration Goals (High Priority)
- ECS must support retained UI node state and hierarchy-friendly querying.
- UI components should be data-only and renderer-agnostic.
- Dirty-flag patterns for UI layout/style updates should be first-class.
- ECS APIs must support efficient UI extraction for rendering.
