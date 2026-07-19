---
title: RunenECS Extraction Boundary Design
description: Decision-complete target ownership and staged repair architecture for extracting ECS, macros, and generic scheduling without Runenwerk spatial, networking, rendering, or lifecycle policy.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../reports/investigations/runenecs-complete-extraction-investigation.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Extraction Boundary Design

## Status

The target repository and ownership model are decision-complete. Extraction and
source repair remain blocked until mandatory local inventory/baseline validation
passes and `PT-RUNENECS-002` converts the repair program into small exact phase
specifications.

The linked investigation owns detailed current-source evidence, defects, consumer
findings, and validation gaps. This document owns the durable target.

## Repository and packages

```text
repository: Crystonix/RunenECS
packages:
  runenecs
  runenecs_macros
  runen_schedule
version: 0.1.0
edition: 2024
license: MIT OR Apache-2.0
publish: false until release gates are accepted
```

Initial shape:

```text
RunenECS/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── runenecs/
│   ├── runenecs_macros/
│   └── runen_schedule/
├── tests/
├── examples/
├── benches/
├── docs/
├── README.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── SECURITY.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

`runen_schedule` remains separately usable without `runenecs`, but stays in the
same repository until a genuine non-ECS release/consumer pressure justifies a
fourth repository.

The old package names `ecs`, `ecs_macros`, and `scheduler` are removed during the
final cutover. No compatibility packages remain.

## Dependency direction

```text
runen_schedule
      ▲
      │
runenecs ◄── runenecs_macros generated implementations
      ▲
      │
Runenwerk adapters/integration
```

Forbidden:

```text
RunenECS -> Runenwerk
runen_schedule -> runenecs
runenecs -> Runenwerk geometry/spatial/network/render/app lifecycle
```

Macros may depend on syntax tooling and generate paths into `runenecs`; the
runtime package must not depend on the macro package except optional re-exports
when the package graph permits it without a cycle.

## RunenECS ownership

`runenecs` owns:

- world-local generational entities;
- components, resources, stateful components, and bundles;
- dense/archetype storage;
- queries and supported filters;
- query-local added/changed/orphaned observation;
- deferred commands;
- system parameters and ECS access declarations;
- ECS runtime integration over `runen_schedule`;
- typed broadcast/event streams;
- typed deterministic FIFO world queues;
- explicit reflection registries;
- optional generic local change journal when consumer evidence requires it;
- structured diagnostics and deterministic reports;
- public derives through `runenecs_macros`.

## RunenSchedule ownership

`runen_schedule` owns:

- typed process-local schedule and system-set labels;
- registered systems over generic context `C`;
- generic access declarations and conflict validation;
- deterministic DAG/stage/wave planning;
- generic plan reports and diagnostics;
- serial reference execution.

It does not own:

- Runenwerk update/render/frame phases;
- ECS world borrowing;
- deferred commands;
- product publication;
- query snapshot publication;
- generation finalization;
- replay/network capture;
- renderer submission;
- a worker pool or global executor;
- process-global telemetry/logging.

The current `ExecutionPhaseKind` and product `BarrierKind` families are removed.
`runen_schedule` returns generic stages/waves. RunenECS flushes commands at its
chosen stage boundary; Runenwerk performs product lifecycle hooks externally.

## Runenwerk ownership

Runenwerk retains:

- app/frame/fixed-update/render/startup/shutdown policy;
- plugin composition and schedule instances;
- spatial indexes and entity-to-spatial adapters;
- simulation tick buffers and provenance;
- ownership/authority routing;
- networking, replication, prediction, rollback, and transport;
- replay/history formats and retention;
- scene/world/render extraction;
- editor synchronization and inspection presentation;
- stable product/network entity/type mappings;
- cross-repository integration tests.

## Entity identity

`Entity` is an opaque world-local generational value.

Requirements:

- fields are private;
- consumers receive read-only diagnostics/accessors only where justified;
- allocator validates free/despawn and rejects stale/double-free identities;
- generation exhaustion retires a slot permanently rather than saturating and
  reusing it;
- raw entity values are not stable persistence or network identities;
- world identity/namespace is included in safety reasoning if entities can cross
  world boundaries;
- Runenwerk explicitly maps entities to product/network IDs.

## Atomic world mutation

Safe public structural mutation must not leave undocumented partial state.

### Bundles

- registration/preflight occurs before mutation;
- insert commits all members or none;
- remove verifies all required members before removing any;
- derived and tuple bundles share one contract;
- tuple arity is not treated as a permanent architecture limit.

### Spawn

- allocation plus bundle insertion is one checked operation;
- failure leaves no live partial entity;
- ordinary failure returns a structured error rather than panicking.

### Commands

Ordinary command queues remain deterministic ordered sequences. They may be
explicitly non-transactional across commands, but each individual command must
preserve its operation's invariants.

A named atomic command batch may be added only with an explicit transaction/
rollback design. The current `BatchCommands` name must not imply atomicity if it
continues to stop after partial application.

Failed system-stage commands are discarded and never replayed later.

## Query safety

The first extracted release does not expose arbitrary third-party implementations
of the low-level raw-pointer `QueryData` contract.

Target:

- low-level query-data implementation is sealed/internal;
- public users compose built-in read/write/entity/optional/tuple query forms and
  filters;
- safe query methods are sound without trusting external access declarations;
- mutable query forms reject duplicate component types before execution;
- unsafe storage/query bridges have complete safety comments and Miri/sanitizer
  tests;
- query state is tied to compatible world identity/generation rather than raw
  pointer coincidence alone;
- query telemetry cannot affect behavior.

A public custom-query extension may be designed later as a separate unsafe API
with external conformance; it is not required for the initial extraction.

## System parameter safety

`SystemParam` remains extensible primarily through `#[derive(SystemParam)]`.

- the unsafe extraction implementation trait is sealed/doc-hidden or explicitly
  unsafe with a complete contract;
- safe manual implementation is not exposed accidentally;
- generated implementations declare exact access and preserve state lifetime;
- raw pointers are scoped to one system invocation;
- parameter values cannot escape their invocation undetected;
- nested/named groups use one recursive descriptor model;
- public params and derives are proven from a downstream package;
- Miri/sanitizer coverage exercises aliasing, escaped commands, query/resource
  combinations, and failure paths.

## Serial and parallel execution

Serial execution is the normative reference for the initial repository.

Parallel execution is deferred until a separate accepted design proves:

- sound disjoint world/resource access;
- deterministic stage and command barriers;
- panic/error/poison behavior;
- cancellation and shutdown;
- worker-pool ownership and lifetime;
- trace/diagnostic ordering;
- bounded queues/tasks;
- serial/parallel equivalence;
- no hidden global executor.

Access declarations and conflict-free waves alone are not sufficient proof.

## Reflection

Reflection uses explicit instance-owned registry authority.

Distinguish:

```text
Rust TypeId             process-local concrete Rust identity
ReflectTypeId           opaque registry-local identity
StableTypeKey           validated persisted/schema identity, when explicitly used
```

Requirements:

- no `OnceLock<Mutex<...>>` global registry;
- no implicit macro registration;
- duplicate stable keys are structured errors, never replacement;
- registry creation and test isolation are deterministic;
- world may own or borrow a registry through an explicit constructor/config;
- unload/reload and descriptor lifetime are documented;
- registry-local IDs are not serialized as durable type IDs;
- schema/version migration is separate from Rust type reflection.

## Messaging

### Events

RunenECS retains typed one-to-many events/broadcast streams.

- lifecycle terminology is host-neutral;
- transient cleanup is triggered by an explicit runtime finalization call, not an
  engine `FrameEnd` concept inside core;
- capacity/retention/overflow are validated;
- no normal panic overflow policy;
- rejected/dropped outcomes are structured and preserve payload where relevant;
- reader cursors report lag/retention loss explicitly;
- sequence exhaustion is handled, not saturated silently;
- observers are notification/diagnostic hooks, not mutation reentrancy.

### FIFO world queues

RunenECS retains typed FIFO queues as world-coordination storage.

- queue semantics are enqueue, inspect, and explicit destructive drain/clear;
- capacity backpressure returns the exact unaccepted payload where ownership
  matters;
- no implicit retry, transport, priority, task execution, claim, or acknowledgement;
- Runenwerk networking decides drop/retry/log policy.

### Tick buffers

Tick-indexed buffers, current/finalized simulation tick state, deduplication
provenance, and network-prediction lifecycle move out of RunenECS.

The current implementation moves to a reviewed Runenwerk simulation/network
owner, initially `engine_sim` plus engine integration. Do not invent a generic
buffer framework without a second independent consumer.

## Spatial indexing

Spatial indexing is not ECS-core ownership.

- delete ECS `SpatialIndex`, `SpatialHashIndex`, and geometry-based world storage;
- remove `geometry` from `runenecs` dependencies;
- entity despawn no longer knows spatial indexes;
- Runenwerk spatial adapters observe entities/components and update the accepted
  `spatial`/`spatial_index` implementation;
- geometry-bearing components remain ordinary downstream component types.

A future RunenSpatial extraction is separate work.

## Change tracking

RunenECS retains:

- local monotonic change tick/sequence;
- query-local `Added`, `Changed`, and removed/orphaned observation;
- entity/component/resource structural facts needed for world correctness;
- optionally one bounded generic change journal if actual consumers require
  historical extraction.

The generic journal, if retained:

- uses one local `ChangeSequence` and cursor;
- has explicit retention and cursor-too-old outcomes;
- contains no engine frame index;
- contains no network/simulation tick type;
- contains no owner/interest filter;
- does not claim stable serialized entity/type identity.

Runenwerk owns frame/tick windows, ownership/interest filtering, replication
snapshot/delta construction, replay, and editor sync policy.

## Ownership and authority routing

Remove current `OwnerId`, `OwnerRole`, owner-routing, resource-owner, and transfer-
log authority from RunenECS.

They are network/product semantics currently consumed by connection handling.
Runenwerk owns them or models them as product components/resources.

RunenECS remains able to store and query any downstream ownership component
without knowing its meaning.

## Errors and panic policy

Public framework APIs use structured errors.

```text
EntityError
WorldMutationError
BundleError
CommandError
QueryError
SystemParamError
ReflectionError
EventError
QueueError
ScheduleBuildError
ScheduleRunError
```

Exact names may consolidate, but consumers must be able to branch without parsing
strings.

`anyhow` is allowed only at Runenwerk/application composition boundaries.

Ordinary stale identity, missing data, duplicate registration, schedule cycle,
capacity, invalid access, and setup failure do not panic. User-system panic policy
is explicit and cannot leave staged commands published as success.

## Telemetry and diagnostics

Remove process-global wall-clock telemetry and hard-coded logging policy from
RunenECS/RunenSchedule.

Retain deterministic counters in operation/plan reports. Runenwerk and benchmarks
measure elapsed time externally.

A future optional observer/sink must be instance-owned, bounded where applicable,
and behaviorally subordinate.

Diagnostics use `runenecs.*` and `runen_schedule.*` namespaces and preserve
structured identity/provenance.

## Macros

`runenecs_macros` owns derives for the accepted public traits.

Requirements:

- resolve renamed `runenecs` dependencies correctly;
- preserve generics, lifetimes, where clauses, visibility, and attributes;
- use supported public/doc-hidden bridge APIs only;
- generate atomic bundle preflight/commit participation;
- generate explicit reflection descriptors, not global registration;
- generate supported system-param bridge implementations;
- produce precise compile diagnostics;
- include external compile-pass and compile-fail conformance;
- emit no Runenwerk path.

## Versioning and persistence

All three packages begin pre-1.0 and unpublished.

Runtime identities, scheduler labels, reflection IDs, event sequences, and change
sequences are process/world/registry local unless explicitly documented.

No raw Rust layout, `TypeId`, or `Entity` value is a persistence format. Stable
schemas and network/replay formats remain separately versioned by Runenwerk
owners.

## Independent conformance

RunenECS must validate without Runenwerk:

- entity stale/double-free/exhaustion behavior;
- atomic bundle/spawn/insert/remove;
- component/resource lifecycle;
- storage/archetype invariants;
- query/filter/change behavior;
- Miri/sanitizer unsafe-boundary tests;
- deferred command ordering/failure isolation;
- event retention/cursors/overflow;
- FIFO queue ordering/backpressure/exact rejected payload;
- explicit reflection registry and duplicate errors;
- generic schedule labels/sets/cycles/conflicts/stages;
- absence of product phases/barriers;
- external macros and package renaming;
- no geometry or Runenwerk dependency;
- serial reference behavior;
- stable/MSRV validation and representative benchmarks.

At least one standalone simulation example uses only public RunenECS APIs.

## Repair sequence

The boundary repair must be divided into reviewable phases. `PT-RUNENECS-002`
will name exact files and validation for each.

Recommended order:

```text
ECS-R1 entity identity and structured core errors
ECS-R2 atomic bundle/spawn/command invariants
ECS-R3 query/SystemParam unsafe-boundary hardening
ECS-R4 explicit reflection registry and macro migration
ECS-R5 remove spatial/geometry and add Runenwerk adapter
ECS-R6 messaging split: events/queues retained, tick buffers migrated
ECS-R7 generic change journal and ownership/network separation
ECS-R8 neutralize runen_schedule phases/barriers/errors/telemetry
ECS-R9 standalone downstream conformance and benchmark baseline
```

Do not combine all repairs into one implementation PR.

## Extraction sequence

### PT-RUNENECS-001 — Complete investigation

Complete subject to mandatory local file/test/consumer and command verification.

### PT-RUNENECS-002 — Decision/spec closure

Produce exact phase specs for R1–R9, including file scopes, API migrations,
validation, stop conditions, dependency order, and temporary seam limits. No broad
source implementation.

### PT-RUNENECS-003 — Boundary repair

Execute R1–R9 through bounded PRs inside Runenwerk. Current packages remain local
until independent conformance passes.

### PT-RUNENECS-004 — Repository creation and transfer

Create `Crystonix/RunenECS`, establish governance/provenance, transfer corrected
packages, and validate independently.

### PT-RUNENECS-005 — Runenwerk cutover

Pin exact revisions, migrate consumers, delete original packages, regenerate the
lockfile, and prove workspace/application/network integration.

### PT-RUNENECS-006 — Closeout

Record compatibility, performance, safety evidence, provenance, deleted paths,
and final ownership.

## Stop conditions

Stop if:

- safe public APIs still trust arbitrary unsafe query/param implementations;
- entity generations can saturate/reuse;
- safe mutation leaves partial undocumented state;
- `runen_schedule` retains Runenwerk phases/barriers;
- ECS retains geometry/spatial ownership;
- reflection remains global;
- tick-buffer/ownership/network policy remains core ECS;
- public branchable errors remain erased into `anyhow`;
- local inventory finds persisted raw entity/type identity dependencies;
- parallel execution begins without soundness/equivalence proof;
- extraction requires compatibility/mirror packages.

## Definition of done

RunenECS is complete only when:

- all three packages validate independently on stable and MSRV;
- unsafe boundaries have Miri/sanitizer and external conformance;
- RunenECS has no Runenwerk/geometry/network/render dependency;
- RunenSchedule has no product lifecycle vocabulary;
- Runenwerk pins exact revisions through one-way dependencies;
- original ECS/macros/scheduler source is deleted;
- no compatibility package, mirror, or moving dependency remains;
- full Runenwerk engine/network/application validation passes;
- provenance and current documentation are complete.
