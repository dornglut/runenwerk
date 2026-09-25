---
title: "Engine Crate"
description: "Documentation for Engine Crate."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-15
---

# Engine Crate

Runtime composition crate for Runenwerk. It owns app boot/run flow, schedule execution,
plugin wiring, and integrated engine-facing systems (scene, render, input, replay, net bridge).

## Where To Start

1. Read crate API surfaces:
   - `src/lib.rs`
   - `src/prelude.rs`
2. Open crate docs hub:
   - `docs/index.md`
3. Run the smallest example:
   - `cargo run -p engine --example runtime_minimal`
4. Open the plugin index:
   - `src/plugins/README.md`
5. Pick an example closest to your change:
   - `examples/README.md`

## Reference Docs

- Docs hub:
  - `docs/index.md`
- Usage guide:
  - `docs/reference/usage-guide.md`
- Advanced guide:
  - `docs/reference/advanced-guide.md`
- Architecture guide:
  - `docs/reference/architecture.md`
- Plugin guides:
  - `docs/reference/plugins/index.md`

## Domain Map

- App
  - `src/app/`
  - Composition root (`App`), plugin registration, run mode selection.
- Runtime
  - `src/runtime/` and `src/app/runtime/`
  - Schedules, system params, fixed-step loop, platform/window adaptation.
- Plugins
  - `src/plugins/`
  - Engine features and cross-feature composition points.
- Render
  - `src/plugins/render/`
  - Render graph, executor registry, shader registry, frame submit path.
- Scene
  - `src/plugins/scene/`
  - Scene lifecycle, world/overlay state publication, snapshot/replay data boundaries.
- Net
  - `src/net/` and `src/plugins/net/`
  - Public net prelude + ECS/runtime integration around standalone RunenNet.
- UI integration
  - `src/plugins/scene/ui/` and `domain/ui/*`
  - Scene overlay integration and renderer-independent UI data contracts.
- Examples and tests
  - `examples/`
  - `tests/`

## App And Runtime Mental Model

- `App` is the user-facing composition API:
  - `App::new()` for windowed mode
  - `App::headless()` for deterministic test/tooling runs
- `App` owns `World`, scheduler runtime, and active runner.
- Bare App construction does not imply fixed cadence or simulation identity/configuration.
- Plugin build methods mutate app composition only (resources/systems/config).
- Runtime frame flow is schedule-driven:
  - `Startup` once
  - per-frame: `PreUpdate -> (FixedStepBegin -> FixedUpdate) 0..N -> Update -> RenderPrepare -> RenderSubmit -> FrameEnd`
- `FixedStepPlugin` is the explicit fixed-cadence selector.
- `SimulationPlugin` supplies Runenwerk's integration of existing `engine_sim` state and advances
  `SimulationTick` at `FixedStepBegin`.
- Windowed and headless modes share the same schedule model, with different platform runners.
- Host selection is stable during execution; bounded advancement never changes a windowed App into a
  headless App.

## Plugin Entry Points

- Plugin trait: `src/plugin.rs`
- Plugin index and docs map: `docs-site/src/content/docs/engine/plugins/README.md`
- Default stack: `engine::plugins::default_plugins()`
  - `TimePlugin`
  - `FixedStepPlugin`
  - `SimulationPlugin`
  - `ReplayPlugin`
  - `InputFinalizePlugin`
  - `DiagnosticsPlugin`

## Public API Ergonomics

- Most users should start with:
  - `engine::App`
  - `engine::Plugin`
  - `engine::prelude::*`
- On `App::headless()`, use `App::run_for_frames(n)` for frame-count advancement.
- On `App::headless()`, use fixed-cadence-owned `AppFixedStepExt::run_for_fixed_steps(n)` only after selecting
  `FixedStepPlugin`; the stop condition is cadence progress, not simulation identity.
- Bounded helpers reject `App::new()` rather than silently changing the selected Host.
- Net-specific integration:
  - `engine::net::prelude::*`
- Schedule and system ordering helpers are re-exported through the prelude/runtime surface.

## Example Map

See [`examples/overview.md`](examples/overview.md) for the full map.

- `runtime_minimal`: smallest headless runtime flow.
- `window_input_demo`: windowed input loop + default plugins.
- `game_of_life_sdf`: windowed public `RenderFlow` example on builtin compiled execution.
- `boids_render_flow`: windowed boids compute with bounded uniform-grid neighbor lookup, procedural draw, fixed-step evidence, aspect-correct impostors, and present flow on the builtin compiled path.
- `sdf_render_flow`: windowed 3D SDF raymarch flow rendered through fullscreen compose and explicit present passes.
- `procedural_sky_sdf_terrain`: windowed procedural sky + noise-shaped terrain SDF raymarch flow.

## Test Map

See `tests/README.md` for integration suite coverage.

## Ownership Boundaries

- Owns runtime loop, plugin composition, replay/runtime integration, and engine-level feature wiring.
- Owns fixed-cadence occurrence/progress but not simulation identity semantics.
- Consumes standalone RunenECS/RunenNet plus local `engine_replay` and `engine_sim` integration owners.
