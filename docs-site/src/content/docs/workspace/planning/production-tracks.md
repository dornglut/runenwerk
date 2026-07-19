---
title: Production Tracks
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
---

# Production Tracks

The repository-family program runs three tracks in parallel at different maturity
levels. It does not run three simultaneous external source moves.

## Coordination model

```text
Primary implementation track
    PT-RUNENSDF

Parallel investigation/design track
    PT-RUNENECS

Parallel investigation/internal-decomposition track
    PT-RUNENRENDER

External independent workstream
    RunenUI — not governed here
```

Shared planning and root files have one owner at a time. After the charter phase,
track PRs use track-specific investigation, design, proof, and closeout files
until a lifecycle transition requires shared authority updates.

## PT-REPOSITORY-FAMILY

Track ID: `PT-REPOSITORY-FAMILY`

Title: Repository Family Governance

Track type: architecture / governance / release coordination

State: active through `PT-REPOSITORY-FAMILY-000`

Goal:

```text
Define repository missions, dependency direction, adapter ownership, versioning,
MSRV/toolchain, diagnostics, identities, persisted formats, provenance,
conformance, clean cutovers, and shared-file coordination.
```

Current phase: `PT-REPOSITORY-FAMILY-000 — Repository Family Charter and Track Activation`

Milestones:

```text
000 Repository-family charter and track activation — active docs-only phase
001 Compatibility matrix and release-policy hardening — deferred until first external repository exists
002 Cross-repository conformance automation — deferred until at least two repositories exist
```

Current blocker: documentation validation and owner review of Phase 000.

Next action: merge Phase 000, then activate the three bounded investigation
phases from current `main`.

## PT-RUNENSDF

Track ID: `PT-RUNENSDF`

Title: RunenSDF Extraction

Track type: reusable framework extraction

State: queued; first extraction track after charter merge

Goal:

```text
Extract reusable SDF mathematics and CPU queries into Crystonix/RunenSDF with no
Runenwerk geometry, ECS, world, material, renderer, or product dependency.
```

Authority:

```text
../../design/active/runensdf-extraction-design.md
../../architecture/repository-family-architecture.md
../../adr/accepted/0014-repository-family-extraction-boundaries.md
```

Milestones:

```text
001 Complete source/test/consumer/public-API investigation
002 Correct geometry, validation, bounds, and query boundaries inside Runenwerk
003 Create RunenSDF and transfer corrected source
004 Cut Runenwerk over and delete domain/sdf
005 Closeout, provenance, compatibility, and branch cleanup
```

Complete investigation gate: incomplete for implementation; Phase 001 is next.

Complete design gate: target direction fixed; implementation details require the
Phase 001 evidence package.

Current blocker: complete consumer/test inventory and local baseline validation.

Activation condition: Phase 001 may start after the charter merges. Phase 002
requires a separate accepted implementation contract.

Next action: open exactly one `PT-RUNENSDF-001` investigation/design PR.

## PT-RUNENECS

Track ID: `PT-RUNENECS`

Title: RunenECS Extraction

Track type: reusable framework architecture and extraction

State: queued parallel investigation/design

Goal:

```text
Extract runenecs, runenecs_macros, and a context-generic runen_schedule package
without carrying Runenwerk spatial, geometry, lifecycle, renderer, networking,
replay, or product policy into the framework.
```

Authority:

```text
../../design/active/runenecs-extraction-boundary-design.md
../../architecture/repository-family-architecture.md
../../adr/accepted/0014-repository-family-extraction-boundaries.md
```

Milestones:

```text
001 Complete ECS/macros/scheduler/spatial/network/replay investigation
002 Close scheduler, spatial, reflection, messaging, replication, identity, error, and concurrency decisions
003 Repair boundaries and add standalone public conformance inside Runenwerk
004 Create RunenECS and transfer corrected packages
005 Cut Runenwerk over and delete original packages
006 Closeout, compatibility, performance, provenance, and branch cleanup
```

Fixed decisions:

```text
ECS core will not depend on Runenwerk geometry.
ECS-owned spatial indexing will be removed.
runen_schedule stays a separate context-generic package in the RunenECS repository.
Runenwerk owns engine lifecycle and network/replay product policy.
Reflection must use explicit registry authority.
```

Complete investigation gate: incomplete.

Complete design gate: incomplete for source changes.

Current blocker: broad public surface and unresolved scheduler/messaging/change
ownership.

Activation condition: Phase 001 investigation may run after charter merge. No ECS
source movement before Phase 002 acceptance and Phase 003 repair.

Next action: open exactly one `PT-RUNENECS-001` investigation PR.

## PT-RUNENRENDER

Track ID: `PT-RUNENRENDER`

Title: RunenRender Decomposition and Extraction

Track type: renderer architecture / internal decomposition / framework extraction

State: queued parallel investigation

Goal:

```text
Separate backend-neutral render planning and a conventional WGPU backend from
Runenwerk ECS, scene, world, material, SDF, UI, editor, window, lifecycle, and
product policy; prove the boundary internally; then extract RunenRender.
```

Authority:

```text
../../design/active/runenrender-decomposition-design.md
../../architecture/repository-family-architecture.md
../../adr/accepted/0014-repository-family-extraction-boundaries.md
```

Milestones:

```text
001 Complete module/shader/macro/example/benchmark/test/control-flow inventory
002 Close graph, producer, resource, surface, frame, sync, error/device-loss, diagnostic, macro, and threading decisions
003 Separate neutral renderer core inside Runenwerk
004 Separate WGPU backend and host/native-window mapping inside Runenwerk
005 Migrate Runenwerk adapters and prove public-boundary-only consumption
006 Create RunenRender and transfer corrected packages
007 Cut Runenwerk over and delete original implementation
008 Closeout, compatibility, GPU/platform evidence, provenance, and branch cleanup
```

Fixed decisions:

```text
RunenRender core must not depend on ECS, SDF, UI, scene, material authoring, Winit, WGPU, or Runenwerk.
RunenRender WGPU owns WGPU surfaces and backend execution.
Runenwerk owns NativeWindowId, event-loop policy, ECS extraction, and domain adapters.
Product-specific graph validation such as built-in UI validation is not renderer-core authority.
External extraction is forbidden until the internal anti-cheating proof passes.
```

Complete investigation gate: incomplete.

Complete design gate: incomplete.

Current blocker: renderer responsibilities are still combined in one engine plugin
and package.

Activation condition: Phase 001 investigation may run after charter merge.
Internal source changes require Phase 002 acceptance. External extraction requires
Phase 005 completion.

Next action: open exactly one `PT-RUNENRENDER-001` investigation PR.

## Retired internal UI track

`PT-UI-RUNTIME-PLATFORM-012` is superseded and closed unmerged. It is not an
active production track.

The separate RunenUI workstream is independent. Historical internal UI closeouts
remain evidence, not current implementation authorization.

## Track rules

- One phase per implementation PR.
- Investigation/design PRs do not move source unless explicitly authorized.
- New branches start from the latest merged `main`.
- Exact revision dependencies only; no moving branches.
- No submodules, source mirrors, compatibility crates, or universal shared core.
- No next phase activation before truthful prior-phase closeout or explicit
  supersession.
