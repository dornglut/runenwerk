---
title: RunenRender Decomposition Design
description: Decision-complete target ownership, package boundaries, host/surface model, producer seams, and internal proof required before extracting RunenRender.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../reports/investigations/runenrender-complete-semantic-inventory.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Decomposition Design

## Status

The target repository, package missions, host/backend split, and staged internal
repair architecture are decision-complete. Source implementation remains blocked
until the mandatory local inventory and command baseline pass and
`PT-RUNENRENDER-002` turns the repair program into small exact phase specs.

The linked investigation owns current-source evidence and module dispositions.
This document owns the durable target.

## Goal

Create an independently usable renderer framework without carrying Runenwerk ECS,
scene, world, material, SDF, UI, editor, lifecycle, window, or product semantics
into the framework.

Runenwerk remains the integration host and translates domain products into
generic renderer contracts.

## Repository and packages

```text
repository: Crystonix/RunenRender
packages:
  runenrender_core
  runenrender_wgpu
  runenrender_macros
version: 0.1.0
edition: 2024
license: MIT OR Apache-2.0
publish: false until release gates are accepted
```

Initial shape:

```text
RunenRender/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── runenrender_core/
│   ├── runenrender_wgpu/
│   └── runenrender_macros/
├── tests/
├── examples/
├── benches/
├── docs/
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

Do not create separate graph, resource, shader, pipeline, surface, residency, or
inspection crates initially. Add a package only after independent dependency or
release pressure exists.

The macro package is justified because Rust proc macros require a separate crate
and current `GpuUniform`/`GpuStorage` derives express reusable GPU layout
conversion. It moves only after ABI conformance is proven.

## Dependency direction

```text
runenrender_core
       ▲
       ├──────── runenrender_macros generated implementations
       │
runenrender_wgpu
       ▲
       │
Runenwerk render host and domain adapters
```

Forbidden dependencies:

```text
runenrender_core -> WGPU, Winit, ECS, Runenwerk, RunenSDF, RunenUI,
                    scene, world, material authoring, app lifecycle
runenrender_wgpu -> Winit, ECS, Runenwerk, domain semantics
RunenRender       -> Runenwerk
```

`runenrender_wgpu` consumes validated core plans and resource/frame inputs.
Runenwerk depends one-way on the framework packages.

## runenrender_core ownership

Core owns backend-neutral:

- opaque graph, pass, resource, producer, frame, view, target, surface, shader,
  and pipeline identities;
- graph and pass declarations;
- resource descriptors and lifetime planning;
- access/dependency validation;
- compute, graphics, fullscreen-raster convenience, copy, and present operations;
- generic target/view and flow-invocation descriptions;
- explicit prepared frame inputs;
- generic producer contributions and replacement/removal;
- backend capability requirements;
- deterministic execution plans;
- GPU parameter/layout traits and values;
- structured diagnostics and provenance;
- read-only planning/inspection reports.

Core does not own:

- ECS resource/state extraction;
- fixed-time or engine phase policy;
- native windows;
- built-in UI/material/SDF/world/editor features;
- product selections or main/offscreen product semantics;
- filesystem asset discovery/hot reload;
- WGPU objects or execution.

## runenrender_wgpu ownership

The WGPU package owns:

- instance, adapter, device, and queue;
- configurable adapter, feature, limit, memory, trace, and fallback policy;
- headless/offscreen initialization;
- GPU buffers, textures, samplers, bind groups, shader modules, and pipelines;
- upload and readback;
- command encoding and submission;
- backend caches and generations;
- WGPU surface creation/configuration/acquisition/presentation/retirement;
- surface and device-loss classification;
- in-flight frame/resource retention;
- backend diagnostics and timing capability facts.

It does not read ECS resources, Runenwerk scenes, material graphs, SDF fields, UI
frames, editor state, product selections, or Winit events.

## runenrender_macros ownership

The macro package owns:

```text
GpuUniform
GpuStorage
```

or reviewed replacement names.

Requirements:

- generated paths resolve `runenrender_core`, including dependency renaming;
- no `engine::plugins::render` or Runenwerk path;
- exact WGSL/WGPU ABI alignment for supported scalar/vector/matrix/array/struct
  shapes;
- deterministic padding and byte layout;
- safe bytemuck use only when generated representation satisfies its invariants;
- precise compile errors for unsupported data;
- generic/where-clause support where sound;
- external compile-pass and compile-fail conformance.

## Runenwerk ownership

Runenwerk retains:

- `RenderPlugin` and app/frame/startup/shutdown scheduling;
- Winit windows and event-loop policy;
- `NativeWindowId`, DPI, resize, monitor, visibility, and window mapping;
- ECS/application state projection;
- scene/world preparation;
- material IR-to-shader/pipeline translation;
- SDF representation and world/residency policy;
- editor picking and editor-specific render workflows;
- UI conversion and font/atlas policy;
- shader filesystem discovery/watch/hot reload;
- asset path and KTX2 material loading;
- product feature and fallback policy;
- debug overlays, capture export, artifact paths, and diagnostics presentation;
- startup readiness and frame pacing;
- integration tests and product evidence.

## Identity policy

RunenRender uses repository-local opaque identities. It does not depend on
Runenwerk `id` or `id_macros`.

Builders/registries allocate identities from an explicit owner/namespace.
Process-global atomic sequences are not canonical deterministic graph identity.
Exhaustion is structured and never saturates into reuse.

Runtime identities are not persisted IDs by default. A future serialized graph or
artifact format must define stable authored keys and versioning separately.

## Graph contract

Core graph operations are only:

```text
Compute
Graphics
FullscreenRaster
Copy
Present
```

There is no `BuiltinUiComposite`, material pass, SDF pass, world pass, editor
pass, or product feature variant.

Graph planning owns:

- pass/resource graph formation;
- dependency and cycle validation;
- read/write/access conflict validation;
- target and binding validation;
- resource lifetime/alias planning;
- capability validation;
- deterministic order and execution-plan generation;
- structured graph diagnostics.

Semantic producers validate their own domain before submitting generic work.
Renderer core never interprets buttons, fields, scene nodes, materials, entities,
or editor tools.

## Host-state and prepared-input boundary

Current APIs such as these remain Runenwerk integration and are removed from
core:

```text
with_state<T: ecs::Resource>
uniform_from_state
dispatch_from_state
host TypeId state requirements
RenderFrameDataRegistry tied to ECS
```

Runenwerk resolves host/ECS/application state first, then submits explicit:

- uniform/storage bytes or validated typed prepared data;
- texture/buffer imports and updates;
- draw and indirect arguments;
- compute dispatch counts;
- target/view descriptions;
- flow invocations;
- producer contributions;
- capability requirements;
- resource generations and revisions.

Prepared inputs have explicit ownership and lifetime. No renderer plan reaches
back into a host world.

## Iteration and fixed-time policy

RunenRender initially has no Runenwerk fixed-step region abstraction.

Runenwerk expands fixed-time/catch-up policy into explicit repeated flow/pass
invocations and prepared iteration uniforms before renderer submission.

A generic iteration-region feature may be designed later only after a second
independent renderer consumer demonstrates the need.

## View, target, and product policy

Core owns explicit opaque `ViewId` and `TargetId` values plus generic target/view
descriptors.

Core does not know:

```text
MainSurface
OffscreenProduct
product selection
viewport embed policy
native window identity
```

Runenwerk forms the invocation set and maps product/native-window concepts to
renderer-local targets and surfaces.

## Producer contributions

A generic producer contract contains:

- producer identity scoped to the host/integration;
- frame/view/target identity;
- resource declarations/updates;
- passes or draw/dispatch/copy contributions;
- dependency/order requirements;
- capabilities;
- provenance and diagnostics;
- deterministic upsert/replacement/removal behavior.

It contains no ECS entities, UI routes/widgets, SDF fields, material graph nodes,
scene nodes, editor tools, or product feature enums.

Runenwerk adapters translate those domains.

## Surface and window ownership

### Runenwerk host

Owns:

- Winit/native window creation and destruction;
- event loop;
- `NativeWindowId`;
- DPI, resize, monitor, focus, visibility, and window policy;
- mapping native windows to renderer-local surfaces;
- product response to unrecoverable presentation failure.

### runenrender_core

Owns:

- opaque renderer-local `SurfaceId`;
- generic desired target size and presentation intent;
- backend-neutral lifecycle and recovery reports.

### runenrender_wgpu

Owns:

- `wgpu::Surface` and capabilities;
- format/usage/alpha/present-mode/frame-latency realization;
- configure/acquire/present;
- surface generations and retirement;
- recoverable and terminal backend outcomes.

The WGPU package has no Winit dependency. It accepts a WGPU-compatible generic
surface target/raw-window-handle contract while the host retains the underlying
window lifetime.

## Headless backend requirement

WGPU initialization must work without a surface/window for:

- compute;
- offscreen rasterization;
- upload/readback;
- pipeline/resource conformance;
- CI-capable adapter/device smoke tests where the environment permits.

Attaching a surface is a later optional operation, not a prerequisite for device
creation.

## Resource lifetime and device loss

RunenRender owns lifetime only for renderer-created GPU resources.

Contracts define:

- logical resource ID and generation;
- create/update/remove/import requests;
- stale-handle behavior;
- CPU upload source ownership;
- in-flight retention;
- transient aliasing/lifetime planning;
- cache invalidation and memory budgets;
- shutdown order;
- device-loss reconstruction report.

Runenwerk/domain adapters retain source assets/data required to reconstruct GPU
resources. The backend reports what must be rebuilt; Runenwerk decides product
recovery, fallback, window closure, or shutdown.

No GPU lifetime relies implicitly on ECS resource lifetime.

## Shader and pipeline boundary

### Core

Owns:

- shader identity/source/interface/revision descriptors;
- validated binding/pipeline descriptions;
- backend-neutral specialization and capability requirements;
- structured compile/validation diagnostics.

### WGPU

Owns:

- WGSL validation/module creation;
- bind-group/pipeline layout realization;
- render/compute pipeline creation and caches;
- backend compile/runtime errors.

### Runenwerk

Owns:

- filesystem paths and discovery;
- asset registries;
- file polling/watchers and hot reload;
- last-known-good product policy;
- material IR and scene-material shader generation;
- SDF/UI/world/editor shader selection;
- material KTX2 file loading.

Invalid flows/shaders are reported. Renderer core does not silently skip them as
application policy.

## Material, SDF, UI, scene, and editor adapters

### Material

```text
Runenwerk MaterialIr/material graph
    -> Runenwerk material render adapter
    -> generic shader/pipeline/resource requests
    -> RunenRender
```

### SDF

```text
RunenSDF field/query semantics
    -> Runenwerk SDF/world render adapter
    -> generic buffers/textures/parameters/passes
    -> RunenRender
```

RunenRender does not depend on RunenSDF.

### UI

RunenUI is a separate workstream. A future Runenwerk adapter converts accepted
paint output to generic contributions. RunenRender has no UI semantic API.

### Scene/editor

Runenwerk maps scenes, world products, picking, captures, and editor workflows to
generic renderer contracts. These do not become core feature families.

## GPU primitive and residency policy

Scan, compaction, counters, indirect draw argument formation, and generic GPU
resource-generation utilities may move only after source/test review proves no
product/domain semantics.

Generic renderer-owned residency may include GPU handle generations, memory
budgeting, eviction, and stale-resource behavior.

World, chunk, SDF, material, texture-product, and streaming policy remains
Runenwerk-owned.

## Error and terminal policy

Public APIs use structured errors, including reviewed equivalents of:

```text
IdAllocationError
GraphBuildError
GraphValidationError
FrameInputError
ResourceError
ShaderError
PipelineError
BackendInitError
SurfaceError
DeviceLostError
SubmissionError
TerminalStateError
```

`anyhow` may wrap these at Runenwerk/application boundaries only.

Ordinary invalid labels, missing resources, invalid flows, unsupported
capabilities, stale handles, surface outcomes, and device loss do not panic.

Surface errors distinguish at least:

```text
Outdated
Lost
Timeout
OutOfMemory
DeviceLost
```

RunenRender reports recovery requirements; Runenwerk chooses policy.

## Threading and synchronization

- core planning is deterministic, GPU-free, and independently testable;
- WGPU execution has one explicit mutable backend owner/render thread initially;
- Device/Queue sharing follows WGPU contracts without global singleton state;
- host callbacks, filesystem watchers, ECS worlds, and Winit events remain outside
  backend locks;
- no process-global flow-ID, shader-registry, telemetry, or backend authority;
- backend submission and in-flight retention ordering are explicit;
- parallel preparation/execution is deferred until ownership and determinism are
  proven.

## Diagnostics and inspection

RunenRender diagnostics use `runenrender.*` codes.

Framework-owned diagnostics include:

- graph/resource/pass validation;
- backend capabilities;
- resource/pipeline generations and cache outcomes;
- submission/presentation/device-loss facts;
- generic pass/resource provenance;
- optional generic capture/readback facts;
- deterministic plan reports.

Runenwerk retains SDF/material/world/product evidence, selector policy, artifact
exports, paths, pixel/texture comparison policy, inspection UI, startup readiness,
and frame-pacing decisions.

Wall-clock timing is evidence, not behavior authority.

## Persistence policy

No current runtime ID, graph object, shader handle, pipeline cache, prepared
frame, or WGPU resource is a stable persisted format.

A future serialized graph/artifact/cache format requires an accepted schema,
version, validation, migration, compatibility, and deterministic encoding design.
Rust layout and backend cache bytes are not portable contracts by default.

## Internal target architecture

Before external extraction, establish package-equivalent boundaries inside
Runenwerk:

```text
neutral renderer core
WGPU backend
Runenwerk render host/plugin adapter
Runenwerk ECS/scene adapter
Runenwerk material adapter
Runenwerk SDF/world adapter
Runenwerk editor/debug adapter
Runenwerk UI adapter
```

Create actual internal crates only where they enforce the intended final package
dependency. Do not create decorative crates that continue reaching through
private engine paths.

## Anti-cheating proof

Runenwerk must consume the internal renderer through the same public seam intended
for external consumers.

Forbidden:

- private module access from Runenwerk adapters;
- ECS derives/types in renderer packages;
- Runenwerk types in renderer signatures;
- product-specific pass/feature/view variants;
- direct material/SDF/UI/scene/editor imports in core/WGPU;
- Winit in WGPU package;
- graph tests requiring an engine world;
- backend tests requiring product features;
- temporary compatibility renderer surviving merge.

The external move is mechanical only after this proof passes.

## Independent conformance

### Core

- identity allocation/exhaustion;
- graph construction, duplicates, missing references, cycles, conflicts, and
  resource lifetimes;
- deterministic plan output;
- generic operations/views/targets;
- explicit frame-input validation;
- producer upsert/replacement/removal/provenance;
- no ECS/WGPU/Winit/Runenwerk dependency;
- external producer using public APIs;
- stable/MSRV tests and planning benchmarks.

### Macros

- WGSL/WGPU ABI fixtures;
- scalar/vector/matrix/array/padding cases;
- package renaming;
- generics/where clauses where supported;
- precise unsupported-type errors;
- external compile-pass/fail tests.

### WGPU

- headless adapter/device creation;
- configured feature/limit negotiation;
- buffer/texture upload and readback;
- shader/pipeline creation;
- compute, graphics, copy, and offscreen rendering;
- resource/pipeline cache generations;
- optional surface attach/configure/acquire/present;
- multiple surfaces and resize;
- outdated/lost/timeout/out-of-memory/device-loss handling;
- shutdown and in-flight retention;
- backend/platform evidence separated from deterministic core tests.

## Repair sequence

`PT-RUNENRENDER-002` will produce exact file scopes and validation for:

```text
RENDER-R1 neutral identities, structured errors, and internal dependency map
RENDER-R2 graph/resource descriptors without UI/ECS/host state/fixed time/products
RENDER-R3 explicit prepared frame inputs and generic producer contributions
RENDER-R4 GPU params and runenrender_macros ABI conformance
RENDER-R5 shader descriptors and filesystem/hot-reload separation
RENDER-R6 WGPU headless device/resource/pipeline executor
RENDER-R7 generic surface target and structured surface/device-loss contract
RENDER-R8 split generic diagnostics/capture/provenance from product inspection
RENDER-R9 migrate scene/material/SDF/UI/editor/runtime to public adapters
RENDER-R10 internal package anti-cheating and performance proof
```

Do not combine all repairs into one PR.

## Extraction sequence

### PT-RUNENRENDER-001 — Complete inventory

Complete for ownership/design decisions subject to mandatory local exact file,
shader, test, benchmark, consumer, and command verification.

### PT-RUNENRENDER-002 — Specification closure

Create exact R1–R10 implementation contracts. No broad source implementation.

### PT-RUNENRENDER-003–005 — Internal decomposition and proof

Execute R1–R10 through bounded PRs, ending with Runenwerk consuming only the
intended public seams.

### PT-RUNENRENDER-006 — Repository creation and transfer

Create RunenRender, establish governance/provenance, transfer corrected packages,
and validate independently.

### PT-RUNENRENDER-007 — Runenwerk cutover

Pin exact revisions, remove original renderer implementation, migrate all apps and
adapters, regenerate the lockfile, and run complete CPU/headless/GPU validation.

### PT-RUNENRENDER-008 — Closeout

Record compatibility, performance/platform evidence, provenance, deleted paths,
and final ownership.

## Stop conditions

Stop if:

- core imports ECS, WGPU, Winit, Runenwerk, UI, SDF, scene, or material authoring;
- WGPU requires Winit or a surface for initialization;
- product pass/feature/view variants remain in core;
- host `TypeId` state extraction remains in compiled plans;
- invalid flows are skipped rather than reported;
- resource/device-loss reconstruction ownership is unresolved;
- Runenwerk uses renderer private internals during internal proof;
- GPU runtime validation is unavailable for backend behavior changes;
- external extraction would retain an original/compatibility renderer.

## Definition of done

RunenRender is complete only when:

- core, WGPU, and macros validate independently on stable and MSRV;
- core has no forbidden dependency or product semantics;
- WGPU supports headless operation and generic host surfaces without Winit;
- Runenwerk uses exact revisions through public adapters only;
- original renderer implementation is deleted;
- no compatibility facade, mirror, or moving dependency remains;
- full Runenwerk headless and GPU application evidence passes;
- provenance, compatibility, and current documentation are complete.
