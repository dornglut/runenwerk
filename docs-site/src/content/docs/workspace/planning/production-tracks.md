---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../specs/pt-runengpu-g1-identities-errors.ron
  - ./active-work.md
  - ./roadmap.md
  - ./completed-work.md
  - ./decision-register.md
---

# Production Tracks

The repository-family program runs tracks at different maturity levels. Multiple
investigations may overlap, but shared manifests, lockfiles, root architecture,
canonical planning summaries, and source cutovers have one active merge owner at a
time.

## Coordination model

```text
Primary independent extraction track
    PT-RUNENSDF — handled in its separate thread/branch

Parallel architecture and investigation track
    PT-GPU-RENDER-SPLIT-001 — docs and S0 preparation only

Prepared track
    PT-RUNENECS — R1 specified, no implementation authorized

Independent workstream
    RunenUI — governed separately
```

This branch must be rebased after active SDF planning changes before merge. It
must preserve newer SDF lifecycle facts.

## PT-REPOSITORY-FAMILY

State: governance baseline complete through PR #109; GPU/render ownership amended
by ADR 0015.

Purpose: own repository missions, dependency direction, adapter ownership,
versioning, diagnostics, identities, formats, provenance, conformance, cutovers,
and compatibility policy.

Future milestones:

```text
001 compatibility matrix and release-policy hardening after first extraction
002 cross-repository conformance automation after at least two repositories
003 multi-framework GPU/render compatibility matrix after both cutovers
```

## PT-RUNENSDF

State: boundary correction complete; standalone transfer owned by its separate
active track.

Goal:

```text
Extract reusable signed-field mathematics and CPU queries into runen-sdf with no
Runenwerk geometry, ECS, world, material, renderer, GPU, UI, or product dependency.
```

Milestones remain SDF-track authority. This branch changes no SDF implementation,
external source, dependency, or deletion state.

## PT-RUNENECS

State: architecture and first repair specification recorded; Rust implementation
not authorized.

Goal:

```text
Produce independently conformant runenecs, runenecs_macros, and runen_schedule
packages without Runenwerk geometry, spatial, lifecycle, renderer, GPU,
networking, replay, or product policy.
```

Internal milestones:

```text
R1 entity identity and structured errors
R2 atomic structural mutation
R3 query and SystemParam unsafe boundaries
R4 explicit reflection and macro migration
R5 remove spatial and geometry ownership
R6 messaging split
R7 change, ownership, and networking separation
R8 neutralize runen_schedule
R9 standalone conformance and performance baseline
```

Only R1 has a current planning specification. Repository creation remains blocked
until R9 acceptance.

## PT-GPU-RENDER-SPLIT-001

State: active planning; documentation and investigation only.

Goal:

```text
Separate general GPU execution from rendering, align repository authority,
classify current ownership, retire the old renderer-only phase, and prepare the
first RunenGPU implementation contract.
```

Delivered planning targets:

```text
RunenGPU:    runengpu_core + runengpu_wgpu
RunenRender: runenrender_core + runenrender_gpu
Dependency:  RunenRender -> RunenGPU
```

Current work:

- accepted split ADR on the branch;
- full RunenGPU architecture;
- revised RunenRender architecture;
- module-family connector inventory;
- new decomposition/extraction roadmap;
- planning-only RunenGPU G1 specification;
- retirement of unimplemented old RunenRender R1.

Remaining before review readiness:

- reconcile branch with active SDF planning changes;
- run docs validator/build and diff checks;
- review authority consistency;
- keep the PR draft until evidence is recorded.

Remaining before G1 activation:

- complete local S0 file and consumer inventory;
- classify every current identity and allocator;
- run Cargo/test/Clippy/docs baseline;
- identify shader/headless/surface/device-loss/benchmark commands;
- update G1 with exact files and consumers;
- grant a separate implementation authorization.

## PT-RUNENGPU

State: architecture accepted on the split branch; G1 planning-only.

Goal:

```text
Provide reusable, validated GPU execution for rendering and non-render compute
without renderer, ECS, SDF, UI, simulation, world, Winit, or Runenwerk semantics.
```

Internal milestones:

```text
S0 complete ownership and command inventory
G1 GPU identities, structured errors, dependency guards
G2 resources, access, ownership, and lifetimes
G3 bounded GPU work fragments and work graph
G4 shader, pipeline, parameter, and optional macro ABI boundary
G5 headless WGPU compute/upload/readback executor
G6 offscreen graphics/copy and compute-to-render proof
G7 surfaces, generations, device outcomes, drop-order contract
G8 shared render/non-render consumer and anti-cheating proof
G9 standalone-boundary and performance conformance
GX1 external repository transfer and standalone proof
GX2 Runenwerk exact-revision cutover and internal-source deletion
```

Only G1 may become the next implementation phase. It is not currently authorized.

## PT-RUNENRENDER

State: architecture revised; old R1 retired; structural implementation blocked by
accepted RunenGPU boundary and cutover.

Goal:

```text
Own prepared render scenes, providers, materials/media, emitters, visibility,
transport, caches, reconstruction, overlays, and presentation while executing
through RunenGPU and retaining Runenwerk/domain integration outside the framework.
```

Internal milestones:

```text
R1 renderer-semantic identities and errors
R2 prepared scene and contribution lifecycle
R3 semantic render planning separated from GPU work
R4 render GPU realization through RunenGPU
R5 material/shader versus authoring/reload boundary
R6 logical targets, overlays, output color, presentation
R7 Runenwerk adapter migration and current behavior parity
R8 standalone/public-boundary and performance conformance
RX1 external repository transfer and standalone proof
RX2 Runenwerk exact-revision cutover and internal-source deletion
```

Read-only planning may overlap with RunenGPU. Structural implementation does not
start by constructing another temporary backend.

## Advanced renderer development

After both cutovers:

```text
F1 reference implicit solid renderer
F2 shell/fiber/liquid/volume providers
F3 many-light direct transport
F4 sparse directional radiance cache
F5 sharp lobe-separated reconstruction and bounded history
F6 preview-to-reference quality ladder
F7 endless-world transport horizons
F8 explicit stylization
F9 production vertical proofs
F10 authoring/tooling/profiling hardening
```

Repository extraction is intentionally not blocked on speculative completion of
this research-heavy renderer destination.

## RunenUI

RunenUI is an independent peer. Its core/runtime do not depend on RunenGPU or
RunenRender.

A future Runenwerk-owned adapter may translate accepted renderer-neutral paint
output into RunenRender overlay contributions after both public seams stabilize.

## Parallel execution rules

Allowed:

- independently authorized SDF work;
- S0 inventory and validation;
- read-only future GPU/render design;
- independent RunenUI work;
- benchmark, shader, headless-GPU, surface, and runtime command discovery.

Forbidden without a bounded phase:

- GPU, ECS, or renderer structural implementation;
- external GPU/render source transfer;
- Runenwerk SDF cutover or deletion from this branch;
- duplicate GPU contexts or renderer paths;
- compatibility repositories or source mirrors;
- universal shared-core/meta repositories;
- direct RunenUI/RunenRender core dependency.
