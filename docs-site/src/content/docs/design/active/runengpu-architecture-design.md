---
title: RunenGPU Architecture Design
description: Decision-complete ownership, workload, resource, capability, WGPU, surface, diagnostics, conformance, and extraction architecture for RunenGPU.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU Architecture Design

## Status

The repository identity, ownership boundary, initial package shape, dependency
direction, WGPU placement, host boundary, and extraction sequence are fixed.

Exact current-file disposition and first implementation scope remain blocked on the
S0 inventory. This document does not authorize Rust changes, source movement, or
repository creation.

## Mission

RunenGPU owns validated execution of GPU resources and workloads.

It answers:

> How are GPU capabilities, resources, accesses, workloads, submissions, results,
> and backend failures represented and executed safely?

It does not answer:

- what an image should contain;
- how light transport works;
- how a field, simulation, material, UI, ECS entity, or world object behaves;
- how an application schedules gameplay;
- how windows and event loops are managed;
- how product recovery is presented to users.

## Repository and package

```text
repository: Crystonix/runen-gpu
package: runen-gpu
crate: runen_gpu
initial backend: WGPU, internal implementation detail
```

Initial repository shape:

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

The module names are directional, not mandatory file names. The public package is
one release unit until real dependency pressure proves otherwise.

Do not initially create `runengpu_core`, `runengpu_wgpu`, facade, macro, testing,
capture, or compatibility packages.

## Dependency rules

The public package must not depend on:

```text
Runenwerk
RunenRender
RunenSDF
RunenECS
RunenUI
Winit
scene/world/material/editor/application domains
```

WGPU may be an internal dependency. Public contracts must not require consumers to
construct or branch on WGPU types unless an explicitly unstable escape hatch is
separately accepted.

Consumers depend downward:

```text
RunenRender --------+
field GPU adapter ---+
simulation adapter --+--> RunenGPU
procedural tools ----+
offline bakers ------+
```

## Ownership

RunenGPU owns:

- GPU context and execution-epoch identities;
- normalized capabilities and requirements;
- backend-neutral resource descriptors and handles;
- access, initialization, lifetime, hazard, and retirement validation;
- shader admission and pipeline realization contracts;
- compute, render, copy, clear, resolve, and present workloads;
- deterministic work-graph composition and validation;
- uploads, asynchronous readback, submission, and completion;
- headless/offscreen initialization;
- low-level surface realization and outcomes;
- WGPU backend mapping;
- GPU diagnostics, timings, and provenance;
- terminal shutdown and device-loss facts.

RunenGPU does not own:

- renderer views, providers, materials, emitters, visibility, transport, or
  reconstruction;
- simulation or field algorithms;
- shader source discovery, file watching, or hot-reload product policy;
- authoritative CPU/domain state;
- ECS storage or scheduling;
- UI semantics or hit testing;
- window/event-loop policy;
- product quality selection, recovery decisions, or diagnostics presentation.

## Context model

A `GpuContext` represents one admitted backend execution authority.

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
- foreign-context and stale-generation values are rejected.

Runenwerk may create one shared context for rendering and non-render workloads.
RunenGPU does not decide how many contexts a product should create.

## Identity model

Runtime identities are opaque, type-distinct, context-scoped, and fallibly
allocated.

Candidate concepts:

```text
GpuContextId
GpuEpochId
GpuBufferId
GpuTextureId
GpuTextureViewId
GpuSamplerId
GpuShaderId
GpuComputePipelineId
GpuRenderPipelineId
GpuSurfaceId
GpuSubmissionId
GpuReadbackId
```

Exact public names remain S0/implementation decisions. Required semantics are:

- no safe arbitrary raw reconstruction;
- no wrapping or saturating reuse;
- explicit exhaustion;
- reserved values never issued;
- foreign-context rejection;
- stale-generation rejection where slots can be reused;
- raw diagnostic values do not imply persistence, replay, cache, or network
  stability;
- deterministic allocation for the same explicit allocator state and operation
  order.

Runtime IDs are not source asset IDs or persisted format keys.

## Capability model

RunenGPU exposes normalized capability facts rather than leaking raw backend feature
enums as universal semantics.

```text
GpuCapabilities
├── limits
├── buffer and texture support
├── format and storage support
├── shader and subgroup support
├── indirect execution
├── timestamps and statistics
├── presentation support
├── external-resource support
├── ray-query support
├── ray-pipeline support
└── backend facts
```

Requirement strength:

```text
Required
Preferred
Optional
Forbidden
```

Every preferred or optional capability must define a fallback, degradation, or
structured rejection policy.

Initial capability profiles:

```text
ComputeBaseline
OffscreenGraphicsBaseline
DesktopPresentationBaseline
```

Hardware ray tracing is optional. Compute-based field traversal must remain a
valid baseline. Experimental backend features do not become stable vocabulary
without cross-backend or cross-consumer evidence.

## Resource model

Initial resource kinds:

```text
Buffer
Texture
TextureView
Sampler
QuerySet
ExternalResource
SurfaceImage
Readback
```

A resource descriptor includes:

- kind, dimensions, and format intent;
- permitted uses;
- initialization state or initial contents;
- lifetime class;
- memory intent;
- debug label;
- provenance;
- reconstruction ownership.

Lifetime classes:

```text
Persistent
FramePersistent
Transient
Imported
Exported
Readback
SurfaceOwned
```

The semantic owner retains authoritative source data and reconstruction policy.
RunenGPU may cache or realize derived state, but device loss must not destroy the
only authoritative copy.

### Resource access

Access declarations support buffer ranges and texture subresources where the
consumer and backend can prove independence.

Access categories include:

```text
Read
Write
ReadWrite
Uniform
StorageRead
StorageWrite
Sampled
CopySource
CopyDestination
ColorAttachment
DepthStencilAttachment
Indirect
Present
```

Validation rejects incompatible overlap, use before initialization, use after
retirement, invalid view/resource relationships, ambiguous writers, and missing
capabilities.

### Imports and exports

Imported resources require explicit ownership and synchronization facts:

```text
owner
runtime identity
access range
validity interval
initial state
required final state
synchronization token
retirement rule
```

Raw backend handles are not a stable public contract by themselves.

## Workload model

Consumers contribute immutable `GpuWorkFragment` values.

Conceptual form:

```text
GpuWorkFragment
├── imported resources
├── declared resources
├── exported resources
├── work nodes
├── explicit dependencies
├── outputs
└── provenance
```

Initial node kinds:

```text
Compute
Render
Copy
Clear
Resolve
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

- identity and kind;
- pipeline intent;
- resource accesses;
- dispatch, draw, copy, clear, resolve, or present intent;
- explicit dependencies;
- capability requirements;
- execution preference;
- debug label and provenance.

Execution preferences:

```text
Automatic
ComputePreferred
GraphicsRequired
TransferPreferred
```

Preferences are hints, not concurrency guarantees. The first backend may serialize
work through one logical queue while preserving dependencies and future scheduling
information.

## Work graph

A `GpuWorkGraph` composes fragments for one bounded execution epoch.

It owns:

- deterministic identity and dependency resolution;
- inferred resource hazards;
- topological ordering;
- initialization and lifetime validation;
- capability admission;
- backend compilation inputs;
- output and completion contracts.

It must reject:

- duplicate, unknown, foreign, or stale identities;
- cycles;
- unknown resources or pipelines;
- read before initialization;
- use after retirement;
- incompatible accesses;
- ambiguous writers;
- invalid surface-image reuse;
- missing capabilities;
- invalid pipeline/resource combinations;
- inconsistent imports/exports.

The graph must not contain ECS systems, gameplay actions, UI routes, SDF nodes,
material graph nodes, renderer feature meaning, or product lifecycle policy.
Higher-level owners lower semantics into GPU work.

## Epoch and submission lifecycle

```text
begin epoch
    -> collect fragments
    -> compose
    -> validate
    -> compile for backend
    -> encode
    -> submit
    -> publish submission
    -> complete/read back
    -> retire transient state
```

The contract defines:

- maximum configured in-flight epochs;
- frame-persistent and history retention;
- transient retirement;
- surface-image ownership;
- asynchronous completion;
- shutdown and cancellation;
- structured terminal outcomes.

Fragment creation may be concurrent. Final context composition and submission
remain context-owned unless a later backend contract proves otherwise.

## Shader and pipeline boundary

A consumer owns shader or kernel meaning and source. RunenGPU owns admission and
backend realization.

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
- pipeline creation and specialization;
- backend cache keys;
- structured failures.

RunenGPU does not own filesystem roots, path discovery, polling, file watching,
user-facing reload UI, or last-known-good product policy.

Logical parameters, byte representation, binding representation, and backend
layout remain distinct. WGSL/WGPU layout is not universal domain semantics.

No macro package is accepted before real consumers prove byte-layout, alignment,
nested type, package-renaming, compile-pass, and compile-fail requirements.

## WGPU backend

The initial implementation may use WGPU internally for:

- instance, adapter, device, and queue;
- resources and views;
- shader modules and pipelines;
- command encoding and submission;
- staging uploads and readback;
- query sets and timings;
- surface creation/configuration/acquisition/presentation;
- capability mapping;
- device and surface outcomes.

The public API must separate normalized RunenGPU semantics from WGPU-specific
facts. Backend-specific extensions may be exposed only through explicitly unstable
or capability-gated interfaces.

A second backend is not required before the first extraction. The architecture is
backend-neutral where semantic value exists, not artificially abstract for its own
sake.

## Surface boundary

Runenwerk or another host owns:

- window creation/destruction;
- event loop;
- raw window/display handle lifetime;
- DPI, monitor, resize, visibility, and presentation timing policy;
- product recovery.

RunenGPU owns:

- admitting host-provided handles;
- low-level surface creation and retirement;
- capability query and configuration;
- image acquisition and lifetime;
- present operation;
- structured surface outcomes.

RunenRender owns output image and color meaning.

Surface configuration and acquired images use generations. An image acquired from
an old generation cannot be presented through a newer configuration.

S0 and the implementation design must prove thread affinity, handle lifetime,
drop order, multi-window, headless operation, resize/reconfiguration, and device
loss.

## Readback

Readback is asynchronous:

```text
ReadbackRequest
    -> ReadbackId
    -> Pending
    -> Ready(bytes) | Failed(error) | Cancelled
```

Readback must not require blocking the submission authority. Callers own decoding
and semantic interpretation.

## Error model

Required categories include:

```text
identity/allocation
capability admission
resource validation
work-graph validation
shader/pipeline realization
submission/completion
readback
surface outcomes
device outcomes
terminal/shutdown
```

Public callers do not branch on generic strings, `anyhow`, panics, or log text.
Deterministic planning failures remain distinct from backend/environment outcomes.

## Diagnostics

RunenGPU exposes structured facts for:

- backend and adapter selection;
- requested/granted capabilities;
- resource creation, residency, and retirement;
- work validation and compilation;
- shader/pipeline realization;
- command encoding and submission;
- uploads and readbacks;
- timings and statistics;
- surface and device outcomes;
- terminal shutdown.

Every fact retains correlation:

```text
GPU fact
    -> resource/work node
    -> work fragment
    -> contributing consumer
    -> source provenance
```

RunenGPU reports facts. Hosts decide severity, storage, user presentation, and
recovery policy.

## Interaction matrix

| Concern | RunenGPU | RunenRender | Domain/framework | Runenwerk |
|---|---|---|---|---|
| Context/device/queue | Owns | Uses | Uses through workloads/adapters | Creates/configures |
| Resources and hazards | Owns/validates | Declares | Declares | Composes |
| Compute/render/copy execution | Owns | Contributes | Contributes | Orders epochs |
| Shader realization | Owns | Supplies render shaders | Supplies kernels | Supplies source revisions |
| Image formation | No | Owns | Supplies prepared data | Selects product policy |
| Simulation/field algorithms | No | No | Owns | Schedules/integrates |
| Window lifecycle | Surface facts only | Logical target intent | No | Owns |
| Product recovery | Reports facts | Adds render context | Adds domain context | Owns |
| UI state/layout/hit testing | No | No | RunenUI owns | Hosts |

## Conformance

Internal proof requires:

1. neutral workload/resource validation can be tested without Runenwerk and without
   constructing a window;
2. one shared context executes a render fragment and a non-render compute fragment;
3. identity tests cover invalid, foreign, stale, exhausted, and non-wrapping use;
4. work-graph tests cover cycles, hazards, initialization, lifetime, imports,
   exports, and capability failure;
5. headless compute and readback pass where the environment supports GPU access;
6. offscreen graphics passes independently of presentation;
7. surface/device outcomes are structured and product recovery remains outside;
8. no renderer/domain meaning appears in RunenGPU public contracts;
9. no Runenwerk type appears in the future-transferable boundary;
10. deterministic planning evidence is separated from environment-dependent GPU
    evidence.

External repository conformance additionally requires:

- independent locked format/test/Clippy/rustdoc validation;
- declared Rust edition and MSRV;
- public downstream consumer proof;
- license and source provenance;
- no Runenwerk source include, submodule, mirror, compatibility package, or branch
  dependency;
- exact-revision Runenwerk integration and deletion of the original authority.

## S0 inventory gate

Before any G1 implementation specification is written, S0 must produce:

- every file under the current GPU/render implementation and macro packages;
- every shader, test, example, benchmark, and generated artifact;
- every Cargo dependency and downstream consumer;
- every current identity, allocator, raw conversion, and handle use;
- persistence, replay, network, cache, and artifact usage of current IDs;
- graph, resource, pipeline, shader, surface, device, frame, and shutdown control
  flows;
- WGPU/Winit/ECS/scene/world/material/SDF/UI/editor/product dependencies;
- shader discovery/reload and macro ABI consumers;
- headless, offscreen, surface, device-loss, shader, benchmark, and runtime command
  inventory;
- exact move/stay/redesign/delete classification;
- current `cargo validate` and relevant GPU/runtime evidence.

S0 is investigation, not implementation. Unknown ownership blocks G1.

## Extraction sequence

```text
S0 complete inventory
G1 identities, errors, and ownership guards
G2 capabilities and resource descriptors
G3 access/lifetime/hazard validation and work fragments
G4 shader/pipeline admission and WGPU realization
G5 headless compute, uploads, and readback
G6 offscreen graphics and shared consumer proof
G7 surfaces, generations, and device outcomes
G8 diagnostics, shutdown, and anti-cheating conformance
GX external repository transfer and clean cutover
```

Phase names and exact scopes may change after S0. Only one next implementation
specification is written at a time from current `main`.

## Definition of done

RunenGPU extraction is complete only when:

- the one-package public boundary validates independently;
- Runenwerk and RunenRender use only public APIs;
- headless compute and offscreen graphics pass;
- at least one non-render consumer proves independent value;
- surface/device lifecycle is accepted;
- exact revision and provenance are recorded;
- every active consumer is migrated;
- the original Runenwerk GPU authority and temporary seams are deleted;
- no duplicate context/resource/workload path survives.
