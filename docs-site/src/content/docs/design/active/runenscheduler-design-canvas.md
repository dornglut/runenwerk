---
title: RunenScheduler Design Canvas
description: Concise product and ownership canvas for the independent domain-neutral RunenScheduler framework.
status: draft
owner: scheduler
layer: domain/scheduler
canonical: false
last_reviewed: 2026-07-29
related_docs:
  - ./runenscheduler-core-semantics.md
  - ../../architecture/repository-family-architecture.md
  - ../../reports/investigations/runenscheduler-ownership-investigation.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../workspace/planning/roadmap.md
---

# RunenScheduler Design Canvas

## Mission

RunenScheduler is a domain-neutral Rust framework for deterministic in-process
dependency and readiness planning.

It accepts task metadata, explicit dependencies, shared or exclusive access claims,
and host-defined placement constraints. It produces immutable and inspectable plans
that external executors can interpret serially or concurrently.

RunenScheduler answers **what work is legal and ready**. It does not decide which
operating-system thread or GPU execution unit physically performs that work.

## Category

RunenScheduler is a general-purpose scheduler within one deliberately bounded
category:

> In-process tasks whose legal execution depends on explicit causality, access
> compatibility, and host placement.

It is not:

- a calendar or cron scheduler;
- a durable distributed workflow engine;
- an async future runtime;
- a production thread pool by default;
- a GPU resource-hazard planner;
- an ECS world or system runtime;
- a hard real-time operating-system scheduler.

## Primary users and proof workloads

### RunenECS

RunenECS derives neutral task and access claims from ECS systems and consumes a
prepared schedule through an ECS-owned execution adapter.

RunenECS retains:

- `World`, entity, component, resource, and query safety;
- `SystemParam` state and extraction;
- deferred command production and application;
- ECS-specific access domains and diagnostics;
- system callback execution.

The proof must demonstrate deterministic planning, read/read compatibility,
read/write and write/write exclusion, explicit semantic ordering, and serial
conformance without moving ECS semantics into RunenScheduler.

### Non-ECS product or derivation pipeline

A genuine second consumer must construct and consume the same neutral contract
without routing through RunenECS.

The preferred proof workload is a Runenwerk product derivation graph above the
existing runtime job executor:

```text
generate source product
    +--> build render product
    +--> build collision product
    +--> build query product
              |
              v
       validate products
              |
              v
       publish generation
```

This proof uses RunenScheduler for legal readiness and the existing Runenwerk job
executor for physical serial, worker-pool, or work-stealing execution.

Re-exported labels, an ordinary worker queue, hypothetical GPU use, or a test-only
example do not count as the second consumer.

### Later adapters

Procedural generation, asset import, editor derivation, offline simulation, physics
job integration, and CPU/GPU causal orchestration are plausible later consumers.
They are evidence workloads, not foundational domain concepts.

## Problems solved

RunenScheduler provides one stable answer to these questions:

1. Which tasks belong to this schedule?
2. Which tasks explicitly depend on which others?
3. Which tasks claim compatible or incompatible access?
4. Which tasks are ready after a particular completion?
5. Why does an edge, exclusion, or rejection exist?
6. What is the deterministic serial interpretation?
7. Can different host executors consume the same legal plan?
8. Can invalid candidate definitions fail before replacing accepted authority?

Existing graph libraries solve graph storage and algorithms. Existing thread pools
execute ready work. Existing ECS schedulers own ECS worlds and systems. RunenScheduler
owns the neutral contract between domain metadata and host execution.

## Neutral core model

```text
ScheduleDefinition
    tasks
    explicit dependencies
    access claims
    placement claims
          |
          v
Deterministic Planner
    normalize identities
    validate references
    detect cycles
    derive incompatibilities
    validate ambiguity policy
          |
          v
PreparedSchedule
    immutable task facts
    predecessor counts
    successor relationships
    explicit exclusions
    edge provenance
    canonical serial order
    structured diagnostics
          |
          v
Host Adapter and Executor
```

The prepared schedule is a readiness DAG. Topological layers may be exposed for
inspection, visualization, or conservative execution, but they do not imply global
runtime barriers.

For:

```text
A -> C
B -> D
```

`C` becomes ready when `A` completes. It does not wait for `B` unless an explicit
relationship requires that wait.

## Core vocabulary

### Identity

Operational identity is checked and schedule-local. Stable keys and labels exist for
inspection, conformance, and plan comparison, but strings are not the primary safety
identity.

### Dependency

An explicit dependency states semantic order:

```text
A before B
```

### Access

The neutral access vocabulary is:

```text
Shared
Exclusive
```

Compatibility is:

```text
Shared    + Shared     compatible
Shared    + Exclusive  incompatible
Exclusive + Exclusive  incompatible
```

Access keys are opaque. Adapters decide whether a key denotes an ECS component,
product, chunk, service, arena, hardware interface, or another non-reentrant
capability.

### Placement

Placement keys are opaque host facts. The core does not define built-in main-thread,
GPU, editor, physics, or background lanes.

### Ambiguity

Access incompatibility proves that two tasks cannot overlap. It does not prove which
one has semantic precedence. The accepted ambiguity policy must reject, warn, or
explicitly permit stable serialization without silently inventing meaning.

## Defining invariants

### Deterministic planning

The same normalized input and planner version produce the same:

- task order;
- dependency and exclusion facts;
- diagnostics and cycle evidence;
- provenance ordering;
- canonical serial order;
- serialized inspection representation.

Physical worker assignment and completion timing may remain opportunistic.

### Inert prepared plans

Prepared plans contain identities and scheduling facts. They do not require stored
callbacks, ECS worlds, thread pools, WGPU objects, or backend commands.

### No unexplained synchronization

Every inferred edge or generated boundary records:

- its origin;
- affected tasks;
- affected access or placement key where applicable;
- the owning adapter or policy;
- whether the relationship is required for correctness or execution policy.

Generated synchronization is allowed. Hidden synchronization is not.

### Transactional preparation

A malformed candidate definition never partially replaces an accepted plan.
Unknown references, cycles, invalid identities, unsatisfied constraints, and rejected
ambiguities return structured failures before activation.

### Canonical serial interpretation

Every accepted plan has one deterministic serial interpretation for reference tests,
debugging, and serial-versus-concurrent conformance.

### Bounded future dynamism

Dynamic child work is not part of V1. A later scoped model may allow child tasks to
inherit or narrow parent authority, but never to broaden access to unrelated global
state or retroactively modify dispatched work.

## Ownership boundary

### RunenScheduler owns

- checked schedule-local identities;
- stable diagnostic keys and provenance;
- explicit dependency relationships;
- opaque access and placement keys;
- shared and exclusive claims;
- deterministic normalization and tie-breaking;
- ambiguity policy;
- cycle, reference, and conflict diagnostics;
- immutable readiness plans;
- canonical serial order;
- inspection and conformance products.

### RunenScheduler does not own

- application or frame lifecycle;
- task callbacks or domain objects;
- ECS queries, borrowing, commands, or world safety;
- production worker creation, work stealing, affinity, and shutdown;
- async I/O polling and timers;
- Runenwerk generations, stale-result policy, and publication;
- physics simulation topology;
- GPU resources, barriers, queues, submissions, or device generations;
- global telemetry, filesystem output, or stdout policy.

## Candidate V1

V1 contains only:

- checked IDs and stable task keys;
- explicit dependencies;
- opaque access keys;
- shared and exclusive access claims;
- deterministic normalization and conflict derivation;
- explicit ambiguity policy;
- cycle and unknown-reference diagnostics;
- immutable readiness plans;
- canonical serial order;
- edge provenance and structured inspection;
- a serial conformance interpreter or harness.

V1 excludes:

- a new production thread pool;
- dynamic child scopes;
- priorities and deadlines;
- NUMA or heterogeneous-core placement;
- external async completion;
- GPU integration;
- generation and stale-result semantics;
- weighted critical paths;
- distributed or persistent execution.

## Program and extraction gates

### Planning gate

The current Runenwerk scheduler authorities, consumers, errors, side effects, and
execution infrastructure are inventoried from exact current main. The design canvas,
core semantics, alternatives review, proof workloads, and migration/deletion map are
accepted through one planning pull request.

### Internal implementation gate

One minimal neutral planner authority is implemented in Runenwerk without preserving
the legacy generic DAG or embedding ECS and Runenwerk lifecycle policy.

### Two-consumer proof

RunenECS and one genuine non-ECS workload pass the same deterministic and inspection
conformance contract using separate adapters.

### Extraction gate

Only after internal proof may Dornglut authorize and bootstrap
`dornglut/runen-scheduler`, transfer one implementation authority, cut consumers over
to exact accepted revisions, and delete the original Runenwerk package and broad
re-exports.

No forwarding crate, alias, source mirror, include, submodule, branch dependency, or
parallel implementation authority survives the cutover.

## Design position

RunenScheduler is justified by a reusable, strict contract rather than by novelty in
graph algorithms or thread execution:

> A deterministic and inspectable readiness authority that separates legal work from
> physical execution and remains neutral across independent consumers.
