---
title: RunenGPU G4 Context, Program, and WGPU Realization Investigation
description: Exact-current-main census and ownership findings for G4 context admission, program interfaces, backend realization, cache compatibility, and renderer cutover.
status: active
owner: gpu
layer: reports
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../../workspace/specs/pt-runengpu-g4a-context-admission.ron
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU G4 Context, Program, and WGPU Realization Investigation

## Question

What authority exists on exact accepted `main` for GPU context/device creation,
capability admission, programs, interfaces, backend resources, pipelines, caches,
generation affinity, and renderer execution, and how must it be decomposed into G4A,
G4B, and G4C without implementing G5, G7, or RunenRender semantics?

## Resolved baseline and authorization

The exact accepted planning base is:

```text
6bbd341691a34763ef54c8ca059940cac8981265
```

The accepted G3 squash merge is:

```text
39d6fe65a334502bdfba0b1a2ce3b365099fcf28
```

The one accepted commit between those revisions changes only:

```text
.github/workflows/ci.yml
.github/workflows/docs-validation.yml
TESTING.md
docs-site/src/content/docs/workspace/engineering-workflow.md
tools/xtask/src/main.rs
```

It hardens verified-head validation, documentation-build evidence, and repository
workflow guards. It changes no RunenGPU or render source, dependency, manifest,
lockfile, program contract, backend contract, or architecture decision. Issue `#183`
is complete. Issue `#182` is open and authorizes only the G4 decision phase. No G4
branch or pull request existed before this investigation branch, and no G4 Rust
implementation is authorized.

## Evidence inspected

The cold-start inspection covered:

- root `AGENTS.md`, `TESTING.md`, `ARCHITECTURE.md`, `.cargo/config.toml`, and the
  repository-owned `cargo validate` implementation;
- `.github/workflows/ci.yml` and `.github/workflows/docs-validation.yml`;
- parent issue `#167`, issue `#182` and every comment, issue `#183`, PR `#181`, the
  accepted G3 merge, and the G3 implementation closeout;
- the canonical RunenGPU architecture, repository-family architecture, roadmap,
  active/completed work, and RunenGPU/RunenRender decomposition plan;
- exact-current-main files under `engine/src/plugins/gpu/**`, the current render
  backend, adapters, pipelines, renderer, shader registry, API bindings and parameter
  helpers, `engine_render_macros/**`, application, example, benchmark, and test
  evidence referenced by the accepted G3 closeout and current module roots.

Repository source, not issue prose, is the authority for the census below. Historical
S0 and issue prose were used only to identify paths that were then checked against
current `main`.

## Existing accepted RunenGPU boundary

`engine/src/plugins/gpu` contains only backend-neutral G1A/G2/G3 contracts:

```text
api/access
api/capability
api/data
api/errors
api/graph
api/handles
api/resource
api/work
api/work_resource_id
```

The complete-subtree guard currently rejects render, ECS, WGPU, Winit, application,
shader-file, and product vocabulary. The accepted boundary owns:

- normalized logical capabilities and requirements;
- validated logical resource descriptors and kind-typed handles;
- prepared host data;
- checked resource access and exact regions;
- typed immutable compute, render, copy, buffer-zero, query-resolve, and logical
  present operations;
- graph-entry initialization, hazards, typed import/export causality, deterministic
  work preparation, and operation-derived requirements.

It does not yet own a context, adapter/device admission, program/interface contracts,
backend objects, realization registries, cache compatibility, execution, progress,
surfaces, or device-loss recovery.

## Context, adapter, device, and queue census

### Current creation and ownership

`engine/src/plugins/render/backend/wgpu_ctx.rs` is the only accepted-main instance,
adapter, device, and queue owner inspected:

- `WgpuCtx<'window>` owns `wgpu::Instance`, `wgpu::Adapter`, a map of WGPU surfaces,
  public `Arc<wgpu::Device>`, public `Arc<wgpu::Queue>`, and one renderer timing fact;
- `new_async(window)` creates a surface first, requests an adapter compatible with that
  surface, requests a device, then configures the initial surface;
- `new(window)` terminates the asynchronous request by calling `pollster::block_on`;
- `attach_surface`, `detach_surface`, `resize`, `surface_config`, and
  `get_current_texture` mix G7 surface authority into the context owner.

`engine/src/plugins/render/backend/device.rs` owns the request policy:

- timestamp-query support is opportunistically enabled;
- every other feature is omitted;
- `wgpu::Limits::default()` is requested without deterministic requirement merging;
- WGPU memory hints and trace policy are selected directly;
- raw `Arc<Device>` and `Arc<Queue>` are returned.

`engine/src/plugins/render/renderer/mod.rs` exposes this owner through public `Gfx`.
`Gfx::new` requires a `winit::window::Window`, and rendering passes raw device and
queue references into renderer code. This prevents headless context admission,
contains no context identity, and makes surfaces a prerequisite for adapter selection.

### Current normalized facts

The only renderer-local normalized adapter fact is
`RenderBackendTimingCapabilities { timestamp_query: bool }`. Current source does not
own a complete normalized backend family, portability class, adapter class, feature
set, limit set, format facts, alignment facts, degradation result, admission report,
or requirement-by-requirement disposition.

### G4 finding

G4A must replace this authority with headless-first `GpuContext` admission. The
library-facing constructor must remain asynchronous. A blocking terminal belongs to a
Runenwerk host adapter and must not be the only reusable entrypoint. Surface
compatibility may be supplied temporarily as optional host admission evidence, but
surface creation, configuration, acquisition, and presentation remain G7-owned.

## Raw WGPU boundary census

Raw WGPU currently crosses module or public boundaries through at least:

| Current path | Raw authority crossing the boundary | G4 disposition |
|---|---|---|
| `render/backend/wgpu_ctx.rs` | `Instance`, `Adapter`, public `Arc<Device>`, public `Arc<Queue>`, `Surface`, `SurfaceConfiguration`, `SurfaceTexture`, `SurfaceError` | redesign in G4A/G4C; retain only a private backend terminal; surface parts remain temporary until G7 |
| `render/backend/device.rs` | `Adapter`, `Features`, `Limits`, `Device`, `Queue` | migrate admission into private G4A backend |
| `render/backend/formats.rs` | `SurfaceCapabilities`, `TextureFormat` | retain only as G7 migration evidence; G4 uses normalized format facts |
| `render/backend/surface.rs` | `Device`, `Surface`, `SurfaceConfiguration`, WGPU surface enums | do not absorb into G4; isolate host compatibility and leave final replacement to G7 |
| `render/renderer/mod.rs` | `DeviceExt`, broad `wgpu::*`, textures/views/layouts/bind groups and public renderer coupling | migrate GPU realization ownership in G4C; render semantics stay in the render tree |
| `render/renderer/pipeline_cache.rs` | shader modules, layouts, pipelines, samplers, bind groups | delete after G4C migration |
| `render/pipelines/flow_keys.rs` | `wgpu::TextureFormat` in public render cache keys | replace GPU correctness inputs with normalized G4B/G4C descriptors; render semantic inputs remain outside RunenGPU |
| renderer runtime/dynamic-target/setup/execute modules | direct resource creation, command-facing objects, view creation, binding and pipeline use | realization moves in G4C; encoding/submission remains G5 |
| current tests/examples/apps | direct access through `Gfx.ctx.device`, `Gfx.ctx.queue`, WGPU smoke fixtures or renderer helpers | migrate to G4 public contracts or retain as explicitly environment-dependent backend proof |

No raw WGPU type becomes stable RunenGPU public authority. Private backend modules may
own WGPU objects and accept normalized descriptors. Narrow temporary host shims may
exist only with an explicit G7 removal owner.

## Resource and view realization census

### Logical authority

G2/G3 logical descriptors and handles are authoritative under
`engine/src/plugins/gpu/api`. They are process-local, backend-neutral values and do not
currently carry context or generation affinity.

### Synthetic construction

`engine/src/plugins/render/adapters/gpu_work.rs` constructs crate-private G2 handles
from render declarations by calling `GpuBufferHandle::from_descriptor`,
`GpuTextureHandle::from_descriptor`, and `GpuQuerySetHandle::from_descriptor`. It also
synthesizes timing query, resolve-buffer, and readback-buffer handles. These paths use
a temporary `RenderFlowId`-derived work-resource owner and are not valid post-G4
resource admission.

### Backend resources

`engine/src/plugins/render/backend/resource_allocator.rs` is an ECS resource that maps
logical IDs to labels and transient `RenderPassId` claims. It does not own WGPU objects,
context identity, device generation, validated descriptors, or retirement. It is not a
usable G4 realization registry and must be replaced, not wrapped.

Renderer modules own actual WGPU buffers, textures, views, samplers, query sets, bind
groups, and per-flow runtime objects. Material preparation currently stores raw
`BindGroupLayout`, `BindGroup`, textures, views, and samplers in
`PreparedMaterialGpuResources`. Dynamic targets and flow runtime caches similarly own
renderer-local WGPU objects.

### G4 finding

G4C must introduce private, generation-bound realization registries keyed by typed
logical identity plus context and device generation. Resource realization is explicit
through context admission APIs:

- persistent/imported buffers, textures, samplers, views, and query sets are realized
  on explicit admit/realize calls;
- transient resources may be realized in a bounded explicit graph-realization pass;
- texture views are lazy only within an already admitted parent texture and are cached
  by the complete normalized view descriptor;
- pipelines are lazy or explicit on demand after their program/interface descriptors
  are admitted;
- no operation may silently create an undeclared logical resource during G5 encoding.

Foreign-context and stale-generation values are rejected before cache lookup or WGPU
calls. Runtime use-after-retirement and delayed destruction remain G5-owned.

## Program, shader source, and revision census

`engine/src/plugins/render/shader/**` owns a Runenwerk product registry:

- filesystem roots, path discovery, file polling, modification times, forced reload,
  and last error;
- string IDs and path aliases;
- process-local `ShaderHandle(usize)`;
- loaded source text and monotonically increasing revisions;
- product-facing discovery/reload events and status lines.

This registry is valid evidence for Runenwerk-owned source discovery and hot-reload
policy. It is not a reusable program/interface contract. Current renderer code also
contains embedded WGSL fallback strings.

The accepted split is:

- Runenwerk retains path discovery, watching, reload scheduling, last-known-good
  selection, and user-facing status;
- callers pass explicit source values with a typed source key and nonzero source
  revision to G4B;
- WGSL is the first concrete source kind;
- RunenGPU owns source admission, entry-point/interface validation, specialization
  validation, module realization, and structured compiler/backend diagnostics;
- source keys and revisions are runtime/cache inputs, not a stable persistence or wire
  format.

## Binding and interface census

Current binding authority is fragmented:

- `render/api/bindings.rs` uses `GpuWorkResourceId`, `TypeId`, type names, ECS `Any`,
  and projected bytes;
- `PassParamBinding` identifies a projected uniform through a logical resource ID but
  does not declare shader group/binding/interface compatibility;
- renderer setup/execute modules construct WGPU bind-group layouts and bind groups;
- `FlowPassPipelineKey` stores an untyped `bind_group_layout_signature_hash: u64`;
- strings, pass IDs, feature IDs, and runtime hashes participate in cache identity.

G4B must establish one typed interface authority:

```text
GpuBindingKey
GpuBindingDeclaration
GpuBindGroupLayoutDescriptor
GpuPipelineLayoutDescriptor
GpuProgramInterfaceDescriptor
```

A binding key is a validated `(group, binding)` pair with deterministic ordering and
hashing. It is not a string, `TypeId`, path, label, resource ID, or renderer pass ID.
Bindings declare resource kind, access, visibility, optional array count, dynamic
offset policy, minimum binding size where applicable, texture sample/view class,
storage texture access/format class, and sampler class. Runtime binding values must be
checked against the admitted declaration before bind-group realization.

## Parameter and ABI helper census

`engine/src/plugins/render/params/**` and `engine_render_macros/**` currently provide:

- `GpuParams`, `GpuUniform`, `GpuStorage` marker traits;
- `ToGpuValue` and `GpuUniformField` conversion/alignment helpers;
- derives that generate bytemuck-compatible raw structures or byte arrays;
- uniform layout assumptions encoded in the derive implementation;
- a hard dependency from generated code to `engine::plugins::render`.

They are used for Runenwerk/render state projection. They do not validate a declared
WGSL interface, nested structs, runtime-sized arrays, storage-layout rules, matrix
orientation, address-space constraints, minimum binding size, package extraction, or
all alignment cases required for a reusable ABI.

Disposition:

- retain them during G4 as transitional Runenwerk byte-preparation helpers;
- explicitly deny them program-interface or universal ABI authority;
- do not move them into `engine::plugins::gpu` in G4A or G4B;
- do not create a macro package;
- G4B contract tests use explicit descriptors and known byte slices;
- G4C migrates consumers so binding compatibility is validated independently of the
  helper that produced bytes;
- a later separately authorized decision may replace, narrow, or retire the helpers
  after real ABI requirements are proven.

## Pipeline descriptor and cache census

### Current keys

`render/pipelines/keys.rs` uses a string-backed `PipelineKey`.
`render/pipelines/specialization.rs` uses only a compute/render phase and string label.
`render/pipelines/flow_keys.rs` defines renderer-owned keys containing:

- `RenderFlowId`, `RenderPassId`, optional `RenderFeatureId`;
- pass kind;
- string shader identity and integer shader revision;
- untyped layout, material, view, runtime, vertex, and raster hashes;
- raw WGPU color/depth formats;
- sample count and topology.

`FlowPassBindGroupKey` adds an untyped resource-generation signature hash.

### Current caches

`render/renderer/pipeline_cache.rs` owns in-memory hash maps for WGPU shader modules,
bind-group layouts, pipeline layouts, compute pipelines, render pipelines, samplers,
and bind groups. Cache behavior is get-or-create only. It has no context/device
generation, adapter/backend compatibility, source/interface schema, structured
rejection, failure record, or correctness-safe fallback. `render/pipelines/cache.rs`
and `render/backend/pipeline_cache.rs` expose only stats or aliases.

No accepted-main persisted binary pipeline cache was found. Current cache-like values
are process-local derived state. Runtime IDs, hash-map entries, WGPU handles, and debug
strings are not persistence authority.

### G4 finding

G4B owns deterministic compute/render pipeline descriptors and semantic equality/hash.
G4C owns derived in-memory realization caches scoped to one context/device generation.
A persisted backend cache remains unsupported unless separately authorized. Any
backend-provided cache data accepted internally must be guarded by a versioned
compatibility envelope containing every correctness input available from WGPU and the
adapter environment. Rejection is structured and falls back to ordinary realization;
a cache hit changes cost only.

## Identity and generation census

Current relevant identities include:

- owner-scoped `GpuWorkResourceId` and kind-typed G2 handles;
- fragment-local and prepared G3 work-node identities;
- `RenderFlowId`, `RenderPassId`, `RenderFeatureId`, and `RenderSurfaceId`;
- string shader IDs, paths, labels, aliases, pipeline keys, and pass labels;
- process-local `ShaderHandle(usize)`;
- shader and feature integer revisions;
- untyped `u64` signature hashes;
- per-renderer resource-generation hashes;
- native-window and surface IDs.

No `GpuContextId` or `GpuDeviceGeneration` exists on accepted `main`. Existing shader,
feature, and renderer resource revisions are not device generations. G4 must introduce
opaque context identity and nonzero device generation. Logical source/resource
identities remain distinct from backend realizations; they do not silently rebind when
a context or device generation changes.

## Execution sidecar census and G4/G5 split

`render/adapters/gpu_work.rs` correctly makes the G3 prepared graph the sole access,
initialization, hazard, requirement, dependency, and topological-order authority. Its
private sidecar is keyed by `GpuPreparedWorkNodeId` and currently stores:

- complete `CompiledPassExecutionPlan` values;
- timing resolve payload;
- timing readback-copy payload.

The sidecar therefore still carries mixed future authority:

- shader/program selection and revision;
- interface/binding information;
- pipeline specialization and layout inputs;
- renderer execution payload used by current encoding.

Exact split:

- G4B removes program, entry-point, interface, binding-layout, specialization, and
  pipeline descriptor truth from the sidecar and places typed references in generic
  RunenGPU work contracts;
- G4C removes backend resource, shader-module, bind-group-layout, pipeline-layout,
  bind-group, pipeline, and cache truth from the sidecar and renderer;
- after G4C the temporary seam may retain only the G5-owned execution payload that is
  still required to encode accepted operations;
- G5 deletes that residual payload while moving encoding, submission, uploads, query
  resolution execution, progress, completion, readback, and retirement into RunenGPU.

No operation, access, hazard, initialization, requirement, program, interface,
pipeline, cache, or realization truth may remain in the sidecar after its owning slice.

## Surface and presentation census

`render/backend/surface.rs`, the surface map inside `WgpuCtx`, `Gfx::attach_surface`,
`Gfx::render`, and `get_current_texture` currently mix host window policy, WGPU surface
lifetime, image acquisition, render execution, and presentation.

G4A may preserve a temporary optional compatible-surface adapter input only to avoid
breaking current hosts while context admission is cut over. G4A/G4B/G4C do not create
a public surface API, acquire a surface image as reusable authority, own presentation,
or define device-loss reconstruction. G7 owns host raw-handle admission, surface
identity/generation/configuration/acquisition/presentation and surface/device outcomes.
Current surface files are migration evidence, not G4 extraction candidates.

## Applications, examples, benchmarks, and tests

The accepted G3 closeout identifies migrated consumers in:

```text
apps/runenwerk_draw/**
apps/runenwerk_editor/**
engine/examples/**
engine/benches/**
engine/tests/**
```

Current G4 migration must revisit every consumer of:

- `Gfx`, `WgpuCtx`, public device/queue fields, renderer setup and pipeline caches;
- shader registry handles, IDs, revisions and loaded source;
- `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, and derived raw bytes;
- render pipeline/bind-group keys and generation hashes;
- synthetic G2/G3 handle construction;
- direct WGPU resources, views, samplers, query sets, layouts, bind groups, pipelines,
  command-facing values, and environment-dependent smoke helpers.

Proof classification:

### Deterministic

- context/generation identity semantics without a live adapter;
- requirement merge and admission against synthetic normalized adapter facts;
- backend/portability normalization tables;
- descriptors, equality, ordering and hashing;
- typed binding keys and interface compatibility;
- specialization schema/value compatibility;
- cache-key completeness and rejection;
- foreign-context/stale-generation rejection using test backend records;
- compile-pass and compile-fail public contract tests;
- source, dependency, reach-through, sidecar, migration and deletion guards.

### Environment-dependent

- actual WGPU instance/adapter/device request;
- backend/driver-reported feature, limit, format and alignment facts;
- WGSL module and pipeline creation on real adapters;
- resource allocation on real devices;
- backend cache behavior exposed by the selected WGPU version;
- optional current-host compatible-surface smoke;
- native/web availability and adapter-specific diagnostics.

Environment-dependent tests must skip or report unsupported environments explicitly;
they cannot replace deterministic contract proof or become an unconditional developer
machine assumption.

## Current-main file disposition

### G4A — redesign or migrate

```text
engine/src/plugins/render/backend/device.rs
engine/src/plugins/render/backend/wgpu_ctx.rs
engine/src/plugins/render/backend/formats.rs only for normalized non-surface facts
engine/src/plugins/render/renderer/mod.rs context/device/queue reach-through
current context/device consumers in apps, examples, benches, and tests
```

New future-transferable ownership belongs under `engine/src/plugins/gpu` in coherent
context/admission/backend modules. `render/backend/surface.rs` remains a temporary G7
migration boundary and may change only where context admission is detached.

### G4B — redesign or migrate

```text
engine/src/plugins/render/shader/** realization-facing source/revision handoff
engine/src/plugins/render/pipelines/**
engine/src/plugins/render/api/bindings.rs binding/interface-facing portions
engine/src/plugins/render/api/handles.rs current equivalents if still present
engine/src/plugins/render/api/resources.rs current equivalents if still present
engine/src/plugins/render/params/** only at the explicit transitional boundary
engine_render_macros/** only for contract classification and compile proof
renderer setup/prepare/render-flow paths that select programs or describe layouts
application/example/test program and binding declarations
```

Filesystem discovery, watching, last-known-good policy, ECS projection, renderer
semantic selection, and domain values remain outside RunenGPU.

### G4C — migrate and delete replaced authority

```text
engine/src/plugins/render/backend/pipeline_cache.rs
engine/src/plugins/render/backend/resource_allocator.rs
engine/src/plugins/render/renderer/pipeline_cache.rs
renderer dynamic-target/runtime resource realization
renderer shader-module/layout/bind-group/pipeline creation
renderer-owned WGPU resources in prepared packets and caches
render/adapters/gpu_work.rs synthetic handles, owner bridge, and G4-owned sidecar truth
all direct raw Device/Queue/resource/program/layout/pipeline/cache consumers
```

Current render files remain in place when they still own render semantics or G5
execution payload. No wholesale rename, move, extraction, or compatibility wrapper is
permitted.

### G5-owned retention after G4C

```text
command encoding and pass encoders
queue submission
uploads and updates
query-resolution execution
device polling and progress
mapping and asynchronous readback
completion/cancellation
runtime retirement and delayed destruction
residual execution-only sidecar payload
```

### G7-owned retention after G4C

```text
host raw-window/display-handle admission
surface identity/configuration/acquisition/presentation
surface generations and leases
surface/device loss classification and reconstruction reports
Winit and product window/event-loop policy in Runenwerk
```

## RunenRender boundary

G4 is GPU/backend decontamination and substrate work. G4A, G4B, and G4C do not
implement RunenRender semantics. The current render tree is migration evidence and is
not renamed, moved, wrapped, or extracted wholesale. Files change only where GPU
context, program/interface, backend realization, or cache authority is removed.

Separate RunenRender planning and R-phase issues remain independently owned. Early
RunenRender planning may use stabilized G4 contracts, but RunenRender Rust
implementation remains governed by its own accepted sequence. RX is a later mechanical
transfer and clean cutover, not the point where renderer architecture is invented.

## Findings requiring binding decisions

The source census establishes these non-deferrable decisions:

1. context construction must be async and headless-first;
2. context identity and device generation are separate opaque values;
3. normalized admission must be deterministic over normalized adapter facts;
4. WGPU remains private backend implementation;
5. resource/program/pipeline realization is explicit or bounded-lazy by kind, never
   accidental during execution;
6. every backend object is context/generation-affine;
7. source, interface, specialization, and pipeline descriptors require deterministic
   semantic equality and hashing;
8. typed binding keys replace strings, paths, `TypeId`, pass IDs and naked hash
   authority;
9. existing parameter derives remain transitional byte helpers only;
10. cache compatibility must include every correctness fact and reject safely;
11. G4C deletes renderer-owned realization, synthetic handles and G4-owned sidecar
    truth; G5 deletes the residual execution payload;
12. deterministic and environment-dependent proof remain separate;
13. no package extraction, dependency addition, persistence format, G5, G7, or
    RunenRender implementation is justified by current evidence.

The focused design and three ordered specifications bind these decisions.