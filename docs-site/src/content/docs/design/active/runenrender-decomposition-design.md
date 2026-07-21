---
title: RunenRender Architecture and Decomposition Design
description: Render-scene, provider, material, transport, reconstruction, overlay, RunenGPU lowering, host integration, and extraction boundaries for RunenRender.
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

RunenRender remains an independent rendering framework, but its earlier direct
WGPU-backend ownership is superseded by ADR 0015.

RunenRender depends on RunenGPU for GPU execution.

Exact public APIs and module dispositions remain phase-owned. No renderer source
movement or broad field/path-tracing rewrite is authorized by this document.

## Mission

RunenRender answers:

> Given prepared views, render providers, materials, media, emitters,
> environments, overlays, changes, and quality policies, how should one or more
> images be formed?

RunenRender owns image formation.

It does not own:

- source authoring;
- ECS or host-world storage;
- procedural world generation;
- field/SDF mathematics;
- simulations;
- UI state, layout, hit testing, or accessibility;
- windows and event loops;
- general GPU execution;
- product lifecycle or recovery.

## Required packages

```text
runenrender_core
runenrender_gpu
```

Deferred until evidence:

```text
runenrender_testing
runenrender_macros
runenrender_capture
bridge packages
facade package
```

There is no `runenrender_wgpu` package. Concrete WGPU behavior belongs to
`runengpu_wgpu`.

## Dependency rules

`runenrender_core` must not depend on:

```text
WGPU
Winit
Runenwerk
ECS
RunenSDF
RunenUI
scene/world/material-authoring domains
editor/application lifecycle
```

`runenrender_gpu` depends on:

```text
runenrender_core
runengpu_core
```

It must not create a device, queue, surface, allocator, or direct WGPU namespace.

Applications wire `runenrender_gpu` to a concrete RunenGPU backend.

## Ownership

### runenrender_core

May own:

- renderer semantic identities;
- prepared render scenes and contribution composition;
- views and logical targets;
- provider capabilities and interaction contracts;
- materials, media, emitters, and environments;
- quality, transport, history, overlay, and presentation policies;
- deterministic render planning;
- render diagnostics and provenance.

### runenrender_gpu

May own:

- render-specific resource realization;
- provider acceleration and residency for rendering;
- visibility and intersection pipelines;
- material and medium evaluation pipelines;
- emitter sampling;
- path transport;
- radiance-cache maintenance;
- reconstruction;
- overlay rasterization;
- display/output encoding;
- lowering into RunenGPU work fragments.

### Runenwerk

Retains:

- `RenderPlugin`-style lifecycle integration;
- ECS/domain extraction;
- windows and event loops;
- scene, world, material-authoring, SDF, UI, editor, and simulation adapters;
- shader source discovery and hot-reload policy;
- product quality selection;
- diagnostics presentation and artifact policy;
- recovery and runtime evidence.

## Prepared scene

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
├── changed regions/generations
├── importance hints
├── regional summaries
└── provenance
```

Planning and execution must not reach back into:

- ECS;
- a UI runtime;
- simulation state;
- authoring graphs;
- host window resources;
- product services.

## Contributions

Independent producers publish immutable contributions:

```text
RenderContribution
├── producer identity
├── contribution identity
├── generation
├── target selection
├── providers
├── instances
├── materials/media
├── emitters/environments
├── overlays
├── changes
└── provenance
```

Required lifecycle:

```text
insert
replace
remove
retire producer
```

Composition defines deterministic producer ordering, replacement, target/overlay
ordering, conflicts, missing references, and diagnostics.

Renderer contributor identities must not be raw ECS entity identities.

## Identity separation

Candidate renderer identities:

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

These are not RunenGPU identities.

Example:

```text
RenderProviderId
    semantic prepared provider

GpuResourceId
    one backend resource realization
```

One provider may have zero, one, or many GPU resources and different realizations
per device, quality policy, or frame.

This distinction is why the old renderer identity phase must not be implemented
before current IDs are classified.

## Views and logical targets

A view contains prepared render facts:

```text
PreparedView
├── identity
├── target
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
└── lifetime
```

Concrete textures and surface images are RunenGPU resources.

## Provider model

RunenRender is provider-oriented and field-first-capable without requiring one
universal representation.

Provider families may include:

```text
Surface
Shell
Fiber
Volume
Analytic
Population
Overlay
RegionalSummary
```

Capabilities may include:

```text
closest_hit
any_hit
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

Backend acceleration may use derived:

- sparse field pages;
- range or distance hierarchies;
- AABB/BVH structures;
- rasterized intermediates;
- hardware acceleration structures;
- temporary microgeometry.

Derived acceleration remains replaceable and non-authoritative.

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
├── provider/instance identity
├── local coordinates
├── approximation error/confidence
└── provenance
```

## Visibility architecture

RunenRender separates:

```text
ray/path selection
from
provider intersection implementation
```

Ray marching or sphere tracing is one local intersection strategy. It is not the
lighting architecture.

Provider strategies may use:

- sphere tracing;
- interval traversal;
- continuous root solving;
- analytic intersection;
- fiber solvers;
- volume integration;
- raster visibility;
- hardware ray queries.

Semantic query:

```text
trace(ray, purpose, error_tolerance, visibility_mask)
    -> RenderQueryOutcome
```

Purposes include primary, shadow, reflection, refraction, indirect, volume,
picking, and reference queries.

## Materials and media

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

Detail-frequency metadata is required:

```text
resolved detail     -> provider/geometric evaluation
transition detail   -> filtered evaluation
unresolved detail   -> effective material/statistical response
```

## Emitters and environments

One emitter model includes:

- directional, point, spot, and area lights;
- emissive surfaces and fields;
- particles and fire;
- environment lighting;
- procedural skies;
- distant regional emitter summaries.

Each emitter provides bounds/distribution, output, importance, linking masks,
generation, and provenance.

Many-light sampling, reservoirs, spatial reuse, and path guiding are estimator
implementations under one emitter contract.

## Transport architecture

RunenRender uses one semantic transport family.

Quality tiers vary budgets and estimators without switching to unrelated lighting
systems.

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

Quality ladder:

| Capability | Preview | Standard | High | Ultra | Reference |
|---|---:|---:|---:|---:|---:|
| Current primary visibility | Yes | Yes | Yes | Yes | Yes |
| Native visibility resolution | Yes | Yes | Yes | Yes | Yes |
| Dynamic direct lighting | Bounded | Many-light | Reservoir | Guided/reused | Progressive |
| Current shadow validation | Yes | Yes | Yes | Yes | Yes |
| Indirect continuation | 0 | 1 | 1-2 | Multiple | Progressive |
| Radiance-cache terminal | Yes | Yes | Yes | Optional/high | Optional |
| Spatial reuse | No | Optional | Yes | Yes | Not required |
| Temporal metadata | No | Optional | Bounded | Bounded | Progressive |
| Mandatory final-color history | No | No | No | Configurable | Progressive |
| Glossy/refraction/volumes | Simplified | Basic | Full | Advanced | Reference |

Each tier reports path depth, cache use, reuse/history policy, reconstruction, bias,
and unsupported transport.

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

The cache is derived and discardable.

It may terminate later bounces, approximate diffuse/rough-glossy transport, and
represent far/regional lighting.

Local geometry, material, medium, emitter, environment, and transform changes
invalidate only dependent cache regions.

Alternative implementations such as radiance cascades, neural caches, probes, or
hashed caches must satisfy the same semantic cache contract rather than create a
second GI API.

## Reconstruction and history

Sharp rendering does not require a ban on temporal information.

The default sharp contract requires:

- current-frame primary intersections;
- current-frame material evaluation;
- current-frame visibility validation;
- native-resolution edge ownership;
- disocclusion/change rejection;
- lobe-separated signals;
- bounded history age;
- generation/confidence validation.

History policies:

```text
None
WorldCacheOnly
ReservoirMetadataOnly
BoundedReservoirHistory
FullTemporal
ProgressiveReference
```

At minimum, reconstruction distinguishes:

```text
direct diffuse
indirect diffuse
glossy/specular
transmission
volume
emission
```

A single broad filter over final color is not sufficient.

## Overlay and RunenUI boundary

2D overlays are image composition, not a second lighting engine.

```text
3D scene transport
    -> tone/display transform boundary
    -> 2D overlay composition
    -> presentation
```

Overlay primitives may include shapes, strokes, clips, transforms, images, glyph
runs, layers, opacity, and damage.

RunenUI owns semantic UI, state, layout, accessibility, hit testing, text shaping,
caret/selection geometry, and renderer-neutral paint output.

Initial integration is Runenwerk-owned:

```text
RunenUI paint output
    -> Runenwerk adapter
    -> RunenRender overlay contribution
```

RunenRender never targets UI input from rendered pixels.

It may realize glyph atlases and composite exact shaped glyph geometry, but it must
not reshape text differently from RunenUI layout.

Normal HUD/editor UI is usually composed after 3D tone mapping. World-space UI may
render to a target consumed by the 3D scene.

## Color and presentation

RunenRender owns:

- scene-referred output;
- exposure;
- tone mapping;
- gamut mapping;
- transfer encoding;
- overlay composition;
- output-target intent.

RunenGPU owns concrete output resources and low-level presentation.

Runenwerk owns windows and product presentation/recovery policy.

## Stylization

Stylization is explicit at:

```text
MaterialStyle
TransportStyle
DisplayStyle
```

Examples:

- lobe and roughness remapping;
- toon response and quantized normals;
- light linking;
- shadow tint and density;
- bounce masks and indirect saturation;
- artistic emission propagation;
- tone curves, grading, outlines, bloom, posterization.

Stylized output uses the same prepared scene, providers, visibility, materials,
emitters, and transport family.

## Integration boundaries

### RunenGPU

`runenrender_gpu` lowers into `runengpu_core`. It does not directly use WGPU.

### RunenSDF

RunenSDF retains signed value, conservative safe-step, exact-distance capability,
bounds, and CPU reference queries.

A Runenwerk-owned adapter initially converts accepted RunenSDF sources into render
providers. A reusable bridge package is justified only after both public APIs are
stable and another host needs it.

### RunenECS

RunenRender does not depend on ECS. Runenwerk extracts explicit contributions.
ECS entity identities are not renderer runtime identities.

### Simulations

Fluid, wind, vegetation, fire, and other simulations retain their algorithms and
state. Adapters publish render providers, materials/media, motion, and changes.

## Error model

Required categories:

```text
RenderIdentityError
RenderSceneError
RenderContributionError
RenderProviderError
RenderMaterialError
RenderPlanError
RenderRealizationError
RenderQueryError
RenderTransportError
RenderHistoryError
RenderPresentationError
RenderTerminalError
```

RunenGPU failures retain their original classification and gain render provenance;
they are not flattened into renderer strings.

## Diagnostics

RunenRender exposes:

- scene and contribution composition;
- provider realization and residency;
- visibility queries and traversal cost;
- ray/sample/path counts;
- emitter candidates;
- radiance-cache usage and invalidation;
- history acceptance/rejection;
- reconstruction confidence;
- overlay ordering;
- presentation transformation;
- quality tier and declared approximations.

Provenance:

```text
render result
    -> render operation
    -> contribution
    -> producer
    -> source object/system
```

## Conformance

Extraction requires:

1. `runenrender_core` builds without Runenwerk, WGPU, Winit, ECS, SDF, or UI.
2. `runenrender_gpu` uses RunenGPU only for GPU execution.
3. renderer planning never reaches into a host world.
4. contribution insertion/replacement/removal is deterministic.
5. renderer semantic IDs and GPU runtime IDs remain distinct.
6. headless offscreen rendering works.
7. current-frame primary visibility is proven for the declared profile.
8. reference rendering provides a convergence oracle.
9. lower tiers document and measure bias toward reference.
10. incompatible generations reject history/cache reuse.
11. UI overlays require no semantic or hit-test access.
12. text compositing consumes accepted shaped geometry.
13. provider adapters compare against their domain CPU references.
14. Runenwerk consumes public seams only.
15. no duplicate renderer path remains.
16. planning, realization, visibility, transport, cache, reconstruction, and
    presentation benchmarks exist.

## Extraction gate

External RunenRender transfer is blocked until:

```text
external RunenGPU cutover complete
prepared scene and contributions proven
semantic render plan separated from GPU work
GPU realization uses RunenGPU only
offscreen parity proven
Runenwerk public-boundary-only consumption proven
no duplicate renderer path
complete validation/performance/provenance evidence
```

Advanced field-ray GI is not a prerequisite for repository extraction. The
framework boundary should be extracted before broad experimental renderer
development adds more code to the monolith.

## Final contract

> RunenRender owns how prepared render information becomes an image. It does not
> own how source worlds are authored, simulated, stored, scheduled, or executed by
> a concrete GPU backend.
