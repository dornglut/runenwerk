# Dependency Rules

## Target repository direction

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk adapters/integration --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Framework repositories must not depend on Runenwerk.

The default rule remains that framework repositories do not depend directly on
one another. A direct dependency requires accepted evidence of independent value,
correct ownership, and absence of a Runenwerk adapter responsibility.

Accepted direct dependency:

```text
RunenRender -> RunenGPU
```

RunenGPU is a lower-level execution framework useful to rendering and non-render
compute. This relationship is not cross-domain translation and therefore does not
belong in a Runenwerk adapter.

In particular:

- RunenGPU does not depend on RunenRender, RunenSDF, RunenECS, RunenUI, or
  Runenwerk;
- RunenRender does not require RunenECS, RunenSDF, RunenUI, or Runenwerk;
- RunenECS does not require Runenwerk geometry, renderer, GPU, networking, or app
  lifecycle;
- RunenSDF does not require Runenwerk geometry, world, renderer, GPU, or ECS;
- RunenUI core/runtime do not require RunenGPU, RunenRender, or Runenwerk;
- Runenwerk owns cross-domain translations and product composition.

## Target package direction

```text
runengpu_core
    -> lower-level external libraries only

runengpu_wgpu
    -> runengpu_core
    -> wgpu

runenrender_core
    -> lower-level external libraries only

runenrender_gpu
    -> runenrender_core
    -> runengpu_core

Runenwerk application/backend wiring
    -> runengpu_wgpu
    -> runenrender_gpu
    -> selected domains/frameworks
```

There is no `runenrender_wgpu` package in the accepted target.

## Current in-repository direction

Until each clean cutover completes:

```text
foundation -> domain crates -> engine/runtime -> apps/adapters/tools
```

Current source location is transitional implementation fact, not permanent
ownership authority.

The current renderer/WGPU code remains under `engine/src/plugins/render` until the
RunenGPU and RunenRender internal anti-cheating gates and clean cutovers pass.

## Foundation rules

Foundation may depend only on justified foundation crates and appropriate
low-level external libraries.

Foundation must not depend on domain, runtime, editor, app, adapter, AI, UI, or
concrete GPU/render/window/input/audio backend code.

Do not create a universal shared-core/meta repository to avoid explicit adapter
boundaries.

RunenGPU is not foundation or a universal core. It is a framework with concrete
GPU execution ownership.

## Domain rules

Domain crates may depend on foundation and carefully selected lower-level domain
contracts.

Domain crates must not depend on runtime, app code, backend adapters, editor app
wiring, AI integrations, or concrete rendering/windowing/input/audio backends
unless the domain explicitly owns that backend.

RunenSDF, simulation, procgen, material-authoring, and UI domains do not depend on
RunenGPU simply because optional adapters may accelerate them.

During extraction, accidental dependencies are removed rather than copied.

## RunenGPU rules

### `runengpu_core`

Must not depend on:

```text
wgpu
winit
Runenwerk
RunenRender
RunenSDF
RunenECS
RunenUI
scene/world/material/simulation/editor/product semantics
```

It may expose only owned GPU execution contracts:

- runtime identities;
- capabilities;
- resources, access, initialization, and lifetimes;
- shader/pipeline intent;
- bounded work fragments and work-plan validation;
- submissions, completions, readback, errors, diagnostics, provenance.

### `runengpu_wgpu`

May depend on WGPU and `runengpu_core`.

Must not depend on:

```text
winit
Runenwerk
RunenRender
ECS
SDF
UI
scene/world/material/simulation/editor/product semantics
```

It owns concrete WGPU realization, not domain algorithms or image formation.

## RunenRender rules

### `runenrender_core`

Must not depend on:

```text
wgpu
winit
Runenwerk
ECS
RunenSDF
RunenUI
scene/world/material-authoring/simulation/editor/application semantics
```

It may own prepared render scenes, contributions, views, logical targets,
providers/interactions, materials/media, emitters/environments, transport/history,
overlays, presentation intent, and render diagnostics.

### `runenrender_gpu`

May depend on `runenrender_core` and `runengpu_core`.

It must not:

- directly depend on WGPU;
- create a competing device, queue, surface, allocator, or resource namespace;
- own window lifecycle;
- import ECS/SDF/UI/domain semantics;
- bypass RunenGPU public work/resource contracts.

It owns render-specific realization and lowering into RunenGPU work.

## Identity dependency rule

Semantic identities do not cross owners unchanged.

```text
GpuResourceId      RunenGPU runtime resource
RenderProviderId   RunenRender semantic provider
Entity             ECS/world identity
NativeWindowId     Runenwerk host identity
Asset key          owning domain/asset format
```

Adapters map identities. Do not solve integration by sharing one universal ID
crate or raw integer namespace.

## Engine/runtime rules

Runtime may depend on foundation, current domains, extracted frameworks, and
backend implementation packages.

Runtime owns:

- lifecycle;
- scheduling;
- windows/event loop;
- GPU context setup;
- framework/domain adapters;
- product capability/quality selection;
- diagnostics presentation;
- recovery.

It must not move product/editor/domain semantics into generic framework APIs.

## Adapter rules

A Runenwerk adapter may depend on Runenwerk and the framework(s) it translates.

Adapters translate:

- identities;
- prepared inputs and outputs;
- lifecycles and generations;
- diagnostics and provenance;
- ownership and persistence boundaries.

Adapters must not duplicate algorithms, become writable parallel authorities,
expose broad compatibility facades, or hide dependency cycles.

Initial RunenUI-to-RunenRender and RunenSDF-to-RunenRender integrations remain
Runenwerk-owned until stable public APIs and a second host justify reusable bridge
packages.

## App/tool rules

Apps and tools may compose higher layers but must not define framework or domain
invariants.

A tool may directly use RunenGPU for an owned GPU workload without depending on
RunenRender when it does not form images.

## Test-support rules

Reusable fixtures live in explicit test-support crates/modules. Production APIs
must not be widened solely for tests.

Framework repositories own public downstream conformance. Runenwerk owns
cross-framework integration compatibility.

Evidence distinguishes deterministic planning/source proof from environment-
dependent GPU, driver, surface, and runtime proof.

## Clean-cutover rules

Completed extraction leaves:

- one external source authority;
- one-way dependency direction;
- exact dependency pinning;
- no original source copy;
- no forwarding package or namespace;
- no submodule;
- no long-lived migration facade;
- no duplicate GPU context/resource namespace;
- no duplicate renderer execution path.

Temporary duplication is allowed only on an unmerged extraction branch.

RunenGPU must cut over before RunenRender GPU realization is externally extracted.

## Boundary escalation

When one owner wants another owner's internals, determine whether the missing
boundary is:

- a public value/DTO;
- command or request;
- diagnostic/report;
- adapter;
- capability contract;
- test-support contract.

Do not solve boundary pressure with a universal abstraction, raw handles, mutable
internals, or a dependency cycle.
