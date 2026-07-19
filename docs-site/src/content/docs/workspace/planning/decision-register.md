---
title: Decision Register
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ./active-work.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
---

# Decision Register

This register records current priority and lifecycle decisions. Detailed completed
phase evidence belongs in closeout reports, [Completed Work](completed-work.md),
pull requests, and Git history.

The previous long-form register was dominated by completed internal UI phase
history. That history remains recoverable and is no longer current planning
authority.

## Repository-family program decision

Date: 2026-07-19

Decision: Establish `PT-REPOSITORY-FAMILY` as the governing program for extracting
RunenSDF, RunenECS, and RunenRender while retaining Runenwerk as the integration
and product repository.

State transition:

```text
repository extraction idea/investigation
    -> accepted repository-family direction
    -> PT-REPOSITORY-FAMILY-000 active docs-only authority phase
```

Reason:

- SDF, ECS, and rendering have independent responsibilities and consumers;
- moving current directories unchanged would preserve accidental coupling;
- one umbrella architecture is required for dependency direction, release,
  diagnostics, identities, persisted formats, provenance, conformance, and clean
  cutovers;
- parallel work is useful only when tracks operate at appropriate maturity.

Authority:

```text
../../architecture/repository-family-architecture.md
../../adr/accepted/0014-repository-family-extraction-boundaries.md
../../reports/investigations/repository-family-current-state-investigation.md
```

Follow-up: validate and merge the charter, then activate three bounded
investigation tracks.

## Dependency-direction decision

Date: 2026-07-19

Decision:

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk adapters/integration --> applications
RunenRender ----+
RunenUI --------+   (separate future integration)
```

Framework repositories must not depend on Runenwerk. Direct dependencies between
framework repositories require a separate ADR.

Rejected alternatives:

- one shared framework monorepository;
- RunenRender depending directly on RunenECS, RunenSDF, or RunenUI;
- Git submodules;
- mirrored source;
- a universal `RunenCore` or shared-meta repository.

Follow-up: enforce one-way dependency and adapter tests in every extraction.

## Parallel track decision

Date: 2026-07-19

Decision: Start parallel work at different maturity levels rather than three
simultaneous extraction implementations.

```text
RunenSDF    complete investigation/design, then extract first
RunenECS    complete investigation and decision closure
RunenRender complete investigation and internal decomposition
```

Reason: all three tracks would otherwise conflict in root manifests, lockfile,
engine dependencies, architecture summaries, and shared planning files while
making regression attribution unreliable.

Follow-up: shared authority changes remain owned by the charter phase; track PRs
use track-specific files until lifecycle transitions.

## RunenSDF first-extraction decision

Date: 2026-07-19

Decision: Make RunenSDF the first extraction implementation candidate.

Evidence:

- `domain/sdf` is one small package;
- its dependencies are only `glam` and Runenwerk `geometry`;
- confirmed public coupling is concentrated in field bounds, primitive bounds,
  transforms, and raymarch input;
- the initial target can remain one crate.

Fixed design:

- package name `runensdf`;
- repository-local validated bounds and ray/query vocabulary;
- no Runenwerk geometry dependency;
- no initial GPU, shader, program, macro, or multi-crate split;
- explicit numerical, validation, error, and conformance policy;
- one final clean Runenwerk cutover.

Follow-up: `PT-RUNENSDF-001` complete investigation before source changes.

## RunenECS boundary decision

Date: 2026-07-19

Decision: Do not extract current ECS directories unchanged.

Fixed design:

- repository packages are `runenecs`, `runenecs_macros`, and context-generic
  `runen_schedule`;
- `runen_schedule` remains usable without `runenecs`;
- ECS-owned spatial indexing is removed;
- ECS core has no Runenwerk geometry dependency;
- Runenwerk owns spatial adapters, engine lifecycle, rendering extraction, and
  network/replay product policy;
- reflection uses explicit registry authority;
- messaging families require semantic review before transfer;
- serial execution is the reference until parallel safety/determinism is proven.

Evidence:

- current ECS exports spatial hash/index types based on `geometry::Aabb3`;
- the workspace already contains separate `spatial` and `spatial_index` domains;
- current scheduler is generic over context but contains product-specific logging
  policy;
- the ECS public root combines storage, spatial, scheduling, reflection,
  messaging, ownership, and change extraction.

Follow-up: `PT-RUNENECS-001` complete investigation; no source movement before
`PT-RUNENECS-002` decision closure and `PT-RUNENECS-003` boundary repair.

## RunenRender decomposition decision

Date: 2026-07-19

Decision: RunenRender must be decomposed and proven inside Runenwerk before
external repository extraction.

Fixed design:

- candidate packages: `runenrender_core` and `runenrender_wgpu`;
- macros move only if independently justified;
- core has no WGPU, Winit, ECS, SDF, UI, scene, material-authoring, or Runenwerk
  dependency;
- WGPU backend owns WGPU resources, execution, surfaces, and presentation;
- Runenwerk owns native-window mapping, app lifecycle, ECS extraction, domain
  adapters, editor/debug policy, and future UI integration;
- product-specific graph validation is removed from renderer core;
- Runenwerk must consume the internal separation through the intended external
  public boundary before extraction.

Evidence:

- current renderer is one engine plugin with graph, backend, features, frame,
  material, procedural, residency, shader, runtime, editor, UI, scene, world,
  SDF, and debug responsibilities;
- `RenderPlugin` initializes product/domain resources and schedules directly
  against Runenwerk and UI schedule sets;
- graph exports `validation_builtin_ui`;
- current surface registry combines `NativeWindowId`, ECS derives, renderer
  identity, diagnostics, and WGPU configuration helpers.

Follow-up: `PT-RUNENRENDER-001` complete semantic inventory. External extraction
remains blocked through the internal anti-cheating proof.

## RunenUI workstream separation decision

Date: 2026-07-19

Decision: RunenUI work is not governed by this program or thread.

Reason: RunenUI has separate repository authority, roadmap, PR sequence, and
implementation ownership. Mixing it into the SDF/ECS/Render extraction program
would create duplicate management and unrelated blockers.

Consequence: this program may require generic future producer seams but must not
design or implement RunenUI APIs.

## Internal UI Phase 012 supersession decision

Date: 2026-07-19

Decision: Supersede `PT-UI-RUNTIME-PLATFORM-012 — Runtime Counter App Product` as
current Runenwerk implementation authority.

State transition:

```text
active-implementation authorization
    -> closed-unmerged historical evidence
```

Evidence: PR #107 was closed without merge. The work does not represent current
repository-family direction.

Consequence: no new Counter product or internal UI framework implementation is
authorized by current planning.

## Rejected extraction commit decision

Date: 2026-07-19

Decision: Classify commit
`b5e9624c594c9f1e3f2a0929bf84028f13fde860` as
`REJECTED_EXTRACTION_ATTEMPT`.

Reason: it deleted source before completing workspace, dependency, consumer,
lockfile, renderer, tests, assets, and documentation cutover.

Consequence: do not merge, rebase, or use it as an implementation base. Retain it
only as evidence that deletion belongs at the end of a clean cutover.

## Clean-cutover decision

Date: 2026-07-19

Decision: Every framework extraction uses one coherent final cutover with:

- corrected boundary before transfer;
- independent public conformance;
- exact revision pinning;
- complete consumer migration;
- deletion of original source;
- no compatibility package, forwarding namespace, mirror, submodule, or moving
  branch dependency;
- provenance, licensing, validation, and closeout evidence.

Temporary parallel source may exist only on an unmerged extraction branch.
