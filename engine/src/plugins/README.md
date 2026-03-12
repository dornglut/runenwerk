# Engine Plugins

This directory is the feature composition layer for `engine`.

## Contract

- Each plugin folder should expose a `mod.rs` and a top-level `README.md`.
- Plugin `build` methods should only compose app/runtime state:
  - initialize resources
  - register systems
  - configure schedule ordering
- Cross-plugin helpers belong in `shared/`.
- Prefer typed schedules/sets from `engine::runtime`.

## Plugin Index

- `time/`
  - README: [`time/README.md`](./time/README.md)
  - Purpose: frame time progression.
- `input/`
  - README: [`input/README.md`](./input/README.md)
  - Purpose: action mapping and frame input pulses.
- `scene/`
  - README: [`scene/README.md`](./scene/README.md)
  - Purpose: scene lifecycle and runtime state publication.
- `render/`
  - README: [`render/README.md`](./render/README.md)
  - Purpose: render graph/executor/shader orchestration.
- `net/`
  - README: [`net/README.md`](./net/README.md)
  - Purpose: network runtime and replication bridge.
- `ui/`
  - README: [`ui/README.md`](./ui/README.md)
  - Purpose: UI domain data and template/text flows.
- `grid/`
  - README: [`grid/README.md`](./grid/README.md)
  - Purpose: gameplay-to-grid runtime config publication.
- `debug_metrics/`
  - README: [`debug_metrics/README.md`](./debug_metrics/README.md)
  - Purpose: diagnostics overlay state and draw commands.
- `scheduler_diagnostics/`
  - README: [`scheduler_diagnostics/README.md`](./scheduler_diagnostics/README.md)
  - Purpose: periodic runtime diagnostics logging.
- `shared/`
  - README: [`shared/README.md`](./shared/README.md)
  - Purpose: shared helper utilities.

## Entry Points

- Plugin trait: `engine/src/plugin.rs`
- Plugin registry module: `engine/src/plugins/mod.rs`
- Default plugin stack helper: `engine::plugins::default_plugins()`
