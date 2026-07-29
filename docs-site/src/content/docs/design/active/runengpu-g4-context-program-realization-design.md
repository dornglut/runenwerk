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
- normalized backend, portability, feature, limit, format and alignment facts;
- deterministic requirement admission and degradation reporting;
- program source, entry-point, interface, binding, specialization and pipeline
  contracts;
- private WGPU resource, program, layout, bind-group and pipeline realization;
- context/device-generation affinity and stale/foreign rejection;
- correctness-complete in-memory cache keys, rejection and fallback facts;
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

`GpuContextDescriptor` contains only backend-neutral policy:

- diagnostic label and provenance, excluded from semantic equality;
- adapter power preference: `Default`, `LowPower`, or `HighPerformance`;
- software/fallback-adapter policy: `Forbid`, `Allow`, or `Require`;
- backend-family allowlist expressed with normalized backend families;
- portability policy;
- merged G2/G3 requirements;
- optional temporary host-compatibility token used only by the Runenwerk adapter while
  the current surface-coupled host is migrated.

It does not contain a Winit window, raw WGPU type, filesystem path, renderer type,
application type, executor, callback, or persistence policy.

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

## Normalized backend vocabulary

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
an independent normalized adapter fact and evaluated by the explicit fallback policy.

`UnknownBackend` preserves an unsupported or unmapped result; it is never folded into
another family. Internal WGPU backend enum values do not cross the public boundary.

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
A CPU class is strong software evidence; an unavailable fact remains `Unknown` rather
than guessed.

`GpuPortabilityClass` distinguishes:

```text
PortableBaseline
PortableWithDeclaredExtensions
BackendSpecialized
Unsupported
```

Meaning:

- `PortableBaseline`: all admitted requirements fit the versioned RunenGPU baseline;
- `PortableWithDeclaredExtensions`: all mandatory requirements are admitted and every
  extension is explicitly represented in normalized facts;
- `BackendSpecialized`: admission intentionally uses a contained backend-specific fact
  requested by the caller; it is not silently treated as portable;
- `Unsupported`: no admitted context may be constructed for the request.

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

`GpuAdmittedDeviceFacts` records the exact enabled feature set and effective limits,
not merely adapter support. Supported and enabled facts remain distinct.

Initial normalized alignments include every current operation-relevant constraint that
can reject G2/G3 work before encoding, including uniform/storage binding offsets, copy
row pitch, copy-buffer offsets, query-resolve destination offsets, and texture block
constraints. A fact is absent only when it is irrelevant or unavailable; absent is
distinct from zero or default.

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

The merged request is normalized before backend selection:

1. validate each requirement value;
2. merge equal requirements;
3. combine minimum limits by maximum requested minimum;
4. combine maximum constraints by minimum permitted maximum where applicable;
5. reject contradictions before backend access;
6. preserve all contributing provenance;
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
5. Reject if any mandatory requirement is missing, any limit is contradictory, or a
   format/alignment relation cannot be satisfied.
6. For each unsupported preferred requirement, require its declared degradation key
   and emit one typed degradation record.
7. Enable only features required by accepted mandatory/preferred requirements plus
   backend features WGPU itself requires. Do not opportunistically enable unrelated
   features.
8. Request the least limits that satisfy the admitted request, bounded by adapter
   support. Do not request broad defaults as hidden policy.
9. Derive portability class from the admitted feature, limit, and format set.
10. Produce a canonical report ordered by typed requirement identity.

Candidate selection is deterministic over the candidate facts the backend can expose:

1. discard rejected candidates;
2. rank explicit fallback requirement, power preference, portability class, adapter
   class, and backend-family preference in that order;
3. use normalized backend family, optional vendor/device facts, and sanitized adapter
   name only as final deterministic tie-break inputs within the observed candidate set;
4. retain the full candidate reports and selected rank reason.

Adapter names and driver strings are not persistence or cross-machine identity. Their
use as a final in-process tie-break changes no compatibility claim.

On platforms where WGPU exposes only a backend-selected adapter rather than an
enumerable candidate set, the same pure admission algorithm evaluates that candidate.
The report states `BackendSelectedCandidate`; RunenGPU does not claim deterministic
hardware selection it did not perform.

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

Every record carries typed facts, human label/provenance where relevant, and corrective
action. Backend error strings may be retained as diagnostic detail but are not the
programmatic category.

Required failure categories include:

- no candidate;
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

A private `WgpuContextState` may own instance, adapter, device, and queue. It is reached
through `GpuContext` methods that return normalized values or opaque generation-bound
handles. No public field, `Deref`, `AsRef<Device>`, broad callback receiving raw WGPU,
or stable escape hatch is accepted.

# G4B — program, interface, binding and pipeline contracts

## Program/source identity

```text
GpuProgramSourceKey
GpuProgramSourceRevision
GpuProgramDescriptor
GpuProgramEntryPointDescriptor
GpuProgramInterfaceDescriptor
```

`GpuProgramSourceKey` is an opaque validated semantic key:

- constructed from a nonempty bounded UTF-8 value or an owner-scoped typed allocator;
- contains no filesystem-path semantics;
- equality, ordering, and hashing use the normalized key value;
- labels and provenance are separate.

`GpuProgramSourceRevision` is nonzero and monotonically meaningful only within its
source owner. A revision change invalidates module and pipeline realization keys. It is
not assumed globally ordered across keys or processes.

`GpuProgramSource` initially supports:

```text
Wgsl { source: Arc<str> }
```

No SPIR-V, GLSL, HLSL, MSL, DXIL, native module handle, filesystem reference, or
persisted source schema is accepted without a concrete consumer and separate decision.
WGSL bytes are runtime source input. This design does not define a stable source-file
format.

`GpuProgramDescriptor` contains source key, revision, source kind/value, declared entry
points, explicit interface, merged requirements, and provenance. Source text
participates in realization correctness through a deterministic content digest in
addition to key and revision. A caller cannot reuse a revision with different source
bytes without a structured `SourceRevisionConflict`.

## Entry points

Entry-point stages are typed:

```text
Compute
Vertex
Fragment
```

Each entry-point descriptor contains a validated name, stage, and interface reference.
A compute pipeline names exactly one compute entry point. A render pipeline names one
vertex entry point and optionally one fragment entry point. Stage mismatches, duplicate
names for the same stage, missing declared entry points, and undeclared backend entry
points required by a pipeline are rejected.

Entry-point names are source-level identifiers, not diagnostic labels. They participate
in equality and hashing exactly as normalized UTF-8 names.

## Typed binding keys

```text
GpuBindingKey { group, binding }
```

Both components are checked `u32` values. Construction validates normalized device
limits during admission, while descriptor construction validates structural bounds and
duplicate keys without a live context. Ordering is lexicographic by `(group, binding)`.
The key is the only binding identity.

The following cannot be binding authority:

```text
String labels
filesystem paths
TypeId or Rust type names
GpuWorkResourceId
RenderFlowId, RenderPassId, or RenderFeatureId
vector position
naked u64 signature hashes
WGPU BindGroupLayoutEntry values
```

Labels, Rust type names, and source spans may accompany diagnostics only.

## Binding declarations

`GpuBindingDeclaration` contains:

- typed key;
- visibility set over compute, vertex, and fragment stages;
- one typed binding kind;
- optional nonzero binding-array count when admitted;
- provenance and diagnostic label excluded from semantic equality.

Initial binding kinds:

```text
UniformBuffer { dynamic_offset, minimum_size }
StorageBuffer { access: ReadOnly | ReadWrite, dynamic_offset, minimum_size }
SampledTexture { sample_class, view_dimension, multisampled }
StorageTexture { access, format, view_dimension }
Sampler { Filtering | NonFiltering | Comparison }
```

Normalized texture format and view vocabularies reuse or extend accepted G2 facts. Raw
WGPU enums do not cross the boundary. Acceleration structures, external textures,
video, ray tracing, bindless/unbounded arrays, and backend-native bindings remain
deferred.

Compatibility checks include:

- exact binding key;
- resource kind and usage;
- visibility required by the entry point;
- minimum buffer size and declared range;
- dynamic-offset policy and admitted alignment;
- texture sample class, view dimension, multisample state, and format role;
- storage texture access and exact normalized format;
- sampler class;
- array cardinality;
- resource context/generation affinity at realization time.

## Program interface

`GpuProgramInterfaceDescriptor` owns an ordered set of binding declarations plus any
stage input/output contracts needed to create initial compute/render pipelines.

Initial vertex-input contracts include typed shader locations, normalized scalar/vector
format, byte offset, stride, step mode, and buffer slot. Initial fragment-output
contracts include typed color locations and normalized format class. Inter-stage
reflection is not assumed. The explicit descriptor is authoritative and backend module
validation must agree with it.

WGSL reflection may be used internally as validation evidence only when the selected
WGPU/Naga facilities provide complete trustworthy facts. It does not replace the
explicit public interface. A mismatch is a structured error, never an inferred silent
layout change.

## Bind-group and pipeline layouts

```text
GpuBindGroupLayoutDescriptor
GpuPipelineLayoutDescriptor
```

A bind-group layout contains one group index and declarations sorted by binding key. It
rejects duplicate bindings and declarations whose keys name a different group. A
pipeline layout contains ordered group-layout descriptors with unique group indices.
Initial G4 has no push constants because no accepted current consumer proves them.

Semantic equality and hashing include every correctness field and exclude labels,
provenance, source spans, backend object addresses, insertion order after
normalization, and diagnostic-only adapter strings.

## Specialization

```text
GpuSpecializationKey
GpuSpecializationType
GpuSpecializationDefault
GpuSpecializationSchema
GpuSpecializationValue
GpuSpecializationSet
```

Initial value kinds are `Bool`, `U32`, `I32`, and finite `F32`. Floating-point values
normalize negative zero and use canonical IEEE bits; NaN and infinity are rejected.
Keys are validated source-level identifiers, not labels. A schema has unique keys,
exact types, optional defaults, and requirement implications. A value set rejects
unknown keys, duplicates, type mismatch, and missing required values, then normalizes
to schema-key order.

Specialization values participate in module/pipeline realization keys only where WGPU
or source transformation actually consumes them. RunenGPU must not pretend unsupported
WGSL override behavior is available. Backend support is admitted explicitly.

## Compute pipeline descriptors

`GpuComputePipelineDescriptor` contains:

- program source key and revision/content identity;
- compute entry-point key;
- exact program-interface identity;
- exact pipeline-layout descriptor;
- normalized specialization values;
- required capability facts;
- label and provenance excluded from semantic equality.

No renderer flow, pass, feature, material, view, or surface identity appears.

## Render pipeline descriptors

`GpuRenderPipelineDescriptor` contains only generic GPU pipeline shape:

- program source key and revision/content identity;
- vertex and optional fragment entry points;
- exact program interface and pipeline layout;
- normalized specialization values;
- vertex-buffer layouts;
- primitive topology, front-face, cull, and polygon rules;
- depth/stencil state where accepted normalized formats support it;
- multisample count, mask, and alpha-to-coverage;
- ordered color-target states with normalized formats and generic blend/write masks;
- required capability facts.

It contains no material meaning, lighting model, view ID, render feature ID, pass ID,
render-flow identity, target alias, product quality policy, shader path, or surface
handle. RunenRender or Runenwerk derives a generic descriptor from semantic decisions.
Initial dynamic state is limited to current accepted consumer needs.

## Descriptor equality, ordering and hashing

All G4B semantic descriptors:

- have private fields and fallible constructors;
- normalize unordered collections into typed-key order;
- implement semantic equality, ordering, and hashing where fields permit;
- canonicalize floating-point values before storage;
- hash typed discriminants and complete field values, never `Debug` text;
- do not expose a naked hash as authoritative identity;
- may expose a diagnostic digest alongside the full descriptor;
- exclude label/provenance only where those fields cannot affect backend correctness.

A cache always verifies full-key equality. Hash collision cannot produce reuse.

## Program and interface diagnostics

Required categories:

```text
invalid source key or revision
source revision conflict
unsupported source kind
WGSL parse or validation failure
entry point missing or stage mismatch
duplicate binding key
binding declaration invalid
program/interface mismatch
runtime binding incompatible
specialization unknown, missing, or type-mismatched
pipeline descriptor invalid
backend module, layout, or pipeline realization failure
```

Backend compiler messages are attached as bounded diagnostic detail with source key,
revision, and entry point. They are not parsed as the public error category.

## Existing parameter helper disposition

`GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField`, and the current
derives remain in the Runenwerk/render boundary during G4. They may prepare bytes for a
logical buffer. They do not prove WGSL structure identity, binding key or visibility,
minimum binding size, storage-buffer ABI, nested/array/runtime-array layout, package
independence, or interface compatibility.

G4B adds no derive. Compile-pass/fail tests prove explicit descriptor construction and
reject ambiguous helper use as interface authority. The implementation must not move
`engine_render_macros` into RunenGPU or create `runengpu_macros`.

## Compile-pass and compile-fail proof

Required passing proof:

- one WGSL compute program with typed storage-read and storage-write bindings;
- one WGSL render program with typed vertex input, uniform binding, and one color
  target;
- equal descriptors built in different insertion orders produce full equality and
  equal hashes;
- specializations normalize independently of caller order;
- current Runenwerk parameter bytes bind only after explicit interface validation.

Required compile-fail or construction rejection:

- raw WGPU type in a public descriptor;
- string, `TypeId`, resource ID, or pass ID used as binding key;
- duplicate `(group, binding)`;
- stage visibility mismatch;
- wrong resource kind, texture class, sampler class, format, or array count;
- missing, unknown, or type-mismatched specialization;
- compute pipeline using a render entry point or the reverse;
- source key/revision reused with different source bytes;
- semantic descriptor field construction bypassing validation;
- derives treated as sufficient interface proof.

# G4C — WGPU realization, cache compatibility and cutover

## Realization model

Logical descriptors are admitted into one `GpuContext` and produce opaque,
generation-bound realized handles for buffers, textures, views, samplers, query sets,
programs, bind-group layouts, pipeline layouts, bind groups, compute pipelines, and
render pipelines.

Exact file and type names remain implementation details. Public handles are `Clone`,
non-`Copy`, contain no raw WGPU access, and expose typed logical identity plus affinity
for inspection. Backend objects remain private.

## Realization timing by kind

The accepted policy is explicit admission with bounded lazy derivation:

| Kind | Policy | Reason |
|---|---|---|
| persistent/imported buffer or texture | explicit before graph execution | allocation/import failure precedes G5 encoding |
| transient graph resource | explicit bounded graph-realization pass after G3 preparation | permits future lifetime-aware allocation without hidden execution-time creation |
| sampler | explicit or cache-backed lazy from complete descriptor | stateless derived object |
| texture view | lazy within an admitted parent texture, cached by complete view descriptor | derived object with mandatory parent affinity |
| query set | explicit | count/type/support failure precedes execution |
| program module | explicit admission or first-pipeline lazy realization | source failure is structured and cacheable |
| bind-group and pipeline layouts | cache-backed lazy from full descriptors | derived state after admission |
| bind group | explicit from typed layout and realized resources | runtime values and generation must be checked |
| compute/render pipeline | explicit or cache-backed lazy before execution | pipeline failure is not an unclassified G5 side effect |

G5 may require all realizations for a prepared graph to resolve before encoding. It
cannot create undeclared resources or bypass compatibility checks.

## Affinity and rejection points

Every realization request validates in this order:

1. logical descriptor validity;
2. context identity of every already realized input;
3. current device generation;
4. resource, program, interface, and pipeline compatibility;
5. admitted features, limits, formats, and alignments;
6. full cache-key lookup and equality;
7. WGPU creation when no compatible hit exists.

Structured failures distinguish foreign context, stale generation, unknown logical
resource, descriptor change for identity, resource-kind mismatch, interface mismatch,
binding-value mismatch, unadmitted requirement, format/alignment incompatibility,
cache rejection, and backend realization failure.

Foreign or stale values are rejected before a WGPU call. A stale cache entry is never
returned. Runtime use-after-retirement and delayed destruction remain G5-owned.

## Realization registries

The context owns private registries separated by responsibility:

```text
resource realizations
program/module realizations
bind-group-layout realizations
pipeline-layout realizations
bind-group realizations
compute-pipeline realizations
render-pipeline realizations
cache diagnostics
```

Registry invariants:

- one context/device generation per registry instance;
- full typed key plus full equality check;
- no renderer, ECS, Winit, domain, or product type;
- no string-only or `TypeId` lookup;
- no WGPU object exposed publicly;
- no authoritative entry without its normalized source descriptor;
- errors leave no partially published entry;
- duplicate equal requests return the same logical realization record;
- one logical identity with a different descriptor is rejected;
- labels do not split otherwise equal realizations;
- provenance remains available for reports;
- no consumer callback occurs under internal locks.

## Realization keys

Resource keys include context identity, device generation, logical identity, complete
normalized descriptor, and import/source generation where applicable. A texture-view
key additionally includes parent realization identity and complete normalized view
range. Sampler and query-set keys include complete semantics. Labels are excluded.

Program/module keys include context identity, generation, source kind/key/revision,
source-content digest, declared entry points, program interface, admitted requirements,
and backend compatibility revision.

Layout keys include affinity, complete descriptor, and relevant admitted limits and
features. Bind-group keys include affinity, full layout identity, ordered typed values,
complete resource realizations and ranges, and resource/import generations.

Pipeline keys include affinity, complete program/module and entry-point identities,
interface, layout, complete compute/render descriptor, specialization, relevant
features/limits/formats/alignments, and backend compatibility revision.

Renderer semantic identities may contribute to the lowered descriptor but do not
appear merely as renderer IDs or hashes in the RunenGPU key.

## Cache scope and compatibility

Initial G4 caches are in-memory and scoped to one `GpuContextId` and
`GpuDeviceGeneration`. They are derived, discardable, and reconstructable.

No stable persisted pipeline-cache or wire format is authorized. If the selected WGPU
version exposes backend cache data, G4C may use it only when the implementation issue
proves it fits this internal envelope without adding a stable format.

Compatibility includes every correctness fact:

```text
RunenGPU internal cache schema revision
RunenGPU descriptor schema revision
WGPU version and backend compatibility revision
normalized backend family
adapter class and independent software/fallback fact
available adapter and relevant driver identity facts
context identity and device generation for in-memory entries
source kind, key, revision, and content digest
program interface and specialization identity
complete resource, layout, and pipeline descriptors
exact enabled features and effective limits
relevant format and alignment facts
```

Behavior:

- full match: reuse and emit `Hit`;
- no entry: create and emit `Miss`;
- incompatible entry: emit typed `Rejected`, discard or quarantine, then realize
  ordinarily;
- corrupt backend data: emit `RejectedCorrupt`, then realize ordinarily;
- backend creation failure: emit `RealizationFailed` and publish no entry.

A cache hit changes cost, never semantics. Cache stats distinguish hits, misses,
rejections, corruption, and creation failures.

## Private backend ownership and the temporary G5 bridge

After G4C, only private RunenGPU backend modules own reusable WGPU buffers, textures,
views, samplers, query sets, shader modules, layouts, bind groups, pipelines, instance,
adapter, device, and queue.

The current renderer still owns G5 execution until G5 is implemented. G4C therefore
permits exactly one narrow crate-private execution bridge with these rules:

- it is located at the RunenGPU backend boundary and is not public or
  future-transferable;
- it is reachable only from the existing bounded render execution adapter;
- it may lend scoped borrowed WGPU references required to encode the already accepted
  G5 execution payload;
- borrowed values cannot be stored, cloned into new ownership, cached, used as lookup
  authority, or returned to other modules;
- it accepts only opaque realized handles already validated for context, generation,
  kind, program, interface, layout, and pipeline compatibility;
- it owns no operation, access, hazard, initialization, requirement, program,
  interface, pipeline, cache, or realization truth;
- source and reach-through guards permit only this named path;
- G5 deletes the bridge when encoding/submission moves into RunenGPU.

This is a migration terminal, not stable raw-WGPU authority or a general escape hatch.
Current render files may otherwise retain WGPU command/pass encoders, surface textures,
and presentation only where explicitly G5/G7-owned.

## Migration order

G4C performs a clean cutover:

1. add generation-bound private RunenGPU realization registries;
2. realize G2 logical resources through the context;
3. realize G4B programs and layouts through the context;
4. realize typed bind groups and pipelines through the context;
5. migrate renderer setup, dynamic targets, material resources, UI passes, flow-runtime
   caches, applications, examples, tests, and benchmarks;
6. replace broad raw device/queue and backend-object access with the one bounded
   crate-private G5 execution bridge;
7. remove renderer cache keys using shader paths, pass/feature IDs, or naked hashes in
   place of complete G4 descriptors;
8. remove renderer-owned WGPU resource/program/layout/bind-group/pipeline registries;
9. remove synthetic logical-handle construction and the `RenderFlowId` owner bridge;
10. remove all G4-owned program/interface/layout/pipeline/cache/realization truth from
    the G3 sidecar;
11. add source, dependency, bridge, sidecar, migration, and deletion guards;
12. prove every current consumer migrated and every replaced owner deleted.

No compatibility alias, forwarding module, duplicate cache, or old/new realization
path is retained.

## Exact deletion and retention split

Delete in G4C after migration:

```text
renderer-owned reusable WGPU resource allocator and registries
renderer-owned shader-module/layout/bind-group/pipeline caches
string-only PipelineKey as GPU correctness authority
FlowPassPipelineKey and FlowPassBindGroupKey as GPU correctness authority
backend aliases that only forward renderer authority
synthetic G2 handle construction from render declarations
RenderFlowId-derived GpuWorkResourceId owner bridge
G4-owned program/interface/layout/pipeline/cache/realization sidecar fields
public or broad WgpuCtx device/queue reach-through
```

A file may remain if it still owns unrelated render semantics, but replaced types,
fields, functions, and re-exports are deleted.

Retain until G5:

```text
prepared G3 graph
residual operation-specific execution payload
command/pass encoding
uploads and updates
query-resolution execution
queue submission
progress and device polling
completion, readback, and cancellation
runtime retirement and delayed destruction
the single crate-private scoped execution bridge
```

Retain until G7:

```text
current Runenwerk window-to-surface registry
surface compatibility shim
surface configuration, acquisition, and presentation
surface-generation and loss gaps
Winit host integration
```

## RunenRender separation

G4 is GPU/backend decontamination and substrate work. It does not define materials,
providers, lighting, visibility, transport, reconstruction, render history, overlays,
views, targets, color policy, image quality, or presentation meaning.

The current render tree is evidence of mixed ownership. It is not moved, renamed,
wrapped, or extracted wholesale. A render file changes in G4 only to remove GPU
context, program/interface, realization, or cache authority, or to consume the new
RunenGPU substrate. Renderer semantic code remains until separately accepted R-phase
work.

The temporary sidecar may remain only for G5-owned execution payload. After the owning
G4 slice, it cannot contain operation, access, hazard, initialization, requirement,
program, interface, pipeline, cache, or realization truth.

RX is a later mechanical repository transfer and clean cutover after R-phase proof. It
is not where renderer architecture is invented.

# Proof and conformance

## Deterministic G4A proof

- context IDs are unique and generations start nonzero;
- foreign/stale affinity is rejected;
- requirement normalization is order-independent;
- contradictions are rejected before backend access;
- synthetic candidate facts produce deterministic admission, degradation, and ranking;
- unsupported preferred requirements require explicit degradation keys;
- unrelated features are not opportunistically enabled;
- portability classes derive exactly;
- normalized backend/adapter/software/limit/format/alignment mappings have exhaustive
  known-variant tests and explicit unknown handling;
- public/source dependency guards prove no raw WGPU, Winit, renderer, ECS, application,
  or product type in the future-transferable boundary.

## Environment-dependent G4A proof

- headless native adapter/device request where an adapter exists;
- explicit unsupported/no-adapter outcome;
- enabled facts equal the request result;
- optional temporary host-compatible selection smoke;
- WebGPU compile/runtime proof in the supported platform matrix when available.

## Deterministic G4B proof

- source key/revision/content consistency;
- descriptor validation, equality, ordering, and hashing;
- typed binding-key uniqueness and compatibility;
- explicit interface versus entry-point stages;
- specialization schema/value normalization;
- compute/render pipeline descriptor validation;
- compile-pass and compile-fail public-contract cases;
- no current derive or `TypeId` can serve as interface authority.

## Environment-dependent G4B/G4C proof

- WGSL module creation and compiler diagnostics;
- one compute and one render pipeline realization on an admitted adapter;
- resource, view, sampler, query-set, bind-group, and layout realization;
- alignment/format rejection using real adapter facts;
- cache hit, miss, rejection, and ordinary fallback;
- exact context/device-generation isolation.

## Migration and deletion proof

Guards inspect the complete relevant source subtree and reject:

- raw public `Device` or `Queue` fields and broad WGPU re-exports;
- any raw-WGPU reach-through except the named crate-private scoped G5 bridge;
- renderer-owned shader-module/layout/bind-group/pipeline cache types;
- `RenderFlowId`-derived GPU resource-owner construction;
- crate-private synthetic G2 handle constructors outside the RunenGPU owner;
- program/interface/pipeline/cache/realization truth in the sidecar;
- string, path, `TypeId`, pass, feature, or naked-hash binding authority;
- direct WGPU realization consumers outside the private backend and explicitly retained
  G5/G7 migration files;
- compatibility aliases or forwarding modules for deleted owners.

Source-string guards supplement typed tests; they do not replace behavior proof.

## Validation

Each implementation slice requires focused checks plus, at its exact reviewed head:

```text
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Repository-owned exact-head GitHub Actions are independent merge evidence. G4A, G4B,
and G4C each update their issue, parent program, and durable planning. No later slice
may begin before its predecessor is accepted.

# Implementation ergonomics

The ordinary consumer path should converge on:

```rust
let context = GpuContext::request(context_descriptor).await?;
let program = context.admit_program(program_descriptor)?;
let pipeline = context.realize_compute_pipeline(compute_pipeline_descriptor)?;
let input = context.realize_buffer(input_descriptor)?;
let output = context.realize_buffer(output_descriptor)?;
```

Names are directional, not an implementation mandate. The implementation may return
future-like results where WGPU requires asynchronous backend work. The contract is one
context owner, explicit admission, typed descriptors, structured failures, no public
raw WGPU reach-through, and no duplicated realization path.

RunenRender and Runenwerk adapters may offer higher-level semantic authoring. They lower
into the same G4B descriptors and G4C realization authority.

# Stop conditions

Implementation stops for owner decision if it requires:

- a new dependency or package;
- a stable persisted cache, source, ABI, capture, replay, or wire format;
- public raw WGPU access or a second raw-WGPU escape path;
- unsafe code not already covered by accepted repository policy;
- a second backend abstraction without a concrete consumer;
- G5 progress, completion, readback, retirement, or shutdown beyond the named temporary
  execution bridge;
- G7 reusable surface, loss, or reconstruction authority;
- RunenRender semantic implementation;
- a compatibility facade, forwarding package, or duplicate cache;
- an ADR changing accepted dependency direction or repository ownership.

Backend API details, adapter availability, and compiler diagnostics are expected
environment facts, not architecture stop conditions when they fit this design.

# Decision summary

G4A creates an async, headless, normalized, generation-aware context admission boundary.
G4B creates explicit typed WGSL-first program, interface, binding, specialization, and
pipeline descriptors with deterministic semantic identity. G4C realizes those values
through private WGPU registries, rejects stale and foreign generations, uses
correctness-complete derived caches, migrates every consumer, and deletes renderer-owned
GPU realization authority. One narrow crate-private G5 execution bridge preserves the
current encoder during the ordered cutover and is deleted by G5.

G5 remains the execution and lifecycle slice. G7 remains the surface and device-loss
slice. RunenRender remains semantic image formation. No package extraction or Rust
implementation is authorized by this planning document alone.