---
title: RunenGPU and RunenRender Split Investigation
description: Connector-backed current-state evidence, ownership classification, identity findings, control-flow findings, and remaining command gates for separating GPU execution from rendering.
status: active
owner: workspace
layer: investigation
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/specs/pt-runengpu-g1-identities-errors.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU and RunenRender Split Investigation

## Question

Which current renderer responsibilities belong to a reusable general GPU execution
framework, which belong to image formation, which remain Runenwerk/domain
integration, and what must be proven before implementation and external transfer?

## Verdict

```text
RunenGPU repository candidate             accepted direction
RunenRender repository candidate          retained
RunenRender -> RunenGPU dependency         accepted direction
move current render directory unchanged   forbidden
old RunenRender R1 implementation         superseded before activation
connector ownership investigation         substantial
complete command/file/consumer baseline   pending local execution
first implementation candidate            PT-RUNENGPU-G1 after S0 gate
external source movement                   forbidden
```

The current `engine/src/plugins/render` root publicly aggregates:

```text
api
backend
composition
features
frame
gpu_primitives
graph
inspect
material_compiler
params
pipelines
procedural
renderer
residency
resource
shader
runtime
texture upload
plugin integration
```

It also re-exports GPU layout macros, renderer execution, runtime, resource,
feature, material, procedural, residency, and shader-reload surfaces from one
namespace.

This is neither a reusable GPU framework nor a clean rendering framework.

## Evidence status

Repository: `Crystonix/Runenwerk`

Reviewed main baseline:

```text
8de096259eab30f8d67672010df9190970d0bfc4
```

Evidence classes:

```text
E2 GitHub repository/commit/PR metadata
E3 connector-backed source and documentation inspection
```

Connector inspection covered:

- renderer root and module-family declarations;
- API identities;
- graph, frame, feature, resource, pipeline, GPU primitive, runtime, shader,
  inspection, procedural, residency, and material-compiler roots;
- WGPU context/device/surface/resource allocator;
- plugin initialization and scheduling;
- frame preparation and submission control flow;
- current repository-family, ADR, design, execution-plan, and planning authority.

The connector does not run Cargo, list every file with authoritative local path
semantics, validate WGSL, initialize WGPU, execute surface/device-loss scenarios,
or benchmark the renderer. Those gates remain mandatory before G1 activation.

## Current root aggregation

The render root exposes all major concerns through one public module:

```text
backend execution
render semantics
GPU primitives
resource planning
material compilation
procedural authoring/lowering
residency
runtime/plugin integration
shader filesystem reload
product inspection
```

The current `renderer` module directly imports:

- `WgpuCtx`;
- WGPU device utilities and public WGPU types;
- Winit `Window`;
- UI render data;
- prepared render frames;
- shader registry;
- capture, timing, and resource inspection;
- `anyhow`.

This confirms that current renderer execution cannot be transferred unchanged.

## Current WGPU ownership

### WGPU context

`WgpuCtx` currently owns:

```text
wgpu::Instance
wgpu::Adapter
Arc<wgpu::Device>
Arc<wgpu::Queue>
BTreeMap<RenderSurfaceId, wgpu::Surface + configuration>
```

Its primary constructor requires `Arc<winit::window::Window>`, creates a surface
before adapter selection, and requests a surface-compatible adapter.

Consequences:

- no independent headless construction path;
- host window lifetime and backend device lifetime are coupled;
- device selection is coupled to one initial surface;
- Runenwerk/Winit policy leaks into backend construction;
- a non-render compute consumer cannot initialize the backend independently.

Future owner:

```text
instance/adapter/device/queue        RunenGPU WGPU backend
surface resource/acquire/present     RunenGPU WGPU backend
window/event-loop/lifecycle policy   Runenwerk
output image/color meaning           RunenRender
```

### Device capabilities

Current device creation:

- enables timestamp queries when supported;
- uses WGPU default limits;
- disables experimental features;
- uses performance memory hints;
- returns `anyhow::Result`.

Future classification:

```text
normalized capabilities              RunenGPU core
WGPU feature/limit mapping            RunenGPU WGPU backend
product capability selection          Runenwerk
render quality selection              RunenRender/Runenwerk policy
```

### Surface registry

The current surface module combines:

- `RenderSurfaceId` allocation;
- `NativeWindowId` mapping;
- ECS resource storage;
- WGPU surface configuration helpers;
- fixed FIFO present policy;
- primary-window product policy;
- saturating ID allocation.

This must split into:

```text
host window -> logical presentation mapping    Runenwerk
low-level surface runtime identity              RunenGPU
logical render target/image intent              RunenRender
```

The existing `RenderSurfaceId` cannot be mechanically retained without deciding
which of those concepts it actually identifies.

## Current identity findings

Current API identities are all produced by one Runenwerk `id_macros::id` macro:

```text
RenderFlowId
RenderPassId
RenderResourceId
RenderFeatureId
RenderFrameProducerId
```

A separate `RenderSurfaceId` is declared in the backend surface module.

Current evidence shows mixed ownership:

| Current ID | Current use | Provisional future classification |
|---|---|---|
| `RenderFlowId` | semantic render-flow definition/invocation | RunenRender semantic ID or redesign |
| `RenderPassId` | compiled pass and transient owner | split semantic render operation from GPU work ID |
| `RenderResourceId` | logical render resources and backend allocator | split render logical resource from `GpuResourceId` |
| `RenderFeatureId` | built-in scene/UI/world/cave/material/product features | Runenwerk producer/product concept or retire |
| `RenderFrameProducerId` | frame/surface contribution producer | RunenRender contribution producer or Runenwerk adapter identity |
| `RenderSurfaceId` | host window mapping and WGPU surface lookup | split logical target, host mapping, and `GpuSurfaceId` |

Current feature constants use safe raw reconstruction and panic on invalid constants.
Current surface allocation uses saturating arithmetic and loops until a free ID is
found. Current resource allocation uses `RenderPassId` and `RenderResourceId` in
an ECS resource.

Therefore, the old identity phase is not safe to implement as a renderer-local
mechanical migration.

## Current graph findings

The graph root contains:

```text
capabilities
diagnostics
execution_plan
flow_graph
merge
pass_graph
pass_shape
planning
prepared_validation
resource_graph
resource_lifetimes
validation
validation_builtin_ui
```

The execution backend accepts pass variants including:

```text
Compute
Fullscreen
Copy
Present
BuiltinUiComposite
Graphics
```

This reveals two distinct models currently combined:

```text
render-semantic planning
GPU execution planning
```

Required split:

### RunenGPU work graph

Contains only bounded GPU execution concepts:

```text
compute
render
copy
clear
resolve
present
resources/access/lifetimes/capabilities
```

### RunenRender plan

Contains image-formation meaning:

```text
views
targets
providers
materials/media
emitters
overlays
visibility/transport/reconstruction intent
```

Product UI, scene, material, SDF, world, cave, editor, and application semantics
must not become variants in either framework core.

## Current feature findings

The feature module defines built-ins for:

```text
scene route
editor picking
UI
world draw
cave interior
procedural world
detail
material
deformation
wind fields
particle VFX
world visual
```

Prepared feature resources derive ECS component/resource traits. The feature
registry also owns product ordering and fallback policy.

Classification:

```text
feature source and domain validation      domain/Runenwerk adapters
prepared generic render contributions     RunenRender
GPU execution nodes                       RunenGPU
product fallback and enablement policy     Runenwerk
```

`RenderFeatureId` is therefore not a general GPU identity.

## Current frame and producer findings

The frame root contains potentially reusable contribution concepts alongside:

- product selection;
- product surface semantics;
- UI-specific prepared contribution re-exports;
- host/window surface information;
- ECS-backed registries.

Frame preparation currently:

- accesses `WorldMut` and `SceneResource`;
- removes and polls the shader filesystem registry;
- writes reload messages into the world UI overlay;
- reads native window lifecycle resources;
- maps native windows to render surfaces;
- collects flow-declared state through `TypeId` and `Any`;
- resolves fixed-time and host state;
- builds product selections and UI bindings;
- publishes ECS resources.

Required target:

```text
Runenwerk/domain adapters
    resolve ECS, windows, time, source assets, and product policy
    publish explicit prepared contributions

RunenRender
    compose and validate prepared render scene

RunenGPU
    execute lowered GPU work
```

No renderer plan reaches back into host state.

## Current runtime and plugin findings

`RenderPlugin` currently initializes and schedules:

- scene resources;
- shader registry;
- flow/fragment/feature registries;
- UI frame, atlas, surface submission, and viewport bindings;
- editor picking;
- world, cave, procedural, material, VFX, deformation, and wind features;
- SDF residency and raymarch acceleration;
- world LOD and runtime caches;
- prepared frames/product selection;
- GPU residency and dynamic texture registries;
- pipeline and backend allocators;
- render surface registry;
- debug, capture, timing, inspection, startup, and pacing state.

This plugin remains Runenwerk-owned composition.

The framework targets are extracted from beneath it. `RenderPlugin` itself is not
moved to RunenRender or RunenGPU.

## Current resource and residency findings

The resource root contains:

```text
descriptors
dynamic targets
imports
lifetimes
transients
usages
```

The backend allocator stores texture/buffer entries and transient claims in an ECS
resource using renderer IDs, but currently does not establish a complete backend
resource lifetime or stale-handle contract.

The renderer also has separate generic GPU residency, SDF residency, world caches,
dynamic target registries, and pipeline caches.

Required distinction:

```text
RunenGPU
    backend-neutral resource/access/lifetime contract
    concrete backend allocation and execution

RunenRender
    render-specific realization/residency/cache policy

Runenwerk/domains
    authoritative source reconstruction and domain residency policy
```

## Current GPU primitive findings

The GPU primitives root contains:

```text
compaction
counters
draw arguments
plan
scan
```

These are candidates for either:

- general RunenGPU algorithm helpers when proven independent and reusable; or
- consumer-owned kernels using RunenGPU execution.

They must not be moved based only on the word `gpu`. Each helper needs independent
consumer, semantic, and release-pressure evidence.

## Current shader and ABI findings

The render shader module currently owns:

- filesystem roots and path normalization;
- file modification tracking;
- poll throttling and forced reload;
- shader registry and handles;
- last-loaded source;
- engine overlay messages;
- test temporary directories and file IO.

Required split:

```text
source asset discovery/watch/reload policy   Runenwerk/domain host
shader source/interface meaning              contributing domain or RunenRender
shader admission and backend realization     RunenGPU
product last-known-good/fallback policy       Runenwerk
```

Current `GpuUniform` and `GpuStorage` derives and parameter modules mix logical
parameters, raw layout, bytemuck representation, and generated engine paths.
They require a separate ABI decision after at least two consumers are inventoried.
No macro package is assumed.

## Current material compiler findings

The material compiler explicitly identifies itself as an engine-owned Material IR
to WGSL compiler. It validates material IR, generates WGSL, validates WGSL, and
creates scene/material resource-binding plans.

Provisional ownership:

```text
material authoring IR/translation       material domain or Runenwerk
renderer material/scattering contract   RunenRender
WGSL/backend pipeline realization       RunenGPU
```

The compiler must not be moved wholesale into either framework before the
material-authoring and renderer-material boundary is accepted.

## Current inspection findings

The inspection module contains neutral-looking facts mixed with product and domain
inspection:

```text
graph/resource/pass provenance and timings
capture and texture inspection
material production/handoff
SDF production/raymarch/residency
world runtime
scale visibility/production
temporal and upscaling product policy
artifact export and readiness
```

Classification:

```text
GPU capability/allocation/submission facts     RunenGPU
render plan/path/cache/reconstruction facts     RunenRender
material/SDF/world/editor/product inspection    Runenwerk/domains
artifact path/export/presentation policy        Runenwerk
```

Deterministic planning proof remains separate from environment-dependent GPU and
window timings.

## RunenUI relationship

RunenUI and RunenRender remain independent peers.

RunenUI owns semantic UI, layout, hit testing, accessibility, text shaping, and
renderer-neutral paint output. RunenRender may consume a prepared overlay scene
through a Runenwerk-owned adapter after the paint protocol stabilizes.

RunenUI core/runtime do not depend on RunenGPU merely because an optional backend
may use GPU acceleration.

## RunenSDF relationship

RunenSDF owns mathematical field contracts and CPU reference behavior. RunenRender
consumes prepared providers through an adapter. RunenGPU may execute provider
realization kernels without owning SDF meaning.

The active RunenSDF external transfer may continue independently. Shared
Runenwerk manifests, lockfiles, and canonical planning files require serialized
merge ownership.

## Module-family disposition matrix

| Current area | Future owner | Required action |
|---|---|---|
| `api/ids` | split GPU/renderer/Runenwerk | redesign after complete consumer inventory |
| `api/flow`, passes | RunenRender semantic plan plus RunenGPU lowering | split models |
| `api/resources`, `resource` | RunenGPU resources plus renderer logical resources | separate identities and lifetimes |
| `backend/device`, `wgpu_ctx` | RunenGPU WGPU backend | remove Winit/surface-coupled construction |
| `backend/surface` | split RunenGPU/RunenRender/Runenwerk | separate surface, target, and host mapping |
| `backend/execution` | RunenGPU work execution | remove `BuiltinUiComposite` semantics |
| `backend/resource_allocator` | RunenGPU backend | remove ECS and renderer-ID coupling |
| `graph` | split RunenRender plan and RunenGPU work graph | remove product/domain variants |
| `frame` | RunenRender prepared scene/contributions plus Runenwerk product extraction | split UI/product/window facts |
| `features` | Runenwerk/domain producers | lower to generic render contributions |
| `gpu_primitives` | evidence-dependent | classify each algorithm independently |
| `params` and macros | RunenGPU ABI candidate or consumer-owned | decide after ABI inventory |
| `pipelines` | split semantic render keys and RunenGPU backend pipelines | redesign |
| `renderer` | RunenRender GPU realization | remove WGPU/Winit/UI/product reach-through |
| `residency` | RunenRender realization or Runenwerk domain policy | classify each cache/source |
| `shader` | split host source policy and RunenGPU realization | remove filesystem from frameworks |
| `material_compiler` | material/Runenwerk plus renderer material contract | do not move wholesale |
| `procedural` | domain/Runenwerk authoring and adapters | keep out of framework cores |
| `inspect` | split GPU/render/domain/product facts | preserve provenance |
| `runtime`, `plugin` | Runenwerk | retain lifecycle/orchestration |

This matrix is module-family authority. A file-level matrix remains required from
local `rg --files` and consumer commands before G1 activation.

## Corrected roadmap

```text
S0 complete inventory and command baseline
G1-G9 internal RunenGPU decomposition and conformance
GX standalone RunenGPU transfer and Runenwerk cutover
R1-R8 internal RunenRender decomposition and conformance
RX standalone RunenRender transfer and Runenwerk cutover
adapters
advanced field-ray renderer
```

## Mandatory local gate before G1

```text
cargo metadata --format-version 1 --locked
cargo tree -p engine --edges normal,build,dev
rg --files engine/src/plugins/render engine_render_macros assets/shaders engine/tests engine/examples engine/benches
rg -n 'wgpu::|winit::|RawWindowHandle|Surface|Device|Queue|CommandEncoder|ComputePass|RenderPass' engine apps domain adapters
rg -n 'Render(Flow|Pass|Resource|Feature|FrameProducer|Surface)Id|try_from_raw|\.raw\(' engine apps domain adapters
rg -n 'plugins::render|RenderPlugin|PreparedRenderFrame|SurfaceFrameSubmission|GpuUniform|GpuStorage' engine apps domain adapters
rg -n 'TypeId|ecs::|World|Resource|material_graph|world_sdf|ui_|Ui|editor|procedural' engine/src/plugins/render
cargo +stable fmt --all --check
cargo test -p engine --lib --locked
cargo test -p engine --tests --locked
cargo clippy -p engine --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git status --short --branch
```

Also identify and run repository-authoritative:

- MSRV validation;
- WGSL/shader validation;
- headless WGPU test/example;
- surface/window test/example;
- device-loss test strategy;
- renderer/GPU benchmarks.

## Gate result

```text
ownership direction                    decision-complete
module-family connector classification substantial and recorded
file-level command inventory            pending
current validation baseline             pending
G1 implementation authorization         blocked
external source movement                 forbidden
```

## Next safe action

Rebase this architecture branch after the active SDF planning PR updates shared
planning authority. Run the mandatory S0 command gate locally. Then update and
activate exactly `PT-RUNENGPU-G1`.

Do not implement the retired RunenRender R1, move WGPU into RunenRender, or create
external RunenGPU source before internal G1-G9 conformance.
