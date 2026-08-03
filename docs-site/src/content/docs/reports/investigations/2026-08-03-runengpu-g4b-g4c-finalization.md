---
title: RunenGPU G4B Contract and G4C Delivery Finalization
description: Exact accepted-main investigation supporting the final G4B ownership decisions, stage-IO proof, and serialized G4C decomposition.
status: active
owner: gpu
layer: reports
last_reviewed: 2026-08-03
related_docs:
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runengpu-shader-authoring-artifact-boundary.md
  - ../../design/active/runengpu-g4b-contracts-g4c-delivery-design.md
  - ./runengpu-g4-context-program-realization-investigation.md
  - ./2026-07-31-runengpu-g4-critical-review.md
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
  - ../../workspace/specs/pt-runengpu-g4c1-resource-realization.ron
  - ../../workspace/specs/pt-runengpu-g4c2-program-binding-realization.ron
  - ../../workspace/specs/pt-runengpu-g4c3-pipeline-cutover.ron
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G4B Contract and G4C Delivery Finalization

## Record classification

| Field | Value |
|---|---|
| Evidence class | Dated planning and architecture investigation |
| Observation date | 2026-08-03 |
| Owning repository | `dornglut/runenwerk` |
| Owning work item | Issue `#209` |
| Accepted starting main | `23bc982703f93d15ac39dd71d61bae9e23854141` |
| Accepted G4A | `501b9fd58e56d33708573e47faf0e5026b5a1ff2` |
| Accepted shader boundary | `23bc982703f93d15ac39dd71d61bae9e23854141` |
| Implementation authorization | None |
| Acceptance authority | Owning issue, reviewed PR, exact-head validation, and repository owner |

This report records evidence and rationale. It does not authorize G4B or any G4C child
and does not replace the implementation specifications.

## Question

After accepted G4A and the shader-authoring clarification, is the existing G4B
specification sufficiently decision-complete for implementation, and is G4C safe as one
or three independent deliveries?

## Executive finding

No implementation should begin from the uncorrected G4B/G4C planning.

Six gaps are material:

1. the program interface combines shader-resource bindings with vertex-input and
   fragment/color-target pipeline state;
2. WGSL reflection is optional despite the need to prove canonical source agreement;
3. source revision conflict is named without one stateful consistency owner;
4. public ergonomics are not demonstrated through concrete compute/render shapes;
5. separating render pipeline state from the resource interface initially lost an
   explicit owner for vertex-input and fragment-output compatibility;
6. a three-way G4C split without temporary successor bridges leaves current consumers
   unable to operate after each ownership cutover unless duplicate renderer authority or
   broad raw-WGPU access is retained.

The smallest safe correction is:

```text
G4B source admission + logical contracts
    -> G4C1 resources
        -> CurrentRenderResourceBridge (deleted by G4C2)
    -> G4C2 programs/layouts/bind groups
        -> CurrentRenderPipelineBridge (deleted by G4C3)
    -> G4C3 pipelines/final realization cutover
        -> CurrentRenderExecutionBridge (deleted by G5)
```

Only one successor bridge remains at an accepted boundary. Every predecessor bridge is
deleted by the immediate successor.

## Evidence inspected

### Accepted repository authority

- `AGENTS.md`;
- `TESTING.md`;
- `ARCHITECTURE.md`;
- repository-family architecture;
- ADR 0015;
- RunenGPU architecture design;
- G4 umbrella design and investigation;
- G4A, G4B, and G4C phase specifications;
- roadmap and active-work documents.

### Accepted delivery evidence

- G4 planning issue `#182`, PR `#185`, and merge
  `62c3949d31a7c03f1f554f8108120d9767139123`;
- G4A issue `#186`, PR `#199`, review history, and merge
  `501b9fd58e56d33708573e47faf0e5026b5a1ff2`;
- shader boundary issue `#203`, PR `#204`, and merge
  `23bc982703f93d15ac39dd71d61bae9e23854141`;
- blocked implementation issue `#187`;
- blocked G4C umbrella issue `#188`;
- parent program issue `#167`.

### Historical evidence disposition

The 2026-07-31 G4 critical review remains dated changes-required evidence for its
recorded heads. It correctly identified the G4B interface/reflection/source-owner and
G4C breadth risks. It is not rewritten as current acceptance evidence.

## Finding 1: program interface contains pipeline state

### Observed authority

The prior G4B specification states that `GpuProgramInterfaceDescriptor` owns ordered
binding declarations plus vertex-input and fragment-output contracts.

### Problem

Shader-visible resources and render pipeline memory/attachment state have different
semantics:

```text
program resource interface
    group/binding identity
    visibility
    resource kind/access/shape
    minimum size and alignment
    array cardinality

render pipeline input/output state
    vertex slots/attributes/strides/step mode
    color target formats/blend/write masks
    primitive/depth-stencil/multisample state
```

Combining them causes otherwise identical resource interfaces to split when only vertex
buffer or attachment configuration changes and contaminates generic binding
compatibility with attachment policy.

### Decision

`GpuProgramInterfaceDescriptor` owns shader-visible resource declarations only.
Vertex-input and color-target state remain complete fields of the render pipeline
descriptor.

## Finding 2: reflection cannot remain optional

### Observed authority

The prior specification describes reflection as optional validation evidence while the
explicit interface is authoritative.

### Problem

Explicit declarations require proof that the actual canonical WGSL accepted by the
pinned backend agrees on entry points, groups/bindings, visibility, resource classes,
access, dimensions, formats, cardinality, and applicable host-layout facts.

Making reflection authoritative would create the opposite failure: public contracts
would become backend-inferred and unstable.

### Decision

Explicit declarations remain authority. WGSL parser/reflection agreement becomes
mandatory evidence before G4C2 module publication.

There is one comparison contract:

- G4B owns expected facts, normalized vocabulary, and deterministic comparison;
- G4C2 invokes the pinned WGPU/Naga path, normalizes observed facts, and supplies them
  to that comparison.

Resource disagreement is structured `ProgramInterfaceMismatch`; reflection cannot
mutate declarations or trigger inferred layouts.

## Finding 3: source conflict requires a stateful owner

### Observed authority

The prior G4B specification requires `SourceRevisionConflict` when one key/revision is
reused with different bytes, but no persistent in-process owner is defined.

### Problem

A pure descriptor constructor cannot detect conflicting admissions occurring at
different times or through different consumers. Enforcing consistency only in G4C
module caches is too late and couples logical source admission to backend realization.

### Decision

G4B owns one bounded process-local `GpuProgramSourceRegistry`.

Its authoritative tuple is:

```text
source owner
+ source key
+ nonzero revision
+ full canonical WGSL bytes
+ digest
+ bounded provenance
```

Equal tuple and bytes is idempotent. Equal owner/key/revision with different bytes is
`SourceRevisionConflict`. Different revisions coexist and do not silently invalidate
older admitted descriptors. Capacity and retained bytes are bounded and pressure is
structured.

## Finding 4: public ergonomics need executable shape

### Observed authority

The prior specification names compile-pass examples but does not bind an understandable
construction shape.

### Problem

A conceptually correct contract can still produce nested construction, backend leakage,
string authority, or ambiguous ownership during implementation.

### Decision

G4B proof must include concrete compile-shaped examples for:

- one compute source with storage input/output bindings;
- one render source with a uniform resource interface, separate vertex-buffer layouts,
  and separate color-target state;
- runtime binding compatibility;
- source conflict rejection;
- insertion-order-independent descriptor identity.

The intended style is private fields plus one bounded fallible builder/admission
terminal per semantic object. Exact names remain implementation details.

## Finding 5: stage-IO proof must survive interface separation

### Observed risk

Moving vertex-input and color-target state out of `GpuProgramInterfaceDescriptor` is
correct, but a resource-only interface does not by itself prove that:

- pipeline vertex attributes satisfy the selected WGSL vertex entry point;
- color-target classes satisfy the selected WGSL fragment outputs.

### Decision

G4B owns separate normalized observed stage-IO vocabulary and deterministic comparison
algorithms.

G4C2 retains, per selected entry point:

- unique vertex input locations and normalized scalar/vector classes;
- unique fragment output locations and normalized scalar/vector classes;
- builtins separately from user locations.

G4C3 compares those signatures against the complete render pipeline descriptor before
backend pipeline creation. Offsets, strides, step modes, target formats, blend state,
and write masks remain explicit pipeline policy and are not inferred.

Mismatch is structured `PipelineStageIoMismatch`.

## Finding 6: decomposition needs serialized migration bridges

### Observed risk

The proposed G4C1/G4C2/G4C3 type split creates intermediate operational dependencies:

1. Once G4C1 becomes the sole resource owner and deletes renderer resource registries,
   existing renderer bind-group/pipeline creation still needs private resource objects
   until G4C2 replaces binding realization.
2. Once G4C2 becomes the sole program/layout/bind-group owner and deletes renderer
   registries, existing renderer pipeline creation still needs private module/layout
   objects until G4C3 replaces pipeline realization.
3. Once G4C3 becomes the sole pipeline owner, current command encoding still needs
   private realized objects until G5 replaces execution ownership.

Without an explicit path, each child must choose one invalid alternative:

- retain old renderer registries as parallel authority;
- absorb successor work into the current child;
- expose raw WGPU broadly;
- break current consumers between accepted slices.

### Decision

Use a serialized successor bridge ladder:

```text
G4C1: CurrentRenderResourceBridge
      immediate owner: current renderer realization call sites
      deletion owner: G4C2

G4C2: CurrentRenderPipelineBridge
      immediate owner: current renderer pipeline creation call sites
      deletion owner: G4C3

G4C3: CurrentRenderExecutionBridge
      immediate owner: current execution encoding call path
      deletion owner: G5
```

Every bridge is:

- crate-private and process-local;
- narrowly typed to audited call sites;
- based on validated opaque handles;
- context/generation/kind checked;
- non-authoritative;
- unable to store, clone, cache, return, persist, or use borrowed backend objects as
  identity;
- forbidden from exposing public access, `Deref`, `AsRef`, a reusable raw handle, a
  generic raw-WGPU callback, arbitrary consumer closure, or broad `Device`/`Queue`;
- source-guarded with exactly one immediate deletion owner.

A predecessor bridge must have zero references before its successor bridge is accepted.
Two realization migration bridges never remain active at one accepted boundary.

## Finding 7: G4C must remain independently reviewable

The full original G4C delivery combines resources, programs/modules, layouts, bind
groups, pipelines, caches, consumer migration, synthetic-handle deletion, raw WGPU
containment, and sidecar cleanup. Commit boundaries are insufficient because they do
not provide independent activation, exact-head review, merge evidence, or accepted-main
dependency gates.

### Decision

Retain `#188` as a non-implementable umbrella and define:

#### G4C1

- buffers, textures, views, samplers, query sets;
- affinity and transactional registries;
- resource cache compatibility;
- resource authority migration/deletion;
- `CurrentRenderResourceBridge` only.

#### G4C2

- canonical WGSL modules;
- mandatory resource-interface agreement;
- observed stage-IO signatures;
- bind-group layouts, pipeline layouts, typed bind groups;
- deletion of `CurrentRenderResourceBridge`;
- `CurrentRenderPipelineBridge` only.

#### G4C3

- compute/render pipelines;
- stage-IO agreement;
- complete pipeline cache keys;
- final current-consumer and authority cutover;
- deletion of `CurrentRenderPipelineBridge`;
- `CurrentRenderExecutionBridge` only.

Each child requires its own accepted predecessor, issue, branch, PR, exact-head CI,
complete-diff review, migration/deletion/bridge census, and accepted merge.

## Alternatives considered

### Keep the prior G4B specification unchanged

Rejected. Architecture decisions would be made inside the Rust PR and duplicate
interface or source authority would be likely.

### Make reflection authoritative

Rejected. It would collapse explicit framework contracts into backend inference.

### Enforce source conflicts only in G4C caches

Rejected. Source admission is a logical contract and must not depend on backend object
creation or cache state.

### Drop stage-IO proof after separating pipeline state

Rejected. Correct resource bindings do not prove vertex attributes or color targets
match the selected entry points.

### Keep old renderer registries until G4C3

Rejected. G4C1/G4C2 would become unused parallel implementations rather than accepted
ownership cutovers.

### Expose a broad raw-WGPU callback between children

Rejected. It would create a de facto public/internal escape hatch with no bounded
semantic surface or deletion proof.

### Collapse G4C1/G4C2 into G4C3

Rejected. It recreates the broad delivery that issue `#209` is correcting.

### Create separate packages now

Rejected. Internal conformance and clean-cutover proof remain incomplete; extraction is
a later GX decision.

## Current authority after this correction

```text
broad G4 architecture
    runengpu-g4-context-program-realization-design.md

shader authoring/toolchain boundary
    runengpu-shader-authoring-artifact-boundary.md

corrected G4B/G4C architecture
    runengpu-g4b-contracts-g4c-delivery-design.md

exact phase requirements
    pt-runengpu-g4b-program-interface-layout.ron
    pt-runengpu-g4c-wgpu-realization-cutover.ron
    pt-runengpu-g4c1-resource-realization.ron
    pt-runengpu-g4c2-program-binding-realization.ron
    pt-runengpu-g4c3-pipeline-cutover.ron

live activation and delivery
    issues #209, #187, #188, and later accepted child issues
```

## Required next action

Accept this documentation-only correction first. After its reviewed merge and
accepted-main validation:

1. update and explicitly activate issue `#187` from exact accepted `main`;
2. implement G4B only;
3. create and activate one G4C1 child issue after accepted G4B;
4. leave G4C2/G4C3 blocked until their accepted predecessors;
5. keep G5, G7, RunenRender implementation, and extraction unauthorized.

## Evidence limitations

This planning work does not execute future G4B/G4C Rust tests and does not prove pinned
WGPU/Naga behavior against real adapters. Those are implementation proof obligations
assigned to G4C2 and G4C3. Repository-owned validation for this documentation change
remains required at the exact pull-request head.
