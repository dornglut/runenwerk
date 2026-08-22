---
title: RunenECS Extraction Boundary Design
description: Repository ownership and evidence gates for extracting RunenECS while consuming the independent neutral RunenScheduler contract through explicit adapters.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ./runenscheduler-design-canvas.md
  - ./runenscheduler-core-semantics.md
  - ../../reports/investigations/runenscheduler-ownership-investigation.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Extraction Boundary Design

## Status

Repository ownership direction is fixed. Public APIs, retained facilities, and
implementation phases remain provisional until the complete source, consumer,
unsafe-boundary, scheduler-adapter, messaging, and networking inventory is verified.

No ECS source movement, scheduler dependency cutover, or broad repair is authorized by
this document.

## Goal

Create an independently useful ECS repository without carrying Runenwerk geometry,
spatial policy, frame/tick lifecycle, rendering extraction, networking, replay,
editor, product behavior, or neutral scheduler implementation into the framework.

## Candidate repository shape

Expected initial packages are:

```text
repository  dornglut/runen-ecs
package     runen-ecs
crate       runen_ecs

package     runen-ecs-macros   only when the required proc-macro boundary is proven
crate       runen_ecs_macros
```

The independent neutral planner belongs to the separate RunenScheduler program:

```text
repository  target dornglut/runen-scheduler
package     runen-scheduler
crate       runen_scheduler
```

RunenECS may depend on an exact accepted RunenScheduler revision after the
cross-repository dependency and cutover gates are accepted. RunenScheduler never
depends on RunenECS or Runenwerk.

Old package names are removed only during coordinated cutover. No long-lived
compatibility packages, aliases, source mirrors, forwarding namespaces, includes,
submodules, or branch dependencies remain.

## Durable ownership

RunenECS owns public ECS semantics such as:

- entity, component, resource, and world lifecycle;
- component/resource storage semantics and iteration guarantees;
- queries and filters;
- deferred structural mutation;
- ECS system access derivation;
- ECS-to-RunenScheduler task/access adapter mapping;
- ECS system execution and command application;
- explicit reflection where accepted;
- repository-local diagnostics and public macro conformance.

The durable architecture does not freeze archetypes, dense columns, sparse sets, or
another storage mechanism as permanent public ownership.

RunenScheduler owns only the accepted neutral dependency/readiness contract:

- checked schedule/task identities;
- explicit dependencies;
- opaque access and placement keys;
- shared/exclusive claims;
- deterministic planning;
- immutable readiness plans;
- edge provenance and structured planning diagnostics;
- canonical serial interpretation.

Runenwerk retains:

- application, frame, fixed-step, render, startup, and shutdown policy;
- plugin and product composition;
- physical worker pools and work-stealing execution;
- generations, stale-result rejection, and product publication;
- general spatial indexes and entity-to-spatial adapters;
- ECS-to-render, scene, and world integration;
- networking, replication, authority, prediction, rollback, and transport;
- replay/history formats and retention;
- editor synchronization and diagnostics presentation.

## Geometry and spatial boundary

RunenECS core has no Runenwerk geometry dependency.

General spatial indexing is not ECS core merely because entries reference entities.
The current ECS-owned spatial hash must be removed, migrated, or proven as a separate
neutral facility before extraction.

RunenECS may expose generic change observation required by a Runenwerk spatial
adapter, but it must not understand AABBs, coordinates, cells, or world-query policy.

## Scheduler boundary

Three owners are distinct:

```text
RunenScheduler
  neutral identities, dependencies, opaque access claims,
  deterministic readiness plans, provenance, and reports

RunenECS
  systems, ECS access derivation, world/resource borrowing,
  command buffers, command application, and ECS execution

Runenwerk
  frame/tick phases, startup/shutdown, physical executors,
  rendering, networking, replay, generations, and product policy
```

The current `domain/scheduler` package is migration evidence, not accepted final
ownership. It currently mixes:

- a legacy callback-owning generic DAG;
- ECS access and `SystemParam` metadata;
- deferred-command assumptions;
- Runenwerk lifecycle and publication barriers;
- execution, telemetry, filesystem, and renderer-specific policy.

The legacy generic DAG is deleted rather than extracted. ECS-specific semantics move
to or remain in RunenECS. Runenwerk-specific lifecycle and execution semantics remain
in Runenwerk. Only the corrected neutral planner contract may later transfer to
RunenScheduler.

Serial execution is the reference behavior until sound parallel access, deterministic
readiness, panic/error policy, host execution, and serial-equivalence are proven.

Topological layers are diagnostic projections. They do not create implicit global
barriers. An ECS command-application boundary is explicit and ECS-owned.

## RunenECS adapter requirements

RunenECS derives neutral claims rather than exposing ECS types through the scheduler
contract.

Illustrative mapping:

```text
component/resource read   -> Shared opaque AccessKey
component/resource write  -> Exclusive opaque AccessKey
system ordering           -> explicit neutral dependencies
system sets               -> deterministic adapter expansion
command producer          -> independent ECS command-buffer output
command application       -> explicit ECS-owned task or boundary
```

ECS broadcast streams, work queues, drains, tick buffers, orphaned components,
structural mutation, query filters, and parameter slots remain ECS-owned concepts.
They may map into neutral claims or explicit tasks where justified, but do not become
built-in RunenScheduler domains or access modes.

The adapter must not:

- expose `World`, query, `SystemParam`, `Commands`, or system callbacks through the
  neutral public API;
- treat a conflict as implicit semantic order;
- depend on registration races, pointer addresses, or unstable Rust type-name suffixes;
- hide generated dependencies or command barriers;
- retain private reach-through after external cutover.

## Safety gates

The complete investigation must review:

- forgeable or stale entity identities and generation exhaustion;
- partial bundle/spawn/command mutation;
- every unsafe block and unsafe trait contract;
- externally implementable query metadata that participates in aliasing safety;
- `SystemParam` raw-pointer and lifetime contracts;
- world/query compatibility and escaped values;
- scheduler adapter identity, access, and ambiguity correctness;
- panic, poisoning, terminal, and capacity behavior.

The first extracted release should prefer sealed/supported low-level query and
system-param internals unless an explicitly unsafe public extension contract is proven
through downstream conformance and Miri/sanitizer evidence.

## Reflection

Reflection authority must be explicit and instance-owned. Process-global mutable
registration is not final authority.

The design must distinguish process-local Rust identity, registry-local identity, and
stable persisted/schema identity. Macros may generate descriptors but do not mutate
hidden global state.

## Messaging and change tracking

Current events, work queues, tick buffers, change extraction, and ownership routing
are not automatically retained in RunenECS.

Provisional classification:

```text
typed events/broadcast       likely RunenECS
FIFO world queues            candidate; requires independent ECS consumer proof
tick buffers/provenance      Runenwerk
change journal               candidate; requires non-network consumer proof
ownership/interest routing   Runenwerk
network/replay packets       Runenwerk
```

The final design follows actual consumer evidence rather than current module
location. Neutral scheduler access keys do not determine ownership of these facilities.

## Identity and errors

Entities are opaque world-local generational values. Raw entity values are not stable
network or persistence identities. Runenwerk maps entities to product and network
identities explicitly.

Framework public boundaries use structured errors where callers branch on failure.
`anyhow`, panics, flattened strings, and process-global telemetry do not define the
public framework contract.

RunenECS adapter errors and RunenScheduler planning errors remain separately owned and
composable. Neither layer discards the other layer's structured evidence.

## Macro policy

Public derives must:

- use only public RunenECS APIs;
- preserve generics and where clauses;
- emit stable compile diagnostics;
- avoid Runenwerk paths and hidden global registration;
- avoid generating private RunenScheduler reach-through;
- pass downstream compile-pass and compile-fail tests.

## Required investigation output

Before implementation, produce:

- complete file and public-API inventory;
- complete package and source-consumer inventory;
- unsafe-boundary and safety-contract inventory;
- exact RunenECS-to-RunenScheduler adapter map;
- scheduler phase/barrier consumer and deletion map;
- spatial/geometry consumer map;
- reflection authority map;
- messaging/change/ownership/network/replay map;
- exact move/stay/redesign/delete matrix;
- current test, Miri/sanitizer, Clippy, MSRV, benchmark, and downstream baseline.

## Sequence

```text
ECS-001 complete and verify investigation
ECS-002 close ownership and safety design
ECS-003 repair ECS boundaries through small ordered phases
ECS-004 consume accepted internal RunenScheduler planning through an adapter
ECS-005 prove standalone downstream conformance
ECS-006 create RunenECS and transfer corrected ECS source
ECS-007 cut Runenwerk over, delete originals, and close provenance
```

RunenScheduler planning and implementation proceed under separate parent `#201`.
ECS identity and atomic structural-mutation work may proceed independently when files
and authority do not conflict. Query and `SystemParam` scheduler integration waits for
the accepted RunenScheduler planning and implementation predecessor.

Only the next executable repair receives a decision-complete implementation
specification.

## Stop conditions

Stop before implementation when:

- source or consumer inventory remains incomplete;
- unsafe extension contracts remain ambiguous;
- RunenECS and RunenScheduler ownership remains mixed;
- scheduler adapter semantics require ECS types in the neutral core;
- messaging or change facilities lack independent ownership evidence;
- geometry/spatial removal requires a new unapproved repository;
- current main is not green for unrelated reasons;
- the plan requires one broad rewrite or long-lived compatibility layer;
- active work conflicts with the same branch, workspace, files, durable authority, or
  unmerged dependency.

## Definition of done

RunenECS is extracted only when framework packages validate independently,
Runenwerk-specific policy is absent, the RunenScheduler dependency is one-way and
exactly accepted, downstream public conformance passes, Runenwerk consumes exact
revisions through explicit adapters, original implementations are removed, and
integration/runtime validation is green.
