---
title: RunenGPU G4B Contracts and G4C Delivery Design
description: Final ownership and delivery boundaries for source admission, shader-resource interfaces, mandatory WGSL agreement, and ordered WGPU realization.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-03
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
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G4B Contracts and G4C Delivery Design

## Status and authority

This design is the focused correction authorized by issue `#209` after accepted G4A
and the accepted shader-authoring boundary:

```text
G4A accepted             501b9fd58e56d33708573e47faf0e5026b5a1ff2
shader boundary accepted 23bc982703f93d15ac39dd71d61bae9e23854141
```

It narrows and supersedes only the conflicting G4B interface/reflection/source-owner
and G4C delivery details in the earlier G4 umbrella design. The earlier design remains
authoritative for the broader G4 mission, G4A ownership, private WGPU direction,
G5/G7 exclusions, and RunenRender boundary.

The implementation specifications own exact phase requirements. GitHub issues own live
activation and delivery. This document owns the durable architectural explanation.

## Problem corrected

The accepted G4 direction was sound, but four G4B decisions and one G4C delivery
boundary remained unsafe:

1. Shader-visible resource bindings were mixed with vertex-buffer and color-target
   pipeline state.
2. Reflection was described as optional even though explicit declarations require
   proof against the actual canonical WGSL accepted by the pinned backend.
3. `SourceRevisionConflict` existed without a stateful owner capable of enforcing it
   across admissions.
4. The public authoring path was described conceptually but not proven through concrete
   compute and render construction shapes.
5. Resource realization, WGSL/layout/bind-group realization, pipeline realization,
   migration, and deletion were combined into one broad G4C delivery.

The correction keeps one architecture while separating ownership and delivery.

# Final ownership model

## Runenwerk owns source production policy

Runenwerk owns:

- source roots and filesystem paths;
- direct WGSL, WESL, Slang, or future authoring-language policy;
- compiler pinning and normalized options;
- module/dependency discovery;
- deterministic canonical WGSL production;
- source maps and authoring diagnostics;
- file watching and reload scheduling;
- atomic publication and last-known-good product fallback;
- persisted artifact and reproducibility policy.

None of these concerns enters the future-transferable RunenGPU public contract.

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
binding keys and declarations
bind-group and pipeline-layout descriptors
specialization schemas and values
compute and render pipeline descriptors
runtime binding compatibility vocabulary
mandatory interface-agreement comparison
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

A source is identified by:

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

## Stateful owner

`GpuProgramSourceRegistry` is the one stateful G4B owner of source consistency.

It provides these semantics:

- equal owner/key/revision and byte-identical canonical WGSL is idempotent;
- equal owner/key/revision with different canonical WGSL is
  `SourceRevisionConflict`;
- a different revision is a distinct admitted record;
- admitting a new revision does not silently invalidate descriptors or realizations
  bound to an older revision;
- capacity and retained-source bytes are explicitly bounded;
- pressure is structured and cannot silently evict live authority;
- publication is transactional;
- no consumer callback executes under registry locks.

Runenwerk may choose capacity policy when creating the registry. Runenwerk does not
become the source-consistency implementation owner.

# Shader-resource interface boundary

## Interface owns bindings only

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
shader-resource bindings and do not belong in bind groups or the program resource
interface.

## Render output state remains pipeline state

The render pipeline descriptor also owns:

- ordered color-target formats;
- blend state;
- write masks;
- depth/stencil state;
- multisample state;
- primitive and other accepted graphics state.

Fragment output declarations in WGSL must agree with the selected entry point and
pipeline state, but color-target attachment policy is not part of the shader-resource
interface.

This separation avoids two failures:

- one program interface being needlessly split by different vertex-buffer or target
  configurations;
- resource-binding compatibility becoming contaminated with renderer attachment
  semantics.

# Mandatory canonical WGSL agreement

## Explicit declarations remain authority

RunenGPU does not infer its public interface from reflection. The explicit typed
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
- texture dimensions, sample/storage facts and multisample state;
- array cardinality;
- applicable host-layout facts.

Those observed facts are compared against the explicit G4B declarations.

The division of responsibility is exact:

```text
G4B
    owns expected facts
    owns normalized comparison vocabulary
    owns the single deterministic comparison algorithm

G4C2
    invokes pinned WGSL/WGPU/Naga parsing and reflection
    normalizes observed facts
    supplies them to the G4B comparison
```

There is no second validator with different semantics.

Reflection may reject disagreement and improve source-span diagnostics. It may not
renumber, add, remove, infer, or mutate authoritative declarations. A disagreement is a
structured `ProgramInterfaceMismatch`, not a fallback to inferred layouts.

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

Exact names may change during implementation, but the contract must preserve one
bounded fallible terminal per semantic object and no raw WGPU.

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

Vertex-buffer input and color-target output state remain visibly separate from the
resource interface.

## Compatibility shape

Prepared bytes and logical resources become eligible for G4C2 bind-group realization
only after explicit compatibility validation against the accepted resource interface.
Current derives may prepare bytes; they do not prove compatibility.

# Ordered G4C delivery

## G4C1 — resource realization

G4C1 owns private realization of:

- buffers;
- textures;
- texture views;
- samplers;
- query sets.

It also owns exact affinity checks, transactional resource registries, complete resource
cache compatibility, migration of resource creation/lookup consumers, and deletion of
fully replaced renderer-owned resource authority.

G4C1 does not parse WGSL or create modules, layouts, bind groups, or pipelines.

## G4C2 — program and binding realization

G4C2 owns:

- canonical WGSL module creation;
- mandatory parser/reflection agreement;
- bind-group layouts;
- pipeline layouts;
- typed bind groups;
- their private registries and caches.

It consumes accepted G4C1 resource handles. It does not create compute or render
pipelines and does not perform the final renderer-wide cutover.

## G4C3 — pipeline realization and final cutover

G4C3 owns:

- compute pipeline realization;
- render pipeline realization;
- complete pipeline cache keys;
- migration of every remaining current realization consumer;
- deletion of renderer-owned program/layout/bind-group/pipeline cache authority;
- deletion of synthetic handles and G4-owned sidecar truth;
- reduction of raw Device/Queue reach-through to one named scoped G5 bridge.

The bridge can lend already validated private references only while current execution
encodes accepted work. It cannot store, clone, cache, return, expose, or treat those
references as authority. G5 owns bridge deletion.

# Delivery and acceptance rules

Each child requires:

1. one explicit issue activated from exact accepted `main`;
2. one branch and one pull request;
3. exact current-main consumer and deletion census;
4. focused deterministic and environment-dependent proof;
5. complete migration of authority owned by that child;
6. deletion of fully replaced authority without aliases or parallel paths;
7. `cargo validate`, `git diff --check`, and the production docs build;
8. repository-owned exact-head CI and Documentation Build;
9. independent complete-diff review;
10. accepted merge before the successor activates.

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
- a G4C child cannot delete the authority it fully replaces in the same slice;
- correct implementation requires a second raw-WGPU bridge;
- one child must consume an unmerged predecessor branch;
- a stable persisted or public backend format becomes necessary;
- implementation crosses into G5, G7, RunenRender, or extraction ownership.
