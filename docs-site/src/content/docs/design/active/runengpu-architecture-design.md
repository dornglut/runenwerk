---
title: RunenGPU Architecture Design
description: Decision-complete ownership, workload, resource, capability, operational, WGPU, surface, diagnostics, conformance, and extraction architecture for RunenGPU.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-28
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ./runen-family-operational-hardening-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-render-s0-identity-consumer-lifecycle.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/investigations/runen-family-operational-hardening-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../reports/closeouts/pt-runen-family-operational-hardening-closeout.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU Architecture Design

## Status

The repository identity, ownership boundary, one-package target shape, dependency
direction, WGPU placement, host boundary, public experience, operational doctrine,
and extraction sequence are fixed.

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        complete at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
G3 decision phase                    complete at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
operational hardening                complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 Rust implementation               candidate corrected after independent review
G4-G8                                pending and not authorized early
GX external extraction               blocked on accepted G2-G8 evidence
```

G3 was implemented from accepted base
`1c645b2bbfcece44dd6ae151cc97559793afa2c2`. The reviewed head
`38abac6bd234d9db3a4544aedbf2dba149538e36` required corrections; corrected code
candidate `905c506e33202405d1bea8c160a05ac92c326c43` remains open, draft, and unmerged
pending fresh exact-head validation and independent review. This document asserts no
G3 merge SHA.

The implementation remains inside Runenwerk until each internal future public
boundary is accepted. This document defines broad architecture. Focused phase designs,
specifications, and owning issues authorize bounded implementation. Accepted G3
resource-access, initialization, hazard, operation, causality, and prepared-graph
semantics remain unchanged by operational hardening.

## Mission

RunenGPU owns validated execution of GPU resources and workloads.

It answers:

> How are GPU capabilities, resources, accesses, workloads, submissions, results,
> progress, pressure, lifecycle outcomes, and backend failures represented and
> executed safely?

It does not answer:

- what an image should contain;
- how light transport works;
- how a field, simulation, material, UI, ECS entity, or world object behaves;
- how an application schedules gameplay;
- how windows and event loops are managed;
- how product recovery is selected or presented;
- how persisted capture bundles or PNG, EXR, MP4, or WebM artifacts are encoded.

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

RunenGPU complements WGPU. It does not reimplement Vulkan, Metal, D3D12, WebGPU,
shader compilers, or operating-system window systems.

## Repository and package

```text
repository: dornglut/runen-gpu
package: runen-gpu
crate: runen_gpu
initial backend: WGPU, internal implementation detail
```

Directional standalone shape:

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

Module names are directional, not pre-authorized file names. The public package
remains one release unit until a second backend, independent consumer, release
boundary, or dependency graph proves another package is necessary.

Do not initially create:

```text
runengpu_core
runengpu_wgpu
runengpu_macros
runengpu_testing
runengpu_capture
facade or compatibility packages
```

The external repository is created only in GX after internal conformance and
extraction-readiness evidence are accepted.

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
product recovery policy
persisted capture or artifact policy
PNG, EXR, FFmpeg, or another codec
raw WGPU types as the universal public API
```

WGPU may be an internal dependency. Narrow backend-specific facts or native-handle
access require a concrete consumer and separately accepted containment. They do not
become the universal contract.

Consumers depend downward:

```text
RunenRender --------+
field GPU adapter ---+
simulation adapter --+--> RunenGPU
procedural tools ----+
offline bakers ------+
```

RunenSDF, RunenECS, and RunenUI remain independent. Cross-framework translation
remains Runenwerk-owned until an independently reusable adapter is proven.

## Ownership

### RunenGPU owns

- context, device-generation, execution-epoch, submission, and readback identities;
- normalized capabilities, limits, format facts, portability classes, and
  requirements;
- backend-neutral logical buffer, texture, texture-view, sampler, and query-set
  descriptions;
- kind-typed logical handles and prepared GPU-data contracts;
- access, graph-entry initialization, lifetime, hazard, and later retirement
  validation;
- immutable compute, render, copy, buffer-zero, query-resolve, and logical present
  work;
- render-attachment and multisample-resolve relationships inside render operations;
- operation/access-derived capability requirements;
- deterministic work composition and validation;
- context/device/backend admission;
- shader admission, interface validation, pipeline realization, and cache
  compatibility facts;
- WGPU resource, command, submission, completion, readback, and low-level surface
  realization;
- headless compute and offscreen graphics execution;
- normalized progress, pressure, cancellation, completion, shutdown, device-loss, and
  reconstruction facts;
- structured backend, timing, provenance, cache, generation, surface, and terminal
  diagnostics.

### RunenGPU does not own

- renderer views, targets, providers, materials, media, emitters, visibility,
  transport, reconstruction, or overlays;
- simulation, field, procedural-world, or application algorithms;
- shader source discovery, file watching, or last-known-good product policy;
- authoritative CPU/domain state;
- ECS storage or scheduling;
- UI semantics, layout, focus, accessibility, hit testing, or text shaping;
- window/event-loop policy;
- product quality selection, recovery decisions, or diagnostics presentation;
- persisted capture selection, retention, redaction, artifact naming, image encoding,
  or video encoding.

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
- tested cross-framework compatibility manifests;
- product recovery decisions;
- persisted capture selection, schema, retention, redaction, and artifact policy;
- versioned namespaced reproducibility bundles;
- offline jobs, ordered frames, manifests, retries, and failure policy;
- PNG/EXR encoding and external FFmpeg or other codec invocation;
- diagnostics presentation.

Runenwerk may create one shared RunenGPU context for rendering and non-render
workloads. Composition responsibility does not make reusable GPU or renderer
semantics Runenwerk-owned.

## Framework relationships

### RunenUI

RunenUI owns semantic UI, state, actions, focus, accessibility, layout, style, text
shaping, hit testing, and renderer-neutral paint output.

```text
RunenUI paint scene
    -> Runenwerk bridge
    -> RunenRender overlay contribution
    -> RunenGPU work
```

RunenRender does not receive widget state or perform hit testing/text shaping.
RunenUI remains usable with independent backends.

### RunenSDF

RunenSDF remains a CPU/backend-neutral field framework owning field values, numerical
contracts, bounds, operators, transforms, capabilities, and reference queries.
GPU or renderer realization is derived integration state. RunenSDF never depends back
on RunenGPU or RunenRender merely because an application accelerates or displays its
output.

### RunenECS

RunenECS owns generic ECS semantics. Runenwerk adapters extract required state into
prepared domain/GPU values. RunenGPU neither stores ECS entities/components nor
schedules ECS systems.

## Public experience

The validated work graph is the shared internal correctness and inspection authority.
It is not mandatory common-path ceremony.

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

Both paths use one preparation, validation, and execution authority. There is no
reduced-validation compatibility path or duplicate graph.

### Progressive disclosure

```text
level 1  domain facade lowers semantic state into GpuWork
level 2  generic typed work authoring
level 3  explicit prepare and inspection
level 4  backend implementation and diagnostics
```

Graph, epoch, admission, realization, progress, and retirement terminology remains
internal or advanced unless explicitly requested.

### Ergonomic invariants

- strings are diagnostic labels, never identity, lookup, binding, dependency, or
  stable-capture authority;
- resource references are kind-typed;
- G4 pipeline interfaces expose validated binding keys;
- builders use lexical or closure scope rather than nested `.finish()` ladders;
- G3 infers data dependencies from declared resource access;
- explicit ordering exists only for real non-data constraints and redundant data
  edges are rejected;
- public handles are `Clone`, non-`Copy` RAII values;
- G5 connects last-handle drop to backend retirement after relevant submissions
  complete;
- accepted work receives one structured terminal outcome;
- pressure never becomes silent loss or unbounded implicit growth;
- errors identify the human operation and resource, preserve typed facts, explain the
  cause, and suggest correction;
- public callers do not branch on panic text, log text, `anyhow`, or backend-only enum
  dumps.

## Context model

A `GpuContext` represents one admitted backend execution authority. G4 creates the
context and realizes backend state; G5 executes work through it.

Conceptual state:

```text
GpuContext
├── identity
├── portability and backend facts
├── granted capabilities
├── device generation
├── resource registry
├── shader/pipeline registry and cache facts
├── submission/progress authority
├── pressure and backing-memory accounting
├── surface registry
├── completion/readback state
└── diagnostics stream
```

Requirements:

- one logical submission authority per live context;
- explicit terminal state and idempotent shutdown;
- no process-global mutable context or implicit product singleton;
- no authoritative domain state stored only on the device;
- foreign-context and stale-generation values are rejected;
- accepted work receives exactly one completion, cancellation, loss, or shutdown
  outcome;
- progress ownership and allowed thread/executor are explicit;
- consumer callbacks are not invoked while internal registry, queue, staging,
  completion, or cache locks are held;
- submissions, uploads, mappings, readbacks, and backing memory are bounded or have
  explicit growth policy;
- RunenGPU does not decide how many contexts a product creates or how product recovery
  is presented.

## Identity model

Runtime identities are opaque, type-distinct, scope-bound, and fallibly allocated.

G1A accepted:

```text
GpuWorkResourceId
    private owner scope
    nonzero local value
    owner-controlled fallible allocator
```

G1A proves no safe arbitrary raw reconstruction, no wrapping/saturating allocation,
explicit exhaustion, reserved-value rejection, deterministic allocation for the same
allocator state and operation order, and foreign-owner rejection.

Later phase concepts include:

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

Handles expose no safe raw constructor or cross-kind reinterpretation. Raw diagnostic
values never imply persistence, replay, cache, network, wire, or external-format
stability.

The temporary crate-private bridge that seeds `GpuWorkResourceIdAllocator` from
`RenderFlowId` remains one bounded G3 adapter seam because live context/work-scope
ownership begins in G4. G4 must delete it.

## Capability and portability model

RunenGPU exposes normalized capability facts rather than raw backend feature enums as
universal semantics.

Requirement strength:

```text
Required
Preferred { explicit fallback/degradation }
Disabled
```

An unmentioned capability is irrelevant. `Optional` is not a fourth state. `Disabled`
is an explicit admission constraint, not a synonym for unsupported.

Compatible requirement merging is deterministic and commutative. `Required` conflicts
with `Disabled`; incompatible preferred fallbacks fail rather than choosing silently.

Profiles are convenience recipes that produce ordinary requirements:

```text
ComputeBaseline
OffscreenGraphicsBaseline
DesktopPresentationBaseline
```

Profiles are not a second authority and cannot silently override explicit
requirements.

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

Initial normalized limit facts include maximum uniform/storage binding sizes, color
attachments, vertex buffers, and bindings per group. Initial format facts include
only formats proven by current descriptors and consumers. Format support is per use.

G4 additionally reports a portability class:

```text
portable_baseline
portable_with_declared_extensions
backend_specialized_internal
unsupported
```

Backend specialization remains contained and does not imply a stable raw native
escape.

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

Compute-based field traversal remains a valid baseline. Experimental backend features
do not enter stable vocabulary without current consumer value.

### Operation-derived requirements

G3 derives requirements mechanically:

```text
Compute operation                    -> Compute
Render operation                     -> RenderPipeline
Copy or buffer-zero Clear            -> Copy
Indirect draw                        -> IndirectDraw
Storage texture access               -> StorageTexture
Depth/stencil attachment             -> DepthAttachment
Timestamp write or query resolution  -> TimestampQuery
Present                              -> Presentation
```

Consumers may add semantic requirements that operation shape cannot infer. An
operation-implied requirement cannot be weakened or disabled. G4 admits the merged
requirements against context facts.

## Resource model

Unrelated properties remain independent:

```text
kind
    buffer, texture, texture view, sampler, query set

lifetime
    transient, retained

ownership
    RunenGPU-owned, imported, surface-acquired

transfer and observation
    initial data, upload/update, copy, query resolution, readback, export

reconstruction
    source-backed, externally reconstructed, non-reconstructable

memory intent
    ordinary device use, upload staging buffer, readback buffer
```

`Imported`, `Exported`, `Readback`, and `SurfaceOwned` are not lifetime classes.
Upload/readback memory intent applies only to buffers. Textures remain device
resources and participate in host transfer through explicit copy relationships.

A resource descriptor includes kind-specific dimensions/format, permitted uses,
initialization, independent lifetime/ownership/memory/reconstruction facts, validated
label, and provenance/source-generation facts where applicable.

Labels and provenance are diagnostic/reconstruction evidence, never identity,
lookup, binding, dependency, persistence, replay, wire, or cache authority.

### Buffer, texture, and query initialization

Buffer and texture initialization are distinct:

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

Texture initialization binds row and image layout so G5 does not invent upload
semantics. Arithmetic is checked.

Query sets have no descriptor-level initialized query contents. They enter graph
preparation without initialized indices unless explicit graph-entry query coverage is
supplied for an imported or retained prior-epoch source.

### Texture views

A texture view references a typed texture handle and checked mip/layer/aspect range.
Its validity cannot exceed its parent texture lifetime, lease, generation, or
subresource range.

### Resource access

G2 descriptors define permitted uses. Accepted G3 defines exact work-time buffer
ranges, texture/query subresources, access categories, graph-entry initialization,
hazards, attachment relations, query resolution, and causality.

Initial access vocabulary includes:

```text
buffer
    UniformRead, StorageRead, StorageWrite, StorageReadWrite
    VertexRead, IndexRead, IndirectRead
    CopySource, CopyDestination, QueryResolveDestination

texture
    SampledRead, StorageRead, StorageWrite, StorageReadWrite
    CopySource, CopyDestination
    ColorAttachment { load, store }
    MultisampleResolveDestination
    DepthStencilAttachment { access, load, store }
    Present

query
    WriteTimestamp, ResolveSource
```

Exact color/depth clear values belong to render attachment operations; access records
retain normalized load/store meaning.

`QueryResolveDestination` is not `CopyDestination`. A typed query resolve consumes
initialized query indices and initializes its exact destination buffer range. Current
timestamp results occupy one `u64` per query; G4/G5 validate backend alignment and
encode the operation.

Multisample texture resolution is an optional relation on a render color attachment,
not standalone work. The destination is initialized regardless of source `Store` or
`Discard`.

Attachment `Load` requires initialized source coverage, `Clear` establishes it,
`Store` preserves it, and `Discard` removes later-readable source coverage.
Texture-view hazards normalize to parent storage. Initial D3 hazards treat an
addressed mip volume as one unit rather than inventing z-slice independence.

G3 rejects invalid overlap, read-before-initialization, invalid views/operations,
missing cross-fragment causality, ambiguous writers, contradictory requirements, and
other context-free failures. Runtime stale-generation, backend lease, and retirement
validation begins in G4/G5.

### Imports and exports

Imported resources require explicit owner, kind, validity, initialized graph-entry
coverage, required final access, synchronization/reconstruction facts, and retirement
rule. Surface-acquired resources remain a G7-owned specialized lease.

Export identifies a typed resource, consumer-owned `GpuExportKey`, required final
access, final initialized coverage, and provenance. It is not a lifetime or kind.

Raw backend handles are not a stable public contract.

## Typed GPU data

Required boundary:

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared value or bytes
            -> RunenGPU upload/update contract
```

Prepared data distinguishes uniform, storage, vertex, indirect, and transfer roles.
Readback uses a separate decoder contract. Texture initialization wraps transfer data
with format/extent/row-layout evidence.

Prepared data records checked byte length, alignment, stride, element count, and
provenance. It does not infer GPU safety from arbitrary Rust memory.

Current `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and
derives are transitional Runenwerk/render adapter mechanisms. G4 decides WGPU/WGSL
layout, validated binding keys, and macro/derive realization. G5 performs uploads,
updates, staging, and readback.

`TypeId` and type names may support process-local diagnostics or adapter lookup. They
are not layout, binding, persistence, replay, wire, cache, shader-interface, or
cross-process authority.

No universal derive may imply one Rust structure has one valid representation for all
GPU roles.

## Workload and prepared-graph model

Consumers contribute immutable `GpuWorkFragment` values after G3:

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

Initial operation variants:

```text
Compute
Render
Copy
Clear      checked buffer zero only
Resolve    query set -> buffer
Present    logical texture consumption
```

A render operation owns ordered color attachments, optional depth/stencil attachment,
ordered draw intents, and render-side timestamp writes. Multisample texture resolve
is an optional color-attachment relation.

Deferred operations include acceleration-structure build, sparse binding, external
interop, video, and multiple hardware queue primitives.

A work node declares one typed operation, typed/derived accesses, merged requirements,
backend-neutral operation shape, optional execution preference, label/provenance, and
explicit order only where no data dependency represents the constraint.

An empty render operation is accepted only when attachment clear or query writes make
it meaningful. Attachment `Store` alone is not work.

`GpuPreparedWorkGraph` composes immutable fragments for one bounded execution epoch.
It owns deterministic typed resolution, operation/access/requirement consistency,
inferred hazards/dependencies, topological order, graph-entry initialization, logical
lifetime validation, admission inputs, backend compilation inputs, and output/
completion contracts.

It rejects duplicate/unknown/foreign identities, cycles, unknown resources,
read-before-initialization, incompatible accesses, missing cross-fragment causality,
ambiguous writers, invalid operation shape, redundant explicit data order,
contradictory requirements, inconsistent imports/exports, and invalid non-data order.

Within a fragment, lexical node order orients access hazards. Fragment collection
position is not scheduling authority. Across fragments, every overlapping access with
at least one write requires a matching typed import/export producer-consumer relation.
Preparation does not guess causality.

Timestamp write -> query resolve and query resolve -> buffer copy dependencies are
inferred through exact query and buffer ranges.

The graph contains no ECS systems, gameplay actions, UI routes, SDF nodes, material
graph nodes, renderer feature meaning, or product lifecycle policy.

## Execution lifecycle

```text
collect fragments
    -> prepare and compose
    -> validate
    -> admit/realize for backend
    -> encode
    -> submit
    -> publish submission
    -> progress/complete/read back
    -> retire transient/backend state safely
```

Phase ownership:

- G3: composition, operations, attachments, accesses, graph-entry initialization,
  hazards, query-resolution intent, derived requirements, and graph validation;
- G4: context/device admission and backend resource/shader/pipeline realization,
  portability, cache compatibility, alignment admission, and stale-generation checks;
- G5: progress, pressure, encode/submit, uploads/updates, query resolution, completion,
  cancellation, asynchronous readback, runtime-retirement checks, and delayed
  retirement;
- G7: surface acquisition/presentation, device/surface generations, loss, and
  reconstruction facts.

Fragment creation may be concurrent. Final context composition and submission remain
context-owned unless a later accepted backend contract proves otherwise.

## Operational contracts

### G4 portability and pipeline-cache compatibility

G4 records normalized admission facts and explicit inherited limitations. Portable
baseline, declared extensions, and contained backend specialization remain distinct.

A persisted/derived pipeline cache is valid only when all correctness facts match,
including:

```text
RunenGPU cache schema
WGPU version/cache key
backend family
adapter identity
relevant driver identity/version
program/interface generation
pipeline descriptor hash
enabled features and limits
```

Incompatible cache data is rejected safely and falls back to realization. A cache hit
changes cost, never semantics.

### G5 progress, pressure, callbacks, and shutdown

G5 specifies:

- who drives native polling and which thread/executor may drive it;
- browser/event-loop progress differences without separate terminal semantics;
- callbacks outside internal locks;
- exactly-once terminal delivery;
- quotas for pending submissions, uploads, mappings, readbacks, and backing memory;
- structured rejection, bounded waits, or caller-owned degradation under pressure;
- cancellation meaning before and after backend submission and during mapping;
- shutdown with pending work and no indefinite API-level wait;
- backing-memory release and high-water diagnostics.

Once work is accepted, RunenGPU never silently discards it.

### G6 cost characterization

G6 compares equivalent RunenGPU and narrow direct-WGPU workloads for known compute,
image-processing, and offscreen graphics proofs. It records preparation/validation
CPU time, allocations, command recording, submission cost, staging/readback bytes,
pipeline cold/warm cost, memory high-water marks, GPU timing where supported, and
artifact equivalence.

Performance facts are diagnostic until a separately accepted controlled budget binds
environment and thresholds. No private bypass may make the framework path appear
faster.

### G7 generations, loss, and reconstruction

G7 distinguishes surface outdated/lost/out-of-memory, device lost/out-of-memory, and
backend failure. Device replacement changes context generation and invalidates old
backend realizations.

```text
source-backed
    RunenGPU reports reconstructable source/prepared facts

externally reconstructed
    external owner must recreate/reimport for the new generation

non-reconstructable
    permanent loss reported explicitly
```

Retained handles do not silently bind to a new generation. Runenwerk chooses retry,
recreate, degrade, pause, exit, or user action.

### G8 capture and reproducibility facts

RunenGPU exposes namespaced, versionable facts for a Runenwerk-owned reproducibility
bundle: framework/backend/capability facts, prepared-work diagnostics, source
provenance, generations, submissions, pressure/loss outcomes, and stable capture keys.

RunenGPU does not persist raw handles, pointers, addresses, unversioned debug strings,
or product artifacts. Runenwerk owns schema, retention, redaction, validation,
migration, and encoding.

## Shader and pipeline boundary

A consumer owns shader/kernel meaning and source. RunenGPU owns admission, interface
validation, and backend realization in G4.

A source descriptor may include stable source key, source revision, language/IR,
entry points, interface declarations, requirements, specialization schema, and
provenance.

RunenGPU owns source admission, backend validation, module realization, resource/
binding layout, validated binding keys, pipeline creation/specialization, compatible
backend cache facts, and structured failures.

RunenGPU does not own filesystem roots, path discovery, polling, file watching,
user-facing reload UI, or last-known-good product policy.

Logical parameters, byte representation, binding representation, and backend layout
remain distinct. WGSL/WGPU layout is not universal domain semantics.

No macro package is accepted before G4 proves byte-layout, alignment, nested type,
package-renaming, compile-pass, and compile-fail requirements.

## WGPU backend

G4 may use WGPU internally for instance/adapter/device/queue, feature/limit/format
mapping, resources/views, shader modules/pipelines, query sets/alignment admission,
and render attachment/resolve compatibility admission.

G5 uses the admitted backend for command encoding/submission, staging, attachment
realization, query resolution, progress, completion/cancellation, asynchronous
mapping/readback, and delayed retirement.

G7 adds surface creation/configuration/acquisition/presentation and device/surface
outcomes.

The public API separates normalized RunenGPU semantics from WGPU facts. A second
backend is not required before extraction. Backend neutrality is applied where
semantic value exists, not for abstraction's own sake.

Current raw public `Device`/`Queue` reach-through and Winit-coupled `WgpuCtx` are
transitional Runenwerk evidence. G4/G8 must contain and remove that reach-through.

## Surface boundary

Runenwerk or another host owns window creation/destruction, event loop, raw handle
lifetime, DPI/monitor/resize/visibility/presentation policy, and product recovery.

RunenGPU G7 owns admitting host-provided raw handles without Winit, low-level surface
creation/retirement, capability/configuration, image acquisition/lease lifetime,
present operation, generations, and structured surface/device outcomes.

RunenRender owns logical output image and color meaning.

An image acquired from an old generation cannot be presented through a newer
configuration. G7 proves affinity, handle lifetime/drop order, multi-surface behavior,
headless independence, resize/reconfiguration, and outcomes by reusing accepted G6
workloads rather than creating a surface-only architecture.

## Readback

G5 readback is asynchronous:

```text
ReadbackRequest
    -> ReadbackId
    -> Pending
    -> Ready(bytes) | Failed(error) | Cancelled
```

Readback does not require blocking the submission authority. RunenGPU returns
normalized bytes/layout/format provenance and completion facts. Callers own semantic
decoding and artifact policy.

Query resolution is not readback. It converts query-set results into a device buffer
through typed work. A later copy/readback request and decoder produce host-visible
evidence.

The current timing path's synchronous `device.poll(wait_indefinitely)` and channel
wait are migration evidence, not the future G5 public progress contract.

## Error model

Required categories include:

```text
identity/allocation
capability requirement/admission
portability/degradation
resource/descriptor validation
typed-data preparation and decoding
work-operation and graph validation
shader/pipeline realization and cache compatibility
pressure/quota/progress
submission/completion/cancellation
readback
surface outcomes
device/loss/reconstruction outcomes
terminal/shutdown
```

Every public error has typed fields for programmatic matching and human-readable
presentation containing operation, label, cause, provenance where applicable, and
corrective action.

Deterministic planning failures remain distinct from backend/environment outcomes.
No panic, generic string matching, `anyhow`, or log text is the normal public
contract.

## Diagnostics

RunenGPU exposes structured facts for:

- backend/adapter selection and portability class;
- requested/granted/degraded capabilities;
- resource creation, realization, use, generations, and retirement;
- work validation and compilation;
- shader/pipeline realization and cache hit/miss/rejection;
- command encoding and submission;
- progress ownership, quota pressure, and backing-memory high-water marks;
- uploads, attachment/query resolution, completion, cancellation, and readbacks;
- timings and direct-path comparison evidence;
- surface/device/loss/reconstruction outcomes;
- terminal shutdown.

Every fact retains correlation:

```text
GPU fact
    -> resource/work node/submission
    -> work fragment
    -> contributing consumer
    -> source provenance
```

RunenGPU reports facts. Hosts decide severity, storage, user presentation, retry,
recovery, persistence, and redaction policy.

## Interaction matrix

| Concern | RunenGPU | RunenRender | Domain/framework | Runenwerk |
|---|---|---|---|---|
| Context/device/queue | Owns | Uses | Uses through work/adapters | Creates/configures policy |
| Resources and hazards | Owns/validates | Declares | Declares | Composes |
| Compute/render/copy execution | Owns | Contributes | Contributes | Orders application epochs |
| Shader realization/cache facts | Owns | Supplies render shaders | Supplies kernels | Supplies source revisions/policy |
| Image formation | No | Owns | Supplies prepared data | Selects product policy |
| Simulation/field algorithms | No | No | Owns | Schedules/integrates |
| Progress/pressure | Owns facts/mechanism | Observes | Observes | Chooses product policy |
| Window lifecycle | Surface facts only | Logical target intent | No | Owns |
| Capture/artifact encoding | Supplies facts/readback | Supplies image meaning | Supplies domain context | Owns persistence/encoding |
| Product recovery | Reports loss/reconstruction | Adds render context | Adds domain context | Owns decision |
| UI state/layout/hit testing | No | No | RunenUI owns | Hosts |

## RenderFlow disposition

Current `RenderFlow` is a transitional combined facade. It is decomposed, not moved,
renamed, or wrapped wholesale.

| Current responsibility | Target owner |
|---|---|
| GPU resource identity, descriptions, access, generic work | RunenGPU |
| WGPU context, resources, pipelines, progress, submission | RunenGPU backend |
| views, targets, rendering, image-formation semantics | RunenRender |
| ECS projection and fixed-step scheduling | Runenwerk adapters |
| shader-file paths, hot reload, windows, built-in UI, capture/export/recovery policy | Runenwerk adapters |

Specific migration rules:

- state projection and dispatch-from-state remain outside RunenGPU;
- shader asset paths remain Runenwerk source authority;
- target aliases, history, fullscreen, graphics, and procedural image meaning remain
  RunenRender authority;
- generic resource descriptions and typed handles move to RunenGPU;
- strings cease to be resource/binding/dependency/capture authority;
- broad `.depends_on` chains are replaced by G3 inferred dependencies;
- redundant explicit data edges are rejected;
- current attachment/timestamp declarations lower into exact G3 facts;
- repeated `.finish()` ladders are not retained;
- ordinary submission does not require a separate `.validate()` call;
- temporary `RenderFlowId` owner bridge remains only through G3 and is removed in G4;
- raw public WGPU `Device`/`Queue` access is contained and removed through G4/G8;
- current product residency/capture/readiness policy remains Runenwerk-owned evidence.

G2-G7 migrate and delete authority incrementally. G8 is final conformance and residual
audit, not delayed bulk migration.

## Proof portfolio

Evidence remains separated:

```text
deterministic conformance
boundary integration
operational pressure
recovery and reproducibility
performance characterization
visual showcase
```

A broad visual demo cannot replace exact correctness evidence. Performance
measurements are not pass/fail thresholds until a controlled specification is
accepted.

### G5 deterministic compute conformance

Prefix scan:

```text
exactly 4,097 fixed integer inputs, all one
complete inclusive and exclusive output comparison
exact total 4,097
workgroup-boundary and multi-level temporary storage
upload, multiple dispatches, completion, asynchronous readback
no renderer, ECS, surface, window, or product types
```

Game of Life:

```text
grid: 160 x 90
seed: 0xC0FF_EE11
steps after seed: 16
boundary: toroidal
expected live cells: 2,063
FNV-1a-64 over little-endian u32 cells: 0xBD710B88594CD584
```

The proof compares the full GPU result with a CPU reference and selected cells.
Simulation state is prepared outside RunenGPU.

A conditional deterministic integer compute-to-texture proof covers texture copy,
row-padding normalization, explicit format handling, selected texels, and
Runenwerk-owned PNG encoding.

### G5 operational conformance

Required proofs:

- submission saturation and recovery of capacity;
- pending-readback and backing-memory saturation;
- staging/upload pressure;
- callbacks outside locks and allowed reentrancy;
- native/WebGPU terminal-semantic equivalence;
- cancellation before/after submission and during mapping;
- shutdown with pending work and no lost terminal outcome.

### G6 graphics and cost

- offscreen known-pattern clear/draw with selected-pixel assertions;
- compute-generated indirect data consumed by graphics through inferred order;
- offscreen boids for representative shared compute/render integration;
- direct-WGPU comparison for equivalent compute, image-processing, and graphics
  proofs;
- cold/warm pipeline and memory/CPU/GPU cost evidence.

Boids validates structure, counts, finite values, bounds, and overflow. It does not
require exact cross-backend floating-point state or pixels.

### G7 lifecycle

Reuse G6 workloads for presentation, resize/reconfiguration, generation changes,
multi-surface behavior where supported, affinity, device/surface outcomes, and the
source-backed/external/non-reconstructable recovery matrix.

### G8 retained conformance

Prove cache compatibility/rejection, pressure, pending-work shutdown, loss and
reconstruction, bounded diagnostics/capture facts, reproducibility bundle inputs,
no raw WGPU reach-through, and retained standalone/downstream public API evidence.

### RunenRender proofs

Procedural sky/SDF terrain is the first semantic image-formation proof after
standalone RunenGPU acceptance. Incremental prepared scenes and synthetic volume add
provider/cache pressure. Boids follows as simulation-to-render integration. SDF
history remains a later temporal/history ownership proof.

## Offline output boundary

Preferred order:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky/SDF/scene sequences after corresponding RunenRender phases.

Runenwerk owns output clock, seeds, job configuration, bounded in-flight readbacks,
ordered filenames, manifests, retry/failure policy, encoding, and external codec
integration. RunenGPU owns completion/readback facts. RunenRender owns image
formation. Neither framework owns MP4/WebM codecs.

## Conformance

Internal proof requires:

1. neutral capability/resource/data validation without Runenwerk or a window;
2. owner-scoped identity tests covering invalid, foreign, exhausted, and non-wrapping
   behavior;
3. accepted G3 access, attachment, operation, requirement, query-resolution, graph,
   cycle, hazard, initialization, causality, import/export, and deterministic-order
   tests;
4. G4 portability, cache compatibility, generation, layout, and backend containment
   proof;
5. G5 exact headless compute, progress, pressure, callback, completion, cancellation,
   readback, pending-work shutdown, and retirement proof;
6. G6 offscreen graphics and direct-WGPU comparison independently of presentation;
7. one shared context executing render and independent non-render work;
8. G7 structured surface/device/loss/reconstruction outcomes with product recovery
   outside RunenGPU;
9. G8 cache, bundle-fact, bounded diagnostic/capture, shutdown, and reach-through
   audit;
10. no renderer/domain meaning or Runenwerk type in future-transferable source;
11. deterministic evidence separated from environment-dependent GPU evidence;
12. simple and inspectable APIs using the same authority;
13. source/dependency/stable-format guards proving no compatibility or duplicate
    authority.

External repository conformance additionally requires independent locked
format/test/Clippy/rustdoc validation, declared edition/MSRV, public downstream
consumer proof, license/provenance, exact-revision Runenwerk integration, and deletion
of original authority with no mirror/submodule/compatibility/moving-branch path.

## S0 and current-source revalidation gate

S0 is complete historical discovery evidence for every GPU/render file, macro,
shader, test, example, benchmark, artifact, dependency, consumer, identity, lifetime,
persistence risk, graph/resource/pipeline/surface/device flow, and move/stay/redesign/
delete classification.

Every phase re-verifies its affected current-main subset. S0 completion does not
authorize later implementation or override newer evidence.

Before source changes, the owning issue verifies exact `main`, runs canonical baseline
validation, repeats direct/transitive consumer census, confirms no stable persisted/
wire/cache/external contract change, and stops for a new ADR, package, dependency,
compatibility path, or premature later-phase owner.

## Extraction sequence

```text
S0 complete inventory                                      complete
G1A owner-scoped logical GPU work-resource identity         complete
G2 capabilities, resources, typed handles, prepared data    complete
G3 decision-complete planning                               complete
operational hardening #176 / PR #178                        complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 access, operations, initialization, hazards, work graph  corrected candidate; new review pending
G4 context/device, shader/pipeline, binding/layout, WGPU     pending
G5 progress/pressure/execution/readback/retirement          pending
G6 offscreen graphics/shared consumers/direct baseline     pending
G7 surfaces/generations/loss/reconstruction                 pending
G8 operational conformance/diagnostics/shutdown/audit       pending
GX external dornglut/runen-gpu clean cutover                blocked
```

Only one next implementation authority is active at a time. Issue `#177` recorded
the exact post-PR-`#180` revalidation before G3 source changes. G4 remains deferred
until G3 is independently reviewed and merged. G2-G7 migrate/delete replaced
authority incrementally. G8 is final conformance and residual audit.

## External cutover

GX is a clean cutover:

1. populate `dornglut/runen-gpu` from accepted internal source;
2. preserve source provenance and license;
3. establish independent validation, MSRV, documentation, downstream, operational,
   and performance evidence;
4. pin Runenwerk to an exact accepted revision;
5. migrate every active consumer;
6. delete internal GPU authority and temporary adapters;
7. prove no mirror, forwarding package, compatibility namespace, duplicate context,
   duplicate descriptor, or duplicate execution path remains.

No submodule, source include, moving-branch dependency, or long-lived dual path is
accepted.

## Strategic reevaluation gates

Reconsider the split if:

- no independent non-render consumer exists by G6;
- ordinary consumers require raw WGPU;
- measured overhead lacks reusable correctness value;
- a direct-WGPU or smaller existing renderer path satisfies accepted needs with less
  ownership;
- backend-neutral contracts repeatedly encode WGPU details;
- progress, pressure, or recovery requires product policy inside RunenGPU.

Reevaluation requires an explicit architecture decision and does not authorize a
hidden bypass.

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
- renderer semantics inside RunenGPU;
- a shared RunenCore, compatibility package, or public raw WGPU escape hatch.

## Definition of done

RunenGPU extraction is complete only when:

- `dornglut/runen-gpu` contains one independently validated public package;
- Runenwerk and RunenRender use only public APIs;
- public contracts contain no Runenwerk, RunenRender, ECS, SDF, UI, Winit,
  application, product, or domain types;
- headless compute, uploads, progress, pressure, completion, asynchronous readback,
  offscreen graphics, and surfaces pass;
- exact conformance and representative showcase evidence remain separate;
- at least one non-render consumer proves independent value;
- cache compatibility, device generations/loss/reconstruction, and terminal shutdown
  are accepted;
- direct-WGPU comparison demonstrates acceptable measured boundary cost;
- simple and inspectable public paths are proven;
- exact revision, MSRV, license, and provenance are recorded;
- every active consumer is migrated;
- original Runenwerk GPU authority and temporary seams are deleted;
- no duplicate context/resource/descriptor/workload path or private WGPU reach-through
  survives.
