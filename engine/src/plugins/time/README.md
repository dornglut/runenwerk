# Time Plugin

## Purpose

Updates frame timing state each engine tick.

## Usage

- Plugin: `TimePlugin`
- Typed schedule: `Update`
- Typed set: `CoreSet::Time`
- Legacy scheduler node: `time`

The plugin advances `Time` once per frame on the typed runtime and `EngineData.time` on the legacy runtime.

## Ownership Boundaries

- Owns frame delta/time progression.
- Does not own downstream consumers of timing state.

## Extension Points

- Add fixed-step utilities/resources tied to frame tick lifecycle.
- Add additional timing diagnostics/state as needed.
