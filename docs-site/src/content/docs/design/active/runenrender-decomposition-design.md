---
title: RunenRender Architecture and Decomposition Design
description: Canonical long-term semantic scene-to-image, representation, planning, scalability, RunenGPU lowering, operational, conformance, and extraction architecture for RunenRender.
status: active
owner: render
layer: framework/render
canonical: true
last_reviewed: 2026-08-04
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./runengpu-architecture-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runengpu-shader-authoring-artifact-boundary.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ./runen-family-operational-hardening-design.md
  - ../../reports/investigations/2026-08-04-runenrender-long-term-capability-and-scalability-review.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../reports/investigations/runen-family-operational-hardening-investigation.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Architecture and Decomposition Design

## Status and authority

This document is the sole canonical RunenRender architecture authority inside
Runenwerk.

The following decisions are fixed:

- repository identity and one-package initial shape;
- dependency `RunenRender -> RunenGPU`;
- no direct WGPU ownership;
- semantic scene-to-image ownership;
- immutable renderer-owned scene revisions;
- separation of scene state, render requests, dynamic inputs, and output bindings;
- heterogeneous representations through narrow versioned protocols;
- device-independent planning followed by execution-environment admission;
- generic lowering through public RunenGPU contracts;
- non-authoritative derived state;
- bounded pressure, diagnostics, and operational outcomes;
- clean internal proof followed by mechanical external cutover.

The following remain future implementation decisions:

- exact Rust field layout and naming below this semantic vocabulary;
- internal scene-storage data structures;
- exact public extension traits;
- shader code-generation strategy;
- method-internal fused, wavefront, raster, ray, regional, or hybrid topology;
- concrete advanced methods and representation families;
- dynamic-library ABI or plugin packaging.

No RunenRender Rust implementation, external repository population, dependency change,
or R-phase activation is authorized by this design alone.

## Mission

RunenRender owns semantic scene-to-image formation.

It answers:

> Given one immutable renderer scene revision, one render request, current typed dynamic
> inputs, and available execution facts, which representations and coherent render
> method should form the requested outputs, with what validity, cost, degradation, and
> provenance?

RunenRender does not own:

- ECS or host-world storage;
- world generation, persistence, or chunk-streaming policy;
- source assets, authoring scene graphs, or editor documents;
- field/SDF mathematical authority;
- simulations or physics;
- UI state, layout, shaping, focus, accessibility, or hit testing;
- windows, event loops, tracking runtimes, or XR sessions;
- generic GPU resources, work validation, realization, submission, progress, or surfaces;
- shader filesystem discovery, watching, authoring compiler selection, or product
  last-known-good policy;
- MaterialX, USD, glTF, OCIO, ACES, or other interchange/configuration authority;
- distributed process/network orchestration;
- image, dataset, or video encoding;
- product recovery, severity, quality presets, or application lifecycle.

## Repository and package

```text
repository: dornglut/runen-render
package: runen-render
crate: runen_render
depends on: runen-gpu
```

RunenRender initially contains one public package. Internal modules carry responsibility
boundaries until an independent consumer, release unit, backend, ABI, compile-time
boundary, or measured build-cost problem proves another package is required.

Do not initially create:

```text
runenrender-core
runenrender-gpu
runenrender-wgpu
runenrender-macros
runenrender-plugins
runenrender-capture
runenrender-compat
```

Concrete WGPU ownership belongs to RunenGPU.

Directional internal modules may include:

```text
scene
change
identity
space
time
view
input
output
representation
protocol
query
material
medium
emitter
appearance
method
planning
admission
derived
session
overlay
diagnostics
gpu
```

The module list is not public API authority.

## Dependency rules

RunenRender may depend on RunenGPU and justified low-level math/data libraries.

It must not depend on:

```text
WGPU
Winit
Runenwerk
RunenECS
RunenSDF
RunenUI
application, editor, world, asset, authoring, or product packages
```

Adapters translate source domains into RunenRender semantic values. RunenRender lowers
those values into RunenGPU work. It does not reach through RunenGPU into backend
objects or recreate GPU identity, resource, graph, progress, surface, or error
authority.

## Neutrality boundary

RunenRender is neutral across:

```text
source domain and host storage
scene representation
query implementation
render method
internal execution decomposition
output destination
physical, technical, sensor, scientific, or stylized appearance
GPU backend
```

RunenRender is intentionally opinionated about:

```text
immutable renderer scene revisions
renderer-local semantic identities
explicit space and time
views and sampling footprints
materials, media, emitters, and environments
narrow versioned query protocols
semantically defined outputs
coherent render methods
deterministic planning and admission
derived-state validity
RunenGPU lowering
structured diagnostics and provenance
```

RunenRender is not an arbitrary compute-to-pixels framework. Generic GPU computation
belongs to RunenGPU consumers.

## Public mental model

```text
RenderSceneStore
    -> commit(RenderSceneUpdate)
        -> RenderSceneCommit
            - RenderSceneSnapshot
            - RenderChangeSet

RenderSceneSnapshot
+ RenderRequest
+ RenderInputSet
    -> RenderMethod
        -> RenderPlan
            -> AdmittedRenderPlan
                -> RenderWorkSet
                    -> RunenGPU
                        -> RenderResult / RenderSession
```

The ordinary API may collapse planning, admission, and lowering into one call. The
separate authorities remain inspectable and testable.

## Identity model

Renderer-local semantic identities include candidate concepts such as:

```text
RenderSceneId
RenderSceneRevision
RenderObjectId
RenderRepresentationId
RepresentationElementId
RenderProducerId
RenderContributionId
RenderViewId
RenderInputSlotId
RenderOutputId
RenderMaterialId
RenderMediumId
RenderEmitterId
RenderEnvironmentId
RenderOverlayId
RenderMethodId
RenderMethodRevision
RenderSessionId
DerivedRenderStateId
```

Required separation:

```text
RenderObjectId             stable renderer semantic object
RenderRepresentationId     one renderable form of that object
RepresentationElementId    one element inside a representation
source-domain identity      ECS entity, scene node, procedural chunk, etc.
asset identity              authored or persisted source identity
RunenGPU resource identity  one logical/realized GPU resource
```

Representation replacement must not silently change semantic object identity.

Runtime renderer identities are not stable persistence, wire, cache, or cross-process
authority unless a separately versioned schema explicitly promotes them.

`RenderObject` is preferred over `RenderEntity` to avoid implying RunenECS ownership.

## Scene store, updates, snapshots, and change sets

### `RenderSceneStore`

`RenderSceneStore` owns renderer scene lineage and publishes immutable snapshots.

It may internally use:

```text
persistent tables
generation-indexed arenas
structural sharing
copy-on-write pages
paged scene databases
lazy query views
compiled dense renderer tables
```

The implementation must not require a deep copy per commit.

### `RenderSceneUpdate`

One update contains atomic insert, replace, remove, relationship, and producer-retirement
operations.

Independent producers may contribute objects, representations, instances, materials,
media, emitters, environments, overlays, relationships, and provenance.

Producer and contribution identities are renderer-local runtime identities. They are
not raw ECS or asset identities.

### `RenderSceneCommit`

A successful commit publishes:

```text
RenderSceneCommit
├── RenderSceneSnapshot
└── RenderChangeSet
```

### `RenderSceneSnapshot`

A snapshot is a cheap immutable view of one committed semantic revision.

It may expose:

```text
objects and representations
instances and transforms
materials and media
emitters and environments
overlays
typed relationships
semantic attributes
scene and table generations
provenance
```

It does not contain an acquired surface image or one invocation's output bindings.

A snapshot is not an ECS mirror. Planning and lowering do not reach back into ECS,
simulation state, source assets, UI runtime state, host windows, or product services.

### `RenderChangeSet`

Change evidence may include:

```text
added and removed objects
changed properties and relationships
changed spatial regions
changed temporal intervals
representation-offer changes
material/medium/emitter/environment changes
availability and residency changes
explicit full resynchronization
```

A non-spatial change must not be fabricated as a spatial region.

Equivalent full and incremental construction must produce the same semantic snapshot.
Adapters unable to provide narrow evidence may request explicit full resynchronization.

## Scene relationships

The scene model must preserve room for typed, versioned relationships such as:

```text
instance/prototype relationships
material assignment
medium boundaries
visibility collections
light and shadow linking
render layers
holdouts and mattes
semantic groups
overrides and variants
overlay ordering
```

Relationships are not arbitrary string tags. Each relationship family defines identity,
referential validity, dependency, and invalidation behavior.

R1 need not implement every relationship family. The scene kernel must not assume every
object is only representation + transform + material.

## Space, units, precision, and transforms

Foundational concepts include:

```text
RenderSpace
CoordinateFrame
UnitScale
SpatialTransform
RenderOrigin
PrecisionContract
SpatialCoverage
```

Source domains retain authoritative coordinate meaning.

RunenRender may lower high-precision or region-indexed source positions into
view-relative, region-relative, or representation-local GPU coordinates.

A representation offer states:

```text
coordinate frame
unit scale
valid spatial domain
precision and error units
transform behavior
```

Field and distance representations explicitly classify transforms as preserving exact,
conservative, approximate, or invalid distance semantics. Nonuniform scaling cannot
silently preserve exact signed-distance guarantees.

## Time, exposure, and motion

Foundational concepts include:

```text
RenderTime
TimeInterval
ExposureModel
TemporalSamplingPolicy
TemporalCoverage
MotionContract
```

Views, representations, inputs, materials, media, emitters, and derived state state the
time intervals for which they are valid.

Supported future observation models may include global shutter, rolling shutter,
camera motion, instance motion, deformation motion, time-dependent fields, and
asynchronous source updates.

Runenwerk owns clocks, scheduling, tracking, late-latching, and product timing policy.
RunenRender consumes immutable time and exposure facts.

## Views and sampling footprints

### `RenderView`

A view describes one semantic observation:

```text
identity
projection or observation model
coordinate frame and transforms
temporal samples
viewport or image region
visibility and relationship masks
sample distribution
provenance
```

Potential future view forms include perspective, orthographic, panoramic, fisheye,
stereo, cubemap, light-field, sensor, and custom versioned models.

### `RenderViewSet`

A request may contain coordinated views for stereo, cubemap, multiview, XR, sensors, or
batch output.

Compatible views may share scene preparation, representation compilation, acceleration,
and derived state.

### `SampleFootprint`

A sample footprint may represent:

```text
pixel footprint
ray cone
ray differentials
surface-space derivatives
volume-filter footprint
temporal footprint
spectral footprint
```

Representations and materials may evaluate at a point, evaluate over a footprint,
return a filtered approximation, provide frequency/error evidence, or request
refinement.

Detail generation is driven by output footprint and declared error, not only geometric
distance.

## Render objects and representations

### `RenderObject`

A render object is a renderer-local semantic object.

It may originate from an ECS entity, scene node, asset instance, procedural region,
simulation, aggregate, or no stable source object.

### `RenderRepresentation`

A representation is one renderer-visible form through which an object participates in
image formation.

Examples include:

```text
analytic primitive
triangle, curve, point, or splat data
signed-distance or general implicit field
dense, sparse, adaptive, or unstructured volume
particle or procedural population
fiber or hair data
procedural generator
neural or learned field
far-field or regional summary
overlay geometry
```

One object may expose multiple representations simultaneously.

Representation categories are not a closed public enum.

## Representation offers

Each available representation publishes a `RepresentationOffer` containing:

```text
representation identity
supported versioned protocols
spatial and temporal coverage
space and unit conventions
accuracy and confidence
freshness/source revision
residency and availability
refinement and fallback
measurement compatibility
provenance
```

Residency states may include:

```text
Resident
PartiallyResident
Pending
Unavailable
Failed
```

A render method may:

```text
use a valid coarser representation
wait within an explicit bound
publish incomplete-result diagnostics
omit an optional output
reject the request
```

It may not silently use stale or semantically incompatible data as a quality
degradation.

## Versioned query protocols

Representations support small versioned semantic protocols, for example:

```text
SurfaceQueryProtocolV1
VisibilityQueryProtocolV1
VolumeTraversalProtocolV1
RasterGeometryProtocolV1
AttributeProtocolV1
MotionProtocolV1
RefinementProtocolV1
PopulationProtocolV1
```

A protocol defines:

```text
inputs
outputs
space and time conventions
accuracy and error meaning
batching expectations
availability and fallback behavior
version compatibility
structured unsupported outcomes
```

A protocol does not prescribe:

- one Rust trait-object call;
- one shader-language interface;
- one dispatch per query;
- one GPU representation.

RunenRender may batch, compile, specialize, fuse, or generate method-specific programs
that satisfy the semantic protocol.

Adding mandatory fields to an accepted protocol requires a new protocol version.

## Query results and lazy attributes

Use narrow semantic results:

```text
SurfaceHit
VisibilityResult
VolumeInterval
TransmittanceResult
```

A minimal `SurfaceHit` identifies:

```text
distance or parameter
object
representation
representation element
local position
surface side
accuracy evidence
provenance
```

Additional information uses explicit protocols:

```text
normal and tangent frame
material binding
medium boundary
motion
derivatives
local coordinates
custom semantic attributes
```

No universal interaction record requires every representation to fabricate UVs,
normals, materials, velocity, emission, or medium data.

## Materials, media, emitters, and environments

### Materials

Separate:

```text
authored/imported material graph
renderer semantic material
method-compiled material program
RunenGPU program/resource realization
```

Renderer material semantics preserve room for:

```text
BSDF surface scattering
EDF emission
VDF volume scattering
medium interfaces
thin-walled behavior
layering
subsurface scattering
displacement and microdetail
measured materials
spectral parameters
stylized and nonphysical models
```

Material authoring graphs and asset import remain outside RunenRender.

### Media

A medium may define absorption, scattering, phase behavior, emission, density source,
majorants/ranges, sampling or reconstruction filter, interface priority, measurement
compatibility, and provenance.

### Emitters and environments

Emitter semantics may cover directional, point, spot, area, emissive
surface/field/population, environment, procedural sky, measured distribution, and
regional summaries.

Many-light sampling, reservoirs, path guiding, and regional reuse remain method or
derived-state implementations.

## Measurement domains and precision

Image formation is not synonymous with RGB display color.

A `MeasurementDomain` may represent:

```text
monochromatic intensity
RGB radiance approximation
spectral radiance
polarized spectral radiance
depth or distance
transmittance
normal or orientation
velocity or motion
identity or segmentation
variance or confidence
derivative
versioned custom measurement
```

Materials, media, emitters, methods, inputs, and outputs must agree on compatible
measurement semantics.

Precision is explicit. A method may admit single, mixed, double, fixed, quantized, or
other declared numerical modes without redefining scene semantics.

## Render requests

`RenderRequest` owns one semantic image-formation request:

```text
view set
render-output set
method selection
accuracy policy
determinism policy
history/session policy
hard render limits
performance goal
appearance
time/exposure intent
provenance
```

Product-facing presets may expand into explicit request values. Preset names are not
semantic authority.

Hard limits and soft performance goals remain distinct.

## Render methods

A `RenderMethod` is a coherent image-formation algorithm family.

Potential methods may use:

```text
raster visibility
direct surface queries
path transport
volume integration
regional or cellular propagation
sensor simulation
reference computation
hybrid combinations
```

These examples do not form an initial closed enum.

Applications do not independently connect arbitrary visibility, transport,
reconstruction, history, and display strategy objects.

A method implementation may internally compose reusable typed operators, but it owns a
coherent valid combination.

Execution topology remains private and may use fused kernels, wavefront queues, raster
passes, compute passes, ray queries, indirect work, persistent queues, or regional
iterations.

The exact public method-extension trait remains unstabilized until at least two
meaningfully different methods prove the shared contract.

## Semantic planning and execution admission

### `RenderPlan`

A render plan is device-independent semantic planning with admissible alternatives.

It records:

```text
requested method and method revision
required query protocols
candidate representations
view and output requirements
measurement and precision requirements
accuracy and determinism requirements
dynamic-input requirements
derived-state dependencies
permitted fallbacks
semantic degradations
diagnostics
```

The plan contains no WGPU object.

### `AdmittedRenderPlan`

Admission selects one concrete valid execution for an environment:

```text
selected representations
selected method variant
measurement and precision mode
residency selections
selected programs and derived state
RunenGPU capability requirements
resource and work estimates
variant estimates
declared degradations
exact protocol, method, and schema revisions
```

This split supports CPU-only tests, preflight tools, multiple devices, remote planning,
replay, and understandable fallback.

The ordinary API may perform planning and admission internally.

## Dynamic render inputs

`RenderInputSet` binds typed `RenderInputSlot<T>` values.

Each slot defines:

```text
semantic meaning
data schema
space and units
temporal coverage
generation
lifetime and access
availability
fallback
```

Bindings may reference:

```text
CPU-prepared values
retained RunenGPU resources
typed exports from preceding RunenGPU work
streamed pages
externally reconstructed data
renderer-derived state
```

A GPU simulation remains authoritative in its source domain. RunenRender consumes typed
current inputs without CPU readback.

RunenGPU typed import/export causality orders producer work before renderer work.

## Render outputs

Use `RenderOutputSet`.

Each `RenderOutputSpec` states:

```text
semantic meaning
extent or output domain
measurement domain
numeric representation
precision
accumulation and merge rule
layout
destination intent
required or optional status
```

Potential layouts include:

```text
FlatImage
MultiSampleImage
DeepImage
TileSet
SparseImage
BufferProduct
ReadbackProduct
PresentationProduct
```

Potential outputs include display color, linear radiance, depth, distance, normals,
albedo/material attributes, object/representation IDs, segmentation, motion, optical
flow, transmittance, emission, variance, confidence, sample count, derivatives,
diagnostic images, and versioned custom products.

Concrete textures, buffers, and acquired surface images are RunenGPU values.

Runenwerk owns artifact encoding and persistence.

## Accumulation and merge semantics

Outputs combine according to their meaning:

```text
radiance        weighted sum or average
depth           nearest or minimum valid sample
normal          normalized weighted accumulation
identity        selected-sample identity
variance        statistical combination
segmentation    categorical reduction
deep output     ordered depth-sample merge
```

These rules support progressive, tiled, multiview, multi-device, and distributed
rendering.

RunenRender owns semantic partitionability and merge meaning. Runenwerk or another
orchestrator owns networking, process management, retries, and job policy.

## Appearance, stylization, and color

### `RenderAppearance`

Broadly applicable appearance contains:

```text
material and medium appearance
output and display intent
method-specific typed appearance extensions
```

Transport, reconstruction, hatching, stippling, brush behavior, regional propagation,
or other specialized controls are not mandatory global style interfaces. A method may
publish typed versioned extension slots.

Stylization does not create a second scene or bypass current visibility, material,
input, and invalidation semantics.

### Color pipeline intent

Separate:

```text
scene measurement
renderer working representation
creative appearance
display transform
output encoding
```

RunenRender owns semantic conversion points and measurement meaning.

Runenwerk owns OCIO/ACES configuration, display selection, persisted color policy, and
artifact encoding.

## Overlay architecture

RunenRender accepts renderer-neutral overlay contributions such as:

```text
shapes and strokes
clips and transforms
glyph runs and images
layers
blend and composite intent
damage regions
```

RunenRender may own GPU glyph/image resources, atlas residency, rasterization, and
compositing.

It does not own text shaping, line breaking, caret/selection semantics, accessibility,
widget state, or UI hit testing.

```text
RunenUI paint scene
    -> Runenwerk bridge
        -> RunenRender overlay contribution
            -> RunenGPU work
```

The bridge consumes paint facts, not widget state.

## Derived render state

Derived state forms an explicit dependency graph.

Kinds may include:

```text
compiled representations
acceleration
residency
transport estimates
history
reconstruction
output accumulation
```

Each node records:

```text
kind and owner method
exact source dependencies
spatial and temporal coverage
measurement domain
accuracy and confidence
residency
memory class
retention priority
update strategy
reconstruction recipe
RunenGPU context/device generation where realized
```

Derived state is non-authoritative, discardable or reconstructable, bounded or
pressure-reporting, validated before reuse, and invalidated through exact dependencies.

A cache hit changes cost, never semantic truth.

## Progressive render sessions

A `RenderInvocation` is one bounded request.

An optional `RenderSession` owns compatible continuity across invocations:

```text
accumulation identity
method revision
output set
scene/view/input compatibility
sample and random-stream ranges
progress and convergence
cancellation
terminal state
```

Each underlying RunenGPU submission still receives exactly one terminal outcome.

Session continuation rejects incompatible scene, request, method, output, measurement,
or input changes unless the method explicitly defines a valid migration.

## Multi-device and distributed compatibility

The lowering result is a `RenderWorkSet`, not a permanent assumption of one
context-local fragment.

A work set may contain:

```text
one device-local fragment
multiple device-local partitions
merge work
readback work
presentation work
```

RunenRender declares semantic partitionability and merge rules.

Runenwerk or another product/orchestration layer owns remote transport, process
lifecycle, retries, scheduling, and cluster policy.

## Differentiable and inverse rendering

The architecture preserves room for:

```text
RenderParameterId
DifferentiableParameterSet
DerivativeRequest
GradientOutput
ForwardMode
ReverseMode
replayable stochastic samples
loss or adjoint inputs
```

Not every value is differentiable.

Stable semantic parameter identities and schemas allow future derivative requests
without making automatic differentiation part of the foundational implementation.

## Neural and learned representations

Neural fields, Gaussian splats, learned materials, neural radiance caches, and learned
reconstruction enter through representations, protocols, methods, outputs, inputs, and
derived state.

They may require:

```text
model schema versions
training versus inference state
quantization and precision facts
large parameter resources
accuracy and confidence evidence
device-specific compilation
```

The architecture does not permanently classify every representation as mesh, surface,
volume, or SDF.

## XR and sensor relationship

RunenRender may consume coordinated view sets, nonuniform sampling density, foveated
inset regions, rolling shutter, lens distortion, and sensor-specific output requests.

Runenwerk/platform adapters own:

```text
tracking and gaze
late-latching timing
runtime swapchains
reprojection policy
display timing
platform session lifecycle
```

RunenRender receives immutable view, time, exposure, and sampling facts.

## Shader and program boundary

RunenRender owns renderer shader/kernel meaning, semantic variants, and method-specific
program families.

Runenwerk owns authoring source roots, module/package policy, compiler selection,
canonical artifact generation, source maps, watching, reload scheduling, and
last-known-good product policy.

RunenGPU owns canonical program admission, explicit interfaces, layouts, specialization,
runtime binding compatibility, backend realization, and cache compatibility.

Shader-language interfaces are implementation tools, not renderer semantic protocol
authority.

Variant selection must be bounded, inspectable, cacheable, and driven by semantic need.
Variant count, cold/warm compilation cost, cache pressure, and fallback behavior require
characterization.

## Trust, extension, and evolution

Extensible semantic families use:

```text
namespaced semantic identity
schema version
required and optional fields
compatibility rules
structured unsupported outcome
```

Initial extension guarantees are source-level Rust contracts, not a stable dynamic
library ABI.

Potential trust classes include:

```text
BuiltInTrusted
ApplicationTrusted
ValidatedThirdParty
UntrustedAuthoringInput
```

Third-party representations, materials, shader graphs, and methods require bounded
source, compiler, variant, resource, and execution budgets; validated interfaces; no
unrestricted host callbacks; no arbitrary backend access; and no unrelated global
resource access.

Runenwerk owns product trust policy.

## Determinism and reproducibility

`DeterminismPolicy` distinguishes:

```text
Structural
NumericalWithinTolerance
Statistical
BitwiseUnderConstrainedEnvironment
```

Reproducibility facts may include:

```text
random generator and revision
seed and stream allocation
sample index ranges
accumulation order
floating-point mode
method and protocol revisions
representation and input revisions
device/backend facts where permitted
```

Runenwerk owns persisted reproducibility bundles, privacy/redaction, artifact schemas,
and presentation.

## Scalability invariants

### Scene

```text
No mandatory deep copy per commit.
No mandatory full rebuild for a local change.
```

### Planning

```text
No mandatory all-object × all-method comparison.
Use indexed protocol, space, time, relationship, and view data.
```

### GPU work

```text
RunenGPU work-node count scales with algorithm stages,
not logical object count.
```

### Submission

```text
No per-object CPU submission requirement.
GPU culling, compaction, indirect execution, and generated work remain possible.
```

### Memory

```text
All scene-derived, method-derived, history, residency,
diagnostic, and output state is bounded or pressure-reporting.
```

### Detail

```text
Refinement is footprint- and error-driven.
Procedurally unbounded detail is never globally materialized.
```

### Views and outputs

```text
Compatible views and outputs may share preparation,
compiled representations, acceleration, and derived state.
```

### Diagnostics

```text
Diagnostics are bounded and aggregatable.
One failing population does not produce an unbounded message storm.
```

### Automatic selection

```text
Exact protocols, representations, methods, revisions,
seeds, and degradations are recorded.
```

The logical world may be finite or unbounded. Every render operates on a finite,
explicitly bounded working set.

## RunenGPU lowering

RunenRender lowers one admitted plan into a `RenderWorkSet` containing generic
RunenGPU work.

RunenRender may own render-specific:

- representation compilation and acceleration descriptions;
- visibility, material, medium, emitter, method, reconstruction, overlay, and output
  program meaning;
- render-specific resource use expressed through RunenGPU contracts;
- derived-state updates;
- output and merge work;
- typed imports from preceding GPU work.

RunenRender does not own:

- GPU context/device admission;
- generic resource identity or allocation;
- generic access/hazard validation;
- backend program/layout/pipeline realization;
- command submission and progress;
- surface lifecycle;
- device-loss classification;
- a second GPU lifetime or error model.

## Host, presentation, and recovery

Runenwerk owns windows, event loops, resize/DPI/visibility policy, presentation timing,
XR/platform sessions, product method/profile selection, artifact encoding, and recovery
decisions.

RunenGPU owns low-level surface operations, generations, and outcomes.

RunenRender owns render-output meaning, composition, and presentation intent.

On device loss, RunenRender reports affected admitted plans, realizations, sessions, and
derived state plus reconstruction facts. It does not decide whether the product retries,
recreates, degrades, pauses, exits, or asks the user for action.

## Diagnostics and observability

RunenRender exposes structured facts for:

```text
scene updates and reference validation
selected and rejected representations
protocol admission
method selection and revision
capability and residency degradation
space/time/footprint decisions
material, medium, emitter, and measurement admission
derived-state hits and invalidation
session and accumulation state
estimated and actual memory/work
variant and pipeline provenance
RunenGPU lowering provenance
full versus incremental preparation evidence
```

Diagnostics retain typed identities and provenance while displaying human vocabulary.

Diagnostics are bounded and aggregatable. RunenRender does not own product severity,
retention, privacy, persistence, or UI presentation.

## Cost estimation and admission

Before lowering, an admitted plan should expose estimates where evidence permits:

```text
retained and transient memory
expected work count
expected shader/program variants
required residency
output storage
required capabilities
```

Estimates may be approximate. They remain inspectable and versioned.

Policy may admit, admit with declared degradation, wait within a bound, select an
alternative, or reject for pressure.

## Founding proof

The first complete render-method proof is intentionally narrow:

```text
one analytic sphere
one analytic plane
one field/SDF surface representation
one diffuse material
one directional emitter
one view
one linear HDR output
compute current visibility and direct lighting
fullscreen render conversion
offscreen output
CPU reference probes
```

It proves:

```text
scene commit and cheap snapshot
change set
object and representation identity
representation offer
versioned surface and visibility protocols
narrow query results
render request
render method
semantic plan
execution admission
output specification
RunenGPU work lowering
```

It does not authorize foundational types named after SDF, direct lighting, preview, or
the first implementation.

Multi-bounce path tracing, regional/cellular transport, volumes, populations, temporal
history, denoising, deep output, differentiability, neural representations, distributed
execution, and XR remain later proofs against the same architecture.

## Conformance

Internal RunenRender proof requires:

1. renderer scene commits and snapshots without ECS, Runenwerk, WGPU, RunenSDF, or
   RunenUI types;
2. no mandatory deep copy per commit;
3. deterministic insert, replace, remove, relationship update, and producer retirement;
4. equivalent full and incremental semantic result;
5. explicit change sets and full-resynchronization fallback;
6. stable semantic object identity across representation replacement;
7. at least two independent producer families;
8. representation offers with exact protocol, coverage, accuracy, freshness, and
   residency facts;
9. narrow protocol and result tests independently;
10. explicit space, time, precision, and footprint contracts;
11. deterministic device-independent planning;
12. deterministic execution-environment admission;
13. RunenGPU lowering through public contracts only;
14. no direct WGPU dependency or private reach-through;
15. typed CPU and GPU-produced dynamic-input proof;
16. output semantics and accumulation rules;
17. derived-state dependency and invalidation proof;
18. bounded memory, pressure, diagnostics, and variants;
19. work-node count that scales with stages rather than objects;
20. multiview/multi-output sharing evidence;
21. one founding method and a later meaningfully distinct method through the same
    semantic boundaries;
22. reproducibility facts supplied without owning persistence policy;
23. Runenwerk adapters consume no private renderer internals;
24. no duplicate old render path after accepted cutover.

External proof additionally requires independent locked validation, public downstream
consumption, exact RunenGPU revision, provenance, operational conformance, and clean
Runenwerk cutover.

## Performance and scalability characterization

R8 must characterize:

```text
full versus incremental scene-update cost
small-change cost against total scene size
representation selection cost
protocol query count and divergence
work-node count against logical object count
CPU submission count
GPU culling/compaction/indirect behavior
resident and transient memory high-water marks
derived-state hit, miss, invalidation, and reconstruction
compatible multiview/multi-output sharing
variant count and cold/warm compilation
progressive-session convergence and cancellation
artifact and reproducibility evidence
comparison with a simpler direct renderer for the same proof
```

Performance evidence is diagnostic until a separately accepted controlled budget
exists. No private RunenGPU/WGPU bypass may improve framework measurements.

## Current decomposition problem

The current `engine/src/plugins/render` mixes:

```text
general GPU execution and WGPU ownership
render graph mechanics and image formation
native-window surfaces
ECS/host projection and frame policy
source-domain and product features
shader discovery and hot reload
diagnostics, captures, artifacts, residency, and pacing
```

Current paths are migration evidence. Moving or extracting the directory unchanged is
forbidden.

## Current-source revalidation gate

Before every R implementation slice:

- verify exact accepted `main`;
- repeat affected declarations and direct/transitive consumer census;
- inspect current scene, frame, resource, shader, pipeline, surface, capture,
  diagnostics, example, test, and benchmark paths;
- verify identity conversions and persisted uses;
- identify host/ECS/source reach-back;
- bind exact public, migration, deletion, proof, and guard scope;
- run canonical baseline validation;
- stop for a new ADR, package, dependency direction, stable format, compatibility path,
  unsafe backend escape, or premature later-phase authority.

The historical S0 inventory remains evidence, not permission to skip current-source
revalidation.

## Definition of done

RunenRender extraction is complete only when:

- one independently validated package exists in `dornglut/runen-render`;
- it depends on an exact RunenGPU revision and not WGPU;
- Runenwerk consumes only public semantic scene/request/input/output seams;
- RunenUI and RunenSDF remain independent;
- the founding method and a meaningfully distinct second method pass;
- multiple representation protocols pass without universal-interface pressure;
- incremental scene lifecycle, representation selection, dynamic inputs, output
  semantics, and derived-state invalidation pass;
- large-scene and bounded-work evidence passes;
- operational, performance, capture, recovery, and reproducibility facts pass;
- downstream and runtime evidence pass;
- every active consumer is migrated;
- exact provenance is recorded;
- original Runenwerk image-formation authority and temporary seams are deleted;
- no mirror, compatibility package, duplicate renderer, or private reach-through
  remains.

## Strategic reevaluation gates

Reconsider RunenRender if:

- a smaller renderer satisfies all accepted proofs;
- protocol or representation interfaces become universal, stringly, or runtime-heavy;
- snapshots require systematic deep copies;
- incremental changes require systematic full rebuilds;
- planning requires all-object × all-method scans;
- work-node or CPU-submission count scales directly with scene objects;
- optional methods cannot share coherent scene, request, input, and output semantics;
- backend-neutral contracts repeatedly leak backend details;
- measured cost materially exceeds simpler alternatives without reusable value;
- no meaningfully distinct second render method or representation family exists.

Reevaluation is explicit architecture work, not permission for a hidden bypass.
