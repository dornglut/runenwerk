---
title: RunenRender Architecture and Decomposition Design
description: Decision-complete prepared-scene, provider, material, transport, reconstruction, overlay, RunenGPU, host, conformance, and extraction architecture for RunenRender.
status: active
owner: render
layer: framework/render
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-architecture-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Architecture and Decomposition Design

## Status

The repository identity, one-package shape, ownership boundary, dependency on
RunenGPU, Runenwerk integration boundary, RunenUI relationship, and target
rendering architecture are fixed.

Exact current-file disposition and the first implementation scope remain blocked
on S0. This document does not authorize Rust changes, source movement, external
repository population, or advanced renderer implementation.

## Mission

RunenRender owns image formation.

It answers:

> Given prepared views, providers, instances, materials, media, emitters,
> environments, overlays, changes, and quality policy, how should one or more
> images be formed?

RunenRender does not own:

- ECS or host-world storage;
- authoring source, scene persistence, or procedural world policy;
- field/SDF mathematics;
- simulations;
- UI state, layout, hit testing, focus, or accessibility;
- windows and event loops;
- general GPU execution;
- WGPU devices, queues, surfaces, resources, or command submission;
- shader filesystem discovery or hot-reload product policy;
- product lifecycle, quality selection, or recovery.

## Repository and package

```text
repository: Crystonix/runen-render
package: runen-render
crate: runen_render
depends on: runen-gpu
```

Initial repository shape:

```text
runen-render/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md
├── src/
│   ├── lib.rs
│   ├── scene.rs
│   ├── view.rs
│   ├── contribution.rs
│   ├── provider.rs
│   ├── interaction.rs
│   ├── material.rs
│   ├── medium.rs
│   ├── emitter.rs
│   ├── visibility.rs
│   ├── transport.rs
│   ├── radiance_cache.rs
│   ├── reconstruction.rs
│   ├── overlay.rs
│   ├── color.rs
│   ├── quality.rs
│   ├── history.rs
│   ├── diagnostics.rs
│   └── gpu.rs
├── shaders/
├── tests/
├── examples/
├── benches/
├── docs/
├── conformance/
└── xtask/
```

Module names are directional. The public package is one release unit until a real
second consumer, backend, or release boundary proves another package is needed.

Do not initially create `runenrender_core`, `runenrender_gpu`,
`runenrender_macros`, facade, testing, capture, bridge, or compatibility packages.

There is no `runenrender_wgpu`. Concrete WGPU ownership belongs to RunenGPU.

## Dependency rules

RunenRender may depend on RunenGPU and appropriate low-level math/data libraries.
It must not depend on:

```text
WGPU directly
Winit
Runenwerk
RunenECS
RunenSDF
RunenUI
scene/world/material-authoring/editor/application domains
```

The package may define renderer semantics and lower them into RunenGPU workloads.
It must not reach through RunenGPU into backend handles or recreate a parallel GPU
resource/submission system.

## Ownership

RunenRender owns:

- renderer semantic identities;
- prepared render scenes and contribution composition;
- views and logical targets;
- providers, instances, and interaction contracts;
- material, medium, emitter, and environment semantics;
- visibility and provider-query policy;
- transport, estimator, and quality policy;
- radiance caches and bounded history;
- reconstruction and anti-aliasing policy;
- 2D/overlay composition and presentation intent;
- color pipeline intent;
- deterministic render planning;
- rendering diagnostics and provenance;
- render-specific lowering into RunenGPU work fragments.

RunenRender does not own:

- source ECS entities or scene nodes;
- world/chunk streaming;
- material graph authoring or asset import;
- SDF fields or numerical query semantics;
- UI widget semantics;
- WGPU realization;
- native windows;
- application lifecycle;
- product-specific feature registries or fallback policy.

## Prepared render scene

RunenRender consumes an immutable render-facing snapshot:

```text
PreparedRenderScene
├── scene generation
├── views
├── logical targets
├── providers
├── instances
├── materials
├── media
├── emitters
├── environments
├── overlays
├── transforms and motion
├── changed regions/generations
├── importance hints
├── regional summaries
└── provenance
```

The prepared scene is renderer-owned input, not a mirror of ECS or authoring state.
Planning and execution must not reach back into:

- ECS;
- UI runtime state;
- simulation state;
- authoring graphs;
- host windows;
- product services.

Runenwerk adapters form the snapshot before submission.

## Contribution model

Independent producers publish immutable contributions:

```text
RenderContribution
├── producer identity
├── contribution identity
├── generation
├── target/view selection
├── providers and instances
├── materials/media
├── emitters/environments
├── overlays
├── changed regions
└── provenance
```

Required lifecycle:

```text
insert
replace
remove
retire producer
```

Composition defines deterministic producer ordering, replacement, target and
overlay ordering, conflict handling, missing references, and diagnostics.

Producer or contribution identities are renderer-local runtime values. They are
not raw ECS entity IDs or stable persisted asset IDs.

## Identity separation

Candidate renderer concepts:

```text
RenderSceneId
RenderViewId
RenderTargetId
RenderProducerId
RenderContributionId
RenderProviderId
RenderInstanceId
RenderMaterialId
RenderMediumId
RenderEmitterId
RenderEnvironmentId
RenderOverlayId
RenderHistoryId
```

Exact names remain implementation decisions. Required separation is:

```text
RenderProviderId   semantic prepared provider
GpuBufferId        one GPU resource realization
EcsEntityId        one Runenwerk source entity
AssetId            one authored/persisted asset identity
```

One provider may have zero, one, or many GPU resource realizations across devices,
quality policies, frames, or caches. This is why current `Render*Id` values must be
classified before migration.

## Views and logical targets

A prepared view contains render-facing facts:

```text
PreparedView
├── identity
├── logical target
├── projection
├── current transform
├── previous transform
├── viewport
├── sample footprint
├── visibility mask
├── quality policy
├── history policy
└── provenance
```

A logical target describes image intent:

```text
RenderTarget
├── extent
├── format intent
├── color-space intent
├── depth requirements
├── sample policy
├── presentation intent
└── lifetime intent
```

Concrete textures and surface images are RunenGPU resources. Native windows are
host-owned.

## Provider architecture

RunenRender is provider-oriented and field-first-capable without requiring one
universal representation.

Provider families may include:

```text
Solid
Shell
Fiber
Liquid
Volume
Analytic
Procedural
Population
Overlay
RegionalSummary
```

Provider capabilities may include:

```text
closest_surface
any_hit_visibility
interval_query
transmittance
raster_visibility
procedural_evaluation
material_attributes
velocity
refinable
streamable
hardware_accelerable
```

Authoring and visible semantics do not require mesh extraction.

Derived acceleration may use:

- sparse field pages or clipmaps;
- range or distance hierarchies;
- AABB/BVH structures;
- rasterized intermediates;
- procedural tables;
- hardware acceleration structures;
- temporary backend triangles, AABBs, or microgeometry.

Derived acceleration remains replaceable, discardable, and non-authoritative.
“No mandatory meshes” does not prohibit backend-local geometry when it is the best
acceleration representation.

## Interaction contract

All provider-specific query strategies produce a common interaction:

```text
RenderInteraction
├── distance
├── world position
├── geometric orientation
├── shading orientation
├── material
├── medium transition
├── emission
├── local velocity
├── provider and instance identity
├── local coordinates
├── approximation error/confidence
└── provenance
```

The interaction separates semantic rendering results from provider-specific
acceleration structures.

## Visibility and query architecture

RunenRender separates path/ray selection from provider intersection strategy.

```text
trace(ray, purpose, tolerance, visibility_mask)
    -> RenderQueryOutcome
```

Purposes include:

```text
primary
shadow
reflection
refraction
indirect
volume
picking
reference
```

Provider strategies may use:

- sphere tracing;
- interval/range traversal;
- continuous root solving;
- analytic intersection;
- fiber solvers;
- volume integration;
- raster visibility;
- hardware ray queries.

Ray marching is an intersection technique, not the lighting architecture.
RunenRender does not reinterpret RunenSDF values directly; a Runenwerk or future
reusable adapter must preserve RunenSDF capability and numerical contracts.

## Materials

A material defines scattering independently of provider representation:

```text
RenderMaterial
├── scattering closure
├── parameter sources
├── layers
├── emission
├── transmission
├── subsurface policy
├── displacement/detail policy
├── material style
└── provenance
```

Material authoring graphs and asset import remain outside RunenRender. An adapter
lowers accepted authored material products into prepared render materials.

Detail-frequency metadata distinguishes:

```text
resolved detail     -> provider/geometric evaluation
transition detail   -> filtered evaluation
unresolved detail   -> effective material/statistical response
```

## Media

A medium defines:

```text
RenderMedium
├── absorption
├── scattering
├── phase behavior
├── emission
├── density source
├── interface priority
└── provenance
```

Surface interfaces identify medium transitions. Transport, visibility, and
transmittance consume one consistent medium contract.

## Emitters and environments

One emitter model includes:

- directional, point, spot, and area emitters;
- emissive surfaces and fields;
- particles and fire;
- environment lighting;
- procedural skies;
- distant/regional summaries.

Each emitter provides bounds/distribution, output, importance, linking masks,
generation, and provenance.

Many-light sampling, reservoirs, spatial reuse, and path guiding are estimator
implementations under the same emitter contract.

## Transport architecture

RunenRender uses one semantic transport family:

```text
generate path segments
    -> trace
    -> classify interaction
    -> evaluate emission
    -> sample emitter
    -> sample continuation
    -> update throughput
    -> terminate, cache, or continue
```

Quality tiers vary budgets and estimators without switching to unrelated rendering
systems.

Initial quality ladder:

```text
Preview
Standard
High
Ultra
Reference
```

All tiers share:

- current prepared scene;
- provider/interaction contracts;
- material/medium/emitter semantics;
- current primary visibility;
- structured capability and degradation reporting.

Budgets may vary:

- path depth;
- direct-light candidate count;
- reservoir/spatial reuse;
- radiance-cache use;
- glossy/refraction/volume support;
- reconstruction and history;
- progressive accumulation.

No tier silently renders stale primary visibility or material state. Unsupported
transport is diagnosed explicitly.

## Radiance cache

The initial scalable GI contract is a sparse directional world-space cache:

```text
RadianceCacheEntry
├── spatial domain
├── directional radiance
├── visibility summary
├── geometry validity
├── material compatibility
├── variance/confidence
├── source generations
├── update age
└── provenance
```

The cache is derived and discardable. It may terminate later bounces, approximate
diffuse or rough-glossy transport, and represent far/regional lighting.

It must not become the only source of current primary visibility or authoritative
scene state.

## Reconstruction and history

History policy is explicit:

```text
None
WorldCacheOnly
ReservoirMetadataOnly
BoundedReservoirHistory
FullTemporal
ProgressiveReference
```

Default product policy should preserve sharp current-frame visibility, material
changes, validation, and disocclusion. History is bounded by source generations,
motion, validity, and confidence.

Final-color history is not mandatory for every quality tier.

## Stylization

Stylization is separated by owner:

```text
MaterialStyle   local scattering/emission/detail
TransportStyle  lighting/path/visibility interpretation
DisplayStyle    color mapping, compositing, presentation
```

Stylization does not require a separate renderer architecture or bypass current
visibility/material validity.

## Overlay architecture

RunenRender accepts renderer-neutral overlay contributions such as:

```text
shapes
strokes
clips
transforms
glyph runs
images
layers
blend/composite intent
damage regions
```

RunenRender may own GPU glyph/image resources, atlas residency, rasterization, and
compositing. It must not own text shaping, line breaking, caret/selection logic,
accessibility, widget state, or UI hit testing.

### RunenUI bridge

RunenUI remains independent:

```text
RunenUI mounted runtime
    -> style/layout/text/hit-test/semantics
    -> renderer-neutral paint scene
    -> Runenwerk bridge
    -> RunenRender overlay contribution
    -> RunenGPU workloads
```

The bridge consumes paint facts, not widget state or actions.

HUD and editor UI normally composite after 3D tone mapping. In-world UI may use an
explicit adapter policy: texture/vector overlay, emissive surface, or physically
lit surface. No current RunenUI milestone depends on RunenRender.

## RunenGPU lowering

RunenRender converts semantic render plans into `GpuWorkFragment` contributions.

The lowering may own:

- render-specific resource realization and cache keys;
- provider acceleration for rendering;
- visibility/intersection pipelines;
- material/medium evaluation pipelines;
- emitter sampling;
- transport wavefronts;
- radiance-cache updates;
- reconstruction;
- overlay rasterization;
- color/output encoding.

It does not own:

- device/queue/surface creation;
- generic resource allocation or hazard validation;
- command submission;
- backend capability mapping;
- a second GPU error or lifetime model.

RunenGPU validates and executes the resulting work.

## Shader boundary

RunenRender owns render shader meaning and source products. RunenGPU owns shader
admission and backend realization.

Runenwerk owns filesystem discovery, source revision policy, file watching,
hot-reload orchestration, user-facing diagnostics, and last-known-good product
policy.

WGSL/WGPU ABI details do not become universal renderer semantics. A macro package
is not accepted before concrete API and conformance pressure exists.

## Host and presentation boundary

Runenwerk owns windows, event loops, resize/DPI/visibility policy, presentation
timing, and product recovery.

RunenGPU owns low-level surface operations and outcomes.

RunenRender owns logical target, output color, compositing, and presentation intent.

The dependency direction remains:

```text
Runenwerk host policy
    -> RunenRender logical image intent
    -> RunenGPU surface/resource execution
```

## Diagnostics

RunenRender exposes structured facts for:

- prepared-scene and contribution validation;
- missing/invalid providers and references;
- quality/capability degradation;
- visibility and interaction failures;
- material/medium/emitter admission;
- transport budgets and unsupported paths;
- cache/history validity;
- reconstruction;
- overlay composition;
- RunenGPU workload provenance.

RunenRender does not decide product severity, storage, UI presentation, or recovery.

## Current decomposition problem

The current `engine/src/plugins/render` combines:

- general GPU execution and WGPU ownership;
- render graph/planning and image formation;
- native-window surfaces;
- ECS and host state projection;
- Runenwerk frame/time policy;
- scene, world, material, SDF, UI, editor, procedural, and product features;
- shader discovery and hot reload;
- diagnostics, capture, artifact export, startup readiness, and frame pacing.

Moving the directory unchanged is forbidden.

## Conformance

Internal RunenRender proof requires:

1. prepared scene and contribution composition test without ECS, Runenwerk, WGPU,
   RunenSDF, or RunenUI;
2. provider/interaction/material/medium/emitter contracts test independently;
3. deterministic render planning;
4. RunenGPU lowering through public workload contracts only;
5. no direct WGPU dependency;
6. at least two independent producer families through the same contribution seam;
7. overlay proof using neutral primitives without UI runtime access;
8. quality/history/cache validity tests;
9. Runenwerk adapters consume no private renderer internals;
10. no duplicate old render path remains.

External repository proof additionally requires independent locked validation,
public downstream consumption, exact RunenGPU revision, provenance, and clean
Runenwerk cutover.

## S0 inventory gate

Before implementation, S0 must classify:

- every current file/module/shader/test/example/benchmark;
- graph, resource, WGPU, surface, shader, pipeline, macro, residency, frame,
  diagnostics, capture, and runtime ownership;
- every current `Render*Id`, allocator, handle, raw conversion, and persisted use;
- every direct consumer in apps, domains, net, adapters, tests, examples, and tools;
- all host/ECS/scene/world/material/SDF/UI/editor/product reach-back;
- device/context/surface/window/drop/shutdown control flow;
- shader discovery/reload and macro ABI consumers;
- validation, headless, offscreen, surface, device-loss, runtime, and benchmark
  commands;
- exact move/stay/redesign/delete disposition.

S0 must first split GPU-execution ownership from renderer semantics. The old
renderer-first R1 identity specification is invalid because current identities mix
GPU, renderer, and Runenwerk owners.

## Definition of done

RunenRender extraction is complete only when:

- the one-package public API validates independently;
- RunenRender depends on exact RunenGPU revision and not WGPU directly;
- Runenwerk consumes only public prepared/contribution/adapter seams;
- RunenUI and RunenSDF remain independent;
- downstream conformance and renderer/GPU/runtime evidence pass;
- every active consumer is migrated;
- exact provenance is recorded;
- the original Runenwerk image-formation implementation and temporary seams are
  deleted;
- no duplicate renderer or private reach-through path remains.
