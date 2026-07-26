---
title: Separate GPU Execution from Rendering
description: Accepted ownership and dependency decision establishing RunenGPU as the shared GPU execution framework beneath RunenRender.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-26
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
related_roadmaps:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# ADR 0015: Separate GPU Execution from Rendering

## Decision

Create two independent framework repositories with one public package each:

```text
product       repository                  package       crate
RunenGPU      dornglut/runen-gpu          runen-gpu     runen_gpu
RunenRender   dornglut/runen-render       runen-render  runen_render
```

The required dependency direction is:

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic GPU execution
            -> WGPU backend
```

Non-render consumers may use RunenGPU directly.

RunenGPU may use WGPU as its first internal backend. RunenRender must not own a WGPU device, queue, surface, allocator, command encoder, or competing GPU resource/error model.

## Context

The current Runenwerk renderer grew as one operational subsystem. It combines:

- generic GPU resource and execution mechanics;
- renderer semantics;
- ECS and application projection;
- window and surface policy;
- shader-file discovery and hot reload;
- fixed-step scheduling;
- built-in UI composition;
- capture and artifact policy;
- product diagnostics and recovery.

That combined shape is useful historical implementation evidence but is not a reusable framework boundary.

A renderer-only extraction would leave simulations, procedural generation, image processing, tools, and future independent consumers either dependent on renderer vocabulary or using WGPU through parallel ad hoc paths. A WGPU-only wrapper would not provide the ownership, validation, typed work composition, lifecycle, and diagnostics required by Dornglut consumers.

The accepted split therefore places a general validated GPU execution framework below a semantic renderer.

## RunenGPU ownership

RunenGPU owns:

- normalized capabilities, limits, format facts, and requirements;
- backend-neutral logical resources and typed handles;
- generic access, initialization, hazard, lifetime, and retirement validation;
- immutable generic GPU work and deterministic preparation;
- context/device/backend admission;
- WGPU resource, shader, pipeline, command, submission, completion, readback, and low-level surface realization;
- structured backend and lifecycle facts.

RunenGPU does not own scenes, views, logical render targets, materials, lighting, transport, reconstruction, overlays, ECS, windows, shader-file policy, capture policy, image encoding, video encoding, or product recovery.

## RunenRender ownership

RunenRender owns:

- prepared scenes and renderer identities;
- views and logical targets;
- materials, media, emitters, and environments;
- visibility and interaction semantics;
- lighting and transport;
- reconstruction and history semantics;
- overlays and image-formation semantics;
- lowering renderer plans into generic RunenGPU work.

RunenRender depends on RunenGPU. It does not expose or own WGPU execution authority.

## Runenwerk ownership

Runenwerk owns:

- ECS and domain extraction;
- application scheduling and fixed-time policy;
- windows and event-loop integration;
- shader-file discovery, watching, revision, and hot reload;
- built-in product/UI integration;
- capture selection and artifact policy;
- offline jobs, ordered frame output, PNG/EXR encoding, and external codec invocation;
- recovery and diagnostics presentation.

Reusable adapters may be extracted later only after an independent consumer proves stable ownership.

## Public API consequence

The validated work graph is RunenGPU's internal correctness and inspection model. It is not mandatory common-path ceremony.

Ordinary use is conceptually:

```rust
let simulation = simulation.gpu_work(&gpu, &state)?;
let rendering = renderer.gpu_work(&gpu, &scene, request)?;
let submission = gpu.submit("frame 42", [simulation, rendering])?;
```

Advanced inspection is conceptually:

```rust
let prepared = gpu.prepare("frame 42", [simulation, rendering])?;
inspect(prepared.diagnostics());
let submission = gpu.submit_prepared(prepared)?;
```

Both paths use one preparation and validation authority. Ordinary submission validates automatically.

## Resource consequence

RunenGPU must model these dimensions independently:

```text
kind
    buffer, texture, texture view, sampler, query set

lifetime
    transient, retained

ownership
    RunenGPU-owned, imported, surface-acquired

transfer and observation
    initial data, update/upload, copy, readback, export relationship

reconstruction
    source-backed, externally reconstructed, non-reconstructable
```

Imported, exported, readback, and surface-acquired are not interchangeable lifetime classes.

## Typed-data consequence

The required boundary is:

```text
Runenwerk or source-domain adapter
    ECS or domain state
        -> explicit prepared typed value or bytes
            -> RunenGPU upload/update contract
```

Uniform, storage, vertex, indirect, transfer, and readback layouts are distinct. No universal derive may imply one valid representation for every purpose.

G2 binds semantic ownership. G4 binds backend layout and derive/macro realization. G5 performs uploads and readback.

## RenderFlow consequence

Current `RenderFlow` is a transitional combined facade. It is decomposed rather than moved, renamed, or wrapped wholesale.

| Current responsibility | Target owner |
|---|---|
| GPU resource identity, descriptions, access, generic work | RunenGPU |
| WGPU context, resources, pipelines, submission | RunenGPU backend |
| views, targets, rendering, image-formation semantics | RunenRender |
| ECS projection and fixed-step scheduling | Runenwerk adapters |
| shader-file paths, hot reload, windows, built-in UI, capture/export policy | Runenwerk adapters |

Useful readability may be reproduced only where it does not retain mixed ownership.

## Extraction sequence

The accepted RunenGPU sequence is:

```text
S0 complete inventory
G1A owner-scoped logical work-resource identity
G2 capabilities, resources, typed handles, prepared-data boundary
G3 access, hazards, generic work, dependency inference, internal graph
G4 context/device admission, WGPU mapping, shaders, pipelines, binding/layout realization
G5 execution, uploads, completion, readback, cancellation, retirement
G6 offscreen graphics and shared compute/render proof
G7 surfaces and device outcomes
G8 final diagnostics, shutdown, conformance, and residual audit
GX external dornglut/runen-gpu clean cutover
```

S0 and G1A are complete. G2 is active. G2-G7 migrate and delete replaced authority incrementally. G8 is the final conformance and residual audit, not a delayed bulk cleanup phase.

The temporary `RenderFlowId` bridge used to seed logical resource owner scope is removed through G3/G4.

## Proof consequence

Evidence remains separated into:

```text
deterministic conformance
boundary integration
visual showcase
benchmark or stress evidence
```

The primary deterministic compute proof is exact `u32` prefix scan/readback. Headless Game of Life is stateful integration. A known-pattern offscreen draw is graphics conformance. Compute-generated indirect draw is GPU-driven composition. Boids is a representative showcase, not the correctness oracle. Surface proof reuses accepted offscreen workloads.

## External cutover rule

External repositories are populated only after the corresponding internal boundary is accepted.

The cutover must:

- preserve source provenance and license;
- establish independent validation and downstream conformance;
- pin consumers to exact accepted revisions;
- migrate all active consumers;
- delete the original internal authority and temporary seams;
- leave no source mirror, forwarding package, compatibility namespace, submodule, or parallel runtime path.

## Rejected alternatives

### Keep WGPU ownership in Runenwerk and extract only helpers

Rejected because it preserves application ownership of the generic GPU execution layer and prevents independent non-render consumers.

### Put GPU execution inside RunenRender

Rejected because simulations, tools, and image processing would depend on renderer semantics or create competing WGPU paths.

### Copy RenderFlow into one of the new repositories

Rejected because RenderFlow combines GPU mechanics, renderer semantics, ECS projection, scheduling, windows, UI, capture, and product policy.

### Wrap every WGPU type one-for-one

Rejected because abstraction is justified only where it adds normalized semantics, ownership, validation, lifecycle, composition, or diagnostics value.

### Create both external repositories immediately

Rejected because clean repository transfer requires accepted internal boundaries, current consumer migration evidence, and deletion readiness.

### Preserve compatibility aliases during migration

Rejected because aliases and forwarding modules would retain duplicate authority and weaken the clean cutover.

## Consequences

### Positive

- rendering and non-render compute share one validated execution framework;
- RunenRender remains semantic and backend-independent;
- WGPU ownership is centralized;
- public resources and lifecycle facts become typed and inspectable;
- independent conformance becomes possible before external extraction;
- later backend or platform evolution is contained behind one boundary.

### Costs

- the current combined renderer must be decomposed incrementally;
- temporary Runenwerk adapters are required while later phases are incomplete;
- current examples and applications must migrate as declaration and execution authority moves;
- external extraction waits for conformance rather than optimizing for immediate repository creation.

These costs are accepted because they remove mixed ownership instead of institutionalizing it.
