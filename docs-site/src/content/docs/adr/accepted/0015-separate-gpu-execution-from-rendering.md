---
title: Separate GPU Execution from Rendering
description: Accepted ownership and dependency decision establishing RunenGPU as the shared GPU execution framework beneath RunenRender and non-render compute domains.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-21
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
related_roadmaps:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# ADR 0015: Separate GPU Execution from Rendering

## Decision

Create `Crystonix/runen-gpu` as an independent lower-level framework repository and
retain `Crystonix/runen-render` as the rendering framework.

The required dependency direction is:

```text
RunenRender ─────────────┐
field/SDF GPU adapters ──┤
simulation GPU adapters ─┼──> RunenGPU
procedural/baking tools ─┘
```

RunenGPU must not depend on RunenRender or any domain consumer.

This ADR supersedes only the RunenRender package/backend ownership selected by
ADR 0014. ADR 0014 remains authoritative for the repository-family clean-cutover,
Runenwerk integration ownership, RunenUI independence, and framework extraction
rules.

## Required package candidates

RunenGPU:

```text
runengpu_core
runengpu_wgpu
```

RunenRender:

```text
runenrender_core
runenrender_gpu
```

`runenrender_gpu` lowers render semantics into RunenGPU work. It does not own a
WGPU device, queue, surface, allocator, or competing WGPU error model.

Macro, facade, testing, capture, and alternative-backend packages remain deferred
until concrete dependency or ABI pressure proves them necessary.

## Ownership

### RunenGPU

RunenGPU owns general GPU execution:

- backend-neutral GPU resources and workload descriptions;
- capability negotiation;
- access, lifetime, and hazard validation;
- shader and pipeline realization contracts;
- compute, render, copy, resolve, clear, and present execution;
- headless device operation;
- uploads and readback;
- low-level surface resources and backend outcomes;
- device-loss facts;
- GPU diagnostics and provenance.

RunenGPU does not own image formation, lighting, material meaning, simulation
algorithms, fields, ECS, UI, world generation, or product lifecycle.

### RunenRender

RunenRender owns image formation:

- prepared render scenes and contributions;
- logical views and targets;
- render providers and interactions;
- materials, media, emitters, and environments;
- visibility and light transport;
- radiance caches;
- reconstruction and bounded history;
- 2D overlay composition;
- color and presentation intent;
- render diagnostics.

RunenRender executes through RunenGPU.

### Runenwerk

Runenwerk retains:

- engine/application lifecycle;
- frame and domain scheduling;
- windows and event-loop policy;
- ECS/domain extraction;
- scene, world, material-authoring, SDF, UI, editor, and simulation adapters;
- shader source discovery and hot-reload policy;
- product capability and quality selection;
- product recovery decisions;
- integration diagnostics and runtime evidence.

### Domain frameworks

RunenSDF, RunenUI, RunenECS, and simulation domains retain their semantics.
They do not depend on RunenRender or RunenGPU merely because an application may
accelerate or display their outputs.

A GPU or renderer adapter is introduced only where independently reusable or kept
Runenwerk-owned when it is integration-specific.

## Rationale

The current Runenwerk render plugin combines two distinct responsibilities:

```text
general GPU execution
image formation
```

Both use GPU devices, resources, pipelines, and command submission, but only the
renderer owns views, visibility, materials, emitters, transport, reconstruction,
and presentation.

Keeping generic compute inside RunenRender would force fluid solvers, field
compilers, procedural tools, and offline GPU workloads to depend on a renderer.
Renaming the whole renderer to RunenGPU would instead erase the rendering
framework boundary and turn one broad repository into a new ownership magnet.

A one-way split preserves reuse and responsibility:

```text
domain algorithm
    -> explicit GPU workload
    -> RunenGPU execution

prepared render scene
    -> rendering plan
    -> RunenGPU workload
    -> image
```

## Sequencing consequence

The previously specified RunenRender R1 phase must not be activated.

Existing `Render*Id` values mix likely GPU-execution identities, renderer-semantic
identities, and Runenwerk producer/product identities. Identity correction must
follow a complete ownership classification.

The required execution sequence is:

1. correct repository and planning authority;
2. complete the current GPU/render inventory;
3. decompose and prove RunenGPU inside Runenwerk;
4. extract and cut Runenwerk over to external RunenGPU;
5. decompose RunenRender on the accepted RunenGPU boundary;
6. extract and cut Runenwerk over to external RunenRender;
7. add reusable adapters after both sides stabilize;
8. develop the advanced field-ray transport renderer.

## Consequences

- RunenGPU is a new independent framework peer.
- RunenRender may directly depend on RunenGPU because the dependency is lower-level,
  independently useful, and not a Runenwerk adapter responsibility.
- RunenRender no longer owns WGPU directly.
- GPU execution planning and render semantic planning remain separate models.
- Runenwerk composes work fragments from rendering and non-render domains.
- The existing renderer-only R1-R10 roadmap is replaced.
- The unimplemented `PT-RUNENRENDER-R1` specification is retired.
- External source movement remains blocked until internal public-boundary and
  anti-cheating conformance passes.

## Rejected alternatives

Rejected:

- keep general compute inside RunenRender;
- rename RunenRender to RunenGPU and merge both responsibilities;
- duplicate GPU contexts and resource systems per domain;
- create a universal shared-core repository;
- make RunenUI or RunenSDF cores depend on RunenGPU;
- extract the current renderer directory before internal decomposition;
- implement the old RunenRender identity phase before ownership classification.

## Fitness functions

The decision is successful only when:

- one live GPU context can serve rendering and at least one non-render workload;
- RunenGPU packages contain no renderer or domain meaning;
- RunenRender contains no concrete WGPU ownership;
- Runenwerk consumes both through public boundaries;
- headless compute and offscreen rendering work independently;
- window/product recovery remains Runenwerk-owned;
- no duplicate GPU or renderer path survives either cutover.
