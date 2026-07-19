---
title: Repository Family Extraction Boundaries
description: Accepted repository-level decision for extracting RunenSDF, RunenECS, and RunenRender while retaining Runenwerk integration ownership and independent RunenUI authority.
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
must not depend on Runenwerk. Integration-specific translation, application
lifecycle, product policy, and cross-domain composition remain in Runenwerk.

RunenUI remains an independent peer repository governed by its own workstream.
This ADR fixes only the repository relationship:

- RunenUI does not depend on Runenwerk or RunenRender by default;
- RunenRender does not depend on RunenUI;
- Runenwerk may later translate accepted renderer-neutral RunenUI output into
  generic RunenRender work;
- standalone RunenUI backends may exist without becoming RunenRender authority.

This ADR does not select RunenUI APIs or authorize RunenUI implementation.

## Dependency direction

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk adapters/integration --> applications
RunenRender ----+
RunenUI --------+
```

A direct dependency between framework repositories requires another ADR proving
independent value and correct ownership. It is not introduced merely to avoid an
adapter.

## Extraction order

The tracks may progress at different maturity levels:

1. RunenSDF: verify the bounded source/API investigation, correct the local
   numerical and geometry boundary, then extract first.
2. RunenECS: complete scheduler, spatial, reflection, messaging, change, safety,
   and replication ownership decisions before source movement.
3. RunenRender: complete semantic inventory, decompose internally, and prove the
   intended public seams through Runenwerk consumption before external transfer.

RunenRender moves last because the current engine plugin combines neutral render
planning with WGPU, windows/surfaces, ECS, scene, material, SDF, UI, editor,
diagnostics, and runtime integration.

## Clean cutover

Every completed extraction must:

- preserve source provenance and licensing;
- establish independent validation and public downstream conformance;
- pin Runenwerk to an exact revision or exact pre-release version;
- migrate all active consumers;
- delete the original Runenwerk implementation in the same completed cutover;
- remove temporary migration seams before merge;
- leave no compatibility package, forwarding namespace, source mirror,
  submodule, or writable parallel authority.

Temporary duplication may exist only on an unmerged extraction branch.

## Ownership decisions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary,
numerical policy, spatial bounds, composition, and CPU reference queries. It does
not own Runenwerk geometry, world streaming, ECS, rendering, materials, or product
policy.

The exact field-sample and query contracts remain track-level design decisions and
must distinguish algorithmically safe distance estimates from values that cannot
support sphere tracing.

### RunenECS

RunenECS owns ECS semantics, not a permanently fixed storage implementation.
General spatial indexing, engine lifecycle, rendering extraction, networking,
replay, and world policy are outside ECS core.

Scheduler ownership is divided between neutral scheduling, ECS integration, and
Runenwerk frame/tick policy. Messaging and change-journal facilities remain
provisional until consumer evidence proves their independent ECS role.

### RunenRender

RunenRender owns only proven backend-neutral render contracts and a conventional
WGPU backend. Runenwerk retains ECS extraction, scene/world/material/SDF/UI
adapters, application lifecycle, editor policy, native-window/event-loop policy,
and product feature selection.

The required initial package candidates are `runenrender_core` and
`runenrender_wgpu`. A proc-macro package is conditional on retaining and proving
the current GPU derives. WGSL/WGPU layout semantics are backend-specific and must
not leak into renderer-neutral core authority.

## Shared infrastructure

Do not create a universal `RunenCore`, shared meta-framework, universal ID crate,
or universal diagnostics crate.

Each repository owns values and identities whose invariants it defines. Adapters
map them explicitly. Diagnostics use repository-specific namespaces and preserve
upstream identity.

## Versioning and formats

Before stable publication, cross-repository dependencies use exact revisions or
exact pre-release versions. Moving branches are forbidden.

Persisted source, artifact, trace, replay, cache, and wire formats each require a
separate owner, identifier, version, validation policy, and migration policy.
Rust API versioning does not implicitly version persisted data.

## Consequences

- Parallel investigation is allowed, but implementation gates differ by track.
- Shared workspace and planning files have one active owner at a time.
- RunenSDF provides the first extraction-workflow proof.
- RunenECS source movement is blocked by unresolved safety and ownership work.
- RunenRender external extraction is blocked until internal public-boundary proof.
- Existing code location is implementation evidence, not permanent ownership.
- Connector-only inspection cannot satisfy command-validation gates.

## Rejected alternatives

The following are rejected:

- extracting all current directories immediately;
- one repository containing SDF, ECS, and rendering;
- Git submodules or source mirrors;
- a universal shared-core repository;
- long-lived compatibility packages;
- making RunenUI depend on RunenRender solely for Runenwerk integration;
- moving Runenwerk-specific product policy into framework cores.

## Fitness functions

The program succeeds only when:

- each framework validates independently;
- Runenwerk consumes it through one-way dependencies;
- independent downstream consumers use public APIs;
- framework repositories contain no Runenwerk assumptions;
- adapters translate rather than duplicate algorithms;
- original Runenwerk implementations are removed after cutover;
- no dependency cycle, source mirror, or compatibility authority remains;
- provenance, licensing, compatibility, and current documentation are complete.