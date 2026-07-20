---
title: Production Tracks
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-20
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ./active-work.md
  - ./roadmap.md
  - ./completed-work.md
---

# Production Tracks

The repository-family program runs tracks at different maturity levels. It does
not run multiple external source transfers simultaneously.

## Coordination model

```text
Primary track
    PT-RUNENSDF — standalone repository planning follows completed boundary repair

Parallel prepared tracks
    PT-RUNENECS — R1 specified, no source implementation authorized
    PT-RUNENRENDER — R1 specified, no source implementation authorized

Independent workstream
    RunenUI — not governed here
```

Shared manifests, lockfiles, root architecture, and canonical planning files have
one owner at a time. Track-specific work remains in track-specific branches until
a lifecycle transition requires shared authority updates.

## PT-REPOSITORY-FAMILY

State: governance baseline complete through PR #109

Purpose: own repository missions, dependency direction, adapter ownership,
versioning, diagnostics, identities, persisted formats, provenance, conformance,
clean cutovers, and cross-repository compatibility policy.

Future milestones:

```text
001 compatibility matrix and release-policy hardening after the first external repository exists
002 cross-repository conformance automation after at least two repositories exist
```

## PT-RUNENSDF

State: active program; `PT-RUNENSDF-002` complete, `PT-RUNENSDF-003` queued planning

Goal:

```text
Extract reusable signed-field mathematics and CPU queries into
Crystonix/RunenSDF with no Runenwerk geometry, ECS, world, material, renderer,
UI, or product dependency.
```

Milestones:

```text
001 investigation and numerical design                         complete
002 correct public/numerical boundary inside Runenwerk         complete through PR #116
003 create RunenSDF and transfer corrected source              queued planning; no authorization
004 cut Runenwerk over and delete domain/sdf                   blocked by 003
005 close provenance, compatibility, release, and ownership    blocked by 004
```

Current blocker: PT-003 must receive an exact repository-creation, provenance,
independent-conformance, and transfer specification. Repository creation may
require an owner action because the connector does not expose repository creation.

## PT-RUNENECS

State: architecture and first repair specification recorded; Rust implementation
not authorized

Goal:

```text
Produce independently conformant runenecs, runenecs_macros, and runen_schedule
packages without Runenwerk geometry, spatial, lifecycle, renderer, networking,
replay, or product policy.
```

Internal milestones:

```text
R1 entity identity and structured core errors
R2 atomic structural mutation
R3 query and SystemParam unsafe boundaries
R4 explicit reflection and macro migration
R5 remove spatial and geometry ownership
R6 messaging split
R7 change, ownership, and networking separation
R8 neutralize runen_schedule
R9 standalone conformance and performance baseline
```

Only R1 has a current phase specification. Repository creation remains blocked
until R9 acceptance.

## PT-RUNENRENDER

State: architecture and first decomposition specification recorded; Rust
implementation not authorized

Goal:

```text
Separate backend-neutral render planning and a conventional WGPU backend from
Runenwerk ECS, scene, world, material, SDF, UI, editor, native-window, lifecycle,
and product policy; prove the boundary internally before extraction.
```

Internal milestones:

```text
R1 neutral identities, errors, and dependency guards
R2 neutral graph and resource descriptors
R3 prepared frame inputs and generic producers
R4 GPU parameter and optional macro ABI
R5 shader and hot-reload boundary
R6 headless WGPU executor
R7 generic surfaces and device loss
R8 diagnostics, capture, and provenance split
R9 Runenwerk adapter migration
R10 internal conformance and performance proof
```

Only R1 has a current phase specification. External repository creation remains
blocked until R10 acceptance.

## RunenUI

RunenUI is an independent peer workstream. RunenUI and RunenRender do not depend
on one another by default. Runenwerk owns any future integration adapter after both
frameworks expose accepted public seams.

## Parallel execution rules

Allowed:

- read-only investigations for later ECS/renderer phases;
- consumer and unsafe-boundary classification;
- benchmark, Miri, shader, headless-GPU, and runtime-command discovery;
- documentation cleanup outside active implementation authority;
- PT-RUNENSDF-003 repository/provenance planning.

Forbidden without a new phase authorization:

- ECS or renderer structural implementation;
- external source transfer;
- Runenwerk SDF cutover or `domain/sdf` deletion;
- duplicate framework implementations;
- compatibility repositories or source mirrors;
- universal shared-core/meta repositories.
