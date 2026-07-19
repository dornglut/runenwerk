---
title: RunenECS Boundary Repair Execution Plan
description: Dependency-ordered repair roadmap from current Runenwerk ECS boundaries to independently conformant RunenECS packages.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ./runenecs-extraction-boundary-design.md
  - ../../reports/investigations/runenecs-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
  - ../../workspace/planning/roadmap.md
---

# RunenECS Boundary Repair Execution Plan

## Purpose

Repair current ECS boundaries through small, dependency-ordered phases before any
external source transfer.

This plan records the whole destination so phases compose coherently. It does not
pre-authorize every phase. Only the next executable phase receives an active RON
specification.

## Sequence

```text
R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8 -> R9
```

Later phases may be investigated in parallel. Implementation cannot skip an unmet
prerequisite or an unclosed safety/ownership decision.

## R1 — Entity identity and structured core errors

Goal:

- make `Entity` opaque and world-local;
- define stale, double-free, cross-world, and generation-exhaustion behavior;
- retire exhausted slots rather than saturating into unsafe reuse;
- introduce structured entity/allocation/world errors;
- remove ordinary panic/error ambiguity from the touched path.

R1 does not redesign storage, bundles, queries, reflection, messaging, spatial,
scheduler, or networking.

Exit condition: entity lifecycle is invariant-preserving and all current consumers
compile through the reviewed public API.

## R2 — Atomic structural mutation

Goal:

- preflight bundle registration and structural changes;
- make insert/remove/spawn individually atomic for their documented scope;
- define command failure and batch semantics;
- ensure failure leaves no live partial entity or half-applied bundle.

Prerequisite: R1 entity/error contract.

## R3 — Query and SystemParam unsafe boundaries

Goal:

- inventory and document every unsafe storage/query/param bridge;
- seal low-level query implementation initially;
- harden world/query compatibility and duplicate mutable access rejection;
- make SystemParam derivation the supported extension path;
- prove safety with Miri/sanitizer and downstream tests.

Prerequisites: R1 and R2 stable lifecycle/mutation behavior.

## R4 — Explicit reflection and macro migration

Goal:

- remove process-global reflection authority;
- introduce explicit registry ownership and duplicate policy;
- separate Rust, registry-local, and stable schema identities;
- migrate derives to descriptor generation without hidden registration;
- add external compile-pass/fail tests.

Prerequisite: R3 public extension and safety contracts.

## R5 — Remove spatial and geometry ownership

Goal:

- remove geometry dependency from ECS core;
- remove ECS-owned general spatial hash/index APIs;
- migrate active consumers to a Runenwerk spatial adapter;
- retain only generic change observation required by the adapter.

Prerequisites: R2 mutation and R4 identity/reflection facts used by consumers.

## R6 — Messaging split

Goal:

- retain only independently justified ECS event/queue semantics;
- define retention, cursor, overflow, terminal, and payload-recovery behavior;
- move tick buffers/provenance and external ingress to Runenwerk;
- remove work/retry/ack semantics that are not ECS-owned.

Prerequisite: complete local consumer map. R5 should precede broad `World` surface
reduction to avoid conflicting migrations.

## R7 — Change, ownership, and networking separation

Goal:

- retain local change observation;
- retain a generic journal only if non-network consumers prove it;
- move tick/window lifecycle, ownership/interest routing, replication, replay,
  prediction, rollback, and transport policy to Runenwerk;
- define product/network entity mapping outside ECS core.

Prerequisites: R1 identities, R4 type registry, and R6 messaging ownership.

## R8 — Neutralize `runen_schedule`

Goal:

- remove Runenwerk phase and barrier enums;
- remove renderer/network/replay exceptions and process-global telemetry;
- expose neutral labels, access conflicts, deterministic stages/waves, and reports;
- keep serial reference execution;
- leave frame/tick/product lifecycle in Runenwerk.

Prerequisites: R3 system access and R7 lifecycle separation.

## R9 — Standalone conformance and performance baseline

Goal:

- prove `runenecs`, `runenecs_macros`, and `runen_schedule` without Runenwerk;
- add downstream public consumer and standalone simulation examples;
- complete Miri/sanitizer, stable, MSRV, Clippy, docs, benchmark, dependency, and
  feature validation;
- record exact move/stay/redesign/delete and provenance matrices.

Prerequisites: R1–R8 closed.

R9 completion authorizes repository-creation planning, not source transfer by
itself.

## Shared invariants

Every repair phase preserves:

- no Runenwerk geometry/spatial/network/render/product policy in target core;
- no process-global reflection or telemetry authority;
- structured errors at public framework boundaries;
- serial reference behavior until parallel equivalence is proven;
- no source mirror or compatibility package;
- no external repository movement before independent conformance;
- exact, truthful evidence for skipped and failed commands.

## Phase-spec policy

Only R1 has a concrete phase spec now.

At each closeout:

1. verify actual delivered behavior and updated consumers;
2. correct the remaining roadmap if implementation facts changed;
3. write the next phase spec from current main;
4. authorize exactly that phase.

Do not retain R2–R9 RON contracts written against pre-R1 assumptions.

## Parallel work

Allowed while R1 is implemented:

- read-only investigation for later phases;
- benchmark and Miri command discovery;
- consumer classification;
- test-gap documentation.

Forbidden:

- concurrent structural changes to R2–R8 paths;
- package renames or source transfer;
- broad `World` rewrite;
- speculative parallel executor;
- new spatial or shared-core repository.

## Final extraction gate

RunenECS repository creation remains blocked until R9 proves:

- framework-independent package graph;
- public downstream use;
- sound extension boundaries;
- accepted messaging/change/scheduler ownership;
- complete validation and performance baseline;
- exact provenance and clean-cutover plan.