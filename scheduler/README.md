# Scheduler Crate

## Purpose

Provides dependency-aware scheduling and node execution ordering.

## Usage

- Crate: `scheduler`
- Used by runtime/plugin systems to register nodes and edges and run execution graphs.

## Ownership Boundaries

- Owns graph validation, ordering, and execution orchestration.
- Does not own domain-specific plugin/system logic.

## Extension Points

- Add scheduling diagnostics and execution controls.
- Extend ordering/validation behavior while preserving deterministic execution.
