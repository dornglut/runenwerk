---
title: Repository Family Extraction Boundaries
description: Accepted repository-level ownership, dependency, clean-cutover, and integration rules for RunenSDF, RunenECS, RunenGPU, RunenRender, RunenUI, and Runenwerk.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-08-25
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
related_roadmaps:
  - ../../workspace/planning/roadmap.md
---

# ADR 0014: Repository Family Extraction Boundaries

## Decision

Runenwerk is the integration and product repository for independently usable peer
frameworks:

```text
product       repository                package / topology
RunenSDF      dornglut/runen-sdf        runen-sdf / runen_sdf
RunenECS      dornglut/runen-ecs        runen-ecs / runen_ecs
                                         + runen-ecs-macros / runen_ecs_macros
RunenGPU      dornglut/runen-gpu        runen-gpu / runen_gpu
RunenRender   dornglut/runen-render     runen-render / runen_render
RunenUI       dornglut/runen-ui         existing workspace topology
```

The table uses current Dornglut repository identities. Historical Crystonix paths
remain provenance only and are not active repository authority.

RunenSDF and RunenUI already exist as independent workstreams. RunenECS,
RunenGPU, and RunenRender remain governed by their accepted boundary designs and
clean-cutover evidence.

Framework repositories must not depend on Runenwerk. Integration-specific
translation, application lifecycle, product policy, and cross-framework
composition remain in Runenwerk.

RunenGPU and RunenRender each begin with one public package. RunenECS begins with
the `runen-ecs` library package plus the technically required
`runen-ecs-macros` proc-macro companion defined by its active extraction design.
This decision does not redefine RunenUI package topology. Further packages require
independently useful dependency, backend, release, ABI, or compile-time pressure.
Repository extraction is not itself justification for package proliferation.

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

The default rule is framework independence. A direct framework dependency requires
a separate ADR proving independent value and correct ownership.

ADR 0015 accepts exactly one such foundational dependency:

```text
RunenRender -> RunenGPU
```

This dependency is valid because image formation requires GPU execution, while
RunenGPU remains independently useful for compute, simulation, field realization,
procedural tools, bakers, and offscreen workloads.

RunenSDF, RunenECS, and RunenUI do not depend on RunenGPU or RunenRender merely
because an application may accelerate or display their outputs.

## Current program state

### RunenSDF

The corrected standalone repository transfer historically completed at:

```text
repository: dornglut/runen-sdf
commit: d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

Later RunenSDF state is owned by current repository-family architecture, code,
and GitHub history; this ADR does not mirror changing live migration state.

### RunenECS

Internal ownership and safety repairs remain required before source movement.
The active RunenECS boundary design and execution plan refine package topology,
ECS-native scheduling ownership, repair order, and the separate transfer gate.

### RunenGPU and RunenRender

The previous renderer-owned-WGPU target is superseded. GPU execution must be
separated before rendering extraction. Current implementation phase and maturity
state are owned by the applicable roadmap, issues, and current architecture rather
than this ADR.

### RunenUI

RunenUI remains an independent peer governed in its own repository. This ADR does
not select its internal APIs or implementation roadmap.

## Extraction order

Tracks may investigate in parallel, but structural changes that share manifests,
lockfiles, identity policy, or canonical architecture must be serialized or
explicitly rebased.

The dependency-constrained family order is:

1. complete any separately approved framework-local repair and conformance;
2. extract/cut over a framework only through its separately accepted transfer
   boundary;
3. preserve foundational dependency order, including RunenGPU before
   RunenRender;
4. consider reusable cross-framework adapters only after public contracts
   stabilize.

RunenECS's internal C0-C9 repair order is owned by its active execution plan and
is not duplicated here.

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

Temporary duplication may exist only on an unmerged extraction branch.

If current Runenwerk has no real consumer for an extracted framework, removal of
the internal package does not require adding an unused external dependency.

## Ownership decisions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary,
numerical policy, bounds, composition, transforms, capabilities, and CPU reference
queries. It does not own Runenwerk geometry, world streaming, ECS, rendering,
materials, GPU execution, or product policy.

### RunenECS

RunenECS owns entity/component/resource lifecycle, storage and query semantics,
deferred structural mutation, system identity/access contracts, explicit semantic
ordering and sets, ECS schedule validation, deterministic serial reference
execution, and explicit reflection.

General spatial indexing, application/frame lifecycle, rendering extraction,
networking semantics, replay policy, world policy, and product scheduling remain
outside ECS core. Runenwerk owns application/product lifecycle scheduling and
cross-framework integration; independently owned framework semantics remain with
their respective repositories.

### RunenGPU

RunenGPU owns validated GPU capabilities, resources, access/lifetimes, workloads,
submissions, uploads/readback, low-level surfaces, backend outcomes, and WGPU as
the initial internal backend.

It does not own renderer semantics, simulations, fields, ECS, UI, windows/event
loops, shader filesystem policy, or product recovery.

### RunenRender

RunenRender owns prepared render scenes, views, providers/interactions,
materials/media, emitters, visibility, transport, radiance caches,
reconstruction, overlays, color, presentation intent, and lowering into RunenGPU
workloads.

It does not own general GPU execution, WGPU directly, ECS extraction, field/SDF
mathematics, UI semantics, native-window policy, or Runenwerk lifecycle.

### RunenUI

RunenUI owns semantic UI, state, actions, focus, accessibility, layout/style,
text, hit testing, and renderer-neutral paint output. It does not depend on
RunenRender or RunenGPU by default.

### Runenwerk

Runenwerk owns application and product lifecycle scheduling, windows/event loops,
ECS/domain extraction and framework invocation, scene/world/material/SDF/UI/editor/
simulation adapters, shader source discovery and reload policy, product
quality/capability selection, diagnostics presentation, recovery, and integration
evidence.

Runenwerk does not redefine ECS-internal schedule semantics merely because it
chooses when an ECS schedule runs.

## Adapter rule

A framework must remain useful without its Runenwerk adapter.

A Runenwerk adapter may depend on Runenwerk and one or more framework contracts
when its purpose is explicitly cross-framework translation. No framework depends
back on the adapter.

Adapters translate identities, prepared inputs, outputs, lifecycle facts,
diagnostics, and ownership. They must not:

- copy framework algorithms;
- mirror source;
- create writable parallel authority;
- hide dependency cycles;
- expose broad compatibility facades.

A reusable bridge is extracted only after at least one independent consumer proves
that the bridge itself has stable ownership.

## Shared infrastructure

Do not create a universal `RunenCore`, shared meta-framework, universal ID
repository, universal diagnostics repository, or generic plugin framework merely
to make extraction convenient.

Each repository owns values and identities whose invariants it defines. Adapters
map them explicitly and preserve upstream diagnostics/provenance.

## Versioning and formats

Before stable publication, cross-repository dependencies use exact revisions or
exact pre-release versions. Moving branches are forbidden.

Persisted source, artifact, trace, replay, cache, and wire formats each require a
separate owner, identifier, version, validation policy, and migration policy.
Runtime IDs are not silently promoted into persisted identity.

## Consequences

- Existing code location is implementation evidence, not permanent ownership.
- RunenECS source movement remains blocked by internal ownership/safety work and
  standalone conformance.
- RunenECS owns ECS-native schedule semantics; Runenwerk owns application/product
  lifecycle scheduling and integration.
- RunenGPU must be proven and extracted before RunenRender where that dependency
  is required by the current accepted family architecture.
- RunenRender consumes RunenGPU through a public one-way dependency.
- RunenUI remains independent.
- Connector inspection does not substitute for executable validation.
- No implementation contract is written before its current-source inventory is
  complete.

## Rejected alternatives

Rejected:

- extracting current directories unchanged;
- one repository containing SDF, ECS, GPU, rendering, and UI;
- Git submodules or source mirrors;
- a universal shared-core repository;
- speculative multi-package repository skeletons beyond technically required
  companions such as a proc-macro package;
- long-lived compatibility packages;
- making RunenUI or RunenSDF depend on rendering for integration convenience;
- retaining WGPU ownership in RunenRender;
- moving Runenwerk-specific product policy into framework packages;
- creating a generic scheduler framework merely to host ECS scheduling semantics.

## Fitness functions

The program succeeds only when:

- each framework validates independently;
- independent downstream consumers use public APIs;
- framework repositories contain no Runenwerk assumptions;
- adapters translate rather than duplicate algorithms;
- original Runenwerk implementations are removed after cutover;
- no dependency cycle, source mirror, compatibility authority, or duplicate path
  remains;
- provenance, licensing, compatibility, and current documentation are complete.
