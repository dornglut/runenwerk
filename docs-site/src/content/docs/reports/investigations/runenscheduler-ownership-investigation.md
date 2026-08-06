---
title: RunenScheduler Ownership Investigation
description: Current Runenwerk scheduler authority census, alternatives review, Option C rationale, proof workloads, and migration gates.
status: draft
owner: scheduler
layer: investigation
canonical: false
last_reviewed: 2026-07-29
related_docs:
  - ../../design/active/runenscheduler-design-canvas.md
  - ../../design/active/runenscheduler-core-semantics.md
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../workspace/planning/roadmap.md
---

# RunenScheduler Ownership Investigation

## Executive decision

Dornglut selects the independent-framework direction represented by parent issue
`#201`.

RunenScheduler is intended to become an independently useful domain-neutral framework
for deterministic in-process dependency and readiness planning.

This decision does **not** authorize immediate repository creation or extraction. The
program first proves one corrected neutral implementation inside Runenwerk through two
independent consumers:

1. RunenECS through an ECS-owned adapter;
2. a genuine non-ECS product or derivation pipeline above an existing host executor.

Only after those proofs may one implementation authority move to
`dornglut/runen-scheduler` and the original Runenwerk package be deleted.

## Evidence status

### Verified through GitHub source inspection

The investigation inspected exact Runenwerk main
`62c3949d31a7c03f1f554f8108120d9767139123` through the GitHub connector.

Verified files include:

- `domain/scheduler/Cargo.toml`;
- `domain/scheduler/src/lib.rs`;
- `domain/scheduler/src/access.rs`;
- `domain/scheduler/src/plan.rs`;
- `domain/scheduler/src/scheduler_core.rs`;
- `domain/scheduler/src/system.rs`;
- `domain/scheduler/src/label.rs`;
- `domain/scheduler/src/builder.rs`;
- `domain/scheduler/src/dag.rs`;
- `domain/scheduler/src/node.rs`;
- `domain/scheduler/src/telemetry.rs`;
- `domain/ecs/Cargo.toml`;
- `domain/ecs/src/system/runtime.rs`;
- `engine/Cargo.toml`;
- `engine/src/lib.rs`;
- `engine/src/runtime/jobs/**` selected execution and product-publication files;
- current repository-family and RunenECS design authority.

### Still required before this report can become canonical

A clean local checkout must execute the complete command gate from issue `#200`,
including Cargo metadata and inverse trees, repository-wide `rg` searches, scheduler
tests, strict Clippy, `cargo validate`, documentation build, and `git diff --check`.

The current authoring environment cannot resolve `github.com` directly. Connector code
search also returned upstream failures during the preliminary census. Therefore this
report does not claim a complete repository-wide consumer inventory or local
validation result.

Exact-head CI for the eventual planning pull request remains mandatory.

## Current package facts

`domain/scheduler` is a private workspace package named `scheduler`, version `0.1.0`,
with direct dependencies on `anyhow` and `tracing`. It exposes an optional local
telemetry feature.

`domain/ecs` directly depends on `scheduler` and forwards the telemetry feature.
`engine` independently declares direct dependencies on both `ecs` and `scheduler`.
`engine/src/lib.rs` broadly re-exports most scheduler modules.

A direct dependency and broad re-export are current implementation facts. They do not
prove an independently useful non-ECS planner consumer.

## Current authority is duplicated

The package contains two materially different scheduler authorities.

### Legacy generic DAG authority

The legacy surface includes:

```text
DAG<C>
Node<C>
NodeId
SchedulerBuilder<C>
Scheduler<C>
```

It owns executable callbacks and a mutable generic context.

Observed problems:

- public raw `NodeId(pub u64)` identity;
- unchecked `next_id += 1` allocation without structured exhaustion;
- one `DAG::topological_sort` path that iterates unordered `HashMap` keys and does not
  detect cycles;
- a second scheduler traversal with separate cycle detection and sorted IDs;
- string operational identity in the builder;
- accumulated string errors flattened into `anyhow`;
- stdout printing and DOT/file export inside the package;
- wall-clock measurement, process-global slow-node logging, and a hard-coded
  `frame_render_submit` exception;
- planning and callback execution in one authority.

Disposition:

> Delete or replace this authority. Do not extract it, wrap it, or preserve it as a
> compatibility path.

### ECS-oriented execution authority

The newer surface includes:

```text
ExecutionScheduler<C>
RegisteredSystem<C>
SystemId
SystemAccess
ScheduleLabel / SystemSet
ExecutionPlan
ExecutionStage / ExecutionWave
ExecutionBarrier
```

Observed problems:

- task callbacks and mutable context are stored inside the planner;
- public raw `SystemId::from_raw` construction;
- `saturating_add` allocation can repeat the terminal ID;
- `plans()` panics on rebuild failure;
- `plan_for()` converts rebuild failure into `None`;
- planning and serial callback execution remain combined;
- the primary shape is stage/wave execution rather than an asynchronous readiness DAG;
- phase meaning is inferred from Rust type-name suffixes;
- barriers hard-code ECS, product, query-snapshot, rendering, generation, replay, and
  networking policy;
- apply-deferred, product-publication, and query-snapshot barriers are generated after
  every wave;
- plan timing is written into process-global atomic telemetry.

Disposition:

> Retain no unchanged public authority. Separate neutral planning facts from ECS
> execution and Runenwerk lifecycle/publication policy.

## ECS-specific authority in the current scheduler

The current access model hard-codes:

```text
Component
OrphanedComponent
Resource
BroadcastStream
WorkQueue
TickBuffer
Structural
```

It also defines ECS-specific `drain` behavior. Structural write/write conflicts are
skipped because deferred command producers are expected to merge at stage end.

`ParamSlotDescriptor` describes `SystemParam` structure inside the scheduler package.
RunenECS converts query metadata into those scheduler domains, owns unsafe parameter
extraction, creates per-system command buffers, serially executes waves, and applies
commands through barrier handlers.

These are ECS semantics, not neutral planner semantics.

RunenECS must retain:

- component/resource/message access derivation;
- query and `SystemParam` safety;
- parameter-slot diagnostics;
- deferred command buffers and application;
- ECS system callbacks and world execution;
- ECS-specific ordering or structural boundaries.

A RunenScheduler adapter receives only normalized opaque task, dependency, and access
claims.

## Runenwerk-specific authority in the current scheduler

The following are host/product policy and remain outside the neutral core:

- `PreUpdate`, `FixedUpdate`, `Update`, `RenderPrepare`, `RenderSubmit`, and
  `FrameEnd` phase meaning;
- product and query-snapshot publication;
- generation finalization;
- replay/network capture;
- renderer wall-time exceptions;
- global telemetry and presentation;
- filesystem export.

Runenwerk retains lifecycle composition and decides when a prepared schedule is run.

## Existing physical execution authority

Runenwerk already owns a runtime product-job executor with:

- serial execution;
- bounded worker-pool execution;
- work stealing;
- bounded submission and backpressure;
- completion queues;
- panic and failure conversion;
- generations and stale-result rejection;
- deterministic staging of accepted product and query-snapshot outcomes.

This is strong evidence against a new production thread pool inside RunenScheduler V1.

It also provides the preferred physical executor for the non-ECS proof. The missing
capability is legal dependency/access readiness above that executor, not another
worker implementation.

## Alternatives review

The review distinguishes a **public contract** from reusable private machinery.

### Bevy ECS schedules

Bevy schedules are mature ECS execution authorities. A schedule owns systems,
metadata, conditions, deferred application, build settings, and an executor against a
Bevy `World`. Access conflicts and ambiguities are ECS-system concepts. Ambiguity
detection is configurable and defaults to ignored in current documentation.

Learning:

- derive access from domain metadata;
- expose conflict diagnostics;
- support explicit sets and conditions;
- do not couple RunenScheduler to another ECS world or executor;
- use stricter ambiguity defaults for framework authority.

Direct adoption is appropriate only if Dornglut adopts Bevy ECS itself.

### EnTT Organizer

EnTT Organizer is the closest architectural reference. It derives a safe execution
graph from resource requirements and returns the graph without executing it. The user
selects the execution mechanism.

Learning:

- planner/executor separation is viable and useful;
- read-only/read-write metadata can derive safe graph relationships;
- an inert graph can support multiple hosts.

Direct adoption is unsuitable because EnTT is C++ and its inference is tied to C++
function and registry semantics.

### Flecs pipelines

Flecs pipelines order ECS systems through phases and deterministic entity-ID ordering.
Systems remain ECS queries/functions executed by a pipeline.

Learning:

- stable declared order helps reproducibility;
- phase systems remain useful as an ECS adapter convenience;
- recycled or domain identity and phase policy must not define a neutral planner's
  durable semantics.

### Shipyard and Legion

Shipyard workloads and Legion schedules combine ECS borrow metadata, workload/system
registration, and execution against their worlds.

Learning:

- borrow-derived scheduling is ergonomic for an ECS adapter;
- ECS-specific workload ownership is not a neutral cross-domain public boundary.

### Dagga

Dagga models named DAG nodes that create, read, write, and consume resources and can
build schedules from those constraints. It is the closest Rust implementation
candidate.

Strengths:

- resource-aware DAG planning exists already;
- create/read/write/consume semantics fit derivation and render-graph workloads;
- using it privately could reduce algorithm implementation.

Risks:

- its public model is broader and more dataflow-oriented than V1 `Shared`/`Exclusive`;
- current documentation does not establish Dornglut's exact deterministic tie-break,
  diagnostics, identity, and provenance contract;
- exposing Dagga types would make external semantics part of the public compatibility
  surface.

Disposition:

> Prototype Dagga privately behind RunenScheduler conformance. Do not expose it as
> public authority.

### Petgraph, Daggy, and stable topological sorting

Petgraph provides graph storage and algorithms. Daggy adds a DAG-oriented wrapper.
`stable_toposort` provides deterministic topological order, deterministic layers, and
cycle evidence.

Strengths:

- established graph machinery can be reused;
- stable ordering directly supports RunenScheduler determinism;
- a low-level backend permits exact Dornglut semantics and diagnostics.

Limitations:

- these libraries do not define access keys, ambiguity, provenance, transactional
  planning, host boundaries, or conformance.

Disposition:

> Use as private machinery when it produces a smaller and clearer implementation than
> the Dagga adapter. Do not expose implementation types publicly.

### Rayon and oneTBB-style executors

Rayon and oneTBB-style runtimes distribute ready CPU work through worker pools and work
stealing. They answer where ready CPU work runs, not which domain work is causally or
resource-safe.

Learning:

- keep physical execution separate;
- avoid subsystem-specific full worker pools and oversubscription;
- use task granularity and bounded queues deliberately.

They are executor candidates, not replacements for the neutral planner.

### Tokio

Tokio schedules woken futures and integrates async I/O and timers. Its runtime offers
fairness conditions but does not promise ready-task order.

Learning:

- external completion and future readiness are a different problem;
- a later scheduler runtime can consume completion events without polling futures or
  replacing Tokio.

### Physics job systems

Jolt's job system supports dependency counters and jobs created by other jobs. Its
documentation states that the complete graph cannot always be constructed beforehand.
Physics engines commonly provide application job-system or dispatcher hooks.

Learning:

- stable engine schedules and dynamic per-step job graphs are separate layers;
- a later bounded fork/join runtime may be useful;
- third-party physics should initially retain its internal graph while sharing host
  execution resources;
- dynamic global graph mutation does not belong in V1 planning.

### GPU scheduling

CPU task readiness and GPU resource hazards are not the same model. GPU work also
requires stage, visibility, resource-state, queue, subresource, submission, and device
generation semantics.

Learning:

- RunenScheduler may later orchestrate CPU preparation, opaque GPU submission,
  completion, and CPU continuation;
- RunenGPU retains all device hazards and backend realization;
- GPU use must not be manufactured merely to justify the second consumer.

## Why own a custom contract

The custom value is not a new topological-sort algorithm or work-stealing queue.

Dornglut needs one combination not supplied as a neutral durable contract by the
reviewed alternatives:

- deterministic normalized planning;
- strict, explicit ambiguity policy;
- distinction between causal edges and access exclusions;
- immutable inert plans;
- stable serial conformance;
- complete edge and synchronization provenance;
- typed/opaque identities rather than string operational identity;
- structured errors and transactional candidate preparation;
- safe use by ECS and non-ECS adapters with different executors;
- implementation machinery that remains replaceable behind the public API.

## Selected proof workloads

### Proof 1: RunenECS

The ECS proof maps systems into neutral definitions while preserving ECS ownership.

Required cases:

- two read-only systems share an ECS key and can be ready together;
- read/write and write/write systems cannot overlap;
- result-sensitive order is explicit rather than inferred from conflict;
- command-producing systems write independent command buffers;
- an ECS-owned command-application task or boundary is explicit and inspectable;
- serial interpretation matches accepted current behavior;
- no `World`, query, `SystemParam`, command, broadcast, work-queue, tick-buffer, or
  drain type enters the neutral public contract.

### Proof 2: Runenwerk product derivation

The non-ECS proof uses a real product graph above `RuntimeJobExecutorResource`.

Required graph:

```text
source product
    +--> render derivation
    +--> collision derivation
    +--> query derivation
              |
              v
         validation
              |
              v
         publication
```

Required evidence:

- independent branches become ready without a global wave barrier;
- at least one shared or exclusive product/access key changes legal concurrency;
- serial, worker-pool, and work-stealing modes produce equivalent accepted outputs and
  deterministic publication order;
- stale generation and publication remain Runenwerk-owned;
- plan inspection explains every dependency and exclusion;
- the proof is not implemented as ECS systems.

## Candidate V1 implementation strategy

1. Define implementation-independent conformance before selecting a graph backend.
2. Prototype a private Dagga-backed planner.
3. Prototype a private stable-toposort or Petgraph-backed planner.
4. Compare adapter size, semantic fit, deterministic behavior, diagnostics, provenance,
   dependency risk, and maintenance cost.
5. Select the smallest backend satisfying the public contract without patches or
   leaked types.
6. Write custom graph algorithms only when both private approaches demonstrably fail.

## Migration and deletion map

### Delete

- legacy `DAG<C>`, `Node<C>`, `SchedulerBuilder<C>`, and callback-owning `Scheduler<C>`;
- duplicate topological algorithms;
- stdout and filesystem graph policy;
- process-global slow-node and plan telemetry policy;
- renderer-specific timing exception;
- broad engine re-exports of legacy scheduler internals;
- any final compatibility package or mirror after cutover.

### Move to or retain in RunenECS

- ECS access derivation and domain classifications;
- `ParamSlotDescriptor` and system-parameter diagnostics;
- unsafe parameter extraction;
- deferred command buffers and application;
- ECS system callbacks and world execution;
- ECS-specific labels and convenience authoring where not neutral.

### Retain in Runenwerk

- application and frame lifecycle;
- physical job executors and backpressure;
- generations and stale-result policy;
- product and query-snapshot publication;
- rendering, networking, replay, editor, and diagnostics presentation policy;
- adapters between framework identities and products.

### Implement as neutral RunenScheduler authority

- checked task/schedule identity;
- stable diagnostic keys;
- explicit dependencies;
- opaque access and placement keys;
- shared/exclusive compatibility;
- ambiguity policy;
- deterministic normalization;
- immutable readiness plans;
- provenance and structured diagnostics;
- canonical serial conformance.

### Transfer later

After two-consumer proof, transfer one accepted implementation authority into
`dornglut/runen-scheduler`, establish standalone validation and release policy, cut
RunenECS and Runenwerk over to exact accepted revisions, and delete
`domain/scheduler` from Runenwerk.

## Remaining planning work

Before this report can be accepted:

1. execute the complete current-main local command census;
2. confirm all manifests, imports, re-exports, tests, and non-ECS consumers;
3. bind exact ID, key-registry, ambiguity, exclusion, error, and conformance APIs;
4. update repository-family architecture and ADR authority;
5. supersede the stale `runen_schedule`-inside-RunenECS design language;
6. reconcile roadmap and active-work after the overlapping RunenGPU G4A branch is
   accepted or explicitly rebased;
7. run exact-head validation and independent complete-diff review;
8. publish exactly one next bounded child under #201.

## Primary references reviewed

- Bevy ECS schedule and schedule-build settings: `docs.rs/bevy_ecs`.
- EnTT Organizer: `github.com/skypjack/entt/wiki/Entity-Component-System`.
- Flecs systems and pipelines: `flecs.dev/flecs/md_docs_2Systems.html`.
- Shipyard workloads: `docs.rs/shipyard`.
- Legion schedules: `docs.rs/legion-systems`.
- Dagga: `docs.rs/dagga`.
- Petgraph, Daggy, and stable topological sorting: `docs.rs/petgraph`,
  `docs.rs/daggy`, and `docs.rs/stable_toposort`.
- Rayon and Tokio runtimes: `docs.rs/rayon` and `docs.rs/tokio`.
- Jolt JobSystem and physics architecture: `jrouwe.github.io/JoltPhysicsDocs`.
