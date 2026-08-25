---
title: RunenECS Extraction Boundary Design
description: Accepted repository ownership and extraction boundary for RunenECS without Runenwerk spatial, networking, rendering, replay, lifecycle, or product policy.
status: accepted
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-08-25
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/runenecs-issue-198-current-main-census.md
  - ./runenecs-boundary-repair-execution-plan.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Extraction Boundary Design

## Status

This accepted design owns the durable RunenECS target boundary. The command-verified
current-main facts used to establish it are recorded in the
[Issue 198 current-main census](../../reports/investigations/runenecs-issue-198-current-main-census.md).
The [Boundary Repair Execution Plan](./runenecs-boundary-repair-execution-plan.md)
owns the one canonical repair sequence. Investigation reports are evidence, not
implementation or sequencing authority.

No ECS source movement, package rename, dependency change, or implementation is
authorized by this design alone. Each implementation slice requires its own owning
GitHub issue.

## Goal

Create an independently useful ECS repository without carrying Runenwerk geometry,
spatial policy, frame/tick lifecycle, rendering extraction, networking, replay,
editor, or product behavior into the framework.

## Target repository shape

Repository:

```text
dornglut/runen-ecs
```

Initial Cargo package and Rust crate identities:

```text
Cargo package       Rust crate
runen-ecs           runen_ecs
runen-ecs-macros    runen_ecs_macros
```

The proc-macro companion is justified by the existing derive boundary and remains
separate while Rust proc-macro packaging technically requires it. Additional
packages require independent proof. ECS-native schedule, access, ordering,
validation, and deferred-command semantics live inside `runen_ecs`; there is no
`runen_schedule` or external RunenScheduler dependency.

Current package names are changed only during a separately accepted cutover. No
long-lived compatibility packages remain after cutover.

## Durable ownership

RunenECS owns public ECS semantics such as:

- entity, component, resource, and world lifecycle;
- component/resource storage semantics and iteration guarantees;
- queries and filters;
- deferred structural mutation;
- system identity, access declarations, explicit ordering/sets, schedule
  validation, and deterministic serial ECS execution;
- explicit reflection where accepted;
- ECS-local change observation and independently justified local messaging;
- repository-local diagnostics and public macro conformance.

The durable architecture does not freeze archetypes, dense columns, sparse sets,
or another storage mechanism as permanent public ownership.

RunenNet owns reusable realtime networking semantics according to its own accepted
authority, including protocol/schema identity, replication consistency,
session/authority semantics, delivery and recovery, transport-independent
interfaces, and separately accepted prediction or interest semantics. RunenECS
does not absorb those semantics merely because networking consumes ECS state.

Runenwerk/application integration retains:

- application, frame, fixed-step, render, startup, and shutdown policy;
- plugin and product composition;
- general spatial indexes and ECS-to-spatial adapters;
- ECS-to-render, scene, and world integration;
- concrete ECS-to-RunenNet identity/state mapping;
- gameplay ownership, relevancy, and world/spatial policy supplied to networking;
- host execution and product/publication barriers;
- archival/editor replay formats and retention;
- editor synchronization and diagnostics presentation.

## Geometry and spatial boundary

RunenECS core has no Runenwerk geometry dependency.

General spatial indexing is not ECS core merely because entries reference
entities. The current ECS-owned spatial hash must be removed or migrated through
the accepted RunenSpatial/Runenwerk integration boundary before extraction.

RunenECS may expose generic local change observation required by a spatial
adapter, but it must not understand AABBs, coordinates, cells, or world-query
policy.

## Scheduler boundary

The scheduler split is semantic, not package-shaped:

```text
runen_ecs
  system identity
  ECS access facts
  explicit ordering and sets
  schedule validation
  deferred-command boundaries
  deterministic serial reference execution

Runenwerk
  frame/tick/startup/shutdown/render lifecycle
  host execution
  product/publication barriers
  application scheduling policy
```

Semantic ordering is not access incompatibility. An access conflict may prevent
concurrent execution without inventing meaningful A-before-B order.

The current `domain/scheduler` package is migration evidence. ECS-owned semantics
move into `runen_ecs`; Runenwerk-owned lifecycle/product policy stays in
Runenwerk; unsupported generic DAG/demo/DOT/filesystem/global-telemetry residue
is deleted after consumer migration. No generic scheduler package survives as a
required dependency.

Serial execution is the correctness/reference behavior until sound parallel
access, deterministic deferred boundaries, panic/error policy, cancellation,
worker ownership, bounded queues, and observational equivalence are proven.

## Identity and errors

`Entity` is an opaque, copyable, comparable, hashable runtime handle comprising:

```text
WorldScopeId + slot/index + generation
```

Each `World` owns one opaque process-local `WorldScopeId`, and its allocator emits
only entities carrying that scope. Every world operation validates scope before
slot/generation. Therefore a token from another world is rejected even when its
slot and generation equal a live local entity.

World scopes are checked, non-reusing process-local runtime identities. Exhaustion
must fail world creation rather than wrap or reuse a scope. A world scope is not
serialized, persisted, transmitted, or promoted to stable/user-visible identity.
Persistence, networking, replay, and editor records use separately owned stable
identities and explicit mappings.

Entity allocation/free operations are fallible where capacity, stale state,
unknown/cross-world identity, double free, index exhaustion, or generation
exhaustion can occur. Rejected operations do not mutate state. Generation
exhaustion retires the slot permanently.

Framework public boundaries use structured errors where callers branch on
failure. `anyhow`, panics, and process-global telemetry do not define the public
framework contract.

## Safety gates

The complete investigation reviews:

- forgeable or stale entity identities, world scope, and exhaustion;
- partial bundle/spawn/command mutation;
- every unsafe block and unsafe trait contract;
- externally implementable query metadata that participates in aliasing safety;
- `SystemParam` raw-pointer and lifetime contracts;
- world/query compatibility and escaped values;
- panic, poisoning, terminal, and capacity behavior.

The first extracted release prefers sealed/supported low-level query and
SystemParam internals unless an explicitly unsafe public extension contract is
proven through downstream conformance and Miri/sanitizer evidence.

## Reflection

Reflection authority is explicit and instance-owned. Process-global mutable
registration is not final authority.

The design distinguishes:

```text
Rust TypeId        process-local concrete Rust identity
registry identity  explicit registry-local identity
stable schema key  separately governed persistence/schema identity
```

Macros may generate descriptors but do not mutate hidden global registration
state.

## Messaging and change tracking

Current events, work queues, tick buffers, change extraction, and ownership
routing are not automatically retained in RunenECS.

Target classification:

```text
typed events/broadcast       RunenECS only with proven ECS-local semantics
FIFO world queues            RunenECS only with proven ECS-local semantics
change observation           RunenECS when local and network-neutral
tick/frame provenance        Runenwerk lifecycle/runtime policy
game ownership/relevancy     Runenwerk/application policy
network protocol/replication RunenNet
network transport semantics  RunenNet / accepted RunenNet adapters
ECS <-> network mapping      Runenwerk/application integration
archival/editor replay       Runenwerk/application policy
```

Unsupported generic work/retry/ack residue is deleted. A networking consumer does
not make network semantics ECS-owned, and Runenwerk integration does not duplicate
RunenNet protocol authority.

## Macro policy

Public derives must:

- use only public `runen_ecs` APIs;
- preserve generics and where clauses;
- emit stable compile diagnostics;
- avoid Runenwerk paths and hidden global registration;
- pass downstream compile-pass and compile-fail tests.

## Evidence gate

The Issue 198 current-main census records the command-verified evidence that was
used to accept this boundary: source/public-surface and consumer inventories,
unsafe/query/SystemParam contracts, entity/allocator behavior, scheduler and
spatial ownership, reflection, messaging/change/network/replay disposition, and
validation-support gaps.

Future implementation slices must re-resolve accepted main and stop if material
source, consumer, ownership, or validation drift invalidates the relevant evidence.

## Repair sequence

The single durable C0-C9 sequence, prerequisites, phase boundaries, and
post-C9 transfer rule are owned by the
[Boundary Repair Execution Plan](./runenecs-boundary-repair-execution-plan.md).
This boundary design does not duplicate that sequence.

C1/R1 is the first implementation slice. It remains unauthorized until a bounded
owning GitHub issue is created from current accepted authority. No implementation
issue is activated by this design or by a retained workspace RON file.

## Stop conditions

Stop before implementation when:

- source or consumer inventory remains incomplete;
- entity world-scope behavior is not mechanically enforceable;
- unsafe extension contracts remain ambiguous;
- scheduler/product ownership remains mixed;
- messaging or change facilities lack independent ownership evidence;
- networking disposition would duplicate or contradict RunenNet authority;
- geometry/spatial removal requires unaccepted dependency or ownership changes;
- current main is not green for unrelated reasons;
- the plan requires one broad rewrite or long-lived compatibility layer.

## Definition of done

RunenECS is ready for external transfer only when the target packages validate
independently, Runenwerk-specific policy and duplicate networking semantics are
absent, downstream public conformance passes, validation/MSRV/safety/performance
requirements are met, and a separate accepted transfer/cutover boundary can move
source and consumers without leaving aliases, forwarding packages, mirrors, or
duplicate implementations.
