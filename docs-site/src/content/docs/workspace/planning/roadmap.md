---
title: Roadmap
description: Maintained high-level sequencing for Runenwerk and its peer frameworks.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-05
related_docs:
  - ../engineering-workflow.md
  - ./active-work.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runengpu-shader-authoring-artifact-boundary.md
  - ../../design/active/runengpu-g4b-contracts-g4c-delivery-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/2026-08-03-runengpu-g4b-g4c-finalization.md
  - ../specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../specs/pt-runengpu-g4c1-resource-realization.ron
  - ../specs/pt-runengpu-g4c2-program-binding-realization.ron
  - ../specs/pt-runengpu-g4c3-pipeline-cutover.ron
---

# Roadmap

This roadmap owns durable sequence and dependency direction. GitHub issues and pull
requests own live status, assignment, delivery, review, and exact accepted revisions.

## Repository family direction

```text
RunenSDF --------+
RunenECS --------+--> Runenwerk adapters and applications
RunenUI ---------+
RunenScheduler --+
                       |
                       +--> RunenRender --> RunenGPU
                       +--> non-render RunenGPU consumers
```

Runenwerk remains the integration and product repository. Framework repositories do
not depend on Runenwerk.

Accepted GPU/render dependency:

```text
RunenRender -> RunenGPU
```

Target layering:

```text
Runenwerk host, source, and product policy
    -> RunenRender semantic image planning
        -> RunenGPU generic resources, programs, work, realization, and execution
            -> private WGPU backend
```

Non-render consumers may lower directly into RunenGPU without depending on RunenRender.

## Accepted RunenSDF cutover

Standalone field mathematics is owned by `dornglut/runen-sdf`. Runenwerk now contains no `domain/sdf` package, source mirror, forwarding namespace, submodule, source include,
or unused external dependency. Runenwerk retains only product/domain integration that
belongs above the standalone public boundary.

## RunenGPU accepted foundation

```text
S0 inventory                         complete
G1A logical resource identity        complete
G2 capabilities and resources        accepted
G3 checked access and work graph     accepted
G4 planning                          accepted
G4A context admission                accepted
shader authoring boundary            accepted
```

Exact accepted revisions and the current authorized slice are recorded in
[Active Work](active-work.md) and the owning GitHub issues.

## RunenGPU current sequence

```text
#209 finalize G4B contracts and G4C decomposition
    -> #187 G4B logical program/interface/pipeline contracts
        -> G4C1 private resource realization
            -> G4C2 private program/layout/bind-group realization
                -> G4C3 private pipeline realization and final cutover
                    -> G5 execution and lifecycle
                        -> G6 offscreen graphics and cost proof
                            -> G7 surfaces and reconstruction
                                -> G8 operational conformance
                                    -> GX standalone extraction
```

Only one RunenGPU implementation slice is active at a time. Documentation-only planning
may precede implementation, but no implementation consumes an unmerged planning branch
as accepted authority.

## G4B — logical program contracts

G4B establishes one backend-neutral WGSL-first authority for:

- bounded process-local source admission and owner/key/revision/full-source consistency;
- typed compute, vertex, and fragment entry points;
- shader-visible resource interfaces and typed binding declarations;
- bind-group and pipeline-layout descriptors;
- specialization schemas and normalized values;
- deterministic compute and render pipeline descriptors;
- runtime binding compatibility;
- one comparison contract for mandatory canonical WGSL agreement;
- understandable public compile-pass and compile-fail examples.

`GpuProgramInterfaceDescriptor` owns resource bindings only. Vertex-buffer input and
fragment/color-target output state remain render pipeline state.

G4B creates no WGPU object. Runenwerk retains source files, authoring compilers, module
resolution, watching, reload scheduling, last-known-good artifacts, product fallback,
and persisted artifact policy.

## G4C — ordered private WGPU realization

Issue `#188` remains the G4C umbrella and is not directly implemented.

### G4C1 — resource realization

Owns private context/device-generation-bound realization of:

- buffers;
- textures;
- texture views;
- samplers;
- query sets;
- transactional resource registries and in-memory compatibility caches.

It migrates and deletes fully replaced renderer-owned resource authority. It does not
parse WGSL or create layouts, bind groups, or pipelines.

### G4C2 — program and binding realization

Owns:

- canonical WGSL module creation;
- mandatory pinned WGPU/Naga agreement with explicit G4B declarations;
- bind-group layouts;
- pipeline layouts;
- typed bind groups;
- their private registries and in-memory compatibility caches.

It consumes accepted G4C1 resource handles and does not create compute or render
pipelines.

### G4C3 — pipeline realization and final cutover

Owns:

- compute and render pipeline realization;
- complete pipeline compatibility keys;
- migration of every remaining current realization consumer;
- deletion of renderer-owned resource/program/layout/bind-group/pipeline cache
  authority and synthetic handle paths;
- removal of G4-owned truth from the temporary execution sidecar;
- one named scoped raw-WGPU bridge retained only for G5 execution migration.

G5 deletes the bridge. G4C3 does not implement G5 execution or G7 surfaces.

## G5-G8 sequence

### G5 — execution and lifecycle

Owns accepted work encoding, uploads, query resolution, queue submission, native/web
progress, pressure, completion, asynchronous readback, cancellation, runtime
retirement, delayed destruction, and pending-work shutdown.

### G6 — offscreen graphics and cost proof

Owns representative offscreen compute/render workloads, shared render/non-render use,
cold/warm characterization, direct-WGPU comparisons, and measured boundary cost.

### G7 — surfaces and reconstruction

Owns raw-handle admission, surface identity/configuration/acquisition/presentation,
thread affinity, device generations after replacement, loss classification, and
reconstruction reports.

### G8 — operational conformance

Owns recovery, reproducibility facts, diagnostics, cache and pressure behavior,
shutdown, performance characterization, and the final no-reach-through audit.

### GX — standalone transfer

Transfers accepted internal RunenGPU authority to `dornglut/runen-gpu`, proves
standalone and downstream conformance, cuts Runenwerk over through public APIs, and
deletes internal duplicate authority without mirrors, forwarding packages, source
includes, submodules, branch dependencies, or compatibility paths.

## RunenRender sequence

RunenRender implementation remains blocked until accepted external RunenGPU cutover and
a separately authorized R-phase issue.

Every R phase preserves this semantic spine:

```text
RenderSceneStore
    -> RenderSceneCommit(RenderSceneSnapshot + RenderChangeSet)

RenderSceneSnapshot + RenderRequest + RenderInputSet
    -> RenderMethod
        -> RenderPlan
            -> AdmittedRenderPlan
                -> RenderWorkSet
                    -> RunenGPU
```

Durable sequence:

```text
R1  scene revisions, identities, relationships, and cheap snapshots
R2  space, time, typed dynamic inputs, and availability
R3  representation offers, sampling footprints, protocols, and narrow results
R4  views, outputs, measurement, materials, methods, and planning
R5  founding analytic and field/SDF method through RunenGPU
R6  derived state, residency, sessions, and invalidation
R7  multiview, multi-output, surface, readback, and merge integration
R8  scalability, extension, operational, and extraction conformance
RX  standalone RunenRender transfer and clean cutover
```

The canonical RunenRender design owns detailed semantics and conformance. This roadmap
owns only sequence and dependency direction.

G4 is GPU/backend decontamination and substrate work. It does not implement or extract
RunenRender semantics.

## Other repository-family tracks

RunenSDF remains the accepted standalone field-mathematics authority. RunenECS,
RunenScheduler, RunenSpatial, and RunenUI continue through separately owned programs and
may proceed in parallel only when repository, branch, workspace, files, authority, and
dependencies do not conflict.

No track consumes an unmerged branch as accepted authority.

## Program rules

- WGPU remains the first backend; do not reimplement native APIs.
- Public RunenGPU contracts remain independent of ECS, Winit, renderer semantics,
  application domains, product policy, and filesystem source policy.
- Work graphs and realization registries are correctness and inspection machinery, not
  mandatory ceremony for ordinary callers.
- Derived caches are non-authoritative and full typed equality follows hashing.
- Accepted work is never silently discarded and receives one terminal outcome.
- Queues, staging, readbacks, caches, captures, and history expose bounds and structured
  pressure.
- No second backend, universal shader IR, custom shader language, macro package, stable
  backend cache, or broad unsafe escape hatch is added without separate accepted need
  and proof.
- Replaced authority is migrated and deleted in the same accepted slice; no forwarding
  alias or parallel old/new path remains.
