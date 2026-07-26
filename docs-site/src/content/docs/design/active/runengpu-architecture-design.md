---
title: RunenGPU Architecture Design
description: Canonical architecture for a standalone GPU execution framework beneath RunenRender and Runenwerk adapters.
status: active
owner: gpu
layer: engine/gpu
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-public-api-ergonomics-review.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU Architecture Design

## Status

The architecture is accepted. Implementation proceeds through bounded internal phases before one clean external cutover.

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        active planning/implementation boundary
G3-G8                                not authorized early
GX external extraction               blocked on accepted G2-G8 evidence
```

The target repository is:

```text
GitHub repository: dornglut/runen-gpu
Cargo package:     runen-gpu
Rust crate:        runen_gpu
```

No external package is created until GX. Current source remains inside Runenwerk while each future-transferable boundary is proved and current consumers migrate.

## Architectural position

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic GPU execution
            -> WGPU backend
```

A non-render consumer may use RunenGPU directly:

```text
simulation, image processing, procedural generation, tooling
    -> RunenGPU generic GPU execution
        -> WGPU backend
```

RunenGPU complements WGPU. It does not reimplement Vulkan, Metal, D3D12, WebGPU, shader compilers, or operating-system window systems.

## Mission

RunenGPU is a standalone Rust framework for validated, deterministic, backend-neutral GPU work authoring and WGPU-backed execution.

It provides:

- normalized capabilities and requirements;
- backend-neutral logical resources and typed handles;
- immutable generic GPU work;
- access, initialization, lifetime, and hazard validation;
- deterministic preparation and composition;
- context/device/backend admission;
- shader, pipeline, resource, command, submission, completion, readback, and surface realization through WGPU;
- structured diagnostics and lifecycle facts;
- headless compute, offscreen graphics, and host-provided surface support.

It does not define rendering meaning, application scheduling, ECS extraction, windows, shader-file discovery, hot reload, capture policy, image encoding, video encoding, or product recovery policy.

## Ownership

### RunenGPU owns

- normalized feature, limit, and format facts;
- required, preferred-with-fallback, and disabled requirements;
- logical buffer, texture, texture-view, sampler, and query-set descriptions;
- typed logical resource handles;
- prepared GPU-data contracts and readback decoding boundaries;
- access, subresource, initialization, hazard, lifetime, and retirement validation;
- immutable compute, render, copy, clear, resolve, and present work after G3/G4;
- deterministic preparation shared by simple and advanced paths;
- WGPU instance, adapter, device, queue, resource, shader, pipeline, command, submission, completion, readback, and low-level surface realization;
- structured backend, submission, completion, device, and surface facts.

### RunenRender owns

- scenes and prepared scenes;
- views and logical targets;
- materials, media, emitters, and environments;
- visibility, lighting, transport, and reconstruction;
- overlays and image-formation semantics;
- renderer quality and degradation policy;
- lowering semantic render plans into generic RunenGPU work.

### Runenwerk owns

- ECS and domain extraction;
- application scheduling and fixed-time policy;
- windows and event-loop integration;
- shader-file discovery, watching, revision, and hot reload;
- capture selection and artifact policy;
- offline batch jobs and bounded frame production;
- PNG and EXR encoding;
- external FFmpeg or other codec integration;
- product recovery and diagnostics presentation.

## Forbidden public dependencies

RunenGPU public contracts must not import or expose:

```text
Runenwerk
RunenRender
ECS
RunenUI
RunenSDF or source-domain types
Winit
application or product types
scene, view, material, lighting, transport, or overlay types
shader filesystem paths
capture, PNG, EXR, or codec policy
fixed-time scheduling
raw WGPU types as the universal public API
```

WGPU exists behind the backend boundary. Narrow native-handle or backend diagnostic access requires a concrete consumer and a separately reviewed containment rule.

## Public experience

The validated work graph is the internal correctness and inspection model. It is not mandatory ceremony for ordinary users.

### Ordinary path

```rust
let simulation = simulation.gpu_work(&gpu, &state)?;
let rendering = renderer.gpu_work(&gpu, &scene, request)?;

let submission =
    gpu.submit("frame 42", [simulation, rendering])?;
```

`submit` validates automatically.

### Inspectable path

```rust
let prepared =
    gpu.prepare("frame 42", [simulation, rendering])?;

inspect(prepared.diagnostics());

let submission =
    gpu.submit_prepared(prepared)?;
```

Both paths use the same preparation, validation, and execution authority. There is no second graph, compatibility path, or reduced validation path.

### Progressive disclosure

```text
level 1  domain facade lowers semantic state into GpuWork
level 2  generic typed work authoring
level 3  explicit prepare and inspection
level 4  backend implementation and diagnostics
```

Graph, epoch, admission, realization, and retirement terminology remains internal or advanced. Common callers should not need to understand it.

## Ergonomic invariants

- strings are diagnostic labels, not identity, binding, or dependency authority;
- resource references are typed;
- admitted pipeline interfaces expose validated binding keys after G4;
- builders use lexical or closure scope rather than nested `.finish()` ladders;
- data dependencies are inferred from resource accesses after G3;
- explicit ordering is only for real non-data dependencies;
- ordinary submission validates automatically;
- public handles are cloneable, non-copy RAII values;
- the last handle drop schedules safe backend retirement only after relevant submissions complete;
- errors identify the human operation and resource, explain the cause, preserve typed facts, and suggest correction;
- no panic, string matching, or backend-only enum dump is the normal failure contract.

## Capability model

The initial requirement vocabulary is:

```text
Required
Preferred { explicit fallback/degradation }
Disabled
```

An unmentioned capability is irrelevant. `Optional` is not a separate state.

Profiles are convenience recipes that produce ordinary requirements. They cannot override explicit requirements or become a parallel admission authority.

Initial normalized features are limited to current consumer evidence:

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

Initial normalized limits are limited to current validation pressure:

```text
maximum uniform-buffer binding size
maximum storage-buffer binding size
maximum color attachments
maximum vertex buffers
maximum bindings per group
```

Format facts are typed and per format/use. Backend names, adapter names, and labels are diagnostic facts.

Deferred until concrete pressure:

```text
subgroups
external-resource interop
ray-query and ray-pipeline baselines
sparse resources
mesh shaders
video
multiple hardware queues
universal shader IR
```

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
host-to-device transfer
GPU-to-host observation
```

`Imported`, `Exported`, `Readback`, and `SurfaceOwned` are not lifetime classes. Readback and export are operations or relationships. Surface acquisition is ownership. Only combinations justified by current consumers enter the initial API.

Descriptors are immutable and validated at construction. Size arithmetic is checked. Labels are not lookup authority. Imported and surface-acquired resources use explicit constructors and cannot claim RunenGPU-owned initialization.

## Typed handles

The initial public handles are kind-typed:

```text
GpuBufferHandle
GpuTextureHandle
GpuTextureViewHandle
GpuSamplerHandle
GpuQuerySetHandle
```

They contain private logical ownership and `GpuWorkResourceId`. They are `Clone`, not `Copy`, expose no safe raw constructor, and cannot be reinterpreted across kinds.

G2 establishes logical ownership shape. G5 connects last-handle drop to delayed backend retirement. G4 adds validated typed binding and layout views. A buffer handle is not parameterized by one universal Rust representation because one buffer may serve storage, vertex, indirect, and transfer roles.

## Typed GPU data

The boundary is:

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

Readback uses a separate decoder contract.

Prepared data records checked byte length, alignment, stride, element count, and provenance. It never infers GPU safety from arbitrary Rust memory.

Current `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and their derives are transitional Runenwerk/render adapter mechanisms. G2 removes them from new RunenGPU signatures and descriptor-size authority. G4 decides backend layout and macro realization. G5 executes uploads, updates, staging, and readback.

`TypeId` and type names may support process-local diagnostics or adapter lookup. They are not layout, binding, persistence, replay, wire, cache, or shader-interface authority.

## Work and validation

After G3, a generic work fragment is immutable and declares typed resource accesses. Preparation composes fragments, infers data order, and rejects:

- cycles;
- ambiguous writers;
- read-before-initialization;
- invalid use combinations;
- invalid capability/resource combinations;
- use after retirement;
- foreign or stale handles;
- incompatible explicit non-data ordering.

RunenGPU does not infer application or renderer semantics. The contributor lowers those semantics into generic work before submission.

## RenderFlow disposition

Current `RenderFlow` is transitional. It is decomposed, not moved or wrapped wholesale.

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
- target aliases, history, fullscreen, graphics, and procedural image meaning remain renderer authority;
- generic resource descriptions and kind-typed handles move to RunenGPU;
- broad string lookup and `.depends_on` chains are replaced by typed references and inferred data ordering;
- repeated `.finish()` ladders are not retained;
- explicit `.validate()` is not required on the ordinary submit path;
- the temporary `RenderFlowId` resource-owner bridge is removed through G3/G4.

## Backend boundary

### G4

G4 owns:

- adapter and device admission execution;
- WGPU feature, limit, and format mapping;
- WGPU resource realization;
- shader and pipeline admission;
- validated binding keys;
- backend data layout and macro realization;
- pipeline and resource caches behind neutral contracts.

### G5

G5 owns:

- headless context execution;
- automatic prepare-and-submit;
- uploads and full/partial updates;
- staging;
- completion and cancellation;
- asynchronous readback;
- delayed safe resource retirement;
- terminal shutdown.

### G7

G7 owns low-level host-provided surface admission, generations, configure/acquire/present, resize, thread affinity, drop order, multi-surface behavior, and structured device/surface outcomes. Window policy and recovery remain Runenwerk responsibilities.

## Proof portfolio

Correctness and demonstration are separate.

```text
deterministic conformance
boundary integration
visual showcase
benchmark or stress evidence
```

### G5

- exact inclusive and exclusive 4,097-element `u32` prefix scan;
- counter reset, scatter/compaction, and indirect-argument focused tests;
- headless Game of Life with fixed grid, seed, tick count, full-grid oracle, live-cell count, checksum, and selected-cell assertions;
- conditional integer compute-to-texture and padded readback proof.

### G6

- offscreen known-pattern graphics conformance;
- compute-generated indirect draw with inferred ordering;
- offscreen boids as a representative showcase with structural and bounded invariants rather than exact cross-backend floating-point equality.

### G7

Reuse the accepted G6 workloads for presentation, resize, reconfiguration, generations, multi-surface behavior, thread affinity, and structured outcomes. Do not create a surface-only execution architecture.

### First RunenRender proof

Procedural sky/SDF terrain is the first image-formation proof after standalone RunenGPU acceptance. Boids follows as simulation-to-render integration. The SDF history flow remains a later temporal/history ownership proof.

## Offline output boundary

Preferred sequence:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky/SDF/scene sequences after matching RunenRender phases.

Runenwerk owns the output clock, seeds, job configuration, bounded in-flight readbacks, ordered filenames, manifests, retries, failure policy, PNG/EXR encoding, and external video encoding. RunenGPU owns completion/readback facts. RunenRender owns image formation. Neither framework owns MP4/WebM codecs.

## Phase program

| Phase | Responsibility | State |
|---|---|---|
| S0 | complete source, consumer, lifecycle, and disposition census | complete |
| G1A | owner-scoped logical GPU work-resource identity | complete |
| G2 | capabilities, resources, typed handles, prepared-data boundary | active |
| G3 | access, hazards, immutable work, dependency inference, internal graph | pending |
| G4 | context/device admission, WGPU mapping, shaders, pipelines, binding/layout realization | pending |
| G5 | headless execution, uploads, completion, readback, cancellation, retirement | pending |
| G6 | offscreen graphics and shared compute/render proof | pending |
| G7 | surfaces and device outcomes | pending |
| G8 | final diagnostics, shutdown, conformance, and residual audit | pending |
| GX | external `dornglut/runen-gpu` clean cutover | blocked on G2-G8 |

G2-G7 migrate and delete replaced authority incrementally. G8 is the final conformance and residual-authority audit, not a bulk cleanup substitute.

## External cutover

GX is a clean cutover:

1. create/populate `dornglut/runen-gpu` from accepted internal source;
2. preserve provenance and license;
3. establish independent validation and downstream conformance;
4. pin Runenwerk to an accepted revision;
5. migrate every active consumer;
6. delete internal GPU execution authority and temporary adapters;
7. prove no source mirror, forwarding package, duplicate context, or duplicate execution path remains.

No submodule, moving-branch dependency, compatibility package, or long-lived dual path is accepted.

## Explicit non-goals

The initial extraction does not include:

- a second backend;
- aggressive transient aliasing;
- pass fusion;
- automatic multi-queue scheduling;
- backend-independent shader IR;
- graph visualization UI;
- hardware ray tracing as a baseline;
- advanced sparse/external-resource interop;
- image or video codec ownership;
- renderer semantics inside RunenGPU.

These require concrete consumer evidence after the accepted framework boundary exists.
