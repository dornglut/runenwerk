---
title: RunenECS Boundary Repair Execution Plan
description: Dependency-ordered repair roadmap from current Runenwerk ECS boundaries to independently conformant RunenECS packages.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-08-25
related_docs:
  - ./runenecs-extraction-boundary-design.md
  - ../../reports/investigations/runenecs-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
  - ../../workspace/planning/roadmap.md
  - ../../reports/investigations/runenecs-issue-198-current-main-census.md
---

# RunenECS Boundary Repair Execution Plan

## Purpose

Repair current ECS boundaries through small, dependency-ordered phases before any
external source transfer.

This plan owns the one durable RunenECS repair sequence. GitHub issue `#198` owns
whether the current reconciliation work is accepted and active; investigation
reports provide source-grounded evidence but do not authorize phases. The former
`R1..R9` and `ECS-001..006` labels are retained only as historical mappings.

## Canonical sequence

```text
C0 -> C1 -> C2 -> C3 -> C4 -> C5 -> C6 -> C7 -> C8 -> C9
```

Later phases may be investigated read-only in parallel. Implementation cannot
skip an unmet prerequisite or an unclosed safety or ownership decision.

## C0 — Current-main census and authority reconciliation

Goal:

- inventory the exact current ECS, macro, scheduler, spatial, messaging, change,
  ownership, networking, reflection, safety, validation, and consumer surfaces;
- reconcile conflicting historical RunenECS sequence and ownership documents;
- leave one durable repair sequence and one explicit next implementation slice.

The command-verified evidence is recorded in the Issue 198 current-main census.
Completion and acceptance remain owned by `#198` and its pull request.

## C1 / R1 — Entity identity and structured core errors

Goal:

- make `Entity` opaque and world-local;
- carry an opaque process-local `WorldScopeId` in every entity token in addition
  to slot/index and generation;
- give each `World` exactly one matching scope and make its allocator emit only
  entities carrying that scope;
- validate scope before slot/generation so an entity from another world cannot
  alias a coincidentally equal local slot;
- allocate world scopes from checked, non-reusing process-local runtime identity
  space; scope exhaustion must fail world creation rather than wrap or reuse;
- define stale, double-free, unknown/cross-world, index-exhaustion, and
  generation-exhaustion behavior;
- retire exhausted slots rather than saturating into unsafe reuse;
- introduce structured entity/allocation/world errors;
- remove ordinary panic/error ambiguity from the touched path.

`WorldScopeId` is ECS runtime identity only. It is not serialized, persisted,
used as a network identity, or promoted to a universal Runen identity.

R1 does not redesign storage, bundles, queries, reflection, messaging, spatial,
scheduler, or networking.

Exit condition: entity lifecycle is invariant-preserving, cross-world rejection
is mechanically enforceable, and all current consumers compile through the
reviewed public API.

## C2 / R2 — Atomic structural mutation

Goal:

- preflight bundle registration and structural changes;
- make insert/remove/spawn individually atomic for their documented scope;
- define command failure and batch semantics;
- ensure failure leaves no live partial entity or half-applied bundle.

Prerequisite: C1 entity/error contract.

## C3 / R3 — Query and SystemParam unsafe boundaries

Goal:

- inventory and document every unsafe storage/query/param bridge;
- seal low-level query implementation initially;
- harden world/query compatibility and duplicate mutable access rejection;
- make SystemParam derivation the supported extension path;
- prove safety with Miri/sanitizer and downstream tests.

Prerequisites: C1 and C2 stable lifecycle/mutation behavior.

## C4 / R4 — Explicit reflection and macro migration

Goal:

- remove process-global reflection authority;
- introduce explicit registry ownership and duplicate policy;
- separate Rust, registry-local, and stable schema identities;
- migrate derives to descriptor generation without hidden registration;
- add external compile-pass/fail tests.

Prerequisite: C3 public extension and safety contracts.

## C5 / R5 — Remove spatial and geometry ownership

Goal:

- remove geometry dependency from ECS core;
- remove ECS-owned general spatial hash/index APIs;
- migrate active consumers to a Runenwerk spatial adapter over accepted
  RunenSpatial ownership;
- retain only generic ECS change observation required by the adapter.

Prerequisites: C2 mutation and C4 identity/reflection facts used by consumers.

## C6 / R6 — Messaging split

Goal:

- retain only independently justified ECS-local event/queue semantics;
- define retention, cursor, overflow, terminal, and payload-recovery behavior;
- move tick buffers/provenance and host/external ingress out of ECS;
- remove work/retry/ack semantics that lack an ECS-local owner.

Prerequisite: C5 should precede broad `World` surface reduction so spatial and
messaging migrations do not compete over the same mixed owner surface.

## C7 / R7 — Change, ownership, networking, replay, and lifecycle separation

Goal:

- retain ECS-local component/resource change observation;
- retain a generic local journal only if non-network consumers prove it;
- remove tick/window lifecycle and gameplay ownership/interest policy from ECS;
- keep reusable networking protocol/schema, replication consistency,
  session/authority, delivery/recovery, and separately accepted prediction or
  interest semantics in RunenNet;
- keep concrete ECS-to-RunenNet identity/state mapping, gameplay ownership and
  relevancy policy, frame/tick integration, and archival/editor replay policy in
  Runenwerk/application integration;
- keep raw ECS `Entity` distinct from stable network and persistence identities.

This phase consumes accepted RunenNet contracts; it does not redesign RunenNet.

Prerequisites: C1 identities, C4 type registry, and C6 messaging ownership.

## C8 / R8 — ECS-native schedule and access semantics

Goal:

- move system identity, ECS access facts, explicit ordering/sets, schedule
  validation, and deferred-command boundaries into the `runen_ecs` crate;
- distinguish semantic order from access incompatibility;
- remove the `ecs -> scheduler` dependency without introducing a replacement
  generic scheduler crate;
- keep deterministic standalone serial execution as the correctness/reference
  behavior;
- leave frame/tick/product lifecycle, host execution, and application barriers
  in Runenwerk;
- delete unsupported generic DAG/demo/DOT/filesystem/global-telemetry scheduler
  residue after consumer migration.

Prerequisites: C3 system-access safety and C7 lifecycle/network separation.

## C9 / R9 — Standalone conformance and performance baseline

Goal:

- prove Cargo package `runen-ecs` / Rust crate `runen_ecs` independently of
  Runenwerk and any generic scheduler dependency;
- prove Cargo package `runen-ecs-macros` / Rust crate `runen_ecs_macros` as the
  proc-macro companion while that split remains technically required;
- add a downstream public consumer and standalone simulation examples;
- complete Miri/sanitizer, stable, declared-MSRV, Clippy, docs, benchmark,
  dependency, feature, license, and provenance validation;
- record final move/stay/redesign/delete and clean-cutover evidence.

Prerequisites: C1-C8 closed.

C9 authorizes consideration of a separate transfer/cutover issue. It does not by
itself authorize populating the external repository, moving source, changing
Runenwerk dependencies, or deleting the internal implementation.

## Post-C9 transfer and cutover

External repository population and Runenwerk consumer cutover are a separately
accepted delivery boundary derived from current C9 evidence. That later work
covers source transfer, exact-revision adoption, consumer migration, deletion of
old source, migration-seam removal, and provenance closure. It is deliberately
not another pre-authorized C-phase.

## Shared invariants

Every repair phase preserves:

- no Runenwerk geometry/spatial/network/render/product policy in target ECS core;
- no duplicate RunenNet semantic authority in RunenECS or Runenwerk adapters;
- no process-global reflection or telemetry authority;
- structured errors at public framework boundaries;
- deterministic serial reference behavior until parallel equivalence is proven;
- no source mirror or compatibility package;
- no external source movement before independent conformance and separate
  transfer authority;
- exact, truthful evidence for skipped, unavailable, and failed commands.

## Handoff policy

GitHub issues own activation and current work state. After each accepted slice:

1. verify actual delivered behavior and updated consumers;
2. correct durable boundary/sequence documents only when implementation facts
   changed them;
3. re-establish the next slice from current accepted main;
4. create exactly one bounded owning GitHub issue when the next slice is ready.

`pt-runenecs-r1-entity-errors.ron` may be retained or reconciled as subordinate
C1/R1 handoff detail if it remains useful. A workspace RON spec does not activate
a phase, and later phases do not require a RON spec merely to become valid work.
Create or update one only when a concrete slice materially benefits from the
machine-readable constraint set.

## Parallel work

Allowed while C1/R1 is implemented:

- read-only investigation for later phases;
- benchmark, Miri, sanitizer, and MSRV command discovery;
- consumer classification;
- test-gap documentation.

Forbidden without separately accepted authority:

- concurrent structural changes to C2-C8 paths;
- package renames or source transfer;
- broad `World` rewrite;
- speculative parallel executor;
- new spatial or shared-core repository.

## Final internal extraction-readiness gate

C9 must prove:

- framework-independent package graph;
- public downstream use;
- sound extension boundaries;
- accepted messaging/change/scheduler/network integration ownership;
- complete validation and performance baseline;
- exact provenance and clean-cutover plan.
