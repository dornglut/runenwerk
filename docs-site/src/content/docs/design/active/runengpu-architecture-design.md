---
title: RunenGPU Architecture Design
description: General GPU execution ownership, packages, identities, resources, workload planning, WGPU realization, surfaces, diagnostics, and extraction conformance.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../workspace/specs/pt-runengpu-g1-identities-errors.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU Architecture Design

## Status

The repository mission, ownership boundary, required initial package candidates,
and dependency direction are fixed by ADR 0015.

Exact public type names and internal module dispositions remain phase-owned until
the current implementation inventory and each bounded implementation phase close.

No external source movement or broad backend rewrite is authorized by this design.

## Mission

RunenGPU is the shared GPU execution framework.

It answers:

> How are GPU capabilities, resources, workloads, submissions, results, and
> backend failures represented and executed safely?

It does not answer:

- what an image should contain;
- how light transport works;
- how a fluid or field evolves;
- what an SDF, UI widget, ECS entity, or world object means;
- how an application schedules gameplay;
- how a product recovers from a device failure.

## Required packages

```text
runengpu_core
runengpu_wgpu
```

Deferred until evidence:

```text
runengpu_testing
runengpu_macros
runengpu_capture
alternative backend packages
facade package
```

Do not create a facade package initially. Consumers should depend on the exact
contract and backend packages they need until setup and capability policy stabilizes.

## Dependency rules

`runengpu_core` must not depend on:

```text
WGPU
Winit
Runenwerk
RunenRender
RunenSDF
RunenUI
RunenECS
scene/world/material/editor/application domains
```

`runengpu_wgpu` may depend on WGPU and `runengpu_core`. It must not depend on
Winit, Runenwerk, a renderer, ECS, SDF, UI, or domain simulations.

Consumers depend downward:

```text
runenrender_gpu ----+
fluid GPU adapter --+
field GPU adapter --+--> runengpu_core
tools/bakers -------+

application backend wiring --> runengpu_wgpu
```

## Ownership

### runengpu_core

May own:

- GPU-context and execution identities;
- normalized capabilities and requirements;
- backend-neutral resource descriptors;
- resource ranges, access, initialization, and lifetime declarations;
- shader-interface and pipeline intent;
- bounded GPU work fragments and work graphs;
- deterministic validation and plan formation;
- submissions, completions, readback contracts, and terminal states;
- neutral GPU diagnostics and provenance.

### runengpu_wgpu

May own:

- WGPU instance, adapter, device, and queue;
- headless/offscreen initialization;
- WGPU buffers, textures, views, samplers, query sets, and bind groups;
- shader-module and pipeline realization;
- command encoding and submission;
- upload and readback staging;
- low-level surface creation/configuration/acquisition/presentation;
- WGPU capability mapping;
- surface and device outcomes;
- backend timings and diagnostics.

### Runenwerk

Retains:

- context creation and product capability selection;
- windows, event loops, DPI, monitor, resize, and visibility policy;
- CPU/domain scheduling and work-fragment composition;
- shader filesystem discovery and reload policy;
- product recovery and diagnostics presentation;
- integration tests and runtime evidence.

### Consumers

A consumer owns:

- algorithm meaning;
- shader or kernel source;
- numerical invariants;
- source and persistent state;
- prepared inputs;
- output interpretation;
- fallback and product policy outside backend capability negotiation.

## Identity model

Runtime GPU identities are opaque, context-scoped, type-distinct, and fallibly
allocated.

Candidate categories:

```text
GpuContextId
GpuEpochId
GpuWorkId
GpuResourceId
GpuShaderId
GpuPipelineId
GpuSubmissionId
GpuReadbackId
GpuSurfaceId
```

Runtime identities are not:

- persisted asset IDs;
- network identities;
- stable source keys;
- shader source identifiers;
- cache-format keys;
- renderer provider or material identities.

Requirements:

- no safe arbitrary raw reconstruction;
- no wrapping or saturating reuse;
- no reserved sentinel issued as valid;
- explicit exhaustion;
- foreign-context rejection;
- stale-generation rejection where allocation can reuse slots;
- read-only raw values permitted only for diagnostics and ordering;
- deterministic allocation for the same explicit allocator state and operation order.

## Capability model

RunenGPU exposes normalized capabilities rather than raw backend feature enums.

```text
GpuCapabilities
├── limits
├── buffer capabilities
├── texture/format capabilities
├── storage capabilities
├── shader capabilities
├── subgroup capabilities
├── indirect execution
├── timestamp support
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

Every non-required request must define a valid fallback or rejection policy.

Initial profiles:

```text
ComputeBaseline
OffscreenGraphicsBaseline
DesktopPresentationBaseline
```

Hardware ray tracing is optional. Compute-based field traversal must remain viable.
Experimental backend features must not become stable core vocabulary.

## Resource model

Initial resource classes:

```text
Buffer
Texture
TextureView
Sampler
ExternalResource
SurfaceImage
Readback
```

Every descriptor contains:

- runtime identity;
- kind and dimensions;
- format where applicable;
- allowed uses;
- initial contents/initialization state;
- lifetime class;
- memory intent;
- debug label;
- provenance.

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
Device loss must not destroy the only authoritative domain state.

### Imports and exports

Imported resources require an explicit contract:

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

Raw backend handles alone are not a stable public contract.

### Subresources

Access declarations must support buffer ranges and texture subresources where
consumers and backends can prove safe independence. Validation must reject
overlapping incompatible accesses.

## Work model

Consumers contribute immutable `GpuWorkFragment` values:

```text
GpuWorkFragment
├── imported resources
├── declared resources
├── exported resources
├── work nodes
├── external dependencies
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

Deferred:

```text
AccelerationStructureBuild
SparseBinding
ExternalInterop
Video
```

A work node declares:

```text
identity
kind
pipeline
resource accesses
dispatch/draw/copy intent
explicit dependencies
capability requirements
execution preference
debug label
provenance
```

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

The validator must reject:

- duplicate or foreign identities;
- graph cycles;
- unknown resources or pipelines;
- read before initialization;
- use after retirement;
- incompatible access;
- ambiguous multiple writers;
- invalid surface-image reuse;
- missing required capabilities;
- invalid pipeline/resource combinations.

## Graph boundary

The GPU work graph represents one bounded execution epoch.

It must not contain:

- ECS systems;
- gameplay actions;
- simulation ownership;
- renderer feature semantics;
- UI routes or widgets;
- SDF or material graph nodes;
- product lifecycle policy.

Higher-level systems lower their semantics into GPU work.

## Execution preferences

```text
Automatic
ComputePreferred
GraphicsRequired
TransferPreferred
```

These are hints, not concurrency guarantees.

The first WGPU backend may serialize work through one logical queue. The core model
must preserve dependencies and preferences without promising asynchronous compute
or multiple hardware queues.

## Frame and submission lifecycle

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

There is one logical submission authority per live context. Fragment creation may
be concurrent; final composition and submission remain context-owned.

Shutdown must be idempotent and must:

- reject new work;
- settle or classify in-flight work;
- cancel or complete readbacks;
- release surfaces before dependent backend state where required;
- publish terminal diagnostics.

## Shader and pipeline boundary

A domain owns shader meaning and source. RunenGPU owns admission and realization.

A source descriptor may include:

```text
stable source key
source revision
language or IR
entry points
declared interfaces
required capabilities
specialization schema
provenance
```

RunenGPU owns:

- source admission;
- backend validation;
- resource-layout realization;
- pipeline creation;
- specialization;
- pipeline caches;
- structured failures.

RunenGPU does not own:

- filesystem roots;
- asset paths;
- directory watching;
- polling;
- user-facing reload UI;
- last-known-good product policy.

Logical parameters, byte representation, binding representation, and backend
layout must remain distinct. WGSL/WGPU layout is not universal core semantics.

No macro package is accepted before byte-layout, alignment, nested type,
package-renaming, and compile-fail conformance exists across at least two real
consumers.

## Surface boundary

The host owns:

- window creation and destruction;
- event loop;
- DPI and resize policy;
- monitor and visibility state;
- presentation timing policy;
- product recovery.

`runengpu_wgpu` owns:

- surface creation from admitted handles;
- capability query;
- configuration;
- image acquisition;
- surface-image lifetime;
- present operation;
- low-level outcomes.

RunenRender owns output image and color meaning.

Surface configuration and acquired images use generations. An image acquired from
an old generation cannot be presented through a newer configuration.

## Readback

Readback is asynchronous:

```text
ReadbackRequest
    -> ReadbackId
    -> Pending
    -> Ready(bytes) | Failed(error) | Cancelled
```

The API must not require blocking the submission authority.

## Error model

Required categories:

```text
GpuIdentityError
GpuPlanError
GpuValidationError
GpuCapabilityError
GpuRealizationError
GpuSubmissionError
GpuReadbackError
GpuSurfaceOutcome
GpuDeviceOutcome
GpuTerminalError
```

Public callers must not branch on generic strings, panics, or `anyhow`.

Deterministic planning failures remain distinct from backend/environment outcomes.

## Diagnostics

RunenGPU exposes structured facts for:

- selected backend and adapter;
- requested/granted capabilities;
- allocation and residency;
- work-plan validation;
- shader and pipeline realization;
- command encoding and submission;
- uploads and readbacks;
- timings;
- surface outcomes;
- device outcomes;
- terminal shutdown.

Every fact retains:

```text
GPU fact
    -> work node/resource
    -> work fragment
    -> contributing package
    -> source provenance
```

RunenGPU reports facts. Hosts decide severity, presentation, storage, and policy.

## Interaction matrix

| Concern | RunenGPU | RunenRender | Domain/framework | Runenwerk |
|---|---|---|---|---|
| GPU context/device/queue | Owns backend | Uses | Uses through adapters | Creates/configures |
| Resources and hazards | Owns | Declares | Declares | Composes |
| Compute/render/copy execution | Owns | Contributes | Contributes | Orders epochs |
| Shader realization | Owns | Supplies render shaders | Supplies kernels | Supplies revisions |
| Image formation | No | Owns | Supplies prepared data | Selects product policy |
| Simulation algorithms | No | No | Owns | Schedules |
| Window lifecycle | Surface facts only | Logical target intent | No | Owns |
| Product recovery | Reports facts | Reports render context | Reports domain context | Owns |
| UI state/layout | No | No | RunenUI owns | Hosts |
| Field/SDF semantics | No | Consumes adapter | RunenSDF owns | Integrates |

## Feature matrix

| Capability | Foundation | Initial production | Advanced optional | Excluded |
|---|---:|---:|---:|---:|
| Backend-neutral identities | Yes | Yes | — | — |
| Capability negotiation | Yes | Yes | Profiles expand | Product policy |
| Buffer/texture descriptors | Yes | Yes | Sparse/external | Domain meaning |
| Work-plan validation | Yes | Yes | Incremental compile | Engine scheduling |
| Headless compute | Yes | Yes | Multi-device | Solver ownership |
| Upload/readback | Yes | Yes | Streaming optimization | Persistence |
| Offscreen graphics | Proof | Yes | Indirect generation | Rendering semantics |
| Surfaces | No in first proof | Yes | HDR/multi-surface | Window lifecycle |
| Timestamp queries | Optional | Preferred | Pipeline statistics | UI presentation |
| Hardware ray queries | No | Optional | Backend specialized | Baseline dependency |
| Multiple hardware queues | No | No guarantee | Backend specialized | Semantic promise |
| Fluid/world/procgen algorithms | No | No | No | Domain-owned |

## Conformance

Internal extraction proof requires:

1. `runengpu_core` builds/tests without WGPU.
2. `runengpu_wgpu` builds without Runenwerk, Winit, renderer, ECS, SDF, UI, or domain packages.
3. identities reject invalid, stale, exhausted, and foreign usage.
4. deterministic plan validation.
5. cycle, hazard, initialization, and lifetime rejection.
6. headless compute and asynchronous readback.
7. compute output consumed by offscreen rendering.
8. optional surface operation without headless regression.
9. one render consumer and one non-render consumer share a context.
10. no public WGPU leakage from core.
11. structured device/surface outcomes.
12. benchmark baselines for planning, allocation, upload, dispatch, submission, and readback.
13. Runenwerk public-boundary-only consumption.
14. no duplicate GPU path.

## Extraction gate

External transfer is blocked until:

```text
internal packages proven
headless compute proven
offscreen graphics proven
shared consumers proven
surface/device boundary accepted
public-boundary anti-cheating proven
complete validation and benchmarks
exact move/stay/redesign/delete matrix
```

The external repository begins from the accepted internal source. It is not an
independent parallel reimplementation.

## Final contract

> RunenGPU owns validated GPU execution. It does not own what the work means.
