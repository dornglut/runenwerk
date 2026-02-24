# Time Plugin

## Purpose

Updates frame timing state each engine tick.

## Usage

- Plugin: `TimePlugin`
- Scheduler node: `time`

The plugin advances `EngineData.time` once per frame.

## Ownership Boundaries

- Owns frame delta/time progression.
- Does not own downstream consumers of timing state.

## Extension Points

- Add fixed-step utilities/resources tied to frame tick lifecycle.
- Add additional timing diagnostics/state as needed.
