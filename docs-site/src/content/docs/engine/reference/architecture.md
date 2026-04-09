---
title: "Engine Architecture"
description: "Documentation for Engine Architecture."
---

# Engine Architecture

Internal architecture and runtime contracts for the `engine` crate.

## Domain Ownership

- App domain: `engine/src/app/`
  - composition root (`App`), plugin registration, run-mode selection
- Runtime domain: `engine/src/runtime/` and `engine/src/app/runtime/`
  - schedules, fixed-step semantics, lifecycle execution, platform adapters
- Plugin domain: `engine/src/plugins/`
  - feature-owned runtime behavior (scene/render/input/net/ui/etc.)

## Runtime Lifecycle Contract

Builtin resource installation:

- Installed during `App` construction via:
  - `App::install_builtin_resources` in `engine/src/app/runtime/bootstrap.rs`
- Includes core resources such as:
  - `Time`, `InputState`, `WindowState`
  - `FixedTimeConfig`, `CatchupBudget`, `FixedTimeState`, `SimulationTick`
  - scene/runtime state resources used by built-in plugins

Startup contract:

- `Startup` runs at most once per runtime state.
- Shared implementation:
  - `run_startup_if_needed` in `engine/src/runtime/frame_lifecycle.rs`

Per-frame schedule order:

1. `PreUpdate`
2. fixed-step loop (`FixedUpdate` zero or more times)
3. `Update`
4. `RenderPrepare`
5. `RenderSubmit`
6. `FrameEnd`

Shared implementation:

- `run_frame` in `engine/src/runtime/frame_lifecycle.rs`

## Fixed-Step Contract

Canonical implementation:

- `run_fixed_update_frame` in `engine/src/runtime/fixed_step_executor.rs`

Rules:

1. Read and clamp `FixedTimeConfig::step_seconds`, frame `Time::delta_seconds`, and `CatchupBudget::max_steps_per_frame`.
2. Add frame delta to `FixedTimeState::accumulator_seconds`.
3. Loop while accumulator has at least one fixed step and budget remains:
   - increment `SimulationTick`
   - run one `FixedUpdate`
   - subtract one step from accumulator
   - update `steps_ran_last_frame`
4. If work remains after budget exhaustion:
   - drop remaining accumulated time
   - increment `saturated_frames`

This contract is shared by headless and windowed runners.

## Headless and Windowed Execution

- Headless path:
  - `engine/src/app/platform/headless.rs`
  - delegates to shared app/runtime lifecycle helpers
- Windowed path:
  - `engine/src/runtime/winit_runner.rs`
  - delegates startup/frame scheduling to shared runtime lifecycle helpers

## Integration Boundaries

- `engine` consumes:
  - `ecs` for world/resources/components/queries
  - `scheduler` for typed schedule execution
  - `engine_sim`, `engine_net`, `engine_replay` for simulation/network/replay domains
- `engine` does not own internals of domain/net crates.

## Related Source Entrypoints

- Public crate surface:
  - [`../../src/lib.rs`](../../src/lib.rs)
- Prelude surface:
  - [`../../src/prelude.rs`](../../src/prelude.rs)
- Runtime schedules:
  - [`../../src/runtime/schedules.rs`](../../src/runtime/schedules.rs)
- Plugin map:
  - [`../../src/plugins/README.md`](../plugins/readme.md)
- Plugin guides:
  - [`plugins/index.md`](plugins/index.md)
