---
title: RunenGPU G4B Contracts and G4C Delivery Design
description: Final ownership and delivery boundaries for source admission, shader-resource and stage-IO compatibility, and ordered WGPU realization.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-10
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g4-context-program-realization-design.md
  - ./runengpu-shader-authoring-artifact-boundary.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/2026-08-03-runengpu-g4b-g4c-finalization.md
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../../workspace/specs/pt-runengpu-g4c1-resource-realization.ron
  - ../../workspace/specs/pt-runengpu-g4c2-program-binding-realization.ron
  - ../../workspace/specs/pt-runengpu-g4c3-pipeline-cutover.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G4B Contracts and G4C Delivery Design

## Status and authority

This focused design is authorized by issue `#209` after accepted G4A and the accepted
shader-authoring boundary:

```text
G4A accepted             501b9fd58e56d33708573e47faf0e5026b5a1ff2
shader boundary accepted 23bc982703f93d15ac39dd71d61bae9e23854141
```

It narrows and supersedes only the conflicting G4B interface, reflection,
source-consistency, stage-IO, and G4C delivery details in the earlier G4 umbrella
design. The earlier design remains authoritative for the broader G4 mission, G4A
ownership, private WGPU direction, G5/G7 exclusions, and RunenRender boundary.

The implementation specifications own exact phase requirements. GitHub issues own live
activation and delivery. This document owns the durable architectural explanation.

## Problems corrected

The accepted direction was sound, but implementation was unsafe in six places:

1. Shader-visible resource bindings were mixed with vertex-buffer and color-target
   pipeline state.
2. Reflection was optional even though canonical WGSL must agree with explicit
   declarations.
3. `SourceRevisionConflict` had no stateful owner.
4. Public compute/render construction was not concrete enough to constrain ergonomics.
5. Reflected vertex inputs and fragment outputs were not carried into later pipeline
   compatibility after separating them from the resource interface.
6. Resource, program/binding, and pipeline cutover were split without a viable
   intermediate path for current consumers.

The correction keeps one architecture while separating semantic ownership, backend
realization, and serialized migration.

# Final ownership model

## Runenwerk owns source production policy

Runenwerk owns:

- source roots and filesystem paths;
- direct WGSL, WESL, Slang, or future authoring-language policy;
- compiler pinning and normalized options;
- module and dependency discovery;
- deterministic canonical WGSL production;
- source maps and authoring diagnostics;
- file watching and reload scheduling;
- atomic artifact publication and last-known-good product fallback;
- persisted artifact and reproducibility policy.

None of those concerns enters the future-transferable RunenGPU public contract.

## Consumers own semantic meaning

RunenRender owns material, medium, lighting, transport, view, target, reconstruction,
and image-formation meaning. A non-render consumer owns its own kernel or simulation
meaning.

Consumers lower that meaning into generic RunenGPU source, interface, resource, work,
and pipeline descriptors. Consumer semantic IDs may select which complete descriptor
to build, but they do not become GPU correctness identity by themselves.

## RunenGPU G4B owns admitted logical contracts

G4B owns:

```text
source owner + source key + source revision + full canonical WGSL
entry-point names and stages
shader-visible resource interfaces
normalized shader-stage-IO compatibility vocabulary
binding keys and declarations
bind-group and pipeline-layout descriptors
specialization schemas and values
compute and render pipeline descriptors
runtime binding compatibility
mandatory resource-interface and stage-IO comparison algorithms
```

G4B creates no WGPU object.

## Private G4C owns backend realization

G4C owns private WGPU realization through three ordered children:

```text
G4C1 resources
    -> G4C2 programs, layouts and bind groups
        -> G4C3 pipelines and final cutover
```

Every realization is bound to exact context and device generation. Public consumers
receive opaque typed handles only.

# Program source admission

## Identity

An admitted source record contains:

```text
GpuProgramSourceOwnerId
GpuProgramSourceKey
GpuProgramSourceRevision
full canonical WGSL bytes
deterministic source digest
bounded provenance
```

The owner ID separates independent source authorities. The key is semantic and
owner-scoped. The revision is nonzero and meaningful only for one owner/key pair.
Neither key nor revision is a path, timestamp, global ordering, persistence identity,
or backend handle.

The full canonical WGSL bytes remain authoritative. The digest accelerates lookup and
diagnostics but cannot authorize equality alone.

## Stateful consistency owner

`GpuProgramSourceRegistry` is the one stateful G4B owner of source consistency.

It provides these semantics:

- equal owner/key/revision and byte-identical canonical WGSL is idempotent;
- equal owner/key/revision with different canonical WGSL is
  `SourceRevisionConflict`;
- a different revision is a distinct admitted record;
- a new revision does not silently invalidate descriptors or realizations bound to an
  older revision;
- maximum records and retained source bytes are explicit;
- pressure is structured and cannot silently evict live authority;
- publication is transactional;
- no consumer callback executes under registry locks.

Runenwerk may choose registry capacity policy. It does not become the
source-consistency implementation owner.

# Resource-interface and stage-IO boundaries

## Program interface owns resource bindings only

`GpuProgramInterfaceDescriptor` owns shader-visible resource declarations:

- typed `(group, binding)` identity;
- stage visibility;
- resource kind and access;
- buffer minimum size and dynamic-offset policy;
- texture dimension, sample class, multisample state, storage format and access;
- sampler class;
- array cardinality;
- requirements needed to validate those declarations.

The interface does not own complete render pipeline memory or attachment state.

## Render input state remains pipeline state

The render pipeline descriptor owns:

- vertex-buffer slots;
- attribute shader locations and formats;
- byte offsets and strides;
- vertex or instance step mode.

These values describe how host buffers feed one pipeline invocation. They are not
shader-resource bindings.

## Render output state remains pipeline state

The render pipeline descriptor also owns:

- ordered color-target formats;
- blend state;
- write masks;
- depth/stencil state;
- multisample state;
- primitive and other accepted graphics state.

Color-target attachment policy is not part of the shader-resource interface.

## Shader-stage IO remains mandatory compatibility evidence

Separating pipeline state from the resource interface must not weaken proof that the
selected WGSL entry points and pipeline state agree.

G4B therefore owns normalized observed-signature vocabulary and comparison algorithms:

```text
GpuObservedVertexInputSignature
    entry point
    unique user shader locations
    normalized scalar/vector classes
    builtins kept separate

GpuObservedFragmentOutputSignature
    entry point
    unique user color locations
    normalized scalar/vector classes
    builtins kept separate
```

G4C2 invokes the pinned WGPU/Naga path, normalizes those signatures, and retains them
with the realized program record. G4C3 compares:

- the selected vertex signature against explicit vertex attributes;
- the selected fragment signature against ordered color-target format classes.

Buffer slots, offsets, strides, step modes, blend state, and write masks remain explicit
pipeline policy and are not inferred from reflection.

Missing, duplicate, extra, location-mismatched, or incompatible scalar/vector facts are
`PipelineStageIoMismatch` before backend pipeline creation.

# Mandatory canonical WGSL agreement

## Explicit declarations remain authority

RunenGPU does not infer its public interface from reflection. Explicit typed
declarations remain authoritative because they provide stable framework-facing
meaning, deterministic construction, structured diagnostics, and backend-neutral
validation.

## Reflection is mandatory evidence

Before G4C2 publishes a program/module realization, the pinned WGPU/Naga path must
observe and normalize:

- entry-point names and stages;
- groups and bindings;
- stage visibility;
- resource classes and access;
- texture dimensions, sample/storage facts, and multisample state;
- array cardinality;
- applicable host-layout facts;
- vertex-input and fragment-output signatures.

The division of responsibility is exact:

```text
G4B
    owns expected resource facts
    owns resource and stage-IO comparison vocabulary
    owns deterministic comparison algorithms

G4C2
    invokes pinned WGSL/WGPU/Naga parsing and reflection
    normalizes observed resource and stage-IO facts
    proves resource-interface agreement
    retains observed stage-IO signatures

G4C3
    compares retained stage-IO signatures to complete render pipeline state
```

Reflection may reject disagreement and improve source-span diagnostics. It may not
renumber, add, remove, infer, or mutate authoritative declarations or pipeline policy.
Resource disagreement is `ProgramInterfaceMismatch`; stage-IO disagreement is
`PipelineStageIoMismatch`.

# Public construction shape

The public path should read as a sequence of semantic objects rather than a nested
backend construction ladder.

## Compute shape

```rust
let source = sources.admit(
    source_owner,
    source_key,
    source_revision,
    canonical_wgsl,
    provenance,
)?;

let interface = GpuProgramInterfaceDescriptor::builder()
    .storage_buffer(GpuBindingKey::new(0, 0)?, ReadOnly, ShaderStages::COMPUTE)?
    .storage_buffer(GpuBindingKey::new(0, 1)?, ReadWrite, ShaderStages::COMPUTE)?
    .build()?;

let program = GpuProgramDescriptor::builder(source)
    .entry_point("scan", GpuShaderStage::Compute, interface.id())?
    .build()?;

let pipeline = GpuComputePipelineDescriptor::builder(program, "scan")
    .layout(interface.pipeline_layout()?)
    .build()?;
```

Exact names may change during implementation, but the contract preserves one bounded
fallible terminal per semantic object and no raw WGPU.

## Render shape

```rust
let interface = GpuProgramInterfaceDescriptor::builder()
    .uniform_buffer(GpuBindingKey::new(0, 0)?, ShaderStages::VERTEX_FRAGMENT)?
    .build()?;

let program = GpuProgramDescriptor::builder(source)
    .entry_point("vs_main", GpuShaderStage::Vertex, interface.id())?
    .entry_point("fs_main", GpuShaderStage::Fragment, interface.id())?
    .build()?;

let pipeline = GpuRenderPipelineDescriptor::builder(program, "vs_main")
    .fragment_entry("fs_main")
    .vertex_buffers(vertex_buffers)
    .color_targets(color_targets)
    .primitive(primitive)
    .multisample(multisample)
    .build()?;
```

Vertex-buffer input and color-target output remain visibly separate from the resource
interface and are checked against observed stage-IO signatures during G4C3.

## Runtime binding shape

Prepared bytes and logical resources become eligible for G4C2 bind-group realization
only after compatibility validation against the accepted resource interface. Current
derives may prepare bytes; they do not prove compatibility.

# Ordered G4C delivery

## Why bridges are required

A clean type-by-type cutover cannot assume that all downstream current consumers vanish
at once:

- after G4C1 owns resources, audited current consumers can still need private resource
  references while their semantic operation remains in G4C2, G4C3, or the unchanged
  G5 execution path;
- after G4C2 owns modules, layouts, and bind groups, audited current consumers can still
  need the proven residual resource references plus private program, layout, and binding
  references while pipeline realization or the unchanged G5 execution path remains;
- after G4C3 owns pipelines, current command encoding still needs private realized
  objects until G5 replaces execution ownership.

Leaving the old registries authoritative would create parallel authority. Exposing raw
objects publicly would violate containment. The solution is a serialized successor
bridge ladder.

## Two distinct temporary seams

The serialized ladder below is the complete **object-reference migration bridge**
ladder. At every accepted G4C boundary, exactly one object-reference migration bridge
may remain.

`CurrentRenderDeviceQueue` is not an object-reference bridge. It is a distinct,
crate-private backend-operation loan to exact current uncut code. It must not be hidden,
counted as a second object-reference bridge, or folded into
`CurrentRenderResourceBridge`. It remains non-public, source-guarded,
non-authoritative, and purpose-bound: it has no `Deref`/`AsRef`, reusable raw handle,
generic callback, or future native-interop meaning.

Its authorized operation classes and exact call sites are an independent monotonic
cutover:

```text
after G4C1
    no generic buffer/texture/view/sampler/query-set creation through
    CurrentRenderDeviceQueue

after G4C2
    additionally no ShaderModule / BindGroupLayout / PipelineLayout / BindGroup
    creation through it

after G4C3
    additionally no compute/render pipeline creation through it

after G5
    encoding / uploads / submission / copy / map / readback users migrate
    and CurrentRenderDeviceQueue is deleted
```

The current source census is evidence for this split: the loan directly enters the
renderer execution path and two runtime-evidence paths, while the transitive renderer
code still performs both realization and G5 operations. Each child must enumerate the
remaining operation classes and exact call sites in source guards, prove that they only
shrink, and leave no broad `Device`/`Queue` access behind.

## Serialized object-reference successor bridge ladder

```text
G4C1
    CurrentRenderResourceBridge
        purpose-typed terminals for exact audited G4C2/G4C3/G5 consumers
        superseded and deleted by G4C2

G4C2
    CurrentRenderPipelineBridge
        residual G4C1 resource terminals plus G4C2 program/layout/bind-group terminals
        superseded and deleted by G4C3

G4C3
    CurrentRenderExecutionBridge
        accepted resource/bind-group/pipeline terminals for the unchanged G5 encoder
        deleted by G5
```

Rules common to all bridges:

- crate-private and process-local;
- exactly one object-reference migration bridge remains at each accepted boundary;
- a successor replaces its predecessor; the predecessor is deleted before the successor
  is accepted, so no bridge overlap remains;
- the set of carried-forward predecessor terminals monotonically shrinks;
- a successor may add only newly realized terminal classes owned by that phase that
  exact-current-main uncut consumers still require;
- migrated predecessor terminals disappear, and no successor may reintroduce a
  predecessor terminal already eliminated by its owning phase;
- accepts only validated opaque handles;
- repeats exact context, generation, and kind checks;
- narrowly typed to audited current call sites;
- no public accessor, `Deref`, `AsRef`, reusable raw handle, generic raw-WGPU callback,
  arbitrary consumer closure, or broad `Device`/`Queue` access;
- borrowed backend values cannot be stored, cloned into ownership, cached, returned,
  persisted, or used as identity/lookup authority;
- owns validated lexical access only: never resource, descriptor, program, layout,
  bind-group, pipeline, encoding, upload, submission, semantic, access, hazard, cache,
  execution, progress, completion, or lifecycle truth;
- the semantic operation remains owned by its phase. For example, an unchanged G5
  operation may lexically use a borrowed resource reference, but no bridge performs or
  owns encoding, uploads, submission, or other execution;
- source guards enumerate its exact call sites and reject a second bridge;
- immediate successor owns deletion.

## G4C1 — resource realization

G4C1 owns private realization of:

- buffers;
- textures;
- texture views;
- samplers;
- query sets.

It owns exact affinity checks, transactional authoritative resource registries,
bounded registry reclamation, migration of resource creation and ownership, and deletion
of replaced renderer resource registries. A derived cache is an optional cost
optimization only; it never substitutes for the registry.

### Logical identity and owner scopes

`GpuWorkResourceId` remains one logical resource identity inside one owner scope. The
owner-scope sequence is:

```text
private RunenGPU process-local monotonic owner-scope authority
    ↓
fresh opaque nonzero owner scope
    ↓
GpuWorkResourceIdAllocator
    ↓
typed G2 logical handles
```

The authority is private to RunenGPU and independent of `GpuContext`,
`GpuDeviceGeneration`, backend/WGPU, `RenderFlowId`, and renderer invocation/history
identity. It issues a fresh nonzero scope; zero is invalid, scopes never wrap or reuse,
and exhaustion is structured. Following the `GpuContextIdAllocator` doctrine, isolated
test allocators cannot mutate or reset production authority. Raw components remain
diagnostic-only, and production callers cannot choose, reconstruct, or inject raw scope
values. Scope acquisition stays inside the already-fallible logical-resource allocation
path (lazily at first successful allocation or equivalently), so `RenderFlow::new` does
not become fallible merely to allocate a scope. No public scalar owner-scope type is
introduced.

The current `RenderFlowId`-derived owner scope is a temporary GPU identity bridge and a
G4C1 deletion target. Renderer flow declarations remain templates and policy, not
generic resource identity. For invocation uniforms and history textures, the renderer
may map its invocation/history key to a distinct retained typed G2 logical handle for
each concurrently distinct resource; G4C1 receives only that handle, its normalized
descriptor, and context/device-generation facts. Two concurrent resources must have
different typed handles before realization.

`invocation_id`, `view_id`, `RenderFlowId`, renderer labels and paths, naked hashes, and
WGPU addresses never become generic G4C1 resource identity or registry-key fields. Do
not introduce a public `GpuResourceInstanceId` unless an exact-current-main proof shows
that typed G2 handles plus GPU-owned owner scopes are insufficient; that finding stops
for a separate decision rather than inventing another identity system.

### Imports, creation, and backend failures

`GpuResourceOwnership::Owned` may create a private backend object in G4C1. Current
renderer import semantics and provenance are not a backend import source: an imported
buffer or texture may be realized only when an explicit accepted import-source contract
admits a concrete source. An import without one yields the structured
`ImportSourceUnavailable` or unresolved-import outcome. `SurfaceAcquired` remains G7
only. G4C1 introduces no public `wgpu::Buffer`/`wgpu::Texture` import, renderer-ID key,
native-handle import, external-memory API, or unsafe generic import escape hatch.

`RenderResourceDeclaration::History` is the current invocation-history path; G4C1 maps
its policy to an invocation-scoped RunenGPU-owned logical/backing texture, not an
external import. Imported-history or external intent without a concrete admitted source
remains unresolved and does not force a generic G4C1 import API.

Deterministic descriptor, capability, and affinity incompatibility is rejected by
RunenGPU before backend creation. An unexpected backend validation rejection after
complete RunenGPU admission is a backend-contract/invariant violation, not ordinary
cache/compatibility control flow. Resource allocation exhaustion or OOM is a distinct
structured backend capacity/resource-exhaustion failure, not an implementation defect;
G5 later owns pressure policy and bounded execution behavior. Unavailable or lost
device/context is a structured context/device outcome: G4C1 owns neither replacement nor
recovery, G7 later owns replacement/reconstruction facts, and Runenwerk owns product
recovery. Backend error text is bounded diagnostics, never semantic classification.
Publication remains transactional and every such failed construction publishes no
realization. Ordinary public realization is not made async merely to manufacture
constructor `Result` semantics.

### Authoritative registry, cache, and liveness

The authoritative registry maps logical identity to a realization record. It is scoped
to one context/device generation, so admission validates relevant context facts before
lookup rather than copying a huge context fact set into every map key. A derived cache
may select a candidate only; hashing never authorizes correctness and full typed equality
does.

```text
same logical identity + same complete descriptor
    -> same realization record
same logical identity + changed descriptor
    -> DescriptorChangedForIdentity
different logical identity + identical descriptor
    -> different logical resource; never aliases merely because descriptors match
```

Cross-logical deduplication is not required. A later sampler or texture-view cache, if
evidence warrants one, is separately derived and non-authoritative; labels, provenance,
and backend pointers are not semantic key fields. Sampler compatibility covers every
semantic field of the accepted normalized `GpuSamplerDescriptor`, not nonexistent
anisotropy state.

Registry reclamation is not GPU completion and not physical backend retirement. G4C1 may
remove or deactivate an unretained record from future authoritative lookup only according
to logical liveness; lookup removal neither proves previous encoded/submitted GPU use is
complete nor invalidates a live realized handle, active bridge borrow, or current
execution reference. G4C1 owns no fence or submission-completion retirement authority and
preserves current backend/execution retention mechanics until G5 replaces them. G5 alone
owns submission, in-flight retention, completion, cancellation, delayed backend
retirement/destruction, and shutdown.

Backend resource-object creation belongs to G4C1. Upload/update/copy/staging,
query-resolution, map/poll, and readback belong to G5. `GpuBufferInitialization` and
`GpuTextureInitialization` remain checked logical intent; `create_buffer_init`,
`queue.write_buffer`, `queue.write_texture`, staging, copies, and readback cannot create
a second G4C1 transfer authority. If creation-time initialization is later retained as
an optimization, it must be semantically equivalent to the one accepted G5 transfer
contract.

It leaves only `CurrentRenderResourceBridge` for the exact audited current consumers
whose operation remains in G4C2, G4C3, or G5. The bridge may lend purpose-typed,
affinity-validated resource references, but it does not perform those operations, parse
WGSL, or create modules, layouts, bind groups, or pipelines.

## G4C2 — program and binding realization

G4C2 owns:

- canonical WGSL module creation;
- mandatory resource-interface agreement;
- normalized observed stage-IO signatures;
- bind-group layouts;
- pipeline layouts;
- typed bind groups;
- their private registries and caches.

It consumes accepted G4C1 resource handles, migrates the resource terminals it now owns
to typed bind-group realization, and deletes `CurrentRenderResourceBridge` as a bridge.
Its single replacement, `CurrentRenderPipelineBridge`, carries forward only proven
residual G4C1 resource terminals together with the new G4C2 program, layout, and
bind-group terminals still required by current pipeline creation or unchanged encoding.

It does not own pipeline realization, G5 execution, or final cutover.

## G4C3 — pipeline realization and final cutover

G4C3 owns:

- compute pipeline realization;
- render pipeline realization;
- stage-IO compatibility;
- complete pipeline cache keys;
- migration of every remaining realization consumer;
- deletion of renderer-owned program/layout/bind-group/pipeline cache authority;
- deletion of synthetic handles and G4-owned sidecar truth.

It migrates the pipeline-bridge terminals it replaces and deletes
`CurrentRenderPipelineBridge`. It leaves only `CurrentRenderExecutionBridge`, carrying
the accepted resource, bind-group, and pipeline terminals needed by the unchanged G5
encoder. That bridge lends validated references only; G5 owns encoding and its deletion.

# Delivery and acceptance rules

Each child requires:

1. one explicit issue activated from exact accepted `main`;
2. one branch and one pull request;
3. exact current-main consumer, raw-reach-through, bridge-call-site, and deletion census;
4. focused deterministic and environment-dependent proof;
5. complete migration of authority owned by that child;
6. deletion of fully replaced authority and predecessor bridge without aliases or
   parallel paths, while the one successor retains only proven residual terminals;
7. source guards proving the successor bridge's exact terminals, call sites, and
   uniqueness;
8. `cargo validate`, `git diff --check`, and the production docs build;
9. repository-owned exact-head CI and Documentation Build;
10. independent complete-diff review;
11. accepted merge before the successor activates.

Issue `#188` remains the G4C umbrella. It is not directly implemented.

# Exclusions retained

This correction does not authorize:

- G4B Rust before issue `#209` is accepted and issue `#187` is activated;
- any G4C child before accepted G4B and a separate activated child issue;
- WESL, Slang, HLSL, GLSL, SPIR-V, or compiler dependencies;
- a custom shader language;
- inferred WGPU layout authority;
- stable source, ABI, cache, capture, persistence, replay, wire, or cross-process handle
  formats;
- overlapping migration bridges or a generic raw-WGPU callback;
- G5 execution ownership;
- G7 surfaces, presentation, replacement, loss, or reconstruction;
- RunenRender image-formation implementation;
- package extraction;
- compatibility facades, source mirrors, generated prompts, task databases, truth
  certificates, execution locks, or temporary authoring workflows.

# Stop conditions

Stop and require another architecture decision when:

- one explicit resource interface cannot describe the shader-visible contract without
  renderer semantic identity;
- mandatory WGSL agreement would require reflection to become sole authority;
- source consistency cannot remain bounded and process-local;
- stage-IO compatibility cannot be expressed without moving vertex or target policy
  into the resource interface;
- a child cannot delete the authority or predecessor bridge it fully replaces while
  preserving only the audited still-required successor terminals;
- correct implementation requires overlapping bridges, a broad callback, or public raw
  backend access;
- one child must consume an unmerged predecessor branch;
- a stable persisted or public backend format becomes necessary;
- implementation crosses into G5, G7, RunenRender, or extraction ownership.
