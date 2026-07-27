---
title: RunenRender Architecture and Decomposition Design
description: Decision-complete prepared-scene, provider, material, transport, reconstruction, overlay, RunenGPU, operational, host, conformance, and extraction architecture for RunenRender.
status: active
owner: render
layer: framework/render
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-architecture-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ./runen-family-operational-hardening-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../reports/investigations/runen-family-operational-hardening-investigation.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Architecture and Decomposition Design

## Status

The repository identity, one-package shape, ownership boundary, dependency on
RunenGPU, Runenwerk integration boundary, RunenUI relationship, prepared-scene
model, provider direction, and target image-formation architecture are fixed.

The S0 current-source, identity, consumer, lifecycle, shader, macro, and file
inventory is complete through issue `#127` / PR `#128`. Current paths remain evidence,
not permanent ownership. No RunenRender Rust implementation or external population is
authorized before accepted external RunenGPU cutover and a separately bounded R-phase
issue.

Operational hardening adds incremental-scene, provider-maturity, cache, capture, and
performance requirements without changing dependency direction or creating a new
phase.

## Mission

RunenRender owns image formation.

It answers:

> Given prepared views, providers, instances, materials, media, emitters,
> environments, overlays, changes, and quality policy, how should one or more images
> be formed?

RunenRender does not own:

- ECS or host-world storage;
- authoring source, scene persistence, or procedural world policy;
- field/SDF mathematics;
- simulations;
- UI state, layout, hit testing, focus, accessibility, or text shaping;
- windows and event loops;
- general GPU execution;
- WGPU devices, queues, surfaces, resources, or submission;
- shader filesystem discovery or hot-reload product policy;
- vertical-domain product systems;
- product lifecycle, quality selection, artifact encoding, or recovery.

## Repository and package

```text
repository: dornglut/runen-render
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

The public package remains one release unit until an independent consumer, backend,
release boundary, ABI, or compile-time boundary proves another package is needed.

Do not initially create `runenrender_core`, `runenrender_gpu`,
`runenrender_macros`, facade, capture, bridge, compatibility, or testing packages.
There is no `runenrender_wgpu`; concrete WGPU ownership belongs to RunenGPU.

## Dependency rules

RunenRender may depend on RunenGPU and appropriate low-level math/data libraries. It
must not depend on:

```text
WGPU
Winit
Runenwerk
RunenECS
RunenSDF
RunenUI
scene/world/material-authoring/editor/application domains
```

RunenRender lowers renderer semantics into RunenGPU work through public contracts.
It must not reach through RunenGPU into backend handles or recreate a second GPU
resource, graph, submission, progress, loss, or error model.

## Ownership

RunenRender owns:

- renderer semantic identities;
- prepared scenes and deterministic contribution composition;
- views and logical targets;
- providers, instances, and narrow interaction capabilities;
- material, medium, emitter, and environment semantics;
- visibility and provider-query policy;
- transport, estimator, and renderer quality semantics;
- render-derived caches and bounded history;
- reconstruction and anti-aliasing policy;
- overlay composition and presentation intent;
- color-pipeline intent;
- deterministic render planning;
- renderer diagnostics and provenance;
- render-specific lowering into RunenGPU work.

RunenRender does not own:

- source ECS entities or scene nodes;
- world/chunk streaming;
- material graph authoring or asset import;
- SDF fields or their numerical query semantics;
- UI widget semantics;
- WGPU realization;
- native windows;
- application lifecycle;
- product feature registries, fallback policy, or recovery;
- domain formats, constraints, simulation policy, or regulated validation.

## Prepared render scene

RunenRender consumes renderer-owned immutable prepared state:

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
├── changed regions and generations
├── importance hints
├── regional summaries
└── provenance
```

The prepared scene is not an ECS mirror. Planning and execution must not reach back
into ECS, UI runtime state, simulation state, authoring graphs, host windows, or
product services. Runenwerk adapters form accepted prepared values before submission.

### Incremental prepared-scene requirement

R1/R2 must support deterministic:

```text
insert contribution
replace contribution
remove contribution
retire producer
```

Every mutation records affected identities, generations, and changed regions.
Unrelated views, providers, instances, materials, media, emitters, targets, and
overlays must not require reconstruction when correctness facts permit narrower
updates.

A source adapter that cannot provide narrow changes may request an explicit full
rebuild. Full rebuild is a declared fallback, not hidden behavior.

Required proof:

- equivalent full and incremental construction produce identical semantic prepared
  state;
- replacement/removal/retirement are deterministic;
- missing references and conflicts produce structured diagnostics;
- affected/unaffected work is inspectable;
- update cost is characterized rather than assumed.

## Contribution model

Independent producers publish immutable values:

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

Composition defines deterministic producer ordering, replacement, target ordering,
overlay ordering, conflict handling, missing references, and diagnostics.

Producer and contribution identities are renderer-local runtime values. They are not
raw ECS entity IDs or stable persisted asset IDs.

The current Runenwerk `PreparedFrameContributions` feature map and collector registry
are migration evidence only. They remain ECS/product-shaped and do not define the
future renderer public API.

## Identity separation

Candidate renderer concepts include:

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

Required separation:

```text
RenderProviderId   semantic prepared provider
Gpu resource ID    one RunenGPU logical or realized resource
ECS entity ID      one Runenwerk source entity
Asset ID           one authored/persisted source identity
```

One provider may have zero, one, or many GPU realizations across devices, quality
policies, frames, and caches.

## Views and logical targets

A prepared view contains:

```text
PreparedView
├── identity
├── logical target
├── projection
├── current and previous transforms
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
Runenwerk-owned.

## Provider architecture

RunenRender is provider-oriented and field-first-capable without requiring one
universal representation or mandatory source meshes.

### Provider maturity

Provider families are planning categories, not commitments to implement every family.

```text
near-term proof
    Procedural
    Analytic
    field-backed Solid sufficient for first SDF terrain
    Overlay

research candidate
    Volume
    Population
    RegionalSummary
    Liquid

fully deferred pending accepted consumer evidence
    Fiber
    broad hardware-specialized variants
    universal provider unification
```

`Shell` may be introduced only when an accepted proof distinguishes it from the
near-term surface/field capabilities.

### Narrow provider capabilities

No universal provider trait may require every provider to implement every query.
Capabilities remain narrow:

```text
surface_query
visibility_query
interval_query
transmittance_query
raster_visibility
procedural_evaluation
material_attributes
motion
refinement
streaming
```

Hardware acceleration is a realization/fallback fact, not a semantic capability that
every provider must implement.

A new provider family requires:

- a concrete accepted consumer;
- owned numerical and semantic contracts;
- a representative proof;
- explicit non-goals;
- no expansion of unrelated provider interfaces.

### Derived acceleration

Derived acceleration may use:

- sparse field pages or clipmaps;
- range/distance hierarchies;
- AABB/BVH structures;
- rasterized intermediates;
- procedural tables;
- hardware acceleration structures;
- temporary backend triangles, AABBs, or microgeometry.

Derived acceleration is replaceable, discardable, source-generation-bound,
validated before reuse, and non-authoritative. “No mandatory meshes” does not
prohibit backend-local geometry when it is the best acceleration form.

## Interaction contract

Provider-specific strategies produce shared semantic interactions:

```text
RenderInteraction
├── distance
├── world position
├── geometric and shading orientation
├── material
├── medium transition
├── emission
├── local velocity
├── provider and instance identity
├── local coordinates
├── approximation error/confidence
└── provenance
```

A provider implements only the interaction/query capabilities needed by accepted
consumers.

## Visibility and query architecture

RunenRender separates path/ray selection from provider query strategy:

```text
trace(ray, purpose, tolerance, visibility_mask)
    -> RenderQueryOutcome
```

Purposes may include primary, shadow, reflection, refraction, indirect, volume,
picking, and reference.

Provider strategies may use sphere tracing, interval/range traversal, continuous root
solving, analytic intersection, volume integration, raster visibility, or later
contained hardware ray queries.

Ray marching is an intersection technique, not the lighting architecture.
RunenRender does not reinterpret RunenSDF values directly; an adapter preserves
RunenSDF numerical and capability contracts.

## Materials, media, emitters, and environments

### Materials

A material defines scattering independently of provider representation:

```text
RenderMaterial
├── scattering closure
├── parameter sources and layers
├── emission and transmission
├── subsurface policy
├── displacement/detail policy
├── material style
└── provenance
```

Material authoring graphs and asset import remain outside RunenRender.

Detail-frequency metadata distinguishes resolved, transition, and unresolved detail
so quality changes do not silently discard semantic material state.

### Media

A medium defines absorption, scattering, phase behavior, emission, density source,
interface priority, and provenance. Interfaces identify medium transitions.

### Emitters and environments

One emitter model may cover directional, point, spot, area, emissive surface/field,
particles/fire, environment lighting, procedural skies, and distant/regional
summaries.

Many-light sampling, reservoirs, spatial reuse, and path guiding are estimator
implementations under the same emitter semantics.

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

Initial quality ladder:

```text
Preview
Standard
High
Ultra
Reference
```

All tiers share current prepared scene, provider/interaction contracts,
material/medium/emitter semantics, current primary visibility, and structured
capability/degradation reporting.

Budgets may vary for path depth, light candidates, spatial reuse, cache use,
glossy/refraction/volume support, reconstruction/history, and progressive
accumulation. No tier silently renders stale primary visibility or material state.

## Derived caches and history

### Cache doctrine

Every render-derived cache records all correctness facts required for reuse:

```text
scene generation
view generation
provider/instance generations
material/medium/emitter generations
quality/algorithm/schema revision
changed-region coverage
RunenGPU context/device generation where realized
reconstruction source or explicit non-reconstructability
```

A cache hit changes cost, never semantics. Incomplete compatibility facts cause
rejection or full rebuild.

### Radiance cache

A scalable GI cache may store spatial domain, directional radiance, visibility
summary, geometry/material validity, variance/confidence, source generations, update
age, and provenance.

It may approximate later transport and far/regional lighting. It must not be the only
source of current primary visibility or authoritative scene state.

### History

History policy is explicit:

```text
None
WorldCacheOnly
ReservoirMetadataOnly
BoundedReservoirHistory
FullTemporal
ProgressiveReference
```

Default product policy preserves sharp current-frame visibility, material changes,
validation, and disocclusion. Final-color history is not mandatory.

R6 must prove narrow invalidation when correctness facts permit and full invalidation
when they do not. Device-generation change invalidates GPU-realized caches/history.
Stale cache use is never a quality-degradation mechanism.

## Stylization

Stylization is separated by owner:

```text
MaterialStyle   local scattering/emission/detail
TransportStyle  lighting/path/visibility interpretation
DisplayStyle    color mapping/compositing/presentation
```

Stylization does not create a second renderer or bypass current visibility/material
validity.

## Overlay architecture

RunenRender accepts renderer-neutral overlay contributions such as shapes, strokes,
clips, transforms, glyph runs, images, layers, blend/composite intent, and damage
regions.

RunenRender may own GPU glyph/image resources, atlas residency, rasterization, and
compositing. It does not own shaping, line breaking, caret/selection, accessibility,
widget state, or UI hit testing.

```text
RunenUI paint scene
    -> Runenwerk bridge
        -> RunenRender overlay contribution
            -> RunenGPU work
```

The bridge consumes paint facts, not widget state or actions.

## RunenGPU lowering

RunenRender lowers semantic render plans into `GpuWorkFragment` values.

RunenRender may own render-specific:

- resource realization and cache keys expressed through RunenGPU contracts;
- provider acceleration;
- visibility/intersection pipelines;
- material/medium evaluation;
- emitter sampling;
- transport wavefronts;
- cache/history updates;
- reconstruction;
- overlay rasterization;
- color/output transformation intent.

RunenRender does not own device/queue/surface creation, generic allocation/hazard
validation, command submission, progress, backend capability mapping, device loss, or
a second GPU lifetime/error model.

## Shader boundary

RunenRender owns render shader meaning and source products. RunenGPU owns program
admission, interface/layout validation, and backend realization. Runenwerk owns
filesystem discovery, revision/watch/reload, user-facing diagnostics, and
last-known-good product policy.

WGSL/WGPU ABI details are not universal renderer semantics. A macro package requires
concrete public-API and conformance evidence.

## Host, presentation, and recovery

Runenwerk owns windows, event loops, resize/DPI/visibility policy, presentation
timing, product quality selection, artifact encoding, and recovery decisions.

RunenGPU owns low-level surface operations, generations, and outcomes.
RunenRender owns logical targets, output color, compositing, and presentation intent.

```text
Runenwerk host/product policy
    -> RunenRender logical image intent
        -> RunenGPU resource/surface execution
```

On device loss, RunenRender reports affected scene/cache/history realizations and
reconstruction facts. It does not decide whether the product retries, degrades,
recreates, or exits.

## Diagnostics, capture, and reproducibility

RunenRender exposes structured facts for:

- prepared-scene and contribution validation;
- missing/invalid providers and references;
- capability/quality degradation;
- visibility and interaction failures;
- material/medium/emitter admission;
- transport budgets and unsupported paths;
- cache/history validity and invalidation;
- reconstruction;
- overlay composition;
- RunenGPU work provenance;
- full versus incremental preparation evidence.

RunenRender does not decide product severity, retention, persistence, UI
presentation, privacy, encoding, or recovery.

Runenwerk may include namespaced RunenRender facts in a versioned reproducibility
bundle. Runtime IDs, pointers, and unversioned diagnostic strings are not stable
capture authority.

## Operational phase requirements

```text
R1/R2
    prepared scene/contribution identity and deterministic lifecycle
    incremental insert/replace/remove/retire-producer proof

R3
    near-term provider proofs through narrow capabilities
    reject universal provider-trait pressure

R4/R5
    image-formation/transport/quality proof on accepted RunenGPU

R6
    cache/history generation and changed-region invalidation

R7
    target/surface integration only through RunenGPU facts

R8
    renderer performance, memory, capture, diagnostics, recovery facts,
    reproducibility, and anti-cheating proof
```

No new R phase is created.

## Performance characterization

R8 must record at least:

- full-scene versus incremental preparation cost;
- affected and unaffected contribution work;
- provider-query counts and divergence evidence;
- cache hit/miss/invalidation;
- current-frame versus history-dependent paths;
- CPU/GPU memory high-water marks;
- cold/warm program/pipeline cost inherited through RunenGPU;
- artifact/capture reproducibility;
- comparison with a simpler renderer or direct path for the same bounded proof.

Performance evidence is diagnostic until a separately accepted controlled budget
exists. No private RunenGPU/WGPU reach-through may make the framework benchmark look
better.

## Application-domain boundary

High-value pressure includes implicit fabrication, scientific volumes, robotics
sensors, geospatial/environmental visualization, digital twins, and offline
procedural output.

Those applications retain constraints, domain formats, data governance, simulations,
streaming, timelines, collaboration, and product workflows. RunenRender provides
image-formation contracts, not complete vertical products.

## Current decomposition problem

The current `engine/src/plugins/render` mixes:

- general GPU execution/WGPU ownership;
- render graph and image formation;
- native-window surfaces;
- ECS/host projection and frame policy;
- source-domain and product features;
- shader discovery/hot reload;
- diagnostics, captures, artifacts, readiness, residency, and pacing.

Current frame contributions are ECS/product-shaped; `WgpuCtx` exposes backend
`Device`/`Queue`; captures are runtime string/byte values; current caches/residency
combine product and backend concerns. Moving the directory unchanged is forbidden.

## Conformance

Internal RunenRender proof requires:

1. prepared scene and contribution composition without ECS, Runenwerk, WGPU,
   RunenSDF, or RunenUI;
2. deterministic insert/replace/remove/retire-producer behavior;
3. equivalent full and incremental prepared results;
4. at least two independent producer families through the same seam;
5. near-term providers using only required narrow capabilities;
6. provider/interaction/material/medium/emitter tests independently;
7. deterministic render planning;
8. RunenGPU lowering through public work contracts only;
9. no direct WGPU dependency or private reach-through;
10. overlay proof using neutral primitives;
11. quality/history/cache validity and invalidation tests;
12. full/incremental performance and memory characterization;
13. reproducibility/capture facts supplied without owning persistence policy;
14. Runenwerk adapters consume no private renderer internals;
15. no duplicate old render path.

External proof additionally requires independent locked validation, public downstream
consumption, exact RunenGPU revision, provenance, operational conformance, and clean
Runenwerk cutover.

## Current-source revalidation gate

Before each R implementation slice:

- verify current `main` and exact accepted base;
- repeat declaration and direct/transitive consumer inventory for affected authority;
- classify current graph, resource, surface, shader, pipeline, macro, residency,
  frame, diagnostics, capture, runtime, example, test, and benchmark paths;
- verify identities, raw conversions, and persisted uses;
- identify all host/ECS/source-domain reach-back;
- bind exact move/stay/redesign/delete scope;
- run the canonical validation baseline;
- stop if a new ADR, dependency direction, package, compatibility path, or stable
  persisted-format change is required.

The accepted S0 inventory remains discovery evidence, not permission to skip current
source revalidation.

## Definition of done

RunenRender extraction is complete only when:

- one independently validated package exists in `dornglut/runen-render`;
- RunenRender depends on an exact RunenGPU revision and not WGPU;
- Runenwerk consumes only public prepared/contribution/adapter seams;
- RunenUI and RunenSDF remain independent;
- near-term provider and image-formation proofs pass;
- incremental scene lifecycle and cache/history invalidation are proven;
- operational/performance/capture/recovery facts pass R8 conformance;
- downstream and runtime evidence pass;
- every active consumer is migrated;
- exact provenance is recorded;
- original Runenwerk image-formation authority and temporary seams are deleted;
- no mirror, compatibility package, duplicate renderer, or private reach-through
  remains.

## Strategic reevaluation gates

Reconsider the RunenRender split if:

- a smaller rend3/Filament-style renderer satisfies all accepted proofs;
- provider abstractions force unrelated query families into one interface;
- prepared scenes systematically require full rebuilds;
- optional history/current-frame quality cannot share coherent semantics;
- RunenRender becomes architecture without multiple real provider families;
- measured cost materially exceeds a simpler renderer without reusable value.

Reevaluation requires an explicit architecture decision and does not authorize a
hidden bypass.
