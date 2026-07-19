---
title: RunenECS Extraction Boundary Design
description: Target package shape and required boundary repairs for extracting ECS, macros, and generic schedule semantics without carrying Runenwerk spatial, networking, rendering, or lifecycle policy.
status: active
owner: ecs
layer: domain/ecs
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenECS Extraction Boundary Design

## Status

This document fixes the target ownership and phase order. ECS source movement is
not authorized until the complete source/consumer inventory and the decisions
listed as implementation gates are proven against current code and tests.

## Goal

Create an independently useful Rust ECS repository with explicit storage, query,
command, scheduling, reflection, messaging, change-tracking, diagnostics, and
macro contracts while keeping Runenwerk product integration outside the
framework.

## Initial repository shape

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
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
└── CHANGELOG.md
```

`runen_schedule` remains a separate package in the RunenECS repository because
current scheduler contracts are generic over an execution context and are a
required dependency of the ECS runtime. It must remain usable without
`runenecs`.

Do not create a fourth RunenSchedule repository during this program. Revisit
repository separation only after a real non-ECS consumer and independent release
pressure exist.

## Package identities

```text
repository: Crystonix/RunenECS
packages:
  runenecs
  runenecs_macros
  runen_schedule
initial version: 0.1.0
edition: 2024
publish: false until release gates are accepted
license: MIT OR Apache-2.0
```

The old package names `ecs`, `ecs_macros`, and `scheduler` are removed during the
final cutover. No compatibility packages remain.

## RunenECS ownership

`runenecs` owns:

- entity allocation and lifetime;
- components, stateful components, bundles, and resources;
- storage and world mutation;
- queries and query-state caching;
- added/changed/orphaned observation;
- deferred commands;
- system parameters and ECS access declarations;
- ECS runtime-plan construction over `runen_schedule`;
- ECS-local event and deterministic tick-message semantics that pass the
  messaging review below;
- generic structural/change journals that do not encode networking policy;
- explicit reflection registries;
- ECS diagnostics, reports, and telemetry contracts;
- public macros through `runenecs_macros`.

## RunenSchedule ownership

`runen_schedule` owns context-generic schedule construction and deterministic
execution planning:

- schedule and set labels;
- registered systems over generic context `C`;
- access declarations and conflict validation;
- DAG construction;
- deterministic plan/topological order;
- plan reports and diagnostics;
- serial schedule execution;
- explicitly designed parallel execution only when deterministic and safety
  contracts are complete.

`runen_schedule` does not own:

- Runenwerk frame or tick phases;
- renderer-specific node exceptions;
- process-global logging switches;
- app lifecycle;
- ECS world storage;
- worker-pool ownership unless an accepted executor design proves it.

The current `frame_render_submit` logging exception and process-global slow-node
logging authority must not survive the extraction.

## Runenwerk ownership

Runenwerk retains:

- app and engine lifecycle;
- fixed-step, variable-step, frame, render, and product phase policy;
- plugin installation and product composition;
- ECS-to-render extraction;
- scene/world synchronization;
- spatial indexing adapters;
- networking, authority, replication, prediction, rollback, and transport;
- replay/history product policy;
- editor and diagnostics presentation;
- cross-repository compatibility tests.

## Spatial-index decision

Spatial indexing is removed from `runenecs` core.

Current ECS exports its own `SpatialHashConfig`, `SpatialHashIndex`, and
`SpatialIndex` using `geometry::Aabb3`, while Runenwerk already contains separate
`spatial` and `spatial_index` domains. Carrying both authorities into RunenECS
would preserve duplication and force a geometry dependency into ECS core.

Final direction:

```text
RunenECS
    owns entities and component data

Runenwerk spatial adapter
    observes selected entity/component changes
    maps entities to spatial keys/entries
    updates the accepted spatial index implementation
```

RunenECS may expose generic change observation required by the adapter. It must
not know AABBs, world coordinates, cells, or spatial query policy.

A future RunenSpatial extraction requires its own investigation and ADR. It is
not part of this program.

## Geometry decision

`runenecs` has no dependency on Runenwerk `geometry`.

Any geometry-bearing component is an application/domain type implemented by a
consumer. The ECS framework stores and queries it without understanding its
meaning.

Removing ECS-owned spatial indexing should eliminate the confirmed direct
geometry dependency. Any remaining use must be classified and moved to an
adapter or consumer.

## Scheduler decomposition

Three scheduling layers are distinct:

### Generic schedule semantics

Owned by `runen_schedule`:

- labels and sets;
- dependency graph;
- access-conflict validation;
- deterministic execution plan;
- generic context execution.

### ECS schedule integration

Owned by `runenecs`:

- transforming systems and system parameters into access declarations;
- world/resource borrowing;
- ECS runtime-plan reports;
- applying deferred commands at defined barriers;
- ECS-specific validation and diagnostics.

### Engine lifecycle policy

Owned by Runenwerk:

- frame/tick schedule instances;
- fixed/variable update policy;
- rendering phases;
- startup/shutdown;
- plugin ordering;
- product-specific phase labels.

Do not merge these into one universal runtime.

## Parallel execution decision

Do not promise unrestricted parallel execution merely because access declarations
exist.

Before public parallel scheduling is accepted, prove:

- sound disjoint world/resource access;
- deterministic barrier and command behavior;
- panic/error/poison policy;
- cancellation/shutdown behavior;
- worker-pool lifetime and ownership;
- ordering of diagnostics and traces;
- equivalence tests against serial execution;
- bounded task and queue behavior;
- no hidden global executor.

Until that gate passes, serial execution is the reference semantics.

## Reflection decision

Reflection uses an explicit registry value. Process-global mutable registration is
forbidden as final authority.

The design must distinguish:

- process-local Rust type identity;
- stable authored/serialized type keys;
- component/resource reflection capabilities;
- registration source and duplicate policy;
- registry lifetime and ownership;
- test isolation;
- plugin unload/reload behavior;
- schema/version compatibility.

A `World` may own or borrow a registry according to the final API design, but
reflection state must be explicit and independently constructible.

Macros may generate registration descriptors. They must not mutate global state
implicitly.

## Messaging decision

The current public surface exposes broadcast streams, tick buffers, and work
queues. They are retained only after each family has one precise semantic role.

Target classification:

### Events/broadcast

- one-to-many observation;
- explicit retention and overflow policy;
- reader cursors/lifetimes;
- no mutation authority;
- deterministic diagnostics.

### Tick-local buffers

- messages associated with a defined logical tick/window;
- deterministic drain/finalization;
- explicit provenance and capacity;
- no external transport semantics.

### Work queues

- one-consumer or claimed-work semantics;
- explicit retry/acknowledgement/failure policy;
- retained only if they are genuinely ECS-world coordination rather than an
  application job system.

### External ingress and networking

Remain Runenwerk-owned. Transport packets, network channels, replication messages,
remote authority, and replay streams do not become ECS messaging merely because
they eventually mutate a world.

The complete investigation must map every current consumer. Any family without a
clear independent ECS use is removed or moved before extraction.

## Change tracking and replication boundary

RunenECS may own generic facts such as:

- component/resource added, changed, and removed facts;
- entity spawn/despawn facts;
- structural deltas;
- bounded extraction windows;
- stable type keys where explicitly registered;
- deterministic journals and cursors.

Runenwerk owns:

- network entity mapping;
- replication schemas;
- authority and ownership policy beyond generic local descriptors;
- packet construction;
- transport;
- prediction and rollback;
- snapshot cadence;
- replay file format and product retention.

The existing ownership and change-extraction API must be reviewed field by field.
Only host-neutral local-world semantics transfer.

## Identity policy

Entity identity remains generational and runtime/world-local unless the current
implementation proves another accepted invariant.

Do not serialize raw process/runtime entity IDs as durable cross-run identities.
Runenwerk networking and persistence own explicit mapping to stable product IDs.

System, schedule, set, parameter-slot, broadcast, buffer, queue, owner, and type
identities each require a documented lifetime and serialization status.

## Error and terminal policy

Replace broad `anyhow` use at public boundaries with structured public errors
where consumers must branch on failure.

`anyhow` may remain an internal/application composition convenience only when it
does not erase public failure categories.

Define behavior for:

- invalid entity generations;
- borrow/access conflicts;
- duplicate registration;
- schedule cycles;
- command failure;
- system failure;
- panic/poisoning;
- capacity exhaustion;
- invalid change windows;
- shutdown and partial execution.

## Macro policy

`runenecs_macros` may generate implementations for public traits and registration
descriptors. Generated code must:

- use only public APIs;
- preserve generic parameters and where clauses;
- produce stable compile diagnostics;
- avoid hidden global registration;
- avoid Runenwerk paths;
- be proven from an external downstream crate;
- have compile-pass and compile-fail coverage.

## Public API review

Before extraction, inventory every public:

- module and re-export;
- trait and derive macro;
- type and public field;
- constructor;
- generic bound;
- unsafe boundary;
- error/result;
- lifetime promise;
- serialization promise;
- feature flag;
- telemetry hook.

The current broad crate-root exports are not accepted automatically.

## Independent conformance

RunenECS must prove, without Runenwerk:

- entity generation and stale-handle safety;
- component/resource lifecycle;
- queries and filters;
- deferred command ordering;
- change observation;
- system parameter/access declarations;
- schedule cycles, sets, barriers, and deterministic plans;
- serial reference execution;
- parallel equivalence if parallel execution is retained;
- explicit reflection registry isolation;
- accepted messaging semantics;
- external macro consumer;
- structured diagnostics and failure behavior;
- stable and MSRV validation;
- representative benchmarks.

At least one standalone simulation example must use only RunenECS public APIs.

## Extraction phases

### ECS-001 — Complete investigation

Read all ECS, macros, scheduler, spatial, networking, replay, renderer, app, tests,
examples, and benchmarks. Produce public API, consumer, and ownership matrices.

### ECS-002 — Decision closure

Finalize:

- scheduler package API;
- spatial deletion/migration;
- reflection registry;
- messaging families;
- change/replication split;
- identity and error policy;
- concurrency contract;
- macro compatibility;
- exact move/stay/redesign/delete map.

### ECS-003 — Boundary repair inside Runenwerk

- remove ECS geometry dependency;
- remove ECS-owned spatial index;
- remove product-specific scheduler behavior;
- make reflection explicit;
- prune or separate messaging families;
- separate networking/replay adapters;
- add independent downstream conformance.

### ECS-004 — Repository creation and transfer

Create RunenECS, establish governance/provenance, transfer corrected packages,
and validate independently.

### ECS-005 — Runenwerk cutover

Pin exact revisions, migrate consumers, delete original ECS/macros/scheduler
packages, regenerate the lockfile, run workspace and product validation, and
remove migration seams.

### ECS-006 — Closeout

Record compatibility, provenance, performance, deleted paths, and final ownership.

## Stop conditions

Stop source movement if:

- scheduler ownership remains ambiguous;
- ECS still depends on geometry;
- two spatial index authorities remain;
- reflection depends on process-global mutation;
- messaging families remain semantically overlapping;
- networking or replay policy remains in core ECS;
- serial/parallel semantics cannot be stated and tested;
- external macro conformance is incomplete;
- a long-lived compatibility package would be required.

## Definition of done

RunenECS is complete only when all three packages validate independently,
Runenwerk consumes exact revisions through one-way dependencies, no Runenwerk
policy remains in the framework, all original source packages are removed, and
standalone plus Runenwerk integration conformance is green.
