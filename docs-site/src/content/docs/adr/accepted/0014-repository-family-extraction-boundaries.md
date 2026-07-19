---
title: Repository Family Extraction Boundaries
description: Accepted decision for extracting RunenSDF, RunenECS, and RunenRender as independent framework repositories while retaining Runenwerk integration ownership.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-19
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenrender-decomposition-design.md
related_roadmaps:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# ADR 0014: Repository Family Extraction Boundaries

## Decision

Create three independent framework repositories through governed clean cutovers:

- `Crystonix/RunenSDF`;
- `Crystonix/RunenECS`;
- `Crystonix/RunenRender`.

Runenwerk remains the integration and product repository. Framework repositories
must not depend on Runenwerk. Integration-specific translation, lifecycle,
product policy, and cross-domain composition remain in Runenwerk.

RunenUI is explicitly outside this program and remains governed by its separate
repository and workstream. This ADR neither blocks nor directs RunenUI work.

## Dependency Direction

The intended direction is:

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk adapters/integration --> applications
RunenRender ----+
RunenUI --------+   (future separate integration)
```

`RunenRender` must not require RunenECS, RunenSDF, or RunenUI. Those domains feed
it through Runenwerk-owned adapters and generic render contracts.

A direct dependency between framework repositories requires a later ADR. It is
not introduced merely to avoid writing an adapter.

## Extraction Order

Use different maturity levels in parallel:

1. RunenSDF: complete design, repair its local boundary, then extract first.
2. RunenECS: complete scheduler, spatial, reflection, messaging, and replication
   decisions before extraction.
3. RunenRender: complete semantic inventory and internal decomposition before
   external extraction.

RunenRender moves last because its current implementation combines neutral render
planning with WGPU, window/surface, ECS, scene, material, SDF, UI, editor,
diagnostics, and runtime integration concerns.

## Clean Cutover

Every extraction must:

- preserve source provenance and licensing;
- establish independent validation and public downstream conformance;
- pin Runenwerk to an exact revision or exact pre-release version;
- migrate all active consumers;
- delete the original Runenwerk implementation in the same completed cutover;
- remove temporary migration seams before merge;
- leave no compatibility crate, forwarding namespace, source mirror, or writable
  parallel authority.

Temporary duplication may exist only on an unmerged extraction branch.

## Ownership Decisions

### RunenSDF

RunenSDF owns reusable SDF mathematics and CPU reference queries. It does not own
Runenwerk geometry, world streaming, ECS, rendering, materials, or product policy.
Its public bounds and ray/query vocabulary must be repository-local.

### RunenECS

RunenECS owns ECS semantics. General spatial indexing, engine lifecycle,
networking policy, rendering extraction, and world policy are not automatically
ECS-owned. Scheduler ownership must be classified between ECS schedule semantics,
generic execution, and Runenwerk frame/tick policy before source movement.

### RunenRender

RunenRender owns only proven backend-neutral render contracts and a conventional
backend implementation. Runenwerk retains ECS extraction, scene/world/material/
SDF adapters, app lifecycle, editor policy, window/event-loop policy, and future
UI integration.

## Shared Infrastructure

Do not create a universal `RunenCore`, shared meta-framework, universal ID crate,
or universal diagnostics crate.

Each repository owns the values and identities whose invariants it defines.
Adapters map them explicitly. Diagnostics use repository-specific namespaces and
preserve upstream identity.

## Versioning And Formats

Before stable publication, cross-repository dependencies use exact revisions or
exact pre-release versions. Moving branches are forbidden.

Persisted source, artifact, trace, replay, cache, and wire formats each require a
separate owner, identifier, version, validation policy, and migration policy.
Rust API versioning does not implicitly version persisted data.

## Consequences

- Parallel work is allowed, but the tracks have different implementation gates.
- Shared workspace and planning files have one active owner at a time.
- RunenSDF provides the first extraction-workflow proof.
- RunenECS extraction is blocked by unresolved internal ownership decisions.
- RunenRender external extraction is blocked until internal package boundaries
  are proven through actual Runenwerk consumption.
- Existing code location is evidence, not ownership authority.

## Rejected Alternatives

Extracting all three current directories immediately was rejected because it
would turn unresolved internal coupling into cross-repository coupling.

A single repository containing SDF, ECS, and rendering was rejected because the
subsystems change for different reasons and have different consumers.

Git submodules and source mirrors were rejected because they preserve operational
and ownership ambiguity.

A universal shared-core repository was rejected because it would attract
unrelated identities, diagnostics, geometry, plugin, and metadata concerns.

Long-lived compatibility crates were rejected because these repositories are
pre-1.0 and controlled by the same owner; a clean coordinated cutover is safer.

## Fitness Functions

The program is successful only when:

- each framework validates independently;
- Runenwerk consumes it through one-way dependencies;
- independent downstream consumers use public APIs;
- framework repositories contain no Runenwerk assumptions;
- adapters contain translation rather than duplicate algorithms;
- original Runenwerk implementations are removed;
- no dependency cycle or source mirror remains;
- provenance, licensing, compatibility, and current documentation are complete.
