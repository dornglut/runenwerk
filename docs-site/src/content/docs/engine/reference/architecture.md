---
title: "Engine Architecture"
description: "Documentation for Engine Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
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

Bare App construction does not install optional capability, Host, product, or presentation state. It contains the RunenECS world/runtime and App lifecycle/composition state only; fixed cadence, simulation identity/configuration, and logical presentation are selected by their owning integration paths.
- Fixed-cadence state is selected through `FixedStepPlugin`.
- Simulation integration state is selected through `SimulationPlugin`.
- `PrimaryPresentationMetricsResource` is shared host-neutral logical-presentation integration state. Native Host realization and Scene selection initialize it idempotently when required; caller-supplied headless metrics are preserved.
- Product/query publication state is selected lazily through `AppPublicationExt`; bare App
  construction does not manufacture publication staging resources or handler registry state.
- Input capability state is selected through `InputFinalizePlugin`, which installs `InputState` and `ActionState`.
- Native window/event providers and frame-pacing state are realized by the selected native Host rather than by bare App construction.

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

Applications select product-publication integration through
`AppPublicationExt::add_product_publication_handler`. The first publication-owner
registration materializes the publication runtime resources and installs the built-in
staged-outcome handler before owner-specific handlers. The built-in handler ratifies
staged outcomes, publishes deterministic journal entries ordered by publication
occurrence and stage sequence, and keeps invalid publication diagnostics inspectable.

## Query Snapshot Runtime

The engine owns the runtime staging resource for query snapshots, not
product-family truth. `QuerySnapshotRuntimeResource` stages
`domain/product` query snapshot descriptors and publishes them only from
`QuerySnapshotPublication` barrier handlers.

Applications select query-publication integration through
`AppPublicationExt::add_query_snapshot_publication_handler`. Selection shares the same
owner-local publication initialization as product publication, so the built-in staged
snapshot handler is installed exactly once before owner-specific handlers. It ratifies
staged snapshots, enforces strict product-domain consumption decisions, preserves prior
snapshots on rejected updates, invalidates snapshots deterministically on
source-generation changes, and keeps accepted, rejected, preserved, and invalidated
decisions inspectable.

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

`WindowStateRegistryResource` in `engine/src/runtime/window.rs` is the native Host authority for
primary and secondary native windows. Each `NativeWindowRecord` owns native title, physical
size/scale, focus, close intent/approval, redraw intent, cursor intent, and lifecycle/failure state.
`PrimaryPresentationMetricsResource` is separate host-neutral logical primary size/scale state.
Headless execution can carry presentation metrics when a selected consumer such as Scene requires them, without manufacturing native-window identity or native lifecycle state in bare App. Windowed execution applies native effects from the keyed record in
`engine/src/runtime/winit_runner.rs::WinitRunner::apply_window_effects`.

## Integration Boundaries

- `engine` consumes the exact standalone `runen-ecs` dependency for ECS world, resource, component, query, system, schedule, and deferred-command contracts.
- `engine` owns Runenwerk host lifecycle, schedule invocation, plugin composition, and product/publication policy around those framework contracts.
- `engine` consumes `engine_sim`, `engine_net`, and `engine_replay` for Runenwerk-local simulation/network/replay integration domains.
- `engine` does not own standalone RunenECS internals or the internals of local domain/net crates.

For reusable RunenECS semantics and current framework architecture, use [standalone RunenECS](https://github.com/dornglut/runen-ecs/blob/main/ARCHITECTURE.md). For API/examples matching this Runenwerk checkout, use the [package guide at the exact consumed revision](https://github.com/dornglut/runen-ecs/blob/6a7af7bbd15da960ce0b68484b446134940fa479/crates/runen-ecs/README.md).

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
