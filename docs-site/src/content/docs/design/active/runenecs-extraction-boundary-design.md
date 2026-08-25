---
title: RunenECS Extraction Boundary Design
description: Provisional repository ownership and investigation gates for extracting ECS and macros without Runenwerk spatial, networking, rendering, replay, lifecycle, or product policy.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-08-25
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runenecs-issue-198-current-main-census.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Extraction Boundary Design

## Status

Repository ownership direction is fixed. The complete current-main source,
consumer, unsafe-boundary, scheduler, messaging, and networking inventory is
recorded in the [Issue 198 current-main census](../../reports/investigations/runenecs-issue-198-current-main-census.md).
That census is the current authority for retained facilities and sequencing.

No ECS source movement or broad repair is authorized by this document.

## Goal

Create an independently useful ECS repository without carrying Runenwerk geometry,
spatial policy, frame/tick lifecycle, rendering extraction, networking, replay,
editor, or product behavior into the framework.

## Candidate repository shape

Expected initial packages:

```text
runenecs
runenecs_macros
```

ECS-native schedule, access, ordering, validation, and deferred-command
semantics live inside `runenecs`. There is no `runen_schedule` crate and no
generic scheduler dependency. Runenwerk retains application and product
lifecycle scheduling.

The old package names are removed only during the final coordinated cutover. No
long-lived compatibility packages remain.

## Durable ownership

RunenECS owns public ECS semantics such as:

- entity, component, resource, and world lifecycle;
- component/resource storage semantics and iteration guarantees;
- queries and filters;
- deferred structural mutation;
- system access declarations and ECS schedule integration;
- explicit reflection where accepted;
- repository-local diagnostics and public macro conformance.

The durable architecture does not freeze archetypes, dense columns, sparse sets,
or another storage mechanism as permanent public ownership.

Runenwerk retains:

- application, frame, fixed-step, render, startup, and shutdown policy;
- plugin and product composition;
- general spatial indexes and entity-to-spatial adapters;
- ECS-to-render, scene, and world integration;
- networking, replication, authority, prediction, rollback, and transport;
- replay/history formats and retention;
- editor synchronization and diagnostics presentation.

## Geometry and spatial boundary

RunenECS core has no Runenwerk geometry dependency.

General spatial indexing is not ECS core merely because entries reference
entities. The current ECS-owned spatial hash must be removed, migrated, or proven
as a separate neutral facility before extraction.

RunenECS may expose generic change observation required by a Runenwerk spatial
adapter, but it must not understand AABBs, coordinates, cells, or world-query
policy.

## Scheduler boundary

Three semantic owners are distinct:

```text
runenecs
  system identity, ECS access facts, explicit ordering/sets, schedule validation,
  deferred-command boundaries, deterministic serial reference execution

Runenwerk
  frame/tick phases, startup/shutdown, rendering, networking, replay, product policy,
  host execution, and product/publication barriers
```

Semantic ordering is not access incompatibility. The current scheduler's
conflict matrix is evidence for the ECS access boundary; it is not a replacement
for explicit semantic order. Generic DAG/demo/DOT/telemetry residue and
Runenwerk-shaped phase/barrier types are deleted after consumer migration.

Serial execution is the reference behavior until sound parallel access,
deterministic barriers, panic/error policy, cancellation, worker ownership, and
serial-equivalence are proven.

## Safety gates

The complete investigation must review:

- forgeable or stale entity identities and generation exhaustion;
- partial bundle/spawn/command mutation;
- every unsafe block and unsafe trait contract;
- externally implementable query metadata that participates in aliasing safety;
- `SystemParam` raw-pointer and lifetime contracts;
- world/query compatibility and escaped values;
- panic, poisoning, terminal, and capacity behavior.

The first extracted release should prefer sealed/supported low-level query and
system-param internals unless an explicitly unsafe public extension contract is
proven through downstream conformance and Miri/sanitizer evidence.

## Reflection

Reflection authority must be explicit and instance-owned. Process-global mutable
registration is not final authority.

The design must distinguish process-local Rust identity, registry-local identity,
and stable persisted/schema identity. Macros may generate descriptors but do not
mutate hidden global state.

## Messaging and change tracking

Current events, work queues, tick buffers, change extraction, and ownership
routing are not automatically retained in RunenECS.

Current-main classification:

```text
typed events/broadcast       RunenECS when local retention/overflow is proven
FIFO world queues            RunenECS only for proven local semantics
tick buffers/provenance      Runenwerk lifecycle/runtime policy
change observation           RunenECS when local and non-network
ownership/interest routing   Runenwerk
network/replay packets       Runenwerk
```

Unsupported generic work/retry/ack residue is deleted. The final design follows
the command-verified consumer map rather than current module location.

## Identity and errors

Entities are opaque world-local generational values. Raw entity values are not
stable network or persistence identities. Runenwerk maps entities to product and
network identities explicitly.

Framework public boundaries use structured errors where callers branch on
failure. `anyhow`, panics, and process-global telemetry do not define the public
framework contract.

## Macro policy

Public derives must:

- use only public RunenECS APIs;
- preserve generics and where clauses;
- emit stable compile diagnostics;
- avoid Runenwerk paths and hidden global registration;
- pass downstream compile-pass and compile-fail tests.

## Required investigation output

Before implementation, produce:

- complete file and public-API inventory;
- complete package and source-consumer inventory;
- unsafe-boundary and safety-contract inventory;
- scheduler phase/barrier consumer map;
- spatial/geometry consumer map;
- reflection authority map;
- messaging/change/ownership/network/replay map;
- exact move/stay/redesign/delete matrix;
- current test, Miri/sanitizer, Clippy, MSRV, and benchmark baseline.

## Sequence

```text
C0 current-main census and authority binding
  -> C1 Entity/errors (R1)
    -> C2 atomic structural mutation (R2)
      -> C3 query/SystemParam safety (R3)
        -> C4 reflection/macros (R4)
          -> C5 ECS-native scheduling/access/deferred semantics (revised R8)
            -> C6 spatial handoff (R5)
              -> C7 messaging/change/lifecycle/network separation (R6/R7)
                -> C8 standalone proof/conformance/benchmark baseline (R9/ECS-004)
                  -> C9 accepted-repository transfer and cutover (ECS-005/006)
```

C1 remains the first implementation slice after #198 acceptance. C9 is not
authorized by this document.

## Stop conditions

Stop before implementation when:

- source or consumer inventory remains incomplete;
- unsafe extension contracts remain ambiguous;
- scheduler/product ownership remains mixed;
- messaging or change facilities lack independent ownership evidence;
- geometry/spatial removal requires a new unapproved repository;
- current main is not green for unrelated reasons;
- the plan requires one broad rewrite or long-lived compatibility layer.

## Definition of done

RunenECS is extracted only when framework packages validate independently,
Runenwerk-specific policy is absent, downstream public conformance passes,
Runenwerk consumes exact revisions through one-way dependencies, original
implementations are removed, and integration/runtime validation is green.
