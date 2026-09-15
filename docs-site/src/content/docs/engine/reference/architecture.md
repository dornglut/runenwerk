---
title: "Engine Architecture"
description: "Documentation for Engine Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
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
- Bare App construction installs only universal App/runtime state. It does not imply fixed cadence or
  simulation identity/configuration.
- Fixed-cadence state is selected through `FixedStepPlugin`.
- Simulation integration state is selected through `SimulationPlugin`.
- Universal bootstrap state still includes current App/platform/publication resources such as:
  - `InputState`, `ActionState`, `WindowState`
  - frame-pacing/window platform state
  - `ProductPublicationRuntimeResource`
  - `QuerySnapshotRuntimeResource`

`Time` is not a bare App builtin. `TimePlugin` owns default `Time` installation and frame-time
progression; `default_plugins()` includes `TimePlugin` for the ordinary engine stack.

Startup contract:

- `Startup` runs at most once per runtime state.
- Shared implementation:
  - `run_startup_if_needed` in `engine/src/runtime/frame_lifecycle.rs`

Per-frame schedule order:

1. `PreUpdate`
2. when `FixedStepPlugin` is selected, zero or more fixed steps:
   - `FixedStepBegin`
   - `FixedUpdate`
3. `Update`
4. `RenderPrepare`
5. `RenderSubmit`
6. `FrameEnd`

Without `FixedStepPlugin`, the frame lifecycle skips fixed-step execution entirely. Public cadence
resource presence alone is not the activation signal.

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

## Execution Fabric Runtime

Engine runtime owns the reusable execution and cache-metadata semantics exposed
by `RuntimeJobExecutorResource` and `RuntimeProductCacheResource`. Resource
installation is application/product composition, not a bare-App invariant.

A bare `App` therefore does not imply product-job execution or runtime product
cache state. Current Draw composition selects both through `DrawingAppPlugin`:
it installs the maintained bounded worker executor only when the application did
not explicitly supply an executor and initializes the runtime product cache
non-overwritingly. Explicit caller resources remain authoritative.

Product-job completion visibility remains governed by the existing product
publication and query snapshot barriers. Selecting executor/cache resources does
not transfer product payload truth or publication authority to the App.

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

## Render GPU Residency Runtime

GPU residency is derived renderer cache state, not product truth. The render
plugin owns `RenderGpuResidencyResource`, which reads prepared
`RenderProductSelection` residency requests before frame preparation and
allocates renderer-owned logical `RenderGpuCacheHandle` values for current
resident products.

Residency derivation allocates, preserves, invalidates, evicts, rejects, and
journals cache state deterministically by prepared selection content, product
identity, generation, priority, and hard-pin state. Inspection exposes logical
cache ids and diagnostics, never mutable backend handles or `wgpu` objects.

## Fixed-Step Contract

`FixedStepPlugin` is the explicit cadence activator and non-overwriting provider of:

```text
FixedTimeConfig
CatchupBudget
FixedTimeState
```

Canonical execution lives in `run_fixed_update_frame` in
`engine/src/runtime/fixed_step_executor.rs`.

For an active cadence frame:

1. Read and clamp `FixedTimeConfig::step_seconds` and `CatchupBudget::max_steps_per_frame`.
2. Use the bounded-advancement one-frame cadence override when present; otherwise use frame
   `Time::delta_seconds` when Time is installed, with the fixed step as the fallback.
3. Add the selected frame delta to `FixedTimeState::accumulator_seconds`.
4. While one fixed step is admitted and budget remains:
   - run `FixedStepBegin`
   - run `FixedUpdate`
   - subtract one step from the accumulator
   - update `steps_ran_last_frame`
   - increment `total_completed_steps`
5. If work remains after budget exhaustion, drop the remaining accumulated time and increment
   `saturated_frames`.

The cadence executor does not import, install, or mutate `SimulationTick`.

## Simulation Integration Contract

`SimulationPlugin` is Runenwerk's explicit integration provider for the existing `engine_sim`
owner state:

```text
SimulationTick
SimulationProfileConfig
SimulationSessionId
SimulationSeed
SimulationRng
```

The plugin preserves explicitly supplied owner state. When the plugin is composed with
`FixedStepPlugin`, it advances `SimulationTick` exactly once in `FixedStepBegin`, before
`FixedUpdate`. Selecting Simulation alone does not activate cadence; selecting FixedStep alone does
not manufacture simulation identity.

`App::set_simulation_profile`, `App::set_authority_role`, and `App::set_simulation_seed` are explicit
composition commands that may materialize the same owner configuration state before plugin
installation. They do not create a parallel App-side simulation authority.

## Bounded Advancement

`App::run_for_fixed_steps(n)` advances by `n` additional completed Runenwerk fixed steps. Its stop
condition is `FixedTimeState::total_completed_steps`, not `SimulationTick`.

The API requires `FixedStepPlugin` to have been selected and returns an error otherwise. Repeated
bounded calls are relative to current cadence progress, so simulation identity may be independently
restored, replayed, or reassigned without changing the App advancement contract.

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
