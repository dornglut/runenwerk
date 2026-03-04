# Engine Crate

## Purpose

Hosts the runtime, plugin composition, and core engine-facing feature implementations.

## Usage

- Crate: `engine`
- Primary entry surface: `engine::App`, `engine::Plugin`, and `engine::prelude`.
- Features are organized under `engine/src/plugins/*`.

### Runtime

The runtime path is now the only supported engine runtime. New code should use:

- `engine::App`
- `engine::App::new()` for the windowed runtime
- `engine::App::headless()` for deterministic tests/tools
- `engine::Plugin`
- default plugin stack: `engine::plugins::default_plugins()`
- networking plugins: `NetworkClientPlugin`, `NetworkServerPlugin`, `ReplicationPlugin`, `PredictionPlugin`
- schedule labels: `Startup`, `PreUpdate`, `FixedUpdate`, `Update`, `RenderPrepare`, `RenderSubmit`, `FrameEnd`
- built-in runtime resources: `WindowState`, `Time`, `InputState`, `FixedTimeConfig`, `FixedTimeState`, `CatchupBudget`, `SimulationTick`
- ordering helpers: `in_set`, `before`, `after`
- system params: `Query`, `Res`, `ResMut`, `Commands`

See [`engine/examples/runtime_minimal/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/runtime_minimal/main.rs) for the smallest headless end-to-end example.
See [`engine/examples/window_input_demo/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/window_input_demo/main.rs) for the primary windowed runtime demo.

## Ownership Boundaries

- Owns engine runtime loop, plugin wiring, and integrated feature implementations.
- Consumes ECS/scheduler crates for data model and execution ordering.
- Does not own ECS core internals or scheduler core internals.

## Extension Points

- Add plugins under `engine/src/plugins/*`.
- Register plugins through app/runtime composition paths.
- Extend plugin-local `README.md` and `requests.md` for feature evolution.
