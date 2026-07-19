---
title: RunenRender Complete Semantic Inventory
description: Current renderer module, API, graph, WGPU, surface, shader, producer, domain-adapter, diagnostics, test, and extraction-readiness investigation.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Complete Semantic Inventory

## Question

Which parts of `engine/src/plugins/render` are reusable renderer framework code,
which are WGPU backend implementation, which are Runenwerk adapters/product
policy, and what internal decomposition is required before external extraction?

## Verdict

```text
EXTRACTION CANDIDATE: yes
MOVE engine/src/plugins/render AS-IS: forbidden
TARGET REPOSITORY: Crystonix/RunenRender
TARGET PACKAGES: runenrender_core, runenrender_wgpu, runenrender_macros
FIRST REQUIRED IMPLEMENTATION: internal neutral-core separation
EXTERNAL SOURCE MOVEMENT: blocked by anti-cheating proof
```

The current renderer contains credible graph, resource, GPU parameter, pipeline,
WGPU, surface, shader, and inspection foundations. It is nevertheless one large
engine plugin whose public authoring path and execution path directly depend on
ECS, Runenwerk lifecycle/time/window state, scene/world/material/SDF/UI/editor
features, Winit, filesystem hot reload, debug products, and application startup
policy.

The correct extraction is an internal strangler cutover through generic renderer
contracts, not a directory move.

## Reviewed baseline

Repository: `Crystonix/Runenwerk`

Published main head:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Repository-family charter base:

```text
docs/repository-family-charter
d14fc0e07ace3c2123ff70fc748b0694114cb6e1
```

## Evidence limits

This investigation inspected current module roots and representative control-flow
and API authority files through GitHub. The connector cannot list the complete
repository tree reliably, run Cargo, compile shaders, initialize a GPU, execute
examples, or run repository-wide grep.

The ownership/classification decisions are complete enough to define the next
architecture/specification phase. Exact file, shader, test, benchmark, and
consumer lists require the mandatory local inventory before source changes.

## Current package boundary

Rendering currently lives inside the `engine` package rather than an independent
crate. `engine/Cargo.toml` directly depends on:

```text
wgpu
winit
naga
rusttype
image
ktx2
bytemuck
pollster
serde/serde_json/ron/postcard
glam
ecs
scheduler
material_graph
world_sdf/world_ops/spatial/scene/product/asset/graph
network/replay packages
former/current UI packages
```

This package graph cannot become RunenRender unchanged.

`engine_render_macros` is a separate proc-macro package deriving `GpuUniform` and
`GpuStorage`, but generated paths target `engine::plugins::render`.

## Complete top-level renderer inventory

Current render module root exposes:

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
texture_upload
plugin
```

### api

Submodules:

```text
bindings
dispatch
flow
handles
ids
passes
resources
```

Current `RenderFlow` and pass builders directly own or reference:

- global atomic flow-ID allocation;
- ECS resource declarations and state projections;
- surface-size state projections;
- fixed-time config/state/catchup budget;
- fixed-step regions;
- procedural and GPU-primitive helpers;
- material-scene shader bundles;
- built-in UI composite passes;
- `MainSurfaceOnly` and `OffscreenProductsOnly` scope;
- `anyhow` validation convenience;
- runtime frame-data projection.

Disposition: `REDESIGN_SPLIT`.

A neutral graph builder moves to `runenrender_core`. ECS state projection,
fixed-time expansion, product view selection, material shader selection, and UI
composition remain Runenwerk adapters.

### backend

Submodules:

```text
device
surface
wgpu_ctx
```

Current backend:

- creates WGPU instance/adapter/device/queue;
- requires a Winit `Arc<Window>` for initialization;
- selects high-performance adapter and default limits internally;
- stores WGPU surfaces and configs;
- exposes timing capability;
- uses `anyhow`;
- current surface registry separately combines `NativeWindowId`, ECS derives,
  renderer IDs/lifecycle, diagnostics, and WGPU config helpers.

Disposition: `REDESIGN_WGPU_BACKEND` plus `RUNENWERK_HOST_ADAPTER`.

`runenrender_wgpu` owns WGPU instance/adapter/device/queue/surfaces, but has no
Winit dependency. Runenwerk supplies a generic raw-window-handle/WGPU surface
target and owns the window/event-loop lifetime and mapping.

### composition

Submodules:

```text
fragment_registry
fragment_validation
fragments
hot_reload
integration
```

The integration registry is an ECS resource and compiles flows using runtime
default capabilities; invalid flows are skipped with tracing warnings. Hot reload
belongs to engine filesystem/lifecycle policy.

Disposition:

- neutral fragment values/validation may move after review;
- ECS registry, synchronization, filesystem watching, and reload policy stay in
  Runenwerk;
- renderer core returns structured compile results and never silently skips an
  invalid flow as application policy.

### features

Submodules:

```text
caves
detail
editor_picking
particle_vfx
ui
world
```

The module defines built-in product feature IDs, ordering, dependencies,
fallbacks, and ECS resource wrappers for scene, world, caves, procedural world,
detail, materials, deformation, wind, particles, UI, and editor picking.

Disposition: `RUNENWERK_DOMAIN_ADAPTERS`.

Only generic contribution status, fallback, producer identity, replacement, and
provenance concepts may move after renaming and removing built-in registration.
There is no built-in product feature registry in RunenRender.

### frame

Submodules:

```text
context
contribution_diagnostics
contribution_registry
contributions
packet
product_selection
product_surface
view
```

The module re-exports UI contribution types and contains product-selection and
product-surface policy.

Disposition: `REDESIGN_SPLIT`.

RunenRender core owns generic `FrameId`, `ViewId`, `TargetId`, validated frame
inputs, flow invocations, producer contributions, and frame plans. Runenwerk owns
product selection, main/offscreen product semantics, native-window mapping, and
UI/domain contribution formation.

### gpu_primitives

Submodules:

```text
compaction
counters
draw_args
plan
scan
```

Disposition: `CANDIDATE_GENERIC_RENDER_UTILITY`.

Move only after a detailed source/test review proves no scene/world/ECS/product
semantics. Generic scan/compaction/indirect-argument descriptors may belong in
RunenRender; policy selecting them remains Runenwerk.

### graph

Submodules:

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

The graph is the strongest `runenrender_core` candidate, but current compiled
plans contain:

- `BuiltinUiComposite` and `CompiledUiCompositeExecutionPlan`;
- direct `UI_RENDER_FEATURE_ID` dependency;
- host `TypeId` state requirements and state-driven dispatch;
- Runenwerk fixed-step regions;
- `MainSurface`/`OffscreenProduct` view categories;
- product feature IDs;
- imported/project product semantics.

Disposition: `REDESIGN_CORE`.

Remove all product/host semantics before extraction. Core graph supports only
generic compute, graphics, fullscreen, copy, and present operations, explicit
prepared inputs, generic targets/views, capabilities, resource lifetimes, and
structured diagnostics.

### inspect

Submodules include generic and product-specific concerns:

```text
artifacts/budgets/capture/config/graph_dump/pass_provenance/plan/prepared_frame/
producer/readiness/report/resource_inspector/texture_view/timings

material_handoff/material_production/product_visual_evidence/query_snapshot/
ray_query/scale*/sdf*/temporal*/world_runtime
```

Disposition: `REDESIGN_SPLIT`.

Generic graph dumps, pass/resource provenance, backend capability, capture facts,
and timing evidence may move. SDF, material, world, query snapshot, product
visual, editor/debug policy, artifact export paths, and diagnostics presentation
stay in Runenwerk.

### material_compiler

Current module explicitly owns engine material IR-to-WGSL compilation,
material-scene table generation, material identities, bindings, validation, and
Naga/WGSL output.

Disposition: `RUNENWERK_MATERIAL_ADAPTER`.

RunenRender may own generic shader/pipeline realization. It does not own
`MaterialIr`, material document IDs, material instance tables, or material output
target semantics.

### params

Submodules:

```text
gpu_params
gpu_value
```

These define generic GPU uniform/storage conversion and ABI helpers.

Disposition: `RUNENRENDER_CORE` after API/safety/conformance review.

The current derives justify a third package, `runenrender_macros`, because Rust
proc macros require a separate crate and explicit GPU layout derivation has
independent renderer value. Generated paths must target renamed packages and be
proven against WGSL/WGPU layout rules.

### pipelines

Submodules:

```text
cache
flow_keys
keys
specialization
```

Disposition: `REDESIGN_SPLIT`.

Backend-neutral pipeline keys/descriptors may live in core. WGPU pipeline objects
and caches live in the WGPU backend. Product flow/material keys stay in
Runenwerk.

### procedural

Submodules:

```text
authoring
camera
descriptors
lowering
population
validation
```

Disposition: `RUNENWERK_PROCEDURAL_RENDER_ADAPTER` by default.

A generic pass-builder helper may move only if it reduces to ordinary core graph
operations without camera/population/product semantics.

### renderer

The current renderer implementation is a large WGPU executor that imports:

- WGPU and Winit;
- prepared Runenwerk frames/flows/views;
- UI font atlas, UI render data, UI shaders, and viewport bindings;
- shader registry and filesystem-backed handles;
- product debug/capture/inspection types;
- material/world/mesh/SDF feature preparation;
- `anyhow` and wall-clock timing.

Disposition: `REWRITE_SPLIT`.

Generic WGPU graph execution/resource/pipeline code moves to
`runenrender_wgpu`. UI/material/world/SDF/editor/debug preparation becomes
Runenwerk adapters. The final backend executor accepts only RunenRender prepared
plans and generic resource/update inputs.

### residency

Current generic handle/resource shell must be reviewed alongside extensive SDF,
world, dynamic texture, and material residency users.

Disposition: `REDESIGN_SPLIT`.

Generic GPU resource generation, stale-handle, budget, and eviction machinery may
move. World/SDF/material residency policy and source reconstruction remain
Runenwerk.

### resource

Submodules:

```text
descriptors
dynamic_target
import
lifetime
transient
usages
```

The descriptor model is broadly renderer-neutral but currently embeds Rust
`TypeId`/`GpuParams`, surface defaults, imported history/product semantics, and
saturating size arithmetic.

Disposition: `REDESIGN_CORE`.

Compiled core descriptors use explicit validated sizes/layouts/formats/usages and
repository-local IDs. Typed authoring helpers may produce those descriptors, but
compiled plans do not depend on ECS or host state extraction.

### shader

Current shader module combines:

- generic handles/types;
- registry and source revisions;
- filesystem roots/path normalization;
- file polling/throttling/force reload;
- Runenwerk shared reload status;
- tracing and test temp directories.

Disposition: `REDESIGN_SPLIT`.

RunenRender core owns shader identity/source/revision/interface descriptors.
RunenRender WGPU owns WGSL validation/module creation/pipeline errors. Runenwerk
owns filesystem discovery, paths, watches, hot reload, asset registry integration,
and last-known-good application policy.

### runtime

Submodules:

```text
debug_eval
dynamic_targets
dynamic_texture_uploads
frame_prepare
frame_submit
```

Current prepare path reads ECS world, scene manager, overlay UI, shader hot reload,
native windows, fixed-time state, product selections, viewport bindings, feature
resources, and Runenwerk diagnostics. Submit path manages startup readiness,
frame pacing, UI font/shaders, product diagnostics/capture export, environment
logging, scheduler global logging, and WGPU execution.

Disposition: `RUNENWERK_RENDER_HOST`.

The final Runenwerk plugin uses RunenRender public APIs; none of this lifecycle
module moves into renderer core.

### texture_upload

Current file loads material-specific KTX2 artifacts from filesystem paths and
validates Runenwerk prepared material descriptors.

Disposition: `RUNENWERK_MATERIAL_ASSET_ADAPTER`.

RunenRender WGPU may accept generic validated texture upload descriptors/bytes.
It does not read material artifact paths or understand material IDs.

### plugin

Current `RenderPlugin` initializes a large set of ECS resources for Runenwerk
scene/world/material/SDF/UI/editor/debug/product features and installs Runenwerk
systems ordered against Runenwerk and UI system sets.

Disposition: `RUNENWERK_INTEGRATION`.

A reusable renderer library does not expose this engine plugin.

## Macro inventory and decision

`engine_render_macros` currently derives:

```text
GpuUniform
GpuStorage
```

It generates raw layout types, bytemuck traits, GPU parameter conversion, and
layout marker implementations. Generated paths target `engine::plugins::render`.

Decision:

- create `runenrender_macros` in the RunenRender repository;
- target `runenrender_core` with package-rename support;
- preserve generics/where clauses where safely possible or emit precise compile
  diagnostics;
- prove uniform/storage layout against WGSL/WGPU alignment cases;
- reject unsupported field types at compile time;
- expose no Runenwerk path;
- maintain external compile-pass/compile-fail conformance.

## Neutral core decisions

### IDs

RunenRender owns opaque repository-local IDs:

```text
RenderGraphId/FlowId
PassId
ResourceId
ProducerId
FrameId
ViewId
TargetId
SurfaceId
ShaderId
PipelineId
```

Exact names are normalized during design closure.

Do not depend on Runenwerk `id`/`id_macros`. Global atomic allocation is not the
canonical deterministic graph identity source. Builders/registries allocate
within an explicit owner/namespace and handle exhaustion structurally.

Runtime IDs are not stable persistence identities by default.

### Graph operations

Core operations are:

```text
compute
graphics
fullscreen raster convenience
copy
present
```

There is no built-in UI pass, material pass, SDF pass, world pass, editor pass,
or product feature variant.

### Prepared inputs

Runenwerk resolves ECS/application state before calling RunenRender.

Core receives explicit:

- uniform/storage bytes or typed prepared payloads;
- texture/buffer imports and updates;
- dispatch counts;
- draw/indirect arguments;
- view/target descriptors;
- flow invocations;
- producer contributions;
- capability requirements.

Remove `with_state<T: ecs::Resource>`, `uniform_from_state`,
`dispatch_from_state`, and host `TypeId` requirements from core.

### Iteration/fixed step

RunenRender core does not own Runenwerk fixed time. The first neutral API does not
include fixed-step regions.

Runenwerk expands required substeps into explicit flow/pass invocations with
prepared iteration uniforms before submission. A generic iteration-region API may
be introduced later only if independent render consumers need it.

### Views and products

Core has explicit view/target IDs and invocation lists. It does not know
`MainSurface`, `OffscreenProduct`, product selections, or viewport embed policy.
Runenwerk forms those invocations.

### Producers

Core supports generic producer/contribution identity, provenance, ordering,
replacement/removal, and capability requirements. It does not register built-in
feature families or semantic dependencies.

## WGPU backend decisions

### Initialization

Support headless/offscreen initialization without a window.

Use explicit configuration for:

- power preference;
- fallback adapter policy;
- required/optional features;
- required limits;
- memory hints;
- trace/debug policy.

Do not hard-code engine labels/policy.

### Window independence

`runenrender_wgpu` does not depend on Winit. It accepts a WGPU surface target or
raw-window-handle-compatible host value according to the reviewed WGPU API.
Runenwerk owns the Winit window and event loop.

### Surface ownership

```text
Runenwerk host:
  native windows, event loop, DPI/resize/visibility, NativeWindowId mapping

RunenRender core:
  opaque SurfaceId, generic desired target/presentation facts, reports

RunenRender WGPU:
  wgpu::Surface, capabilities, configuration, acquire, present, backend errors
```

The backend must classify lost/outdated/timeout/out-of-memory/device-lost outcomes
and return recovery requirements. Runenwerk decides product retry/recreate/close/
shutdown policy.

### Resource and device loss

The backend owns GPU resource generations, pipeline caches, submissions, in-flight
retention, and backend reconstruction reports. Runenwerk/domain adapters own the
source data needed to repopulate resources after device loss.

No ECS resource lifetime is implicit renderer lifetime.

## Shader and asset boundary

Core/WGPU accept validated source/interface descriptors and explicit revisions.
Runenwerk resolves files/assets and chooses last-known-good/hot-reload policy.

Material/SDF/UI/world shaders are producer assets, not renderer semantic kinds.

## Error policy

Replace public `anyhow` and panic-based builder lookups with structured errors:

```text
GraphBuildError
GraphValidationError
FrameInputError
ResourceError
ShaderError
PipelineError
BackendInitError
SurfaceError
DeviceLostError or structured terminal state
SubmissionError
```

Exact taxonomy may consolidate, but consumers must branch without parsing
strings. Invalid flows are not silently skipped by renderer core.

## Threading policy

- core graph/planning is deterministic and usable without a GPU;
- WGPU backend has one explicit mutable execution owner/render thread unless a
  later design proves concurrent access;
- `Device`/`Queue` sharing follows WGPU guarantees but does not create hidden
  global renderer state;
- filesystem watchers, ECS worlds, and Winit callbacks stay outside backend locks;
- no global flow-ID, shader-registry, telemetry, or backend singleton authority.

## Diagnostics and persistence

RunenRender diagnostics use `runenrender.*` codes and structured IDs/provenance.
Runenwerk inspection UIs and artifact exporters consume them.

No current graph, runtime ID, shader handle, pipeline cache, or WGPU resource is a
stable persisted format. A future serialized graph/artifact format requires its
own schema/version/migration design.

## Module disposition matrix

| Current module | Disposition | Final owner |
|---|---|---|
| `api` | Split neutral graph builder from host state projection | RunenRender core + Runenwerk |
| `backend` | Remove Winit/anyhow, support headless/surface targets | RunenRender WGPU + Runenwerk host |
| `composition` | Move neutral fragments only; keep registry/reload integration | Core + Runenwerk |
| `features` | Remove built-ins from framework | Runenwerk adapters |
| `frame` | Split generic frame/invocation/contribution from products/UI | Core + Runenwerk |
| `gpu_primitives` | Move only proven neutral scan/compaction/indirect utilities | Core/WGPU or stay |
| `graph` | Remove UI/state/fixed-time/product views | RunenRender core |
| `inspect` | Split generic provenance/capture/timing from product evidence | Core/WGPU + Runenwerk |
| `material_compiler` | Keep material semantics | Runenwerk material adapter |
| `params` | Move after ABI conformance | RunenRender core |
| `pipelines` | Split descriptors/keys from WGPU objects/product keys | Core/WGPU + Runenwerk |
| `procedural` | Keep by default; move only ordinary graph sugar | Runenwerk |
| `renderer` | Rewrite/split generic WGPU executor from feature prep | WGPU + Runenwerk adapters |
| `residency` | Split generic GPU cache/generation from SDF/world policy | WGPU/core + Runenwerk |
| `resource` | Remove TypeId/state coupling, validate explicit descriptors | Core |
| `shader` | Split descriptors/WGPU compile/filesystem reload | Core/WGPU/Runenwerk |
| `runtime` | Keep engine lifecycle | Runenwerk |
| `texture_upload.rs` | Keep material artifact loading | Runenwerk material adapter |
| `plugin.rs` | Keep engine plugin composition | Runenwerk |
| `engine_render_macros` | Rename and migrate after ABI proof | RunenRender macros |

## Independent conformance requirements

### Core

- graph construction and deterministic validation;
- duplicate/missing/cycle/access/lifetime errors;
- generic pass/resource/target/view planning;
- producer replacement/removal and provenance;
- explicit frame-input validation;
- no ECS/WGPU/Winit/Runenwerk dependency;
- external producer using public APIs;
- stable/MSRV tests and planning benchmarks.

### Macros

- uniform/storage layout fixtures covering scalars/vectors/matrices/arrays/padding;
- WGPU/WGSL layout agreement;
- package rename;
- generics/support errors;
- external compile-pass/fail tests.

### WGPU

- headless adapter/device initialization;
- configured feature/limit negotiation;
- resource/pipeline creation and reuse;
- compute and raster/copy execution;
- offscreen readback proof;
- optional surface attach/configure/acquire/present;
- multiple surfaces;
- resize/outdated/lost/timeout/out-of-memory/device-loss behavior;
- shutdown and in-flight retention;
- backend/platform evidence clearly separated from deterministic core tests.

### Runenwerk anti-cheating proof

- engine owns no renderer-internal private access;
- all ECS state projection occurs before renderer submission;
- scene/material/SDF/UI/editor features submit generic work;
- no product-specific pass/feature enum remains in renderer packages;
- renderer packages build/test without engine;
- Runenwerk applications work through the same public seam intended externally.

## Required local inventory gate

Before `PT-RUNENRENDER-002` implementation/spec closure, run:

```text
git status --short --branch
git rev-parse HEAD
cargo metadata --format-version 1 --locked
cargo tree -p engine --edges normal,build,dev
find engine/src/plugins/render engine_render_macros assets/shaders engine/examples engine/tests engine/benches -type f | sort
rg -n 'ecs::|crate::runtime|crate::plugins::scene|material_graph|world_sdf|ui_|Ui|NativeWindowId|winit::|wgpu::|anyhow|panic!|unwrap\(|expect\(' engine/src/plugins/render engine_render_macros
rg -n 'plugins::render|RenderFlow|RenderPlugin|PreparedRenderFrame|Gfx' apps domain net adapters engine/tests engine/examples
cargo test -p engine --lib --locked
cargo test -p engine --tests --locked
cargo clippy -p engine --all-targets --locked -- -D warnings
```

Run every repository-authoritative docs, MSRV, shader-validation, headless, and GPU
command. Record adapter/device/platform for GPU evidence.

## Next design/specification phase

`PT-RUNENRENDER-002` must convert this inventory into exact small implementation
phases. Required sequence:

```text
RENDER-R1 neutral IDs/errors and package-internal dependency map
RENDER-R2 graph/resource descriptors without UI/ECS/host state/fixed time/product views
RENDER-R3 explicit prepared frame inputs and generic producer contributions
RENDER-R4 GPU params and runenrender_macros ABI conformance
RENDER-R5 shader descriptors and filesystem/hot-reload separation
RENDER-R6 WGPU headless device/resource/pipeline executor
RENDER-R7 WGPU surface target and structured surface/device-loss contract
RENDER-R8 split generic diagnostics/capture/provenance from Runenwerk product inspection
RENDER-R9 migrate scene/material/SDF/UI/editor/runtime to public adapters
RENDER-R10 internal package anti-cheating and performance proof
```

No external repository source movement occurs before R10 passes.

## Stop conditions

Stop implementation if:

- core still imports ECS, WGPU, Winit, Runenwerk, UI, SDF, scene, or material
  authoring;
- WGPU backend requires Winit rather than generic surface targets;
- product feature/pass/view variants remain in core;
- host `TypeId` state extraction remains in compiled plans;
- invalid flows are skipped instead of reported;
- resource/device-loss reconstruction ownership is unresolved;
- Runenwerk uses private renderer internals during internal proof;
- GPU runtime validation is unavailable for backend behavior changes;
- external extraction would retain an original renderer or compatibility facade.

## Gate status

```text
Top-level module inventory: complete
Representative graph/API/control-flow inspection: complete
WGPU/surface inspection: complete
Macro inspection: complete
Domain/product coupling classification: complete
Target module disposition: complete
Exact file/shader/test/benchmark inventory: local verification required
Command/GPU validation: not run; connector limitation
Complete investigation gate: complete for design closure subject to local inventory
Complete design gate: PT-RUNENRENDER-002 required for implementation phases
Internal decomposition authorization: blocked
External extraction authorization: blocked through R10
```

## Next action

After the repository-family charter and this inventory are reviewed and locally
validated, execute one bounded `PT-RUNENRENDER-002` specification PR. Do not move
renderer source to an external repository yet.
