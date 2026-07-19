---
title: Roadmap
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
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Roadmap

This roadmap contains current and future execution authority only. Detailed
historical UI phases remain in [Completed Work](completed-work.md), closeout
reports, pull requests, and Git history.

## Program destination

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration/adapters --> applications
RunenRender ----+
RunenUI --------+   (separate external workstream)
```

Framework repositories do not depend on Runenwerk. Runenwerk owns cross-domain
integration and products.

## Current program

### PT-REPOSITORY-FAMILY-000

ID: `PT-REPOSITORY-FAMILY-000`

Title: Repository Family Charter and Track Activation

State: `active-implementation` for one docs-only authority PR

Owner: workspace

Authority:

```text
../../architecture/repository-family-architecture.md
../../adr/accepted/0014-repository-family-extraction-boundaries.md
../../reports/investigations/repository-family-current-state-investigation.md
```

Evidence:

- current Runenwerk `main` inspected at
  `c078bd8609dc407d68269e86a1472c9234932213`;
- SDF manifest and public geometry/query coupling inspected;
- ECS, macro, scheduler, and duplicate spatial-index package boundaries inspected;
- render module root, graph, plugin, backend, and surface coupling inspected;
- PR #107 is closed unmerged;
- commit `b5e9624c...` is an incomplete deletion and not a valid extraction base.

Complete investigation gate: complete for repository-family direction and track
ordering.

Complete design gate: complete for this documentation/authority phase.

Known gaps: local documentation validation and owner review remain required
before merge.

Next action: merge this phase after docs validation, then activate the three
bounded investigation tracks below.

---

# RunenSDF track

## PT-RUNENSDF-001 — Complete extraction investigation

State: `queued`; becomes active after `PT-REPOSITORY-FAMILY-000` merges

Owner: SDF

Goal:

- read every source/test/benchmark/example;
- inventory every consumer;
- inventory every public API item;
- map all `geometry` coupling;
- inspect numerical, validation, serialization, and shader/GPU assumptions;
- produce an exact move/stay/redesign/delete matrix;
- run the current baseline.

Authority:

```text
../../design/active/runensdf-extraction-design.md
../../reports/investigations/repository-family-current-state-investigation.md
```

Implementation authorization: source changes forbidden.

Exit criteria:

- complete source and consumer inventory;
- public API inventory;
- evidence-backed design corrections;
- local Cargo baseline recorded;
- exact `SDF-002` implementation contract.

Next action: open one investigation/design PR from current `main`.

## PT-RUNENSDF-002 — Boundary correction

State: `blocked` by `PT-RUNENSDF-001`

Owner: SDF

Goal:

- replace public `geometry::Aabb3` and `geometry::Ray3` coupling;
- add repository-local validated bounds/ray/query settings;
- correct disjoint intersection and invalid-input semantics;
- remove `geometry` from the SDF manifest;
- add downstream/public conformance;
- migrate Runenwerk consumers through explicit adapters.

Implementation authorization: requires a separate exact phase spec after
`PT-RUNENSDF-001` acceptance.

Exit criteria:

- `domain/sdf` validates independently inside the workspace;
- no geometry dependency remains;
- all consumers and tests pass;
- external repository transfer is mechanical rather than architectural.

## PT-RUNENSDF-003 — Create RunenSDF and transfer source

State: `blocked` by `PT-RUNENSDF-002`

Owner: SDF/repository governance

Goal:

- create `Crystonix/RunenSDF`;
- establish governance, licenses, metadata, validation, and provenance;
- transfer corrected `runensdf` source and tests;
- validate the independent repository.

Manual dependency: repository creation must be available to the owner or an
installed connector/tool.

## PT-RUNENSDF-004 — Runenwerk cutover

State: `blocked` by `PT-RUNENSDF-003`

Goal:

- pin Runenwerk to the exact RunenSDF revision;
- migrate all consumers;
- delete `domain/sdf`;
- remove old workspace/lockfile entries;
- retain no compatibility package or source mirror;
- run full integration and runtime evidence.

## PT-RUNENSDF-005 — Closeout

State: `blocked` by `PT-RUNENSDF-004`

Goal: record provenance, compatibility, validation, deleted paths, and final
ownership; remove temporary branches and migration authority.

---

# RunenECS track

## PT-RUNENECS-001 — Complete ECS boundary investigation

State: `queued`; parallel investigation after `PT-REPOSITORY-FAMILY-000`

Owner: ECS

Goal:

- read ECS, macros, scheduler, spatial, networking, replay, renderer, and app
  consumers;
- inventory public API and unsafe/lifetime contracts;
- map scheduler ownership;
- map reflection registry behavior;
- classify broadcast, tick-buffer, and work-queue semantics;
- classify change extraction, ownership, replication, and replay boundaries;
- produce exact move/stay/redesign/delete matrices.

Authority:

```text
../../design/active/runenecs-extraction-boundary-design.md
```

Implementation authorization: source changes forbidden.

Exit criteria: all boundary decisions have evidence and `PT-RUNENECS-002` can be
specified without implementation invention.

## PT-RUNENECS-002 — Decision closure

State: `blocked` by `PT-RUNENECS-001`

Goal: accept final APIs and ownership for:

```text
runenecs
runenecs_macros
runen_schedule
spatial deletion/migration
explicit reflection registry
messaging families
change tracking versus replication
serial/parallel scheduling
identity and error policy
```

No extraction occurs in this phase.

## PT-RUNENECS-003 — Boundary repair

State: `blocked` by `PT-RUNENECS-002`

Goal:

- remove ECS geometry dependency and ECS-owned spatial index;
- remove Runenwerk-specific scheduler policy;
- make reflection explicit;
- prune/separate messaging;
- separate network/replay adapters;
- add standalone public conformance.

## PT-RUNENECS-004 — Create RunenECS and transfer packages

State: `blocked` by `PT-RUNENECS-003`

Goal: create the repository, transfer corrected `runenecs`,
`runenecs_macros`, and `runen_schedule`, preserve provenance, and validate
independently.

## PT-RUNENECS-005 — Runenwerk cutover

State: `blocked` by `PT-RUNENECS-004`

Goal: pin exact revisions, migrate all consumers, delete original packages,
regenerate the lockfile, and prove standalone plus product integration.

## PT-RUNENECS-006 — Closeout

State: `blocked` by `PT-RUNENECS-005`

Goal: record compatibility, performance, provenance, deleted paths, and final
ownership.

---

# RunenRender track

## PT-RUNENRENDER-001 — Complete semantic inventory

State: `queued`; parallel investigation after `PT-REPOSITORY-FAMILY-000`

Owner: render

Goal:

- read every render module, shader, macro, example, benchmark, and test;
- trace preparation, graph planning, resource creation, submission,
  presentation, surface lifecycle, and failure control flow;
- classify every item by future owner;
- identify current hard-coded UI, ECS, scene, material, SDF, editor, Winit, WGPU,
  and runtime coupling.

Authority:

```text
../../design/active/runenrender-decomposition-design.md
```

Implementation authorization: source changes forbidden.

Exit criteria: complete classification and exact `PT-RUNENRENDER-002` design
questions.

## PT-RUNENRENDER-002 — Seam and package design closure

State: `blocked` by `PT-RUNENRENDER-001`

Goal: accept public contracts for graph, producer submissions, resources,
surfaces, frames, synchronization, errors/device loss, diagnostics, macros, and
threading.

## PT-RUNENRENDER-003 — Neutral core separation

State: `blocked` by `PT-RUNENRENDER-002`

Goal:

- remove ECS and product types from neutral graph/resource/frame contracts;
- remove product-specific graph validation;
- add independent core tests;
- preserve behavior through Runenwerk adapters.

## PT-RUNENRENDER-004 — WGPU/backend separation

State: `blocked` by `PT-RUNENRENDER-003`

Goal:

- isolate WGPU device/resource/pipeline/execution ownership;
- split native-window mapping from WGPU surface ownership;
- define surface error and device-loss recovery contracts;
- add headless/GPU conformance.

## PT-RUNENRENDER-005 — Adapter migration and internal package proof

State: `blocked` by `PT-RUNENRENDER-004`

Goal: migrate ECS/scene/material/SDF/editor/UI/plugin concerns into Runenwerk
adapters and make Runenwerk consume only the intended external boundary.

## PT-RUNENRENDER-006 — Create RunenRender and transfer packages

State: `blocked` by `PT-RUNENRENDER-005`

Goal: create repository, transfer corrected core/WGPU packages, preserve
provenance, and validate independently.

## PT-RUNENRENDER-007 — Runenwerk cutover

State: `blocked` by `PT-RUNENRENDER-006`

Goal: pin exact revisions, remove original implementation, migrate applications,
and run complete CPU/headless/GPU validation.

## PT-RUNENRENDER-008 — Closeout

State: `blocked` by `PT-RUNENRENDER-007`

Goal: record compatibility, provenance, performance/platform evidence, deleted
paths, and final ownership.

---

# Retired current-work authority

## PT-UI-RUNTIME-PLATFORM-012

State: `superseded/closed-unmerged`

PR #107 was closed without merge. It is not active implementation authority.

The separate RunenUI repository/workstream now owns standalone reusable UI
framework development. Historical Runenwerk UI phases remain evidence only.

No work in this roadmap authorizes RunenUI changes or immediate deletion of
Runenwerk UI source.

## Rejected extraction attempt

Commit `b5e9624c594c9f1e3f2a0929bf84028f13fde860` is classified
`REJECTED_EXTRACTION_ATTEMPT`. It must not be used as an extraction base.

---

# Coordination rules

- One implementation PR receives one bounded phase.
- New implementation branches start from current merged `main`.
- Shared root manifests, lockfile, and canonical planning files have one active
  owner at a time.
- Parallel investigation/design branches avoid shared-file edits after this
  charter merges.
- Source movement is the final step after boundary repair, not the first step.
- No source mirror, submodule, compatibility crate, or moving-branch dependency.
- RunenUI remains outside this program.
