---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runensdf-extraction-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../reports/investigations/runenecs-extraction-investigation.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../specs/pt-runenecs-r1-entity-errors.ron
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../specs/pt-runengpu-g1-identities-errors.ron
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

## Primary extraction track

ID: `PT-RUNENSDF-003`

Title: Standalone RunenSDF Repository Creation and Corrected Source Transfer

The SDF track is handled independently. Its own branch/specification owns source
transfer authority. This GPU/render architecture branch does not modify SDF Rust,
external repository source, Runenwerk SDF dependencies, or `domain/sdf`.

`PT-RUNENSDF-002` boundary correction is complete through PR #116. Detailed
evidence is recorded in the SDF closeout report.

Shared planning files in this branch must be rebased and reconciled after the
active SDF planning PR lands. Neither track may silently overwrite the other's
current lifecycle facts.

## Parallel architecture track

ID: `PT-GPU-RENDER-SPLIT-001`

Title: Separate General GPU Execution from Rendering

Lifecycle state: `active-planning`

Implementation authorization: **documentation, connector-backed investigation,
and local command-gate preparation only**

Branch:

```text
docs/gpu-render-split-architecture
```

Mission:

- establish RunenGPU as a separate framework beneath RunenRender;
- replace the renderer-owned WGPU package target;
- classify current renderer/GPU ownership at module-family level;
- retire the unimplemented old RunenRender R1 specification;
- replace the renderer-only roadmap;
- prepare exactly one planning-only RunenGPU G1 contract;
- preserve the active SDF track and avoid Rust/Cargo/shader changes.

## Decisions fixed by this track

```text
RunenGPU
  required candidates: runengpu_core, runengpu_wgpu
  owns general GPU execution

RunenRender
  required candidates: runenrender_core, runenrender_gpu
  owns image formation
  depends on RunenGPU

Runenwerk
  owns lifecycle, windows, ECS/domain extraction, adapters,
  source reload policy, product policy, diagnostics presentation, and recovery
```

There is no `runenrender_wgpu` target.

RunenUI and RunenRender remain independent peers. RunenUI core/runtime do not
depend on RunenGPU. Initial UI-to-render translation remains Runenwerk-owned.

## Current evidence

Connector-backed inspection confirmed:

- the render root aggregates API, backend, composition, features, frame,
  GPU primitives, graph, material compilation, params, pipelines, procedural,
  renderer, residency, resources, shader reload, inspection, runtime, and plugin
  integration;
- WGPU context construction currently requires a Winit window and creates a
  surface before adapter/device selection;
- current graph execution contains compute, graphics, copy, present, fullscreen,
  and built-in UI semantics;
- current feature registries contain scene, UI, world, cave, material, VFX,
  deformation, wind, and editor product semantics;
- frame preparation reaches into ECS/world/window/time/shader/product state;
- `Render*Id` families mix probable GPU, renderer, and Runenwerk ownership.

This is substantial source evidence, not the complete local command gate.

## Mandatory S0 gate before implementation

Before activating `PT-RUNENGPU-G1`, complete and report:

1. full local file inventory for render code, macros, shaders, tests, examples,
   and benchmarks;
2. complete current consumer/import map;
3. classification of every current `Render*Id` and allocator;
4. persistence/replay/network/cache use of raw IDs;
5. shader/pipeline/macro consumer inventory;
6. surface/window/device lifecycle trace;
7. Cargo/test/Clippy/docs baseline;
8. headless GPU, surface, shader, device-loss, and benchmark command inventory;
9. exact file-level move/stay/redesign/delete matrix.

The commands are recorded in the split investigation and execution plan.

## Next implementation candidate

```text
PT-RUNENGPU-G1
GPU Identities, Structured Errors, and Dependency Guards
```

State: `active-planning-only`

No Rust implementation is authorized until:

- this architecture PR is reviewed and merged;
- the S0 local command gate passes;
- the G1 spec is updated with exact files/consumers against current main;
- a separate activation decision grants implementation authority.

## Program allocation

```text
RunenSDF     separate active extraction track
RunenECS     R1 specified; no Rust implementation authorized
RunenGPU     architecture/inventory planning; G1 not active
RunenRender  old R1 retired; implementation waits for accepted RunenGPU boundary
RunenUI      independent workstream
```

## Allowed parallel work

- SDF standalone transfer in its own thread/branch;
- local GPU/render inventory and command execution;
- read-only later GPU/render investigation;
- independent RunenUI work;
- benchmark and shader/headless/surface command discovery.

## Forbidden without a later phase authorization

- GPU or renderer Rust restructuring;
- Cargo/lockfile changes from this architecture branch;
- external RunenGPU or RunenRender source transfer;
- implementing the retired RunenRender R1;
- duplicate GPU contexts or renderer paths;
- broad field/path-tracing feature work in the current plugin;
- RunenSDF cutover or deletion;
- direct RunenUI/RunenRender core dependency;
- compatibility packages, source mirrors, or universal shared-core repositories.
