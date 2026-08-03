---
title: RunenGPU G4B Contract and G4C Delivery Finalization
description: Exact accepted-main investigation supporting the final G4B ownership decisions and ordered G4C decomposition.
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
implementation delivery?

## Executive finding

No. The existing direction remains valid, but implementation must not begin from the
uncorrected specification.

Four G4B gaps are material:

1. the program interface combines shader-resource bindings with vertex-input and
   fragment/color-target pipeline state;
2. WGSL reflection is optional despite the need to prove the canonical source agrees
   with explicit declarations;
3. source revision conflict is named without one stateful consistency owner;
4. public ergonomics are not demonstrated through concrete compute and render shapes.

G4C is also too broad as one delivery. It combines resource realization, program and
binding realization, pipeline realization, complete consumer migration, broad raw-WGPU
containment, and deletion. Those concerns have different dependencies and review
surfaces.

The smallest safe correction is:

```text
G4B source admission + logical contracts
    -> G4C1 resource realization
        -> G4C2 program/layout/bind-group realization
            -> G4C3 pipeline realization and final cutover
```

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
buffer or attachment configuration changes. It also contaminates generic binding
compatibility with renderer-facing attachment state.

### Decision

`GpuProgramInterfaceDescriptor` owns shader-visible resource declarations only.
Vertex-input and color-target state remain complete fields of the render pipeline
descriptor. Entry points reference one resource interface; a render pipeline combines
that interface with separate input/output state.

## Finding 2: reflection cannot remain optional

### Observed authority

The prior specification describes reflection as optional validation evidence while the
explicit interface is authoritative.

### Problem

Explicit declarations are useful only when the actual canonical WGSL accepted by the
pinned backend is proven to agree. Without mandatory agreement, the framework can
publish an explicit interface that diverges from:

- entry-point names or stages;
- group/binding identities;
- visibility;
- resource classes and access;
- texture dimensions or formats;
- array cardinality;
- applicable host-layout facts.

Making reflection authoritative would create a different failure: public contracts
would become inferred, backend-shaped, and unstable.

### Decision

Explicit declarations remain authority. WGSL parser/reflection agreement becomes
mandatory evidence before G4C2 module publication.

There is one comparison algorithm:

- G4B owns expected facts, normalized comparison vocabulary, and deterministic
  comparison;
- G4C2 invokes the pinned WGPU/Naga path, normalizes observed facts, and supplies them
  to that comparison.

Mismatch is structured `ProgramInterfaceMismatch`; reflection cannot mutate the
explicit contract or trigger inferred layouts.

## Finding 3: source conflict requires a stateful owner

### Observed authority

The prior G4B specification requires `SourceRevisionConflict` when one key/revision is
reused with different bytes, but no persistent in-process owner is defined.

### Problem

A pure descriptor constructor cannot detect conflicting admissions that occur at
different times or through different consumers. Enforcing consistency only in G4C
module caches is too late and couples logical source admission to backend realization.

### Decision

G4B owns one bounded process-local `GpuProgramSourceRegistry`.

Its key contract is:

```text
source owner
+ source key
+ nonzero revision
+ full canonical WGSL bytes
+ digest
+ bounded provenance
```

Equal tuple and bytes is idempotent. Equal owner/key/revision with different bytes is
`SourceRevisionConflict`. Different revisions coexist; they do not silently invalidate
older admitted descriptors. Capacity and retained bytes are bounded and pressure is
structured.

Filesystem paths, compiler state, watcher state, reload scheduling, and last-known-good
policy remain Runenwerk-owned.

## Finding 4: public ergonomics need executable shape

### Observed authority

The prior specification names compile-pass examples but does not bind an understandable
construction shape.

### Problem

A conceptually correct contract can still create excessive nested construction,
backend leakage, string authority, or ambiguous ownership during implementation.

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

## Finding 5: G4C must be decomposed

### Observed authority

The previous G4C specification and issue `#188` define one delivery containing:

- resources;
- programs/modules;
- layouts;
- bind groups;
- pipelines;
- registries and caches;
- complete consumer migration;
- synthetic-handle deletion;
- raw Device/Queue containment;
- sidecar cleanup;
- temporary G5 bridge creation.

### Problem

The delivery has multiple irreversible cutovers and cannot be reviewed safely as one
broad branch. Resource realization is a prerequisite for typed bind groups. Program and
layout realization is a prerequisite for pipelines. Final deletion depends on all
prior owners being accepted.

### Decision

Retain `#188` as the umbrella outcome, but prohibit direct implementation.

#### G4C1

Own resources only:

- buffers;
- textures;
- texture views;
- samplers;
- query sets;
- affinity and transactional registries;
- resource cache compatibility;
- deletion of replaced resource authority.

#### G4C2

Own program and binding realization:

- canonical WGSL modules;
- mandatory parser/reflection agreement;
- bind-group layouts;
- pipeline layouts;
- typed bind groups;
- their caches and replaced-authority deletion.

#### G4C3

Own pipelines and final cutover:

- compute/render pipelines;
- complete cache keys;
- every remaining consumer migration;
- deletion of renderer-owned realization/cache authority;
- synthetic-handle and G4 sidecar cleanup;
- one named scoped G5 execution bridge.

Each child requires its own accepted predecessor, issue, branch, PR, exact-head CI,
complete-diff review, migration/deletion census, and accepted merge.

## Alternatives considered

### Keep the prior G4B specification unchanged

Rejected. Implementation would have to make architecture decisions inside the Rust PR,
which would obscure review and likely create duplicate interface or source authority.

### Make reflection authoritative

Rejected. It would collapse explicit framework contracts into backend inference and
make authoring frontends or pinned parser changes silently alter public layout meaning.

### Enforce source conflicts only in G4C caches

Rejected. Source admission is a logical contract and must not depend on backend object
creation or cache state.

### Keep one G4C issue and use many commits

Rejected. Commit boundaries do not provide independent activation, exact-head review,
merge evidence, or accepted-main dependency gates.

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

This planning work does not execute the future G4B/G4C Rust tests and does not prove
pinned WGPU/Naga reflection behavior against real adapters. Those are implementation
proof obligations assigned to G4C2. Repository-owned validation for this documentation
change remains required at the exact pull-request head.
