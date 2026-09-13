---
title: "Repository Family Extraction Boundaries"
description: "Accepted repository-level ownership, dependency, clean-cutover, and integration rules for RunenSDF, RunenECS, RunenGPU, RunenRender, RunenUI, and Runenwerk."
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/accepted/runenecs-extraction-boundary-design.md
  - ../../design/accepted/runenrender-decomposition-design.md
related_roadmaps:
  - ../../workspace/planning/roadmap.md
---

# ADR 0014: Repository Family Extraction Boundaries

## Decision

Runenwerk is the integration and product repository for independently usable peer
frameworks:

```text
product       repository                 package       crate
RunenSDF      dornglut/runen-sdf         runen-sdf     runen_sdf
RunenECS      target dornglut/runen-ecs  topology refined by accepted RunenECS design
RunenGPU      dornglut/runen-gpu         runen-gpu     runen_gpu
RunenRender   dornglut/runen-render      runen-render  runen_render
RunenUI       dornglut/runen-ui          existing workspace; current packages include runenui_core and runenui_runtime
```

Historical `Crystonix/*` paths remain provenance only; active repository identity uses
the `dornglut/*` namespace.

RunenSDF, RunenGPU, and RunenUI have standalone repository authority. RunenECS and
RunenRender remain governed by their own accepted boundaries and current cutover state.
This ADR owns the durable dependency and ownership laws, not a duplicate copy of any
standalone framework's semantics or live work status.

Framework repositories must not depend on Runenwerk. Integration-specific translation,
application lifecycle, product policy, and cross-framework composition remain in
Runenwerk.

RunenGPU and RunenRender each begin with one public package. This decision does not
redefine RunenUI package topology. The accepted RunenECS extraction design refines its
initial topology to Cargo package `runen-ecs` / Rust crate `runen_ecs` plus the
technically required Cargo package `runen-ecs-macros` / Rust crate
`runen_ecs_macros` proc-macro companion. Additional packages require independently
useful dependency, backend, release, ABI, or compile-time pressure. Repository
extraction is not itself justification for package proliferation.

## Dependency direction

The accepted family direction is:

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

The default rule is framework independence. A direct framework dependency requires a
separate ADR proving independent value and correct ownership.

ADR 0015 accepts exactly one such foundational dependency:

```text
RunenRender -> RunenGPU
```

This dependency is valid because image formation requires GPU execution, while
RunenGPU remains independently useful for compute, simulation, field realization,
procedural tools, bakers, and offscreen workloads.

RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely because
an application may accelerate or display their outputs.

## Current program state

### RunenSDF

Current reusable RunenSDF architecture and implementation authority belongs to
[`dornglut/runen-sdf`](https://github.com/dornglut/runen-sdf/blob/main/ARCHITECTURE.md).
The corrected standalone source transfer completed at:

```text
repository: dornglut/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

`PT-RUNENSDF-004` later completed the retirement-only Runenwerk cutover after proving
zero real code consumers. Current Runenwerk source contains no tracked `domain/sdf`
package, workspace member, or duplicate implementation and carries no unused external
RunenSDF dependency; Runenwerk retains only product/world integration such as
`domain/world_sdf`. Exact migration and retirement evidence remains in the
[PT-RUNENSDF-004 closeout](../../reports/closeouts/pt-runensdf-004-internal-sdf-retirement-closeout.md).

### RunenECS

RunenECS work remains governed by its accepted repository-local boundary and standalone
repository authority where transferred. This ADR does not duplicate its current
implementation or live cutover status.

### RunenGPU

The standalone transfer and Runenwerk consumer cutover are complete. Current reusable
RunenGPU semantics, implementation, public API, conformance, validation, and framework
evolution belong to
[`dornglut/runen-gpu`](https://github.com/dornglut/runen-gpu/blob/main/ARCHITECTURE.md).

Runenwerk consumes `runen-gpu` through an exact accepted Git revision. At this review the
workspace pin is:

```text
77c7c8d5ad6922b6f46c6b25e31b1a224c1314a4
```

That pin is Runenwerk's integration compatibility claim; it is not a second semantic
authority and does not freeze standalone RunenGPU evolution. Runenwerk no longer keeps
active G-phase semantic designs as current framework authority.

### RunenRender

RunenRender remains Runenwerk-owned semantic-rendering authority until its separately
accepted external cutover. It depends only on public RunenGPU contracts and must not
reintroduce direct/private WGPU ownership or duplicate RunenGPU semantics.

### RunenUI

RunenUI remains an independent peer governed in its own repository. This ADR does not
select its internal APIs or implementation roadmap.

## Extraction order

Tracks may investigate in parallel, but structural changes that share manifests,
lockfiles, identity policy, or canonical architecture must be serialized or explicitly
rebased.

The durable dependency constraint is:

```text
standalone RunenGPU authority
    -> RunenRender implementation/proof
        -> standalone RunenRender cutover
```

Historical RunenGPU G-phase ordering and proof evidence are provenance, not a current
Runenwerk work queue. Current activation belongs to GitHub issues and the Engineering
Portfolio. Current reusable RunenGPU evolution belongs to `dornglut/runen-gpu`.

## Clean cutover

Every completed extraction must:

- preserve source provenance and licensing;
- establish independent validation and public downstream conformance;
- pin Runenwerk to an exact revision or exact pre-release version;
- migrate every active consumer;
- delete the original Runenwerk implementation in the completed cutover;
- remove temporary migration seams before merge;
- leave no compatibility package, forwarding namespace, source mirror, submodule,
  branch dependency, or writable parallel authority.

Cross-repository handoff follows Dornglut Engineering ADR 0008. Before successor
acceptance, a copied candidate may exist only on an unmerged successor branch while
Runenwerk remains the semantic source authority. After successor acceptance, the
accepted successor revision becomes the sole semantic source authority and any
still-present Runenwerk implementation is a frozen predecessor copy: it may exist only
for the serialized downstream cutover, may not receive independent semantic or
implementation changes, and must be deleted when consumers migrate or retired through
an explicit accepted reversal. The completed cutover still contains one implementation
copy and no compatibility or forwarding path.

If current Runenwerk has no real consumer for an extracted framework, removal of the
internal package does not require adding an unused external dependency.

## Ownership decisions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary, numerical
policy, bounds, composition, transforms, capabilities, and CPU reference queries. It
does not own Runenwerk geometry, world streaming, ECS, rendering, materials, GPU
execution, or product policy.

### RunenECS

RunenECS owns entity/component/resource lifecycle, storage and query semantics, deferred
structural mutation, system identity and access contracts, explicit ECS ordering/sets,
ECS schedule validation, deterministic serial reference execution, and explicit
reflection.

General spatial indexing, application/frame lifecycle scheduling, rendering extraction,
networking, replay, world policy, and product scheduling remain outside ECS core.

### RunenGPU

Reusable RunenGPU semantics are owned by the standalone repository. The durable boundary
retained here is that RunenGPU owns generic validated GPU execution rather than renderer,
domain, ECS, UI, window/event-loop, shader-filesystem, or product-recovery policy.
Runenwerk and RunenRender must consume that owner through public contracts rather than
restating its API or validation rules here.

### RunenRender

RunenRender owns prepared render scenes, views, providers/interactions,
materials/media, emitters, visibility, transport, radiance caches, reconstruction,
overlays, color, presentation intent, and lowering into RunenGPU workloads.

It does not own general GPU execution, WGPU directly, ECS extraction, field/SDF
mathematics, UI semantics, native-window policy, or Runenwerk lifecycle.

### RunenUI

RunenUI owns semantic UI, state, actions, focus, accessibility, layout/style, text, hit
testing, and renderer-neutral paint output. It does not depend on RunenRender or
RunenGPU by default.

### Runenwerk

Runenwerk owns application/product lifecycle scheduling, windows/event loops,
ECS/domain extraction, scene/world/material/SDF/UI/editor/simulation adapters, shader
source discovery and reload policy, product quality/capability selection, diagnostics
presentation, recovery, and integration evidence. It chooses when an ECS schedule runs
but does not redefine RunenECS-internal ordering, access, validation, deferred-command,
or reference-execution semantics.

## Adapter rule

A framework must remain useful without its Runenwerk adapter.

A Runenwerk adapter may depend on Runenwerk and one or more framework contracts when its
purpose is explicitly cross-framework translation. No framework depends back on the
adapter.

Adapters translate identities, prepared inputs, outputs, lifecycle facts, diagnostics,
and ownership. They must not:

- copy framework algorithms;
- mirror source;
- create writable parallel authority;
- hide dependency cycles;
- expose broad compatibility facades.

A reusable bridge is extracted only after at least one independent consumer proves that
the bridge itself has stable ownership.

## Shared infrastructure

Do not create a universal `RunenCore`, shared meta-framework, universal ID repository,
universal diagnostics repository, or generic plugin framework merely to make extraction
convenient.

Each repository owns values and identities whose invariants it defines. Adapters map
them explicitly and preserve upstream diagnostics/provenance.

## Versioning and formats

Before stable publication, cross-repository dependencies use exact revisions or exact
pre-release versions. Moving branches are forbidden.

Persisted source, artifact, trace, replay, cache, and wire formats each require a
separate owner, identifier, version, validation policy, and migration policy. Runtime
IDs are not silently promoted into persisted identity.

## Consequences

- Existing code location is implementation evidence, not permanent ownership.
- RunenSDF and RunenGPU have completed standalone authority transfer.
- Runenwerk consumes standalone frameworks at exact accepted revisions where integrated.
- Runenwerk does not keep active predecessor semantic designs after accepted authority
  transfer.
- RunenRender consumes RunenGPU through the standalone public boundary.
- RunenRender extraction remains separately governed by its accepted architecture and
  owning issues.
- RunenECS owns ECS-native scheduling semantics; Runenwerk owns application/product
  lifecycle scheduling and integration.
- RunenUI remains independent.
- Connector inspection does not substitute for executable validation.

## Rejected alternatives

Rejected:

- extracting current directories unchanged;
- one repository containing SDF, ECS, GPU, rendering, and UI;
- Git submodules or source mirrors;
- a universal shared-core repository;
- speculative multi-package repository skeletons;
- long-lived compatibility packages;
- making RunenUI or RunenSDF depend on rendering for integration convenience;
- retaining WGPU ownership in RunenRender;
- moving Runenwerk-specific product policy into framework packages;
- creating a generic scheduler framework merely to host ECS scheduling semantics;
- retaining Runenwerk-local RunenGPU semantic documents as current authority after the
  standalone transfer.

## Fitness functions

The program succeeds only when:

- each framework validates independently;
- Runenwerk consumes exact revisions through one-way public dependencies;
- independent downstream consumers use public APIs;
- framework repositories contain no Runenwerk assumptions;
- adapters translate rather than duplicate algorithms;
- original Runenwerk implementations are removed after cutover;
- no dependency cycle, source mirror, compatibility authority, or duplicate path remains;
- provenance, licensing, compatibility, and current documentation are complete.
