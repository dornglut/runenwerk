---
title: "Engine Architecture"
description: "Documentation for Engine Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-13
---

# Engine Architecture

Internal architecture and runtime contracts for the `engine` crate.

## Domain Ownership

- App domain: `engine/src/app/`
  - composition root (`App`), plugin registration, run-mode selection
- Runtime domain: `engine/src/runtime/` and `engine/src/app/runtime/`
  - schedules, fixed-step semantics, lifecycle execution, platform adapters
- Plugin domain: `engine/src/plugins/`
  - feature-owned runtime behavior (scene/render/input/net/world/etc.)

## Runtime Lifecycle Contract

Builtin resource installation:

- Installed during `App` construction via:
  - `App::install_builtin_resources` in `engine/src/app/runtime/bootstrap.rs`
- Includes core resources such as:
  - `Time`, `InputState`, `WindowState`
  - `FixedTimeConfig`, `CatchupBudget`, `FixedTimeState`, `SimulationTick`
  - `ProductPublicationRuntimeResource`
  - `QuerySnapshotRuntimeResource`
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

## Product Publication Runtime

The engine owns the runtime staging resource for product publication outcomes,
not product-family truth. `ProductPublicationRuntimeResource` stages
`domain/product` publication outcomes and publishes them only from
`ProductPublication` barrier handlers.

Plugins install product-agnostic barrier behavior through
`engine::App::add_barrier_handler`. The default engine handler ratifies staged
outcomes, publishes deterministic journal entries ordered by barrier and stage
sequence, and keeps invalid publication diagnostics inspectable.

## Query Snapshot Runtime

The engine owns the runtime staging resource for query snapshots, not
product-family truth. `QuerySnapshotRuntimeResource` stages
`domain/product` query snapshot descriptors and publishes them only from
`QuerySnapshotPublication` barrier handlers.

The default engine handler ratifies staged snapshots, enforces strict
product-domain consumption decisions, preserves prior snapshots on rejected
updates, invalidates snapshots deterministically on source-generation changes,
and keeps accepted, rejected, preserved, and invalidated decisions inspectable.

## Render Product Selection Runtime

Render selection production is prepared-frame state, not renderer-owned product
truth. The render plugin owns `PreparedRenderProductSelectionResource`, which
stores producer-scoped `domain/product` `RenderProductSelection` contributions
keyed by `RenderFrameProducerId`.

Producers replace their own contribution before
`engine/src/plugins/render/runtime/frame_prepare.rs::frame_render_prepare_system`
publishes the prepared frame. The prepared frame snapshots selections together
with views, flow invocations, dynamic targets, and surface bindings. Render
submit consumes this prepared data and does not perform live ECS extraction to
discover product truth.

Prepared-frame inspection exposes selected product ids, generations, typed
freshness/residency/authority/query-policy state, required target descriptors,
residency requests, and diagnostics without backend handles.

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

`WindowState` in `engine/src/runtime/window.rs` is also the runtime-owned place
for platform window effects that app systems request declaratively. It currently
stores title, size, scale factor, close/redraw requests, and `WindowCursorIcon`.
Windowed execution applies the cursor icon in
`engine/src/runtime/winit_runner.rs::WinitRunner::apply_window_effects`; app
systems set the intent on `WindowState` rather than calling winit directly.

## Integration Boundaries

- `engine` consumes:
  - `ecs` for world/resources/components/queries
  - `scheduler` for typed schedule execution
  - `engine_sim`, `engine_net`, `engine_replay` for simulation/network/replay domains
- `engine` does not own internals of domain/net crates.

## Related Source Entrypoints

- Public crate surface:
  - `engine/src/lib.rs`
- Prelude surface:
  - `engine/src/prelude.rs`
- Runtime schedules:
  - `engine/src/runtime/schedules.rs`
- Plugin map:
  - [`../plugins/README.md`](../plugins/README.md)
- Plugin guides:
  - [`plugins/index.md`](plugins/index.md)
