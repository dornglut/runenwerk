# Grid Crate

## Purpose

Hosts standalone grid-related domain code shared by engine/gameplay systems.

## Usage

- Crate: `grid`
- Included as a workspace dependency where grid algorithms/data are needed.

## Ownership Boundaries

- Owns grid-domain data structures and algorithms.
- Does not own engine scheduling/runtime orchestration.

## Extension Points

- Add reusable grid primitives and utilities.
- Keep APIs data-oriented and engine-agnostic.
