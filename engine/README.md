# Engine Crate

## Purpose

Hosts the runtime, plugin composition, replay integration, and core engine-facing feature implementations.

## Usage

- Crate: `engine`
- Primary entry surface: `engine::App`, `engine::Plugin`, and `engine::prelude`.
- Feature plugins live under `engine/src/plugins/*`.

### Runtime

The engine runtime path is the active and only supported path. Use:

- `engine::App`
- `engine::App::new()` for the windowed runtime
- `engine::App::headless()` for deterministic tests/tools
- `engine::Plugin`
- default plugin stack: `engine::plugins::default_plugins()`
- networking plugins: `NetworkClientPlugin`, `NetworkServerPlugin`, `ReplicationPlugin`, `PredictionPlugin`
- schedule labels: `Startup`, `PreUpdate`, `FixedUpdate`, `Update`, `RenderPrepare`, `RenderSubmit`, `FrameEnd`
- built-in runtime resources: `WindowState`, `Time`, `InputState`, `FixedTimeConfig`, `FixedTimeState`, `CatchupBudget`, `SimulationTick`
- shared runtime state: `SceneRuntimeState`, `GameplayRuntimeConfig`, `UiOverlayState`, `SessionRuntimeState`
- ordering helpers: `in_set`, `before`, `after`
- system params: `Query`, `Res`, `ResMut`, `Commands`

See [`engine/examples/runtime_minimal/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/runtime_minimal/main.rs) for the smallest headless end-to-end example.
See [`engine/examples/window_input_demo/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/window_input_demo/main.rs) for the primary windowed runtime demo.
See [`engine/examples/scene_manager_ui/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/scene_manager_ui/main.rs) and [`engine/examples/sdf_renderer/main.rs`](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/examples/sdf_renderer/main.rs) for the fuller feature path.

### Current Networking and Replay Integration

The engine already integrates:

- `ReplayPlugin` for authoritative scene replay/checkpoint recording and seek/validation
- `NetworkClientPlugin` / `NetworkServerPlugin` for session state and runtime task bridging
- `ReplicationPlugin` / `PredictionPlugin` for the current dedicated-authority path

The current production-leaning profile is `SimulationProfile::DedicatedAuthority`.

Implemented on that path now:

- live QUIC runtime task bridging
- reconnect support
- authoritative scene snapshot replication
- real scene delta snapshots
- client-side authoritative apply plus first-pass prediction correction
- admitted session state publication through `SessionRuntimeState`

## Ownership Boundaries

- Owns engine runtime loop, plugin wiring, replay/runtime integration, and integrated feature implementations.
- Consumes ECS/scheduler crates for data model and execution ordering.
- Does not own ECS core internals or scheduler core internals.

## Extension Points

- Add plugins under `engine/src/plugins/*`.
- Register plugins through app/runtime composition paths.
- Extend plugin-local `README.md` and `requests.md` for feature evolution.
