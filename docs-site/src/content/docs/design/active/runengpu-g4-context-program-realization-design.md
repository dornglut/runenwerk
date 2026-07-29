---
title: RunenGPU G4 Context, Program, and WGPU Realization Design
description: Decision-complete architecture for headless context admission, program and interface contracts, generation-bound WGPU realization, cache compatibility, and renderer decontamination.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-g4-context-program-realization-investigation.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../../workspace/specs/pt-runengpu-g4a-context-admission.ron
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU G4 Context, Program, and WGPU Realization Design

## Status and authority

This design binds the G4 decision phase authorized by issue `#182` against exact
accepted base:

```text
6bbd341691a34763ef54c8ca059940cac8981265
```

The accepted G3 implementation merge is:

```text
39d6fe65a334502bdfba0b1a2ce3b365099fcf28
```

The intervening accepted commit is validation/workflow maintenance only. G4 Rust
implementation remains unauthorized until this planning authority is accepted and a
separate ordered implementation issue is active.

G4 is implemented as three independent, ordered slices:

```text
G4A context and adapter/device admission
 -> G4B program, interface, binding and pipeline contracts
 -> G4C WGPU realization, cache compatibility and cutover
```

The slices must not be collapsed into one issue or pull request.

## Mission

G4 establishes one future-transferable RunenGPU owner for:

- headless context and adapter/device admission;
- normalized backend, portability, feature, limit, format, and alignment facts;
- deterministic requirement admission and degradation reporting;
- program source, entry-point, interface, binding, specialization, and pipeline
  contracts;
- private WGPU resource, program, layout, bind-group, and pipeline realization;
- context/device-generation affinity and stale/foreign rejection;
- correctness-complete in-memory cache keys, rejection, and fallback facts;
- migration and deletion of renderer-owned GPU realization authority.

G4 does not encode or submit work. It does not own progress, completion, readback,
retirement, reusable surfaces, presentation, device-loss recovery, or image-formation
semantics.

## Ownership boundaries

### G4 owns

```text
context identity
current device generation identity
adapter/device admission
normalized backend and portability facts
requirement admission and degradation
program/source/entry-point/interface contracts
typed binding keys and declarations
bind-group and pipeline-layout descriptors
specialization schemas and values
compute/render pipeline descriptors
resource/program/layout/bind-group/pipeline realization
realization registries and generation affinity
cache compatibility and structured cache facts
private WGPU containment
```

### G5 owns later

```text
execution epochs
command encoding
uploads and updates
query-resolution execution
queue submission
native/web progress
pressure and bounded waits
completion and cancellation
mapping and asynchronous readback
runtime use-after-retirement checks
delayed backend destruction
shutdown with pending work
```

### G7 owns later

```text
host raw-window/display-handle admission
surface identity and generation
surface creation/configuration/acquisition
surface-image leases
presentation
surface and device loss classification
device replacement and reconstruction reports
```

### RunenRender owns later

```text
prepared render scenes
views and logical render targets
materials and media
emitters and environments
visibility and provider interaction
lighting and transport
reconstruction and render history
color, overlay and image-formation semantics
lowering semantic render plans into RunenGPU work
```

### Runenwerk retains

```text
window and event-loop policy
application scheduling
ECS/domain extraction
shader source discovery, paths, watching and reload scheduling
last-known-good and product fallback policy
cross-framework composition and compatibility manifests
product recovery decisions
diagnostics presentation
persisted capture/reproducibility/artifact policy
```

## Architectural shape

```text
Runenwerk source and host policy
    -> RunenRender or non-render consumer descriptors
        -> RunenGPU G2/G3 logical resources and prepared work
        -> RunenGPU G4 admitted context/program/realization authority
            -> private WGPU backend objects
        -> RunenGPU G5 execution
```

Logical identity, program identity, backend realization, and execution identity remain
separate. A logical handle is not a WGPU object. A program descriptor is not a shader
module. A pipeline descriptor is not a pipeline. A prepared graph is not a submission.

# G4A — context and adapter/device admission

## Public context construction

The reusable entrypoint is asynchronous and headless-first:

```text
GpuContext::request(GpuContextDescriptor) -> Future<Result<GpuContext, GpuContextRequestError>>
```

The RunenGPU library does not call `pollster::block_on`, create a Winit window, or
require a surface. Runenwerk may provide a synchronous terminal wrapper for a concrete
binary or existing host integration. That wrapper is outside the future-transferable
RunenGPU API.

`GpuContextDescriptor` contains only reusable backend-neutral policy:

- diagnostic label and provenance, excluded from semantic equality;
- adapter power preference: `Default`, `LowPower`, or `HighPerformance`;
- software/fallback-adapter policy: `Forbid`, `Allow`, or `Require`;
- backend-family allowlist expressed with normalized backend families;
- portability policy;
- merged G2/G3 requirements.

It contains no Winit window, raw host/window/display handle, WGPU type, surface,
filesystem path, renderer type, application type, executor, callback, or persistence
policy.

Temporary current-host compatibility is supplied separately through one crate-private
Runenwerk request envelope. It is not a field, builder option, feature, or variant of
the public `GpuContextDescriptor`.

## Async and terminal boundary

The backend may await instance, adapter, and device requests. The future:

- is cancellation-safe before a context is returned;
- publishes no partially admitted public context;
- performs no callback into consumer code while holding internal locks;
- does not promise a particular executor or thread beyond backend platform
  requirements;
- returns structured terminal success or failure.

Native blocking belongs to a host adapter. Web integration uses the same async public
contract. G5 later binds progress after work acceptance; G4A only binds context-request
completion.

## Context identity and generation

```text
GpuContextId
GpuDeviceGeneration
GpuContextAffinity { context_id, device_generation }
```

Rules:

1. `GpuContextId` is opaque, nonzero, process-local, and unique for every successfully
   admitted context instance.
2. `GpuDeviceGeneration` is opaque, nonzero, and starts at `1` for a newly admitted
   context.
3. G4A does not replace a live device. G7 increments generation when device replacement
   is later implemented.
4. Every G4C backend realization stores complete `GpuContextAffinity`.
5. Logical G2/G3 descriptors remain backend-neutral and do not acquire affinity merely
   by existing.
6. Public realized handles are `Clone`, non-`Copy`, and expose affinity only through
   typed inspection.
7. Context IDs, generation values, backend handles, and pointer-like values are not
   stable persistence or wire authority.
8. Equality of affinities is exact pair equality. Labels, provenance, and diagnostics
   do not participate.

## Normalized backend and adapter vocabulary

`GpuBackendFamily` contains only backend/API families the selected WGPU version reports
and RunenGPU can normalize without guessing:

```text
Vulkan
Metal
Direct3D12
OpenGl
BrowserWebGpu
UnknownBackend
```

Software execution is not a backend family. A software adapter may use Vulkan,
Direct3D12, OpenGL, or another backend family. Software/fallback status is recorded as
an independent normalized adapter fact and evaluated by explicit policy.

`UnknownBackend` preserves an unsupported or unmapped result; it is never folded into
another family. Internal WGPU enums do not cross the public boundary.

`GpuAdapterClass` distinguishes:

```text
Discrete
Integrated
Virtual
Cpu
Other
Unknown
```

The class and independent software/fallback fact remain distinct from backend family.
A CPU class is strong software evidence; unavailable information remains `Unknown`
rather than guessed.

`GpuPortabilityClass` distinguishes:

```text
PortableBaseline
PortableWithDeclaredExtensions
BackendSpecialized
Unsupported
```

Meaning:

- `PortableBaseline`: admitted requirements fit the versioned RunenGPU baseline;
- `PortableWithDeclaredExtensions`: all mandatory requirements are admitted and every
  extension is explicitly represented in normalized facts;
- `BackendSpecialized`: admission intentionally uses a contained backend-specific fact
  requested by the caller;
- `Unsupported`: no context may be admitted for the request.

Portability class is a result of admitted requirements, not a marketing claim about an
adapter.

## Normalized adapter and device facts

`GpuAdapterFacts` records:

- normalized backend family and adapter class;
- independent software/fallback status;
- sanitized adapter name for diagnostics only;
- vendor/device identifiers when supplied, as optional diagnostic compatibility facts
  rather than public identity promises;
- driver name/version when supplied;
- normalized supported features;
- normalized limits;
- normalized format capabilities;
- normalized alignment constraints;
- WGPU version and internal backend compatibility revision for cache diagnostics.

`GpuAdmittedDeviceFacts` records exact enabled features and effective limits, not merely
adapter support. Supported and enabled facts remain distinct.

Initial normalized alignments include every current operation-relevant constraint that
can reject G2/G3 work before encoding, including uniform/storage binding offsets, copy
row pitch, copy-buffer offsets, query-resolve destination offsets, and texture block
constraints. An absent fact is distinct from zero or default.

## Requirement model

G2/G3 normalized requirements remain the semantic input. G4A adds admission policy per
requirement:

```text
Mandatory
Preferred { degradation_key }
```

Mandatory means context admission fails if unsupported. Preferred means admission may
continue only if the caller explicitly supplied a degradation key and the requirement
has a defined degradation result. No requirement is silently downgraded.

The request is normalized before backend selection:

1. validate every requirement value;
2. merge equal requirements;
3. combine minimum limits by maximum requested minimum;
4. combine maximum constraints by minimum permitted maximum where applicable;
5. reject contradictions before backend access;
6. preserve contributing provenance;
7. sort by typed requirement identity for deterministic diagnostics.

## Deterministic admission algorithm

For one normalized candidate fact set, admission is a pure deterministic function:

```text
admit(candidate_facts, normalized_request)
    -> Admitted(candidate_report)
     | Rejected(rejection_report)
```

Algorithm:

1. Reject a disallowed backend family, adapter class, or software/fallback status.
2. Evaluate every mandatory feature, limit, format, and alignment requirement in typed
   identity order.
3. Evaluate preferred requirements in the same order.
4. Record exact requested, supported, granted, and degraded values.
5. Reject missing mandatory facts, contradictory limits, or unsatisfied
   format/alignment relations.
6. Require an explicit degradation key for each unsupported preferred requirement.
7. Enable only features required by admitted requirements plus backend features WGPU
   itself requires.
8. Request the least limits satisfying the admitted request, bounded by adapter support.
9. Derive portability class from the admitted feature, limit, and format set.
10. Emit a canonical report ordered by typed requirement identity.

Candidate selection is separate from per-candidate admission:

1. discard rejected candidates;
2. rank accepted candidates by fallback policy, power preference, portability class,
   adapter class, and backend-family preference;
3. use optional normalized vendor/device facts and sanitized adapter name only as final
   deterministic tie-break inputs within the observed candidate set;
4. retain complete candidate reports and the selected rank reason.

Adapter names and driver strings are not persistence or cross-machine identity. If two
accepted candidates remain indistinguishable after every permitted normalized
tie-break, admission returns `AmbiguousAdapterSelection` and requires a stronger caller
filter. Backend enumeration order is never authority.

On platforms where WGPU exposes only a backend-selected candidate, the same pure
admission algorithm evaluates it. The report states `BackendSelectedCandidate`;
RunenGPU does not claim deterministic hardware selection it did not perform.

## Admission diagnostics

Structured diagnostics include:

```text
GpuContextAdmissionReport
GpuCandidateAdmissionReport
GpuRequirementAdmission
GpuDegradationRecord
GpuContextRequestError
GpuBackendRequestFailure
```

Required failure categories include:

- no candidate;
- ambiguous adapter selection;
- backend family forbidden;
- software/fallback policy violation;
- mandatory feature missing;
- limit below required minimum or above permitted maximum;
- unsupported format role;
- alignment incompatibility;
- contradictory request;
- backend adapter request failure;
- backend device request failure;
- temporary host-compatibility failure.

Backend error strings may be attached as bounded diagnostic detail but are not the
programmatic category.

## Raw WGPU containment

WGPU types live only in private backend modules, except for one explicitly temporary
crate-private G5 execution bridge defined under G4C. The future-transferable public API
must not expose:

```text
wgpu::Instance
wgpu::Adapter
wgpu::Device
wgpu::Queue
wgpu::Features
wgpu::Limits
wgpu::TextureFormat
wgpu::Surface or surface configuration
wgpu resource, layout, bind-group, shader, or pipeline objects
```

A private `WgpuContextState` may own instance, adapter, device, and queue. Public
methods return normalized values, structured results, or opaque generation-bound
handles. No public field, `Deref`, `AsRef<Device>`, broad callback receiving raw WGPU,
or stable escape hatch is accepted.

## Temporary host compatibility

The current host needs surface-compatible adapter selection. G4A may retain exactly
one Runenwerk-owned crate-private request path that pairs a public
`GpuContextDescriptor` with opaque temporary compatibility evidence for private WGPU
adapter selection.

Rules:

- no temporary compatibility value appears in the public descriptor or standalone API;
- it does not transfer surface ownership to RunenGPU G4;
- it does not configure, acquire, resize, present, or publicly identify a surface;
- headless admission is independently tested and remains the default architecture;
- the compatibility path has explicit G7 deletion ownership;
- source guards reject a second compatibility path.

# G4B — program, interface, binding, and pipeline contracts

## Program and source identity

```text
GpuProgramSourceKey
GpuProgramSourceRevision
GpuProgramDescriptor
GpuProgramEntryPointDescriptor
GpuProgramInterfaceDescriptor
```

`GpuProgramSourceKey` is an opaque validated semantic key:

- constructed from a nonempty bounded UTF-8 value or owner-scoped typed allocator;
- contains no filesystem-path semantics;
- equality, ordering, and hashing use the normalized key value;
- labels and provenance remain separate.

`GpuProgramSourceRevision` is nonzero and meaningful only within its source owner. A
revision change invalidates module and pipeline realization keys. It is not globally
ordered across keys or processes.

`GpuProgramSource` initially supports only:

```text
Wgsl { source: Arc<str> }
```

No SPIR-V, GLSL, HLSL, MSL, DXIL, native module handle, filesystem reference, or
persisted source schema is accepted without separate evidence and authorization.

`GpuProgramDescriptor` contains source key, revision, source text, declared entry
points, explicit interface, merged requirements, and provenance. Source text
participates in full semantic equality. A deterministic content digest may accelerate
hashing and diagnostics but never substitutes for full source-byte comparison. A
caller cannot reuse one key/revision with different source bytes; the registry returns
`SourceRevisionConflict` before realization or cache reuse.

Source keys, revisions, text, and digests are runtime/cache inputs, not a stable source,
wire, persistence, capture, or replay format.

## Entry points

Entry-point stages are typed:

```text
Compute
Vertex
Fragment
```

Each descriptor contains a validated source-level name, stage, and interface reference.
A compute pipeline names exactly one compute entry point. The initial render pipeline
uses one admitted program source containing one vertex and zero or one fragment entry
point. Separate vertex/fragment source modules remain deferred until a current consumer
proves the need.

Stage mismatches, duplicate same-stage names, missing declared entries, and pipeline
references not declared by the program are rejected. Entry-point names are semantic
source identifiers and participate in equality and hashing.

## Typed binding keys

```text
GpuBindingKey { group, binding }
```

Both components are checked `u32` values. Descriptor construction validates structural
bounds and duplicates without a live context; realization validates admitted device
limits. Ordering is lexicographic by `(group, binding)`. This pair is the only binding
identity.

The following cannot be binding authority:

```text
String labels
filesystem paths
TypeId or Rust type names
GpuWorkResourceId
RenderFlowId, RenderPassId, or RenderFeatureId
vector position
naked u64 signature hashes
raw WGPU layout entries
```

Labels, Rust type names, and source spans may accompany diagnostics only.

## Binding declarations

`GpuBindingDeclaration` contains typed key, stage visibility, one typed binding kind,
optional nonzero binding-array count, and diagnostic provenance excluded from semantic
equality.

Initial kinds:

```text
UniformBuffer { dynamic_offset, minimum_size }
StorageBuffer { access: ReadOnly | ReadWrite, dynamic_offset, minimum_size }
SampledTexture { sample_class, view_dimension, multisampled }
StorageTexture { access, format, view_dimension }
Sampler { Filtering | NonFiltering | Comparison }
```

Raw WGPU enums do not cross the boundary. Acceleration structures, external textures,
video, ray tracing, bindless/unbounded arrays, and backend-native bindings remain
deferred.

Compatibility checks include:

- exact binding key;
- resource kind and usage;
- stage visibility;
- minimum buffer size and selected range;
- dynamic-offset policy and admitted alignment;
- texture sample class, view dimension, multisample state, and format role;
- storage texture access and exact normalized format;
- sampler class;
- array cardinality;
- resource context/generation affinity at realization time.

## Program interface

`GpuProgramInterfaceDescriptor` owns ordered binding declarations plus the initial
stage input/output contracts required by accepted compute/render consumers.

Vertex inputs include typed shader location, normalized scalar/vector format, byte
offset, stride, step mode, and buffer slot. Fragment outputs include typed color
location and normalized format class. The explicit descriptor is authoritative.
Reflection may be internal validation evidence only when complete and trustworthy; it
cannot silently infer or replace the public contract. Source/backend disagreement is a
structured `ProgramInterfaceMismatch`.

## Bind-group and pipeline layouts

`GpuBindGroupLayoutDescriptor` owns exactly one group and declarations sorted by key. It
rejects duplicate bindings and declarations naming another group.

`GpuPipelineLayoutDescriptor` owns ordered unique group layouts. Initial G4 has no push
constants because no accepted current consumer proves them.

Semantic equality and hashing include every correctness field and exclude labels,
provenance, source spans, backend addresses, normalized-away insertion order, and
diagnostic-only adapter strings.

## Specialization

Initial specialization types are `Bool`, `U32`, `I32`, and finite `F32`.

- keys are validated source-level identifiers;
- schemas have unique keys, exact types, optional defaults, and requirement
  implications;
- value sets reject unknown or duplicate keys, missing required values, and type
  mismatches, then normalize to schema-key order;
- negative zero is canonicalized;
- NaN and infinity are rejected;
- specialization participates in realization keys only where the admitted WGPU/WGSL
  path consumes it;
- unsupported override behavior is rejected explicitly.

## Pipeline descriptors

`GpuComputePipelineDescriptor` contains complete program/source identity, compute entry
point, program interface, pipeline layout, normalized specialization, and capability
requirements. It contains no renderer flow, pass, feature, material, view, or surface
identity.

`GpuRenderPipelineDescriptor` contains complete program/source identity, vertex and
optional fragment entries, program interface, pipeline layout, specialization,
vertex-buffer layouts, primitive state, depth/stencil state, multisample state, ordered
color-target states, and capability requirements.

The render descriptor contains no material meaning, lighting model, view ID,
`RenderFlowId`, `RenderPassId`, `RenderFeatureId`, target alias, product quality policy,
shader path, or surface handle. Initial state is limited to accepted current WGPU
consumer needs; no universal graphics-state abstraction is invented.

## Deterministic descriptor identity

All semantic descriptors:

- have private fields and fallible constructors;
- normalize unordered collections into typed-key order;
- implement semantic equality, ordering, and hashing where fields permit;
- canonicalize floating-point values before storage;
- hash typed discriminants and complete field values, never `Debug` text;
- never expose a naked hash as authoritative identity;
- compare full values after hashing;
- exclude label/provenance only when those fields cannot affect correctness.

Hash collision cannot authorize cache reuse.

## Existing parameter helper disposition

`GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and current
derives remain transitional Runenwerk/render byte-preparation helpers during G4. They
do not prove WGSL structure identity, binding key/visibility, minimum binding size,
storage ABI, nested/array/runtime-array layout, matrix policy, package independence, or
interface compatibility.

G4B adds no derive, moves no current macro into RunenGPU, and creates no macro package.
Binding compatibility is validated independently of whichever helper prepared bytes.
A later separately authorized decision may replace or retire these helpers after real
ABI requirements are proven.

## Program/interface diagnostics and proof

Required structured categories include invalid source key/revision, source revision
conflict, unsupported source kind, WGSL parse/validation failure, missing or wrong-stage
entry point, duplicate/invalid binding, interface mismatch, runtime binding mismatch,
specialization mismatch, invalid pipeline descriptor, and backend realization failure.
Backend compiler strings are bounded detail, not public categories.

Required compile-pass proof:

- one WGSL compute program with typed storage read/write;
- one WGSL render program with typed vertex input, uniform, and color output;
- equal descriptors from different insertion orders compare/hash equally;
- specialization order normalization;
- current prepared bytes bind only after explicit interface validation.

Required compile-fail or construction rejection:

- public raw WGPU descriptor fields;
- string, `TypeId`, resource ID, pass ID, or naked hash as binding key;
- duplicate binding;
- stage mismatch;
- incompatible resource, texture, sampler, format, or array shape;
- invalid specialization;
- wrong-stage pipeline entry;
- reused key/revision with different source;
- field construction bypassing validation;
- current derives treated as sufficient interface proof.

# G4C — WGPU realization, cache compatibility, and cutover

## Realization model

Logical descriptors are admitted into one `GpuContext` and produce opaque,
generation-bound handles for buffers, textures, views, samplers, query sets, programs,
bind-group layouts, pipeline layouts, bind groups, compute pipelines, and render
pipelines.

Exact file/type names remain implementation details. Public handles are `Clone`,
non-`Copy`, expose typed logical identity plus affinity for inspection, and contain no
raw WGPU access.

## Realization timing by kind

| Kind | Policy |
|---|---|
| persistent/imported buffer or texture | explicit before graph execution |
| transient graph resource | explicit bounded graph-realization pass after G3 preparation |
| sampler | explicit or complete-descriptor cache-backed lazy |
| texture view | lazy only within an admitted parent texture |
| query set | explicit |
| program module | explicit admission or first-pipeline lazy realization |
| bind-group/pipeline layout | complete-descriptor cache-backed lazy |
| bind group | explicit from typed layout and realized values |
| compute/render pipeline | explicit or cache-backed lazy before execution |

Backend or imported-resource failure must precede G5 encoding. G5 may require all
realizations for a prepared graph to resolve before encoding, but cannot create
undeclared resources or bypass compatibility.

## Affinity and rejection

Every request validates in this order:

1. logical descriptor validity;
2. context identity of every realized input;
3. current device generation;
4. resource/program/interface/pipeline compatibility;
5. admitted feature, limit, format, and alignment facts;
6. full cache-key lookup and equality;
7. WGPU creation when no compatible hit exists.

Structured failures distinguish foreign context, stale generation, unknown logical
resource, descriptor change for identity, resource-kind mismatch, interface mismatch,
binding mismatch, unadmitted requirement, format/alignment incompatibility, cache
rejection, and backend realization failure.

Foreign or stale values are rejected before WGPU access. Runtime use-after-retirement
and delayed destruction remain G5-owned.

## Realization registries

The context owns separate private registries for resources, modules, bind-group layouts,
pipeline layouts, bind groups, compute pipelines, render pipelines, and cache
diagnostics.

Invariants:

- one context/device generation per registry instance;
- full typed key plus full equality;
- no renderer, ECS, Winit, domain, application, product, `TypeId`, or string-only
  lookup;
- no public WGPU object;
- complete normalized source descriptor retained with every entry;
- transactional publication after backend success;
- equal duplicate requests return the same logical record;
- changed descriptor for one logical identity is rejected;
- labels do not split equal realizations;
- no consumer callback under locks.

## Realization and cache keys

Resource keys include affinity, logical identity, complete normalized descriptor, and
import/source generation. Texture-view keys additionally include parent identity and
complete normalized view range. Sampler/query keys include complete semantics.

Program/module keys include affinity, source kind/key/revision, full admitted source
record, declared entries, interface, admitted requirements, and backend compatibility
revision. A digest accelerates hashing only; registry source conflict validation and
full source equality remain authoritative.

Layout keys include affinity, complete layout, and relevant admitted limits/features.
Bind-group keys include affinity, complete layout, ordered typed values, full realized
resource identities/ranges, and resource/import generations.

Pipeline keys include affinity, complete program/module and entry identities, interface,
layout, complete compute/render descriptor, specialization, relevant
features/limits/formats/alignments, and backend compatibility revision.

Renderer semantic identities may affect the lowered descriptor but never appear merely
as opaque renderer IDs, strings, paths, labels, or naked hashes.

## Cache scope and compatibility

Initial caches are in-memory, derived, discardable, reconstructable, and scoped to one
context/device generation. No stable persisted pipeline cache, source schema, capture,
ABI, or wire format is authorized.

Compatibility includes:

```text
internal cache and descriptor schema revisions
WGPU version and backend compatibility revision
backend family, adapter class, software/fallback fact
available adapter and relevant driver facts
context identity and device generation
source kind, key, revision, full admitted source identity, and digest
program interface and specialization
complete resource, layout, and pipeline descriptors
exact enabled features and effective limits
relevant formats and alignments
```

Behavior:

```text
Hit                 reuse
Miss                ordinary realization
Rejected            structured incompatibility, then ordinary realization
RejectedCorrupt     structured corruption, then ordinary realization
RealizationFailed   no entry published
```

A cache hit changes cost, never semantics. Full equality follows hashing. Stats
separate hits, misses, rejections, corruption, and creation failures.

## Private WGPU ownership and temporary G5 execution bridge

After G4C, only private RunenGPU backend modules own reusable WGPU resources, modules,
layouts, bind groups, pipelines, instance, adapter, device, and queue.

The current renderer still owns G5 encoding until G5. G4C therefore permits exactly one
named crate-private scoped execution bridge:

- it lives at the private RunenGPU backend boundary;
- only the existing bounded render execution adapter may call it;
- it lends borrowed WGPU references only while encoding already accepted G5 payload;
- borrowed values cannot be stored, cloned into ownership, cached, returned, or used
  as lookup authority;
- it accepts only opaque realizations already validated for context, generation, kind,
  program, interface, layout, and pipeline compatibility;
- it owns no operation, access, hazard, initialization, requirement, program,
  interface, pipeline, cache, or realization truth;
- guards permit only this path;
- G5 deletes it when encoding/submission moves into RunenGPU.

This is a migration terminal, not stable raw-WGPU authority or a general escape hatch.
Current render files may otherwise retain WGPU command/pass encoders, surface textures,
and presentation only where explicitly G5/G7-owned.

## Migration order

1. add context/generation-bound RunenGPU realization registries;
2. realize G2 logical resources through the context;
3. realize G4B programs and layouts through the context;
4. realize typed bind groups and pipelines through the context;
5. migrate renderer setup, targets, materials, UI passes, flow-runtime caches, apps,
   examples, tests, and benchmarks;
6. replace broad raw backend access with the single named scoped G5 bridge;
7. remove renderer keys substituting paths, pass/feature IDs, or naked hashes for full
   G4 descriptors;
8. remove renderer-owned resource/program/layout/bind-group/pipeline registries;
9. remove synthetic logical-handle construction and the `RenderFlowId` owner bridge;
10. remove G4-owned program/interface/layout/pipeline/cache/realization truth from the
    G3 sidecar;
11. add source, dependency, bridge, sidecar, migration, and deletion guards;
12. prove every current consumer migrated and every replaced owner deleted.

No compatibility alias, forwarding module, duplicate cache, second bridge, or old/new
realization path remains.

## Exact deletion and retention split

Delete in G4C:

```text
renderer-owned reusable WGPU resource allocator and registries
renderer-owned module/layout/bind-group/pipeline caches
string-only PipelineKey as GPU correctness authority
FlowPassPipelineKey/FlowPassBindGroupKey as GPU correctness authority
backend aliases forwarding renderer cache authority
synthetic G2 handle construction outside RunenGPU
RenderFlowId-derived GpuWorkResourceId owner bridge
G4-owned sidecar fields
public or broad WgpuCtx backend-object reach-through
```

Retain until G5:

```text
prepared G3 graph
residual operation-specific execution payload
command/pass encoding
uploads and query-resolution execution
queue submission and polling
completion, readback, cancellation, retirement, shutdown
the single named scoped execution bridge
```

Retain until G7:

```text
Runenwerk window-to-surface registry and Winit integration
temporary compatible-surface admission path
surface configuration, acquisition, and presentation
surface/device generation and loss gaps
```

A current render file may remain when it still owns unrelated render semantics or G5/G7
execution evidence, but replaced G4 types, fields, functions, and re-exports are
deleted.

## Sidecar split

After G4B, no source, entry-point, interface, binding, layout, specialization, or
pipeline descriptor truth remains in the sidecar. After G4C, no backend resource,
module, layout, bind group, pipeline, cache, generation, or realization truth remains.
Only G5-owned execution payload keyed by prepared-node identity may remain.

The temporary execution bridge is not sidecar truth; it only resolves already validated
opaque realizations for scoped current encoding. G5 deletes both the residual payload
and bridge.

## RunenRender separation

G4 is GPU/backend decontamination and substrate work. It does not define materials,
providers, lighting, visibility, transport, reconstruction, history, overlays, views,
targets, color policy, image quality, or presentation meaning.

The current render tree is migration evidence and is not moved, renamed, wrapped, or
extracted wholesale. Files change only to remove GPU context, program/interface,
realization, or cache authority, or to consume the new substrate. Renderer semantic
code remains until independently accepted R-phase work.

RX is a later mechanical transfer/cutover after R-phase proof. It is not where renderer
architecture is invented.

# Proof and conformance

## Deterministic G4A

- context IDs unique and generations nonzero;
- exact foreign/stale rejection;
- order-independent requirement normalization;
- contradictions rejected before backend access;
- synthetic deterministic admission, degradation, ranking, and ambiguity;
- no opportunistic features;
- exact portability derivation;
- exhaustive known backend/adapter/software/limit/format/alignment mapping and explicit
  unknowns;
- no public temporary host compatibility;
- public/source dependency guards.

## Environment-dependent G4A

- headless native adapter/device request where available;
- explicit no-adapter/unsupported outcome;
- enabled facts match the admitted report;
- optional current-host compatibility smoke;
- WebGPU coverage where supported.

## Deterministic G4B

- source key/revision/full-content consistency;
- descriptor validation and full equality/hash;
- typed binding uniqueness and compatibility;
- entry-stage/interface validation;
- specialization normalization;
- compute/render descriptor validation;
- compile-pass/fail contracts;
- no derive or `TypeId` interface authority.

## Environment-dependent G4B/G4C

- WGSL compiler/module diagnostics;
- one compute and one render pipeline;
- representative resource, view, sampler, query, layout, and bind-group realization;
- real format/alignment rejection;
- cache hit/miss/rejection/fallback;
- context/generation isolation;
- one representative compute and render encoding through the temporary G5 bridge.

## Migration and deletion guards

Guards reject:

- public raw `Device`, `Queue`, or WGPU re-exports;
- any raw-WGPU reach-through except the single named scoped G5 bridge;
- a public or second host-compatibility path;
- renderer-owned reusable module/layout/bind-group/pipeline caches;
- `RenderFlowId`-derived GPU resource ownership;
- synthetic G2 construction outside RunenGPU;
- G4-owned sidecar truth;
- string/path/`TypeId`/pass/feature/naked-hash binding or cache authority;
- compatibility aliases and forwarding modules;
- incomplete consumer or deletion inventory.

Source-string guards supplement typed behavior tests; they do not replace them.

## Validation

Each implementation slice requires focused checks plus, at its exact reviewed head:

```text
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Repository-owned exact-head Actions are independent merge evidence. No later slice may
begin before its predecessor is accepted.

# Implementation ergonomics

Directional ordinary use:

```rust
let context = GpuContext::request(context_descriptor).await?;
let program = context.admit_program(program_descriptor)?;
let pipeline = context.realize_compute_pipeline(compute_pipeline_descriptor)?;
let input = context.realize_buffer(input_descriptor)?;
let output = context.realize_buffer(output_descriptor)?;
```

Exact method names remain implementation choices. The contract is one context owner,
explicit admission, typed descriptors, structured failures, no public raw-WGPU
reach-through, and no duplicate realization path.

RunenRender and Runenwerk may provide higher-level semantic authoring, but they lower
into the same G4B/G4C authority.

# Stop conditions

Implementation stops for owner decision if it requires:

- a new dependency or package;
- a stable persisted cache, source, ABI, capture, replay, or wire format;
- public raw-WGPU access or a second raw-WGPU bridge;
- public host compatibility or reusable G7 surface authority;
- unsafe code outside accepted policy;
- a speculative second backend abstraction;
- G5 progress, completion, readback, retirement, or shutdown beyond preserving current
  encoding through the one named bridge;
- RunenRender semantic implementation;
- a compatibility facade, forwarding package, or duplicate cache;
- an ADR changing accepted dependency direction or repository ownership.

Backend API details, adapter availability, and compiler diagnostics are expected
environment facts when they fit this design.

# Decision summary

G4A creates an async, headless, normalized, generation-aware context admission boundary
with no public host/surface compatibility. G4B creates explicit typed WGSL-first
program, interface, binding, specialization, and pipeline contracts with full semantic
identity. G4C realizes those values through private WGPU registries, rejects stale and
foreign generations, uses correctness-complete derived caches, migrates every consumer,
and deletes renderer-owned GPU realization authority. One narrow crate-private G5
execution bridge preserves current encoding during the ordered cutover and is deleted
by G5.

G5 remains execution and lifecycle. G7 remains surfaces and device loss. RunenRender
remains semantic image formation. No package extraction or Rust implementation is
authorized by this planning document alone.