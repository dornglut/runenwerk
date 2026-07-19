---
title: RunenECS Boundary Repair Execution Plan
description: Dependency-ordered implementation plan that converts the accepted RunenECS target into nine bounded pre-extraction repairs.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ./runenecs-extraction-boundary-design.md
  - ../../reports/investigations/runenecs-complete-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/specs/pt-runenecs-r1-entity-errors.ron
  - ../../workspace/specs/pt-runenecs-r2-atomic-mutation.ron
  - ../../workspace/specs/pt-runenecs-r3-unsafe-extension-boundaries.ron
  - ../../workspace/specs/pt-runenecs-r4-reflection-macros.ron
  - ../../workspace/specs/pt-runenecs-r5-spatial-geometry-removal.ron
  - ../../workspace/specs/pt-runenecs-r6-messaging-split.ron
  - ../../workspace/specs/pt-runenecs-r7-change-ownership-separation.ron
  - ../../workspace/specs/pt-runenecs-r8-scheduler-neutrality.ron
  - ../../workspace/specs/pt-runenecs-r9-standalone-conformance.ron
---

# RunenECS Boundary Repair Execution Plan

## Purpose

This plan converts the accepted RunenECS architecture into nine bounded repairs
inside Runenwerk. The packages remain local until every repair and standalone
conformance gate passes.

Every RON spec is `active-planning-only`. Implementation requires local inventory,
baseline validation, a current-main recheck, and a separate activation record for
the named phase.

## Dependency order

```text
R1 entity/errors
  -> R2 atomic mutation
  -> R3 unsafe extension boundaries
  -> R4 reflection/macros
  -> R5 spatial/geometry removal
  -> R6 messaging split
  -> R7 change/ownership separation
  -> R8 scheduler neutrality
  -> R9 standalone conformance
  -> external repository transfer
```

A later phase may be investigated in parallel, but implementation does not skip
an unmet dependency.

## R1 — Entity identity and structured core errors

Owner: ECS core.

Primary scope:

```text
domain/ecs/src/entity.rs
domain/ecs/src/errors.rs
domain/ecs/src/world/entity/**
domain/ecs/src/world/state.rs only for allocator/world identity wiring
domain/ecs/tests/entity_identity*.rs
domain/ecs/tests/world_error*.rs
```

Outcomes:

- private opaque entity fields;
- checked stale/double-free/despawn behavior;
- exhausted generations retire slots;
- world/namespace compatibility is explicit if required by current cross-world
  behavior;
- structured entity/world errors replace ordinary panic/anyhow paths in this
  scope;
- no raw entity value becomes a persistence/network promise.

R1 must not change bundles, queries, system params, reflection, messaging,
spatial, networking, or scheduler semantics beyond compilation migration.

## R2 — Atomic bundle, spawn, and command invariants

Owner: ECS world mutation.

Primary scope:

```text
domain/ecs/src/bundle.rs
domain/ecs/src/commands/**
domain/ecs/src/world/entity/**
domain/ecs/src/world/component*.rs if present
domain/ecs/src/world/resource*.rs if required for command atomicity
domain/ecs_macros/src/lib.rs only for Bundle derive participation
domain/ecs/tests/bundle*.rs
domain/ecs/tests/commands*.rs
domain/ecs/tests/spawn*.rs
```

Outcomes:

- bundle insert/remove preflight before mutation;
- spawn allocation plus bundle insertion is atomic;
- failed individual commands preserve operation invariants;
- ordered queues are explicitly non-transactional across commands unless a named
  transaction API is introduced;
- misleading batch names do not imply rollback;
- no partial live entity after failed spawn;
- structured errors and deterministic command ordering.

## R3 — Query and SystemParam unsafe boundary hardening

Owner: ECS unsafe core.

Primary scope:

```text
domain/ecs/src/query/**
domain/ecs/src/system/params.rs
domain/ecs/src/system/extract.rs
domain/ecs/src/system/mod.rs
domain/ecs/src/system/runtime.rs only for checked extraction/failure propagation
domain/ecs/src/storage/** only for safety bridge changes
domain/ecs_macros/src/lib.rs only for SystemParam derive
domain/ecs/tests/query*.rs
domain/ecs/tests/system_param*.rs
domain/ecs/tests/unsafe_boundary*.rs
```

Outcomes:

- low-level QueryData implementation sealed for the initial public release;
- safe query forms cannot trust arbitrary external access declarations;
- duplicate mutable component access rejected;
- SystemParam derive uses one sealed/doc-hidden or explicitly unsafe bridge;
- raw-pointer lifetimes scoped to invocation;
- complete safety comments;
- Miri/sanitizer plan and tests;
- no parallel-execution expansion.

## R4 — Explicit reflection registry and macro migration

Owner: ECS reflection and macros.

Primary scope:

```text
domain/ecs/src/reflect/**
domain/ecs/src/world/state.rs only for explicit registry ownership
domain/ecs/src/world/mod.rs only for constructor/API wiring
domain/ecs_macros/src/lib.rs reflection derives
domain/ecs/tests/reflect*.rs
domain/ecs_macros tests/fixtures
```

Outcomes:

- no process-global mutable registry;
- explicit registry construction/ownership;
- separate Rust TypeId, registry-local ReflectTypeId, and stable schema key;
- duplicate stable key is structured rejection, not replacement;
- macros produce descriptors/registration operations without global mutation;
- package-renaming and external derive conformance;
- deterministic test isolation.

## R5 — Remove ECS spatial and geometry ownership

Owner: ECS core plus selected Runenwerk spatial integration owner.

Primary ECS scope:

```text
domain/ecs/Cargo.toml
domain/ecs/src/indexing/**
domain/ecs/src/world/** only spatial-index storage/hooks
domain/ecs/src/lib.rs exports
domain/ecs/tests/indexing*.rs
domain/ecs/tests/spatial*.rs
```

Integration scope is selected during activation from the current consumer
inventory. It must be one existing Runenwerk spatial/world/engine owner or one
narrow adapter module; no speculative new framework crate.

Outcomes:

- `geometry` removed from ECS dependencies;
- ECS spatial index types and world hooks removed;
- generic entity/component change observation remains sufficient for adapter
  updates;
- despawn behavior is integrated through the Runenwerk spatial adapter rather
  than ECS semantic knowledge;
- separate `spatial`/`spatial_index` authority is not duplicated.

## R6 — Messaging split

Owner: ECS events/queues plus Runenwerk simulation/network migration.

Primary ECS scope:

```text
domain/ecs/src/world/messaging/**
domain/ecs/src/system/params.rs messaging params only
domain/ecs/src/world/state.rs messaging storage only
domain/ecs/src/world/runtime.rs lifecycle finalization only
domain/ecs/tests/broadcast*.rs
domain/ecs/tests/work_queue*.rs
domain/ecs/tests/tick_buffer*.rs
```

Runenwerk migration scope is limited to current tick-buffer consumers in
`engine_sim`, network integration, and focused tests identified by local search.

Outcomes:

- host-neutral typed event streams remain in ECS;
- normal overflow never panics;
- lag/drop/rejection outcomes explicit;
- FIFO world queues retain exact unaccepted payload under backpressure;
- no transport/task/ack/retry semantics in queues;
- tick buffers and simulation provenance move out of ECS;
- no engine `FrameEnd` semantic in ECS messaging.

## R7 — Generic change observation and ownership/network separation

Owner: ECS change tracking plus Runenwerk network/product integration.

Primary ECS scope:

```text
domain/ecs/src/world/change_tracking.rs
domain/ecs/src/world/change_extraction/**
domain/ecs/src/world/ownership/**
domain/ecs/src/world/state.rs related fields
domain/ecs/src/lib.rs exports
domain/ecs/tests/change*.rs
domain/ecs/tests/ownership*.rs
```

Runenwerk scope is limited to current network/replay/editor consumers found by
local inventory.

Outcomes:

- query-local added/changed/removed facts remain ECS-owned;
- one generic local ChangeSequence/journal retained only if consumer evidence
  requires history;
- no engine frame/simulation tick sentinels in ECS journal;
- owner roles/routing/transfer logs removed from ECS;
- interest, authority, replication, replay, and stable network IDs remain
  Runenwerk-owned;
- raw Entity/ReflectTypeId are not persisted/network identities.

## R8 — Neutralize runen_schedule

Owner: scheduler package plus ECS runtime integration.

Primary scope:

```text
domain/scheduler/**
domain/ecs/src/system/runtime.rs
domain/ecs/src/system/mod.rs
domain/ecs/src/errors.rs only ECS schedule integration errors
domain/scheduler tests
domain/ecs/tests/runtime_plan*.rs
engine schedule/lifecycle call sites only for migration to host-owned phases/hooks
```

Outcomes:

- remove Runenwerk ExecutionPhaseKind and product BarrierKind;
- generic labels, access declarations, DAG stages/waves, and reports only;
- structured schedule build/run errors;
- no global telemetry or hard-coded render logging;
- RunenECS flushes deferred commands after accepted generic stages;
- Runenwerk owns frame/update/render/publication/replay hooks;
- system ID exhaustion structured;
- serial reference semantics preserved.

## R9 — Standalone conformance and benchmark baseline

Owner: RunenECS public conformance.

Primary scope:

```text
domain/ecs/tests/**
domain/ecs_macros tests/fixtures
domain/scheduler tests
a non-publishable downstream test package under tests/runenecs_external or accepted equivalent
domain/ecs/benches/**
docs-site/src/content/docs/domain/ecs/** current API alignment
proof reports and phase closeout docs
```

Outcomes:

- external components/resources/bundles/system params/reflection use public APIs;
- standalone simulation uses no engine;
- Miri/sanitizer unsafe-boundary suite;
- entity/atomicity/query/commands/events/queues/reflection/schedule proofs;
- no geometry/Runenwerk dependency;
- stable/MSRV/Clippy/docs validation;
- benchmark baseline for entity/storage/query/commands/schedule/events;
- exact transfer inventory for RunenECS repository creation.

R9 does not create the external repository or delete local packages.

## Shared constraints

Every repair:

- starts from current merged `main` after truthful prior-phase closeout;
- has one bounded PR and one phase spec;
- preserves current unrelated behavior;
- does not rename packages to final external names unless the phase spec explicitly
  authorizes a no-alias cutover;
- adds no source mirror, compatibility package, universal core/meta crate, hidden
  global authority, or parallel writable API;
- reports command validation honestly;
- updates current docs only when public behavior changes.

## Activation gate

Before R1 activation, run the complete local inventory and baseline from the
investigation. Each later phase re-runs affected package tests plus the full
workspace envelope required by current repository authority.

If local evidence changes ownership or reveals persisted raw entity/type
contracts, update the Markdown design before any implementation spec is
activated.
