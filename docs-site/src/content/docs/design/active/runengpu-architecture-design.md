---
title: RunenGPU Architecture Design
description: Decision-complete ownership, workload, resource, capability, WGPU, surface, diagnostics, conformance, and extraction architecture for RunenGPU.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU Architecture Design

## Status

The repository identity, ownership boundary, one-package target shape, dependency direction, WGPU placement, host boundary, and extraction sequence are fixed.

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        complete through issue #172 and PR #173
G3 decision phase                    active through issue #174; implementation unauthorized
G4-G8                                pending and not authorized early
GX external extraction               blocked on accepted G2-G8 evidence
```

The current implementation remains inside Runenwerk until the internal future public boundary is accepted. This document defines architecture; the active phase specification and owning issue authorize bounded work. Issue `#174` authorizes G3 planning only and does not authorize Rust implementation.

## Mission

RunenGPU owns validated execution of GPU resources and workloads.

It answers:

> How are GPU capabilities, resources, accesses, workloads, submissions, results, and backend failures represented and executed safely?

It does not answer:

- what an image should contain;
- how light transport works;
- how a field, simulation, material, UI, ECS entity, or world object behaves;
- how an application schedules gameplay;
- how windows and event loops are managed;
- how product recovery is presented to users;
- how PNG, EXR, MP4, or WebM artifacts are encoded.

## Architectural position

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic GPU execution
            -> WGPU backend
```

Independent non-render consumers may use RunenGPU directly:

```text
field or simulation adapter --+
procedural/image tool ---------+--> RunenGPU --> WGPU
baker or offline job ----------+
```

RunenGPU complements WGPU. It does not reimplement Vulkan, Metal, D3D12, WebGPU, shader compilers, or operating-system window systems.

## Repository and package

```text
repository: dornglut/runen-gpu
package: runen-gpu
crate: runen_gpu
initial backend: WGPU, internal implementation detail
```

Directional standalone repository shape:

```text
runen-gpu/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
├── src/
│   ├── lib.rs
│   ├── capability.rs
│   ├── context.rs
│   ├── data.rs
│   ├── error.rs
│   ├── graph.rs
│   ├── pipeline.rs
│   ├── resource.rs
│   ├── shader.rs
│   ├── surface.rs
│   ├── workload.rs
│   └── backend/
│       └── wgpu.rs
├── tests/
├── examples/
├── benches/
├── docs/
├── conformance/
└── xtask/
```

Module names are directional rather than pre-authorized file names. The public package is one release unit until a second backend, independent consumer, release boundary, or dependency graph proves another package is necessary.

Do not initially create:

```text
runengpu_core
runengpu_wgpu
runengpu_macros
runengpu_testing
runengpu_capture
facade or compatibility packages
```

The external repository is created only in GX after the internal boundary and extraction-readiness evidence are accepted.

## Dependency rules

The future public package must not depend on or expose:

```text
Runenwerk
RunenRender
RunenSDF or source-domain types
RunenECS or another ECS
RunenUI
Winit
scene, view, material, lighting, transport, overlay, editor, or application types
shader filesystem paths or hot-reload policy
fixed-time scheduling
capture or artifact policy
PNG, EXR, FFmpeg, or another codec
raw WGPU types as the universal public API
```

WGPU may be an internal dependency. Narrow backend-specific facts or native-handle access require a concrete consumer and separately reviewed containment; they do not become the universal contract.

Consumers depend downward:

```text
RunenRender --------+
field GPU adapter ---+
simulation adapter --+--> RunenGPU
procedural tools ----+
offline bakers ------+
```

RunenSDF, RunenECS, and RunenUI remain independent. Cross-framework translation remains Runenwerk-owned until an independently reusable adapter is proven.

## Ownership

### RunenGPU owns

- GPU context and execution-epoch identities;
- normalized capabilities, limits, format facts, and requirements;
- backend-neutral logical buffer, texture, texture-view, sampler, and query-set descriptions;
- kind-typed logical handles and prepared GPU-data contracts;
- access, initialization, lifetime, hazard, and retirement validation;
- immutable compute, render, copy, clear, texture/query resolve, and present work;
- deterministic work composition and validation;
- context/device/backend admission;
- shader admission, interface validation, and pipeline realization;
- WGPU resource, command, submission, completion, readback, and low-level surface realization;
- headless compute and offscreen graphics execution;
- structured backend, timing, provenance, device, surface, and shutdown facts.

### RunenGPU does not own

- renderer views, targets, providers, materials, media, emitters, visibility, transport, reconstruction, or overlays;
- simulation, field, procedural-world, or application algorithms;
- shader source discovery, file watching, or last-known-good product policy;
- authoritative CPU/domain state;
- ECS storage or scheduling;
- UI semantics, layout, focus, accessibility, hit testing, or text shaping;
- window/event-loop policy;
- product quality selection, recovery decisions, or diagnostics presentation;
- capture selection, artifact naming, image encoding, or video encoding.

### RunenRender owns

- prepared scenes and renderer identities;
- views and logical render targets;
- materials, media, emitters, and environments;
- visibility and provider interaction semantics;
- lighting, transport, and estimator policy;
- reconstruction, radiance caches, and history semantics;
- overlays, color, output, and image-formation semantics;
- lowering renderer plans into generic RunenGPU work.

### Runenwerk owns

- ECS and domain extraction;
- application scheduling and fixed-time policy;
- windows, event loops, DPI, monitor, resize, and visibility policy;
- shader source discovery, revision, watching, and hot reload;
- composition of contributions from multiple framework/domain consumers;
- capture selection and artifact policy;
- offline jobs, ordered frames, manifests, retries, and failure policy;
- PNG/EXR encoding and external FFmpeg or other codec invocation;
- product recovery and diagnostics presentation.

Runenwerk may create one shared RunenGPU context for rendering and non-render workloads. This composition responsibility does not make reusable GPU or renderer semantics Runenwerk-owned.

## Framework relationships

### RunenUI

RunenUI owns semantic UI, state, actions, focus, accessibility, layout, style, text shaping, hit testing, and renderer-neutral paint output.

A Runenwerk-owned bridge may lower accepted paint primitives into a RunenRender overlay contribution:

```text
RunenUI paint scene
    -> Runenwerk bridge
    -> RunenRender overlay contribution
    -> RunenGPU work
```

RunenRender does not receive widget state or perform hit testing/text shaping. RunenUI remains usable with independent backends.

### RunenSDF

RunenSDF remains a CPU/backend-neutral field framework owning field values, numerical contracts, bounds, operators, transforms, capabilities, and reference queries.

GPU or renderer realization is derived integration state. RunenSDF never depends back on RunenGPU or RunenRender merely because an application accelerates or displays its output.

### RunenECS

RunenECS owns generic ECS semantics. Runenwerk adapters extract required state into prepared domain/GPU values. RunenGPU neither stores ECS entities/components nor schedules ECS systems.

## Public experience

The validated work graph is the shared internal correctness and inspection authority. It is not mandatory common-path ceremony.

### Ordinary path

```rust
let simulation = simulation.gpu_work(&gpu, &state)?;
let rendering = renderer.gpu_work(&gpu, &scene, request)?;

let submission = gpu.submit("frame 42", [simulation, rendering])?;
```

Ordinary submission validates automatically.

### Inspectable path

```rust
let prepared = gpu.prepare("frame 42", [simulation, rendering])?;
inspect(prepared.diagnostics());
let submission = gpu.submit_prepared(prepared)?;
```

Both paths use one preparation, validation, and execution authority. There is no reduced-validation compatibility path or duplicate graph.

### Progressive disclosure

```text
level 1  domain facade lowers semantic state into GpuWork
level 2  generic typed work authoring
level 3  explicit prepare and inspection
level 4  backend implementation and diagnostics
```

Graph, epoch, admission, realization, and retirement terminology remains internal or advanced unless explicitly requested.

### Ergonomic invariants

- strings are diagnostic labels, never identity, lookup, binding, or dependency authority;
- resource references are kind-typed;
- G4 pipeline interfaces expose validated binding keys;
- builders use lexical or closure scope rather than nested `.finish()` ladders;
- G3 infers data dependencies from declared resource access;
- explicit ordering exists only for real non-data constraints and redundant data edges are rejected;
- public handles are `Clone`, non-`Copy` RAII values;
- G5 connects last-handle drop to backend retirement after relevant submissions complete;
- errors identify the human operation and resource, preserve typed facts, explain the cause, and suggest correction;
- public callers do not branch on panic text, log text, `anyhow`, or backend-only enum dumps.

## Context model

A `GpuContext` represents one admitted backend execution authority. G4 creates the context and realizes backend state; G5 executes work through it.

Conceptual state:

```text
GpuContext
├── identity
├── backend facts
├── granted capabilities
├── device generation
├── resource registry
├── shader/pipeline registry
├── submission authority
├── surface registry
├── completion/readback state
└── diagnostics stream
```

Requirements:

- one logical submission authority per live context;
- explicit terminal state;
- idempotent shutdown;
- no process-global mutable context;
- no implicit product singleton;
- no authoritative domain state stored only on the device;
- foreign-context and stale-generation values are rejected;
- RunenGPU does not decide how many contexts a product creates.

## Identity model

Runtime identities are opaque, type-distinct, scope-bound, and fallibly allocated.

G1A accepted:

```text
GpuWorkResourceId
    private owner scope
    nonzero local value
    owner-controlled fallible allocator
```

G1A proves no safe arbitrary raw reconstruction, no wrapping/saturating allocation, explicit exhaustion, reserved-value rejection, deterministic allocation for the same allocator state and operation order, and foreign-owner rejection.

Later phase concepts may include:

```text
GpuContextId
GpuEpochId
GpuShaderId
GpuComputePipelineId
GpuRenderPipelineId
GpuSurfaceId
GpuSubmissionId
GpuReadbackId
```

G2 public resource ownership is expressed through distinct logical handles:

```text
GpuBufferHandle
GpuTextureHandle
GpuTextureViewHandle
GpuSamplerHandle
GpuQuerySetHandle
```

Handles expose no safe raw constructor or cross-kind reinterpretation. Raw diagnostic values never imply persistence, replay, cache, network, wire, or external-format stability.

The temporary crate-private bridge that seeds `GpuWorkResourceIdAllocator` from `RenderFlowId` remains one bounded G3 adapter seam because live context/work-scope ownership begins in G4. G4 must delete it.

## Capability model

RunenGPU exposes normalized capability facts rather than raw backend feature enums as universal semantics.

Requirement strength:

```text
Required
Preferred { explicit fallback/degradation }
Disabled
```

An unmentioned capability is irrelevant. `Optional` is not a fourth state. `Disabled` is an explicit admission constraint, not a synonym for unsupported.

Compatible requirement merging is deterministic and commutative. `Required` conflicts with `Disabled`; incompatible preferred fallbacks fail rather than choosing silently.

Profiles are convenience recipes that produce ordinary requirements:

```text
ComputeBaseline
OffscreenGraphicsBaseline
DesktopPresentationBaseline
```

Profiles are not a second authority and cannot silently override explicit requirements.

Initial normalized feature vocabulary is limited to current consumer evidence:

```text
compute
render pipelines
copy operations
indirect draw
storage textures
depth attachments
timestamp queries
presentation
```

Initial normalized limit facts are:

```text
maximum uniform-buffer binding size
maximum storage-buffer binding size
maximum color attachments
maximum vertex buffers
maximum bindings per group
```

Initial format facts include only formats proven by current descriptors/capture consumers. Format support is per use: sampled, filterable, storage read/write, attachment, depth/stencil, and copy.

Deferred until concrete pressure:

```text
subgroups
external-resource interop
ray queries and ray pipelines
sparse resources
mesh shaders
video
multiple hardware queues
universal shader IR
```

Compute-based field traversal remains a valid baseline. Experimental backend features do not enter stable vocabulary without current consumer value.

## Resource model

Unrelated properties are independent.

### Kind

```text
buffer
texture
texture view
sampler
query set
```

### Lifetime

```text
transient
retained
```

### Ownership

```text
RunenGPU-owned
imported
surface-acquired
```

### Transfer and observation

```text
initial data
upload or update
copy
query resolution
readback request
export relationship
```

### Reconstruction

```text
source-backed
externally reconstructed
non-reconstructable
```

### Memory intent

```text
ordinary device use
upload staging buffer
readback buffer
```

`Imported`, `Exported`, `Readback`, and `SurfaceOwned` are not lifetime classes. Import and surface acquisition are ownership; readback is an operation/result; export is a relationship/final-state contract.

Upload/readback memory intent applies only to buffers. Textures remain device resources and participate in host transfer through explicit copy relationships.

A resource descriptor includes:

- kind-specific dimensions and format intent;
- permitted uses;
- initialization contract;
- independent lifetime, ownership, memory, and reconstruction facts;
- validated human label;
- provenance and source-generation facts where applicable.

Labels and provenance are diagnostics/reconstruction evidence, never identity, lookup, binding, dependency, persistence, replay, wire, or cache authority.

### Buffer and texture initialization

Buffer and texture initialization are distinct.

```text
buffer
    uninitialized
    zeroed
    prepared transfer data

texture
    uninitialized
    zeroed
    prepared texture data {
        format,
        extent,
        bytes_per_row,
        rows_per_image,
        prepared transfer bytes
    }
```

Texture initialization binds row and image layout so G5 does not invent upload semantics. Arithmetic is checked; no saturating multiplication silently normalizes overflow.

### Texture views

A texture view references a typed texture handle and a checked mip/layer/aspect range. Its effective validity cannot exceed its parent texture's logical lifetime, ownership lease, or subresource range.

### Resource access

G2 descriptors define permitted uses. G3 defines work-time buffer ranges, texture/query subresources, access categories, initialization flow, and hazards. G3 extends the accepted G2 buffer usage vocabulary with `GpuBufferUsage::QueryResolve` because the current timestamp path requires a distinct query-resolve destination usage.

Initial exact categories are:

```text
buffer
    UniformRead
    StorageRead
    StorageWrite
    StorageReadWrite
    VertexRead
    IndexRead
    IndirectRead
    CopySource
    CopyDestination
    QueryResolveDestination

texture
    SampledRead
    StorageRead
    StorageWrite
    StorageReadWrite
    CopySource
    CopyDestination
    ColorAttachment { load, store }
    DepthStencilAttachment { access, load, store }
    Present

query
    WriteTimestamp
    ResolveSource
```

`QueryResolveDestination` is not `CopyDestination`. A typed query resolve consumes initialized query indices and initializes its exact destination buffer byte range. Current timestamp results occupy one `u64` per query; checked destination length is `query_count * 8`. G4/G5 validate backend-specific destination-offset alignment and encode the operation.

Attachment `Load` requires initialized coverage, `Clear` establishes coverage, `Store` preserves it, and `Discard` removes later readable coverage. Texture-view hazards normalize to parent storage. D3 hazards initially use whole addressed mip volumes rather than inventing z-slice independence.

Validation rejects incompatible overlap, use before initialization, use after retirement, invalid view/resource relationships, invalid query resolve shape or usage, ambiguous writers, missing capabilities, and invalid ownership/lifetime combinations.

### Imports and exports

Imported resources require explicit ownership, provenance, reconstruction, validity, and synchronization facts. Surface-acquired resources are a G7-owned specialized lease and remain transient.

Conceptual import facts:

```text
external owner
logical resource kind
validity interval
initialized graph-entry coverage
required final access
synchronization fact
reconstruction owner
retirement rule
```

Raw backend handles are not a stable public contract by themselves.

Export identifies a typed resource, consumer-owned `GpuExportKey`, required final access state, final initialized coverage, and provenance. It is not a lifetime or resource kind.

## Typed GPU data

The required boundary is:

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared value or bytes
            -> RunenGPU upload/update contract
```

Prepared data distinguishes:

```text
uniform
storage
vertex
indirect
transfer
```

Readback uses a separate decoder contract. Texture initialization wraps transfer data with format/extent/row-layout evidence.

Prepared data records checked byte length, alignment, stride, element count, and provenance. It does not infer GPU safety from arbitrary Rust memory.

Current `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and their derives are transitional Runenwerk/render adapter mechanisms. G2 removes them from new RunenGPU signatures and descriptor-size authority. G4 decides WGPU/WGSL layout, validated binding keys, and macro/derive realization. G5 performs uploads, updates, staging, and readback.

`TypeId` and type names may support process-local diagnostics or adapter lookup. They are not layout, binding, persistence, replay, wire, cache, shader-interface, or cross-process authority.

No universal derive may imply one Rust structure has one valid representation for uniforms, storage, vertices, indirect data, transfer, and readback.

## Workload model

Consumers contribute immutable `GpuWorkFragment` values after G3.

Conceptual form:

```text
GpuWorkFragment
├── imported resources
├── declared resources
├── exported resources
├── work nodes
├── explicit non-data dependencies
├── outputs
└── provenance
```

Initial typed operation variants are:

```text
Compute
Render
Copy
Clear
Resolve {
    Texture
    QuerySet
}
Present
```

Deferred until evidence:

```text
AccelerationStructureBuild
SparseBinding
ExternalInterop
Video
Multiple hardware queue primitives
```

A work node declares:

- identity and one typed `GpuWorkOperation`; node kind is derived from the operation rather than stored twice;
- an admitted pipeline/interface reference after G4, with a temporary render-owned sidecar during G3;
- typed resource accesses, including accesses derived automatically from copy, clear, texture/query resolve, indirect draw, and present operations;
- backend-neutral dispatch, draw, copy, clear, resolve, or present shape;
- capability requirements;
- optional execution preference;
- debug label and provenance;
- explicit order only when no data dependency represents the constraint.

Execution preferences may include:

```text
Automatic
ComputePreferred
GraphicsRequired
TransferPreferred
```

Preferences are hints, not concurrency guarantees. The first backend may serialize through one logical queue while preserving dependencies and future scheduling information.

An empty render operation is accepted only when attachment clear or another explicit side effect makes it meaningful. Attachment `Store` alone preserves prior content and is not work.

## Work graph

A `GpuPreparedWorkGraph` composes immutable fragments for one bounded execution epoch after G3. It is internal/advanced authority, not the mandatory user-facing authoring surface.

It owns:

- deterministic typed identity and reference resolution;
- operation/access consistency;
- inferred data dependencies and resource hazards;
- topological ordering;
- initialization and lifetime validation;
- capability admission inputs;
- backend compilation inputs;
- output and completion contracts.

It rejects:

- duplicate, unknown, foreign, or stale identities;
- cycles;
- unknown resources or pipelines;
- read before initialization;
- use after retirement;
- incompatible accesses;
- ambiguous writers;
- invalid copy, clear, texture resolve, query resolve, or present shape;
- redundant explicit ordering that duplicates an inferred data edge;
- invalid surface-image reuse;
- missing capabilities;
- invalid pipeline/resource combinations;
- inconsistent imports/exports;
- incompatible explicit non-data ordering.

The graph contains no ECS systems, gameplay actions, UI routes, SDF nodes, material graph nodes, renderer feature meaning, or product lifecycle policy. Higher-level owners lower those semantics first.

Within one fragment, lexical node order orients access-derived hazards. Fragment collection position does not create dependencies or resolve writer ambiguity. Cross-fragment causality requires shared typed resources plus matching typed imports/exports.

Timestamp writes followed by query resolution infer a dependency through the query range. Query resolution followed by a buffer copy infers a dependency through the resolved destination byte range.

## Epoch and submission lifecycle

```text
collect fragments
    -> prepare and compose
    -> validate
    -> admit/realize for backend
    -> encode
    -> submit
    -> publish submission
    -> complete/read back
    -> retire transient/backend state safely
```

Phase ownership:

- G3: fragment composition, typed operations, accesses, initialization, hazards, query-resolution intent, and graph validation;
- G4: context/device admission and backend resource/shader/pipeline realization, including query-resolve alignment admission;
- G5: encode/submit, uploads/updates, query-resolution encoding, completion, cancellation, asynchronous readback, and delayed retirement;
- G7: surface acquisition/presentation and surface generations.

The lifecycle contract defines:

- maximum configured in-flight submissions/epochs;
- retained/history resource validity;
- transient retirement;
- surface-acquired image ownership;
- asynchronous completion and cancellation;
- shutdown and terminal outcomes.

Fragment creation may be concurrent. Final context composition and submission remain context-owned unless a later accepted backend contract proves otherwise.

## Shader and pipeline boundary

A consumer owns shader/kernel meaning and source. RunenGPU owns admission, interface validation, and backend realization in G4.

A source descriptor may include:

```text
stable source key
source revision
language or IR
entry points
interface declarations
capability requirements
specialization schema
provenance
```

RunenGPU owns:

- source admission and backend validation;
- module realization;
- resource-layout and binding realization;
- validated binding keys;
- pipeline creation and specialization;
- backend cache keys;
- structured failures.

RunenGPU does not own filesystem roots, path discovery, polling, file watching, user-facing reload UI, or last-known-good product policy.

Logical parameters, byte representation, binding representation, and backend layout remain distinct. WGSL/WGPU layout is not universal domain semantics.

No macro package is accepted before G4 proves byte-layout, alignment, nested type, package-renaming, compile-pass, and compile-fail requirements. G2 introduces no universal derive.

## WGPU backend

G4 may use WGPU internally for:

- instance, adapter, device, and queue;
- normalized feature, limit, and format mapping;
- resources and views;
- shader modules and pipelines;
- query sets, query-resolve alignment admission, and timing realization.

G5 uses the admitted WGPU backend for:

- command encoding and submission;
- staging uploads and updates;
- query-set resolution encoding;
- completion and cancellation;
- asynchronous mapping/readback;
- delayed backend resource retirement.

G7 adds:

- surface creation/configuration/acquisition/presentation;
- surface and device outcomes.

The public API separates normalized RunenGPU semantics from WGPU-specific facts. A second backend is not required before extraction. The architecture is backend-neutral where semantic value exists, not abstract for its own sake.

## Surface boundary

Runenwerk or another host owns:

- window creation/destruction;
- event loop;
- raw window/display handle lifetime;
- DPI, monitor, resize, visibility, and presentation timing policy;
- product recovery.

RunenGPU G7 owns:

- admitting host-provided raw handles without depending on Winit;
- low-level surface creation and retirement;
- capability query and configuration;
- image acquisition and lifetime;
- present operation;
- structured surface/device outcomes.

RunenRender owns logical output image and color meaning.

Surface configuration and acquired images use generations. An image acquired from an old generation cannot be presented through a newer configuration.

G7 proves thread affinity, handle lifetime, drop order, multi-surface behavior where supported, headless independence, resize/reconfiguration, and device/surface outcomes. It reuses accepted G6 execution workloads rather than creating a surface-only architecture.

## Readback

G5 readback is asynchronous:

```text
ReadbackRequest
    -> ReadbackId
    -> Pending
    -> Ready(bytes) | Failed(error) | Cancelled
```

Readback does not require blocking the submission authority. RunenGPU returns normalized bytes/layout/format provenance and completion facts. Callers own semantic decoding and artifact policy.

Query resolution is not itself readback. It converts opaque query-set results into a device buffer through typed work. A later copy/readback request and decoder convert those bytes into host-visible timing evidence.

## Error model

Required categories include:

```text
identity/allocation
capability requirement/admission
resource/descriptor validation
typed-data preparation and decoding
work-operation and graph validation
shader/pipeline realization
submission/completion/cancellation
readback
surface outcomes
device outcomes
terminal/shutdown
```

Every public error has typed fields for programmatic matching and human-readable presentation containing operation, label, cause, provenance where applicable, and corrective action.

Deterministic planning failures remain distinct from backend/environment outcomes. No panic, generic string matching, `anyhow`, or log text is the normal public contract.

## Diagnostics

RunenGPU exposes structured facts for:

- backend and adapter selection;
- requested/granted/degraded capabilities;
- resource creation, realization, use, and retirement;
- work validation and compilation;
- shader/pipeline realization;
- command encoding and submission;
- uploads, query resolution, completion, cancellation, and readbacks;
- timings and statistics;
- surface and device outcomes;
- terminal shutdown.

Every fact retains correlation:

```text
GPU fact
    -> resource/work node/submission
    -> work fragment
    -> contributing consumer
    -> source provenance
```

RunenGPU reports facts. Hosts decide severity, storage, user presentation, retry, and recovery policy.

## Interaction matrix

| Concern | RunenGPU | RunenRender | Domain/framework | Runenwerk |
|---|---|---|---|---|
| Context/device/queue | Owns | Uses | Uses through work/adapters | Creates/configures policy |
| Resources and hazards | Owns/validates | Declares | Declares | Composes |
| Compute/render/copy execution | Owns | Contributes | Contributes | Orders application epochs |
| Shader realization | Owns | Supplies render shaders | Supplies kernels | Supplies source revisions |
| Image formation | No | Owns | Supplies prepared data | Selects product policy |
| Simulation/field algorithms | No | No | Owns | Schedules/integrates |
| Window lifecycle | Surface facts only | Logical target intent | No | Owns |
| Capture/artifact encoding | Completion/readback facts | Image meaning | No | Owns |
| Product recovery | Reports facts | Adds render context | Adds domain context | Owns |
| UI state/layout/hit testing | No | No | RunenUI owns | Hosts |

## RenderFlow disposition

Current `RenderFlow` is a transitional combined facade. It is decomposed, not moved, renamed, or wrapped wholesale.

| Current responsibility | Target owner |
|---|---|
| GPU resource identity, descriptions, access, generic work | RunenGPU |
| WGPU context, resources, pipelines, submission | RunenGPU backend |
| views, targets, rendering, image-formation semantics | RunenRender |
| ECS projection and fixed-step scheduling | Runenwerk adapters |
| shader-file paths, hot reload, windows, built-in UI, capture/export policy | Runenwerk adapters |

Specific migration rules:

- `with_state`, `uniform_from_state*`, `dispatch_from_state`, and `project_uniforms` remain outside RunenGPU;
- `shader_asset` paths remain Runenwerk source authority;
- target aliases, history, fullscreen, graphics, and procedural image meaning remain RunenRender authority;
- generic resource descriptions and kind-typed handles move to RunenGPU;
- string maps cease to be resource/binding/dependency authority;
- broad `.depends_on` chains are replaced by inferred data dependencies in G3;
- redundant explicit data edges are rejected rather than retained beside inferred edges;
- repeated `.finish()` ladders are not retained;
- ordinary submission does not require a separate `.validate()` call;
- the temporary `RenderFlowId` owner bridge remains only through G3 and is removed in G4.

G2-G7 migrate and delete the authority each phase replaces. G8 performs final residual audit; it is not a delayed bulk migration phase.

## Proof portfolio

Evidence is classified separately:

```text
deterministic conformance
boundary integration
visual showcase
benchmark or stress evidence
```

A broad visual demo cannot replace exact correctness evidence. Performance measurements are not pass/fail thresholds until hardware, driver, OS, backend, power state, build mode, workload, and method are separately bound.

### G5 deterministic compute conformance

Use existing `u32` prefix-scan machinery:

- 4,097 fixed integer inputs, all one;
- complete inclusive and exclusive output comparison;
- exact total 4,097;
- workgroup-boundary and multi-level temporary storage coverage;
- upload, multiple dispatches, completion, and asynchronous readback;
- no renderer, ECS, surface, window, or product types.

Counter reset, scatter/compaction, and indirect-argument primitives remain intended RunenGPU authority with focused exact tests.

### G5 stateful integration

Use headless Game of Life:

```text
grid: 160 x 90
seed: 0xC0FF_EE11
steps after seed: 16
boundary: toroidal
expected live cells: 2,063
FNV-1a-64 over little-endian u32 cells: 0xBD710B88594CD584
```

The proof compares the full GPU result with a CPU reference and selected cells. Simulation state is prepared outside RunenGPU.

### G5 conditional texture proof

When admitted by G5 scope, use deterministic integer compute-to-texture output, texture-to-buffer copy, row-padding normalization, explicit format handling, selected texels, and Runenwerk-owned PNG encoding.

### G6 graphics conformance

Use an offscreen known-pattern clear/draw, texture readback, and selected-pixel assertions with documented tolerances where normalization/rasterization requires them.

### G6 GPU-driven composition

Use compute-generated count, compacted data, or indirect arguments consumed by graphics. Ordering is inferred from shared resource access through one context. Validate structure, generated arguments, and selected pixels.

### G6 showcase

Use offscreen boids for shared compute/render, bounded-grid primitives, ping-pong state, indirect/generated draw data, and image artifacts. Validate structure, agent counts, finite values, bounded positions/ranges, and overflow. Do not require exact cross-backend boid state or pixels.

### G7 surface proof

Reuse the accepted G6 known-pattern and boids workloads for presentation, resize, reconfiguration, generations, multi-surface behavior where supported, thread affinity, and device/surface outcomes.

### RunenRender proofs

Procedural sky/SDF terrain is the first semantic image-formation proof after standalone RunenGPU acceptance. Boids follows as simulation-to-render integration. The SDF history flow is a later temporal/history ownership proof.

## Offline output boundary

Preferred order:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky/SDF/scene sequences after corresponding RunenRender phases.

Runenwerk owns output clock, seeds, job configuration, bounded in-flight readbacks, ordered filenames, manifests, retry/failure policy, PNG/EXR encoding, and external FFmpeg/codec integration. RunenGPU owns completion/readback facts. RunenRender owns image formation. Neither framework owns MP4/WebM codecs.

## Conformance

Internal proof requires:

1. neutral capability/resource/data validation without Runenwerk and without a window;
2. owner-scoped identity tests covering invalid, foreign, exhausted, and non-wrapping behavior;
3. G3 access, typed-operation, query-resolution, graph, cycle, hazard, initialization, import, export, and deterministic-order tests;
4. G5 headless compute, completion, and asynchronous readback where GPU access is supported;
5. G6 offscreen graphics independently of presentation;
6. one shared context executing render and independent non-render work;
7. G7 structured surface/device outcomes with recovery outside RunenGPU;
8. no renderer/domain meaning in RunenGPU public contracts;
9. no Runenwerk type in future-transferable source;
10. deterministic evidence separated from environment-dependent GPU evidence;
11. simple and inspectable APIs using the same authority;
12. source/dependency guards proving no compatibility or duplicate authority.

External repository conformance additionally requires:

- independent locked format/test/Clippy/rustdoc validation;
- declared Rust edition and MSRV;
- public downstream consumer proof;
- license and source provenance;
- no Runenwerk source include, submodule, mirror, compatibility package, or moving-branch dependency;
- exact-revision Runenwerk integration and deletion of original authority.

## S0 inventory gate

S0 is complete. Its inventory and disposition reports remain historical evidence for:

- every current GPU/render file, macro, shader, test, example, benchmark, and generated artifact;
- Cargo dependencies and downstream consumers;
- identities, allocators, raw conversions, and handle use;
- persistence, replay, network, cache, wire, and artifact risk;
- graph, resource, pipeline, shader, surface, device, frame, and shutdown flows;
- WGPU/Winit/ECS/scene/world/material/SDF/UI/editor/product dependencies;
- exact move/stay/redesign/delete classification;
- validation and runtime evidence.

Each later phase re-verifies its affected current-main subset. S0 completion does not authorize later implementation or override newer evidence.

## Extraction sequence

```text
S0 complete inventory                                      complete
G1A owner-scoped logical GPU work-resource identity         complete
G2 capabilities, resources, typed handles, prepared data    complete
G3 decision-complete planning                               active through issue #174
G3 access, operations, initialization, hazards, work graph  blocked on planning acceptance
G4 context/device, shader/pipeline, binding/layout, WGPU     pending
G5 execution, uploads, completion, readback, retirement     pending
G6 offscreen graphics and shared consumer proof             pending
G7 surfaces, generations, thread affinity, device outcomes  pending
G8 final diagnostics, shutdown, residual audit              pending
GX external dornglut/runen-gpu clean cutover                blocked
```

Only one next decision or implementation specification is active at a time. Issue `#174` owns G3 planning. Create one separate bounded G3 implementation issue only after the planning PR is independently reviewed and merged. G2-G7 migrate and delete replaced authority incrementally. G8 is the final conformance and residual-authority audit.

## External cutover

GX is a clean cutover:

1. create/populate `dornglut/runen-gpu` from accepted internal source;
2. preserve source provenance and license;
3. establish independent validation, MSRV, documentation, and downstream conformance;
4. pin Runenwerk to an exact accepted revision;
5. migrate every active consumer;
6. delete internal GPU authority and temporary adapters;
7. prove no mirror, forwarding package, compatibility namespace, duplicate context, duplicate descriptor, or duplicate execution path remains.

No submodule, source include, moving-branch dependency, or long-lived dual path is accepted.

## Explicit non-goals

The initial extraction does not include:

- a second backend;
- aggressive transient aliasing;
- pass fusion;
- automatic multi-queue scheduling;
- backend-independent shader IR;
- graph visualization UI;
- hardware ray tracing as a baseline;
- sparse/external-resource interop without current evidence;
- image/video codec ownership;
- renderer semantics inside RunenGPU.

## Definition of done

RunenGPU extraction is complete only when:

- `dornglut/runen-gpu` contains one independently validated public package;
- Runenwerk and RunenRender use only public APIs;
- public contracts contain no Runenwerk, RunenRender, ECS, SDF, UI, Winit, application, product, or domain types;
- headless compute, uploads, completion, asynchronous readback, offscreen graphics, and surfaces pass;
- exact conformance and representative showcase evidence remain separate;
- at least one non-render consumer proves independent value;
- surface/device lifecycle and terminal shutdown are accepted;
- simple and inspectable public paths are proven;
- exact revision, MSRV, license, and provenance are recorded;
- every active consumer is migrated;
- original Runenwerk GPU authority and temporary seams are deleted;
- no duplicate context/resource/descriptor/workload path survives.
