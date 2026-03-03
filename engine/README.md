# Engine Crate

## Purpose

Hosts the runtime, plugin composition, and core engine-facing feature implementations.

## Usage

- Crate: `engine`
- Primary entry surface: `engine::App`, `engine::Plugin`, and `engine::prelude`.
- Legacy entry surface: `engine::legacy`, `engine::runtime`, and `engine::platform`.
- Features are organized under `engine/src/plugins/*`.

### Typed Runtime

The typed runtime path is now the preferred API for new code:

- `engine::App`
- `engine::Plugin`
- schedule labels: `Startup`, `Update`, `RenderPrepare`, `RenderSubmit`
- system params: `Query`, `Res`, `ResMut`, `Commands`

See [`engine/examples/runtime_minimal/main.rs`](/Users/joshua/Projekte/grotto-quest/engine/examples/runtime_minimal/main.rs) for the smallest headless end-to-end example.

## Ownership Boundaries

- Owns engine runtime loop, plugin wiring, and integrated feature implementations.
- Consumes ECS/scheduler crates for data model and execution ordering.
- Does not own ECS core internals or scheduler core internals.

## Extension Points

- Add plugins under `engine/src/plugins/*`.
- Register plugins through app/runtime composition paths.
- Extend plugin-local `README.md` and `requests.md` for feature evolution.
