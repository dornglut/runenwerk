---
title: Separate GPU Execution from Rendering
description: Accepted ownership and dependency decision establishing RunenGPU as the shared GPU execution framework beneath RunenRender.
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
---

# ADR 0015: Separate GPU Execution from Rendering

## Decision

Create two independent framework repositories with one public package each:

```text
product       repository                 package       crate
RunenGPU      Crystonix/runen-gpu        runen-gpu     runen_gpu
RunenRender   Crystonix/runen-render     runen-render  runen_render
```

The required dependency direction is:

```text
RunenRender -> RunenGPU
```

RunenGPU may use WGPU as its initial internal backend. RunenRender must not own a
WGPU device, queue, surface, allocator, command encoder, or competing GPU resource
and error model.

Each repository begins with one public package. Internal modules provide ownership
boundaries until a second backend, independently reusable consumer, release unit,
or dependency graph proves that another package is necessary.

Do not initially create:

```text
runengpu_core
runengpu_wgpu
runengpu_macros
runengpu_testing
runenrender_core
runenrender_gpu
runenrender_macros
runenrender_testing
facade or compatibility packages
```

This ADR amends the RunenRender backend ownership in ADR 0014. ADR 0014 remains
authoritative for repository independence, Runenwerk integration ownership, clean
cutover, provenance, and removal of duplicate source authority.

## RunenGPU ownership

RunenGPU owns validated execution of GPU resources and workloads:

- context, adapter, device, queue, and execution epochs;
- normalized capabilities and requirements;
- buffers, textures, views, samplers, pipelines, and query resources;
- resource access, initialization, lifetime, hazard, and retirement validation;
- compute, render, copy, clear, resolve, and present workloads;
- shader admission and backend pipeline realization;
- uploads, asynchronous readback, submission, and completion;
- headless compute and offscreen graphics;
- low-level surface creation, configuration, acquisition, and presentation;
- backend and device outcomes;
- GPU diagnostics, timings, and provenance.

RunenGPU does not own image formation, visibility semantics, lighting, material
meaning, field mathematics, simulations, ECS, UI, world generation, window/event
loop policy, or product recovery.

## RunenRender ownership

RunenRender owns how prepared render-facing data becomes one or more images:

- prepared render scenes and deterministic contribution composition;
- views and logical targets;
- providers, instances, and interaction contracts;
- materials, media, emitters, and environments;
- visibility and provider intersection policy;
- light transport and estimator policy;
- radiance caches and bounded history;
- reconstruction, overlays, color, and presentation intent;
- render quality profiles and diagnostics;
- lowering render work into RunenGPU workloads.

RunenRender does not own ECS extraction, source authoring, procedural world policy,
field/SDF mathematics, simulations, UI state/layout/hit testing/accessibility,
windows/event loops, shader filesystem watching, general GPU execution, or product
recovery.

## Runenwerk ownership

Runenwerk retains:

- application and engine lifecycle;
- frame and domain scheduling;
- windows, event loops, DPI, monitor, resize, and visibility policy;
- ECS, scene, world, material-authoring, field/SDF, UI, editor, and simulation
  extraction/adapters;
- shader source discovery, revision, filesystem watching, and hot-reload policy;
- product quality and capability selection;
- cross-framework work composition;
- diagnostics presentation, artifact policy, and product recovery;
- integration and runtime evidence.

Runenwerk may create one shared RunenGPU context and compose work from RunenRender
and non-render consumers. Runenwerk does not gain ownership of reusable GPU or
rendering semantics merely because it performs composition.

## Framework independence

RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely
because an application may accelerate or display their outputs.

The default shape is:

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

A reusable adapter package may be extracted only after both public contracts are
stable and at least one consumer outside Runenwerk proves independent value. Until
then, cross-framework translation remains Runenwerk-owned.

## RunenUI relationship

RunenUI owns semantic UI, state, actions, focus, accessibility, layout, style, text
shaping, hit testing, and renderer-neutral paint output.

A future Runenwerk-owned bridge may translate accepted paint primitives into a
RunenRender overlay contribution:

```text
RunenUI paint scene
    -> Runenwerk bridge
    -> RunenRender overlay contribution
    -> RunenGPU workloads
```

The bridge does not expose RunenUI widget state to RunenRender. RunenRender does
not perform UI hit testing or text shaping. RunenUI remains usable with independent
standalone backends.

## RunenSDF relationship

RunenSDF remains a CPU/backend-neutral field framework. It owns field values,
numerical contracts, bounds, operators, transforms, capabilities, and reference
queries.

Rendering or GPU realization is derived integration state. A future reusable
adapter may depend on RunenSDF and RunenRender/RunenGPU, but RunenSDF never depends
back on it.

## Rationale

The current Runenwerk renderer aggregates two distinct responsibilities:

```text
general GPU execution
image formation
```

Both use GPU resources and command submission, but only image formation owns
views, visibility, materials, emitters, transport, reconstruction, overlays, and
presentation intent.

Keeping generic compute inside RunenRender would force field compilers, simulation
solvers, procedural tools, bakers, and offline workloads to depend on a renderer.
Renaming the entire renderer to RunenGPU would erase image-formation ownership and
create another broad repository magnet.

The one-way split permits:

```text
domain algorithm -> GPU workload -> RunenGPU
prepared render scene -> render plan -> GPU workloads -> RunenGPU -> image
```

## Sequencing

No implementation phase is authorized by this ADR.

The required sequence is:

1. **S0 inventory:** classify every current file, consumer, identity, shader,
   pipeline, resource, surface, macro, test, example, benchmark, persistence risk,
   and lifecycle path.
2. **Internal RunenGPU proof:** separate reusable GPU execution behind the intended
   one-package public boundary while still inside Runenwerk.
3. **External RunenGPU cutover:** create `runen-gpu`, prove independent conformance,
   pin Runenwerk to an exact revision, migrate consumers, and delete the original
   Runenwerk implementation.
4. **Internal RunenRender proof:** separate image-formation semantics from
   Runenwerk adapters and consume RunenGPU only through its public API.
5. **External RunenRender cutover:** create/populate `runen-render`, prove
   conformance, pin exact revisions, migrate consumers, and delete the original
   implementation.
6. **Adapters and advanced rendering:** extract reusable bridges only after proof;
   then develop advanced field-ray transport on stable foundations.

A first implementation specification is written only after S0 names exact current
files and consumers. The stale renderer-first identity specification is retired.

## Consequences

- RunenGPU becomes a lower-level independent framework peer.
- RunenRender may directly depend on RunenGPU because the dependency is
  independently useful and lower-level, not a disguised Runenwerk adapter.
- WGPU remains an implementation detail of RunenGPU until another backend creates
  real package pressure.
- GPU execution planning and render semantic planning remain different models.
- Runtime identities are classified by semantic owner before migration.
- External source movement remains blocked until internal public-boundary and
  anti-cheating conformance passes.
- No compatibility package, forwarding namespace, source mirror, or duplicate
  execution path survives a completed cutover.

## Rejected alternatives

Rejected:

- keep general GPU compute inside RunenRender;
- rename the entire renderer to RunenGPU;
- create separate GPU contexts and resource systems per domain;
- make RunenSDF, RunenECS, or RunenUI core depend on RunenGPU;
- create speculative `core`, `wgpu`, `gpu`, facade, macro, capture, or testing
  packages before dependency pressure exists;
- move `engine/src/plugins/render` unchanged;
- implement renderer-local identities before classifying GPU, renderer, and
  Runenwerk-owned identities;
- preserve both old and new GPU/render paths after cutover.

## Fitness functions

The decision is successful only when:

- one RunenGPU context can serve RunenRender and at least one non-render workload;
- RunenGPU contains no renderer or domain meaning;
- RunenRender contains no direct WGPU ownership;
- Runenwerk consumes both through public boundaries;
- headless compute and offscreen rendering work independently;
- RunenUI and RunenSDF remain independent;
- window/product recovery remains Runenwerk-owned;
- each external repository validates independently;
- no duplicate GPU or render implementation survives cutover.
