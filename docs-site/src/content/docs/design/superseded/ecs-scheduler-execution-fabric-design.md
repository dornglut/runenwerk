---
title: ECS Scheduler Execution Fabric Design
description: Superseded draft for execution fabric, scheduler planning, product jobs, and diagnostics.
status: superseded
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/execution-fabric-and-product-jobs-design.md
---

# ECS Scheduler Execution Fabric Design

## Status

Superseded draft.

Replaced by `../accepted/execution-fabric-and-product-jobs-design.md`.

This document defines the long-term execution architecture for Runenwerk.

It is not an ECS rewrite plan, not a scheduler-only roadmap, and not a direct copy of Bevy, Flecs, Unity DOTS, Rayon, or any other runtime. It synthesizes useful ideas from those systems into a Runenwerk-specific architecture that respects the repository doctrine:

- domain crates own concepts and invariants
- runtime composes executable behavior
- important mutations go through explicit boundaries
- projected and derived state must not silently become authority
- diagnostics explain failures
- tests protect invariants
- descriptions and execution are separate

The design is intended to support the SDF-first production architecture, product systems, renderer product flows, procgen, physics, AI, streaming, multiplayer, editor tooling, replay, and future multithreading.

---

# Purpose

Runenwerk needs a production execution model for:

- ECS systems
- deterministic schedule planning
- safe multithreaded execution
- deferred commands and structural mutation
- deferred and snapshot queries
- field-product formation jobs
- render-product preparation jobs
- procgen jobs
- physics/contact jobs
- AI/influence jobs
- streaming/residency jobs
- VFX/product update jobs
- diagnostics jobs
- network replication jobs
- replay validation jobs
- background and budgeted work

The existing runtime can execute deterministic stages serially. The long-term system must evolve into an **execution fabric**:

```text
domain declarations
  -> formed execution products
  -> scheduler planning
  -> execution waves and barriers
  -> runtime job execution
  -> deferred mutation merge/apply
  -> product generation/freshness updates
  -> diagnostics and replay/network records
```

---

# Core Thesis

Runenwerk should not build a single universal graph that owns ECS, products, rendering, physics, procgen, and networking.

Instead:

```text
ECS remains live world truth.
Field products remain formed derived products.
Scheduler owns deterministic planning.
Runtime owns actual execution and threads.
Graph owns neutral graph structure.
Diagnostics explain state and failures.
```

The execution fabric coordinates these systems without collapsing their ownership.

---

# Design Goals

1. Preserve domain/runtime separation.
2. Keep ECS as live runtime state, not a generic product graph.
3. Keep field/product dataflow as derived product maintenance, not ECS replacement.
4. Support multithreaded execution through explicit access metadata and execution waves.
5. Support staged/deferred mutation with deterministic merge/apply points.
6. Support deferred queries and query snapshots without unsafe hidden reads.
7. Support product jobs for field, render, procgen, physics, AI, streaming, and diagnostics.
8. Support deterministic replay and multiplayer authority.
9. Support visual-only nondeterminism where explicitly allowed.
10. Expose actionable diagnostics for conflicts, stale queries, failed commands, budget misses, and authority violations.
11. Allow the runtime executor to evolve independently from domain scheduling contracts.
12. Keep implementation incremental, with serial fallback at every phase.

---

# Non-Goals

The execution fabric is not:

- a universal editor graph runtime
- a replacement for ECS storage
- a replacement for the scheduler crate
- a replacement for the graph crate
- a renderer
- a product registry
- a networking protocol
- a thread-pool API exposed to domain crates
- a global mutable job registry
- a "run anything anywhere" task system
- a license to bypass commands, ratifiers, or domain invariants

---

# Existing Baseline

Runenwerk already has the architectural seeds required for this design:

- ECS owns live entity/component/resource state, queries, systems, events, and deferred command concepts.
- Scheduler owns deterministic schedule planning, access validation, graph validation, labels, and execution plans.
- Runtime executes schedules and owns app lifecycle, plugins, runtime resources, rendering execution, and composition.
- Graph owns neutral graph substrate and must not become runtime execution authority.
- Semantic graph and gameplay lowering designs already expect authored graphs to lower into formed ECS/scheduler/runtime products.
- Adaptive Field Product System drafts define product identity, scope, lineage, freshness, residency, and diagnostics.
- Render-product architecture drafts expect render preparation and product resolution before render submission.

This design connects those pieces into a single long-term execution model.

---

# External Inspiration

This design takes inspiration from several domains without copying their APIs.

## Bevy-inspired ideas

Useful concepts:

- deferred commands
- explicit apply barriers
- parallel-safe systems based on access
- command application as a scheduled boundary
- fallible deferred entity operations

Runenwerk adaptation:

- deferred mutation becomes a first-class execution product and scheduler barrier
- failed deferred commands must produce diagnostics
- command application order must be deterministic

## Flecs-inspired ideas

Useful concepts:

- readonly execution windows
- per-thread or per-stage deferred command queues
- staged mutation during iteration
- deterministic merge points
- explicit staging API boundaries

Runenwerk adaptation:

- execution waves can run in readonly mode
- mutations are staged into buffers
- merge/apply is explicit and diagnosable
- ECS storage remains domain-owned

## Unity DOTS / job-system-inspired ideas

Useful concepts:

- explicit job dependencies
- handles for scheduled work
- dependency completion barriers
- read/write dependency tracking

Runenwerk adaptation:

- runtime may use internal job handles
- domain scheduler describes dependencies and barriers without owning thread-pool implementation
- product jobs declare inputs, outputs, access, budgets, and determinism

## Work-stealing executor inspiration

Useful concepts:

- worker-local queues
- work stealing
- scoped tasks
- parallel execution waves

Runenwerk adaptation:

- engine/runtime may use a work-stealing executor
- the scheduler remains executor-neutral
- serial fallback remains available for testing and debugging

## Compiler inspiration

Useful concepts:

- lowering from source to IR
- typed intermediate products
- dependency graphs
- passes and validation
- source maps
- diagnostics

Runenwerk adaptation:

- authored/editor/gameplay graphs lower into execution products
- schedule plans are formed artifacts
- diagnostics preserve source references and product lineage

## Database/build-system inspiration

Useful concepts:

- materialized views
- incremental invalidation
- cache keys
- rebuild graphs
- stale/fresh states

Runenwerk adaptation:

- field products are materialized views over world/authored/simulated state
- product jobs update freshness/generation
- dataflow is used for product maintenance, not for all ECS execution

---

# Ownership Boundaries

## `domain/ecs`

Owns:

- entity identity and lifecycle semantics
- component and resource storage contracts
- query descriptors and access metadata
- event channels where ECS-owned
- deferred ECS command descriptors
- ECS system interfaces
- ECS diagnostics and ratification where applicable

Does not own:

- runtime thread pools
- renderer execution
- product job execution
- app plugin orchestration
- backend-specific resources

## `domain/scheduler`

Owns:

- deterministic execution planning
- labels and ordering constraints
- access conflict detection
- execution waves/stages
- barriers
- apply-deferred nodes
- plan diagnostics
- schedule validation
- plan inspection DTOs

Does not own:

- ECS storage internals
- runtime worker threads
- GPU submission
- editor graph execution
- product registry authority

## `domain/graph`

Owns:

- graph identity
- nodes/ports/edges
- traversal
- cycle policy
- structural validation

Does not own:

- ECS runtime semantics
- scheduler policy
- field-product authority
- runtime execution

## Adaptive Field Product System

Owns:

- product identity
- product family/kind
- scope
- scale band
- lineage
- freshness
- residency
- rebuild/retention policy
- product diagnostics
- query contracts

Does not own:

- live ECS truth
- runtime worker execution
- backend GPU resources

## `engine/runtime`

Owns:

- actual execution of plans
- worker pool / executor implementation
- serial fallback executor
- job dispatch
- job handles or internal dependency primitives
- command buffer storage at runtime
- runtime metrics
- panic/error containment
- plugin orchestration
- render prepare/submit sequencing

Does not own:

- domain invariants
- field-product truth
- editor-only concepts in generic runtime APIs

## `net/*`

Owns:

- simulation tick identity
- command frames
- replay/history
- replication contracts
- transport-agnostic network state
- authoritative/deterministic simulation vocabulary

Does not own:

- local visual-only job execution
- renderer caches
- editor-only diagnostics

---

# Conceptual Model

## Execution Product

An **execution product** is a formed, validated description of work.

Examples:

- ECS system execution product
- ECS query product
- event flow product
- schedule plan product
- field-product job
- render-product prepare job
- procgen job
- collision/contact job
- AI influence job
- diagnostics job
- network replication job
- replay validation job

Execution products are descriptions. They are not worker-thread tasks by themselves.

## Execution Node

An **execution node** is a unit in a scheduler plan.

Possible node kinds:

- system node
- product job node
- query snapshot node
- deferred command apply node
- stage merge node
- barrier node
- main-thread node
- GPU submit node
- diagnostics node

## Execution Wave

An **execution wave** is a group of nodes that may execute concurrently if their access/dependency rules allow it.

A wave is not necessarily a thread. It is a scheduler-planned unit of parallel opportunity.

## Barrier

A **barrier** is an explicit boundary where progress must wait.

Examples:

- apply deferred commands
- merge worker command queues
- complete product job dependencies
- enter render submit
- finish authoritative simulation step
- publish product generation
- capture replay state

## Runtime Job

A **runtime job** is the engine-runtime execution object created from one or more execution nodes.

Runtime jobs may use worker threads, work stealing, serial fallback, or backend-specific constraints.

Domain crates should not depend on runtime job objects.

---

# Execution Flow

```text
1. Domain systems declare access and execution metadata.
2. Product systems declare inputs, outputs, freshness, budget, and authority metadata.
3. Scheduler receives formed execution descriptors.
4. Scheduler validates dependencies, conflicts, barriers, and determinism.
5. Scheduler emits an ExecutionPlan.
6. Runtime converts plan nodes to runtime jobs.
7. Runtime executes safe waves in parallel or serial fallback.
8. Deferred commands collect in stage/worker queues.
9. Barriers merge/apply deferred commands deterministically.
10. Product jobs publish outputs with generation/freshness updates.
11. Diagnostics record conflicts, failures, stale use, and budget misses.
12. Replay/network layers record authoritative execution facts where needed.
```

---

# Access Model

Runenwerk's access model must grow beyond ECS components.

## Access classes

| Access Class | Meaning |
|---|---|
| ECS component read | Reads component data. |
| ECS component write | Writes component data. |
| ECS resource read | Reads resource data. |
| ECS resource write | Writes resource data. |
| ECS structural mutation | Spawns/despawns/entities/components/resources. |
| Event channel read | Reads event stream/channel. |
| Event channel write | Emits events. |
| Product read | Reads formed product. |
| Product write | Forms or mutates product output. |
| Product freshness write | Updates product generation/freshness metadata. |
| Runtime cache read | Reads derived runtime cache. |
| Runtime cache write | Updates runtime cache. |
| Network authoritative write | Writes authoritative replicated/replay-relevant state. |
| Main-thread-only | Must run on main thread. |
| GPU-submit-only | Must run during GPU submit/backend phase. |
| Blocking IO | May block; must be scheduled away from hot frame path. |
| Background CPU | Can run as background work. |
| Visual-only | Does not affect authoritative state. |

## Access invariants

1. Conflicting writes cannot run in the same parallel wave.
2. Structural ECS mutation must be deferred unless a node has exclusive world access.
3. Product writes must declare output product identity.
4. Product freshness writes must be generation-safe.
5. Runtime cache writes cannot become product authority.
6. Main-thread-only nodes must be isolated.
7. GPU-submit-only nodes cannot run during domain planning.
8. Authoritative network/replay writes must be deterministic.

---

# Plan Model

The scheduler should evolve from simple stages toward explicit waves and barriers while preserving backwards compatibility.

## Target structure

```text
ExecutionPlan
  metadata
  diagnostics
  phases[]
    phase_id
    waves[]
      wave_id
      nodes[]
    barriers[]
```

## Phase examples

- PreUpdate
- FixedUpdate
- Update
- RenderPrepare
- RenderSubmit
- FrameEnd
- Background
- EditorInspection
- ReplayValidation

## Node metadata

Each node should declare:

- node identity
- node kind
- source descriptor
- read access
- write access
- dependency edges
- affinity class
- budget class
- determinism class
- authority class
- failure policy
- diagnostics hooks

## Barrier metadata

Each barrier should declare:

- barrier identity
- barrier kind
- dependencies completed
- queues merged
- products published
- command buffers applied
- replay/network capture behavior
- diagnostics behavior

---

# Deferred Mutation Model

Runenwerk already flushes deferred stage commands. This should become more formal.

## Deferred command sources

- ECS systems
- gameplay systems
- field/product jobs that request domain mutation
- editor commands routed through runtime
- network command frames
- replay playback
- physics/contact systems
- AI/gameplay systems

## Command queue levels

Candidate queue levels:

| Queue Level | Use |
|---|---|
| Per-system queue | Easy deterministic order, more overhead. |
| Per-worker queue | Efficient parallel writes, needs merge ordering. |
| Per-stage queue | Simple, lower parallelism. |
| Per-authority queue | Useful for network/replay authoritative commands. |
| Per-product queue | Useful for product mutation requests. |

Recommended long-term model:

```text
per-node command buffer
  -> optional per-worker staging
  -> deterministic merge by execution plan order
  -> apply-deferred barrier
```

## Apply rules

1. Application order is deterministic.
2. Failed commands produce diagnostics.
3. Entity lifecycle races are expected and reported, not panics by default.
4. Commands cannot silently mutate undeclared domains.
5. Commands that enqueue additional commands during apply must follow explicit policy.
6. Apply barriers are visible in the plan.

## Command failure classes

- target entity missing
- component missing
- resource missing
- authority rejected
- stale generation
- product missing
- ratification failed
- command not allowed in phase
- structural mutation forbidden in readonly window

---

# Deferred Query and Snapshot Model

Deferred queries should not mean "read ECS later and hope it is still valid."

Runenwerk should support three query modes.

## 1. Immediate Query

A system reads current ECS/world state during execution.

Requirements:

- access declared
- schedule conflict checked
- lifetime bounded to execution

## 2. Snapshot Query Product

A snapshot product captures query results at a known generation.

Use cases:

- renderer preparation
- AI planning
- diagnostics
- gameplay graph products
- editor inspection
- background product jobs

Properties:

- source generation
- scope
- freshness
- invalidation policy
- consumer class
- diagnostics

## 3. Deferred Query Request/Response

A query request is enqueued and answered later.

Use cases:

- expensive spatial queries
- async-ish editor inspection
- AI background planning
- procgen validation
- streaming/relevance checks
- diagnostics snapshots

Properties:

- request identity
- requested scope
- required freshness
- allowed fallback
- response generation
- diagnostics

## Query invariants

1. Query results carry source generation.
2. Stale query products are diagnosable.
3. Strict consumers can reject stale/fallback query results.
4. Deferred queries cannot mutate world state.
5. Background queries cannot silently read exclusive live ECS state.
6. Query snapshots are derived state unless explicitly designed otherwise.

---

# Product Job / Dataflow Model

Product jobs are how the Adaptive Field Product System forms and refreshes products.

## Product job examples

- field product formation
- SDF chunk product build
- render product preparation
- material product formation
- procgen chunk generation
- collision product formation
- influence field update
- vegetation density update
- water/wetness update
- diagnostics aggregation
- asset import/preview product generation
- streaming residency planning

## Product job descriptor

A product job should declare:

| Field | Meaning |
|---|---|
| Job identity | Stable execution product identity. |
| Job kind | Field, render, procgen, collision, influence, etc. |
| Inputs | Product/source dependencies. |
| Outputs | Product identities generated or refreshed. |
| Scope | Chunk, region, view, sector, basin, etc. |
| Scale band | Near, mid, far, summary, preview, etc. |
| Read/write access | ECS/product/runtime access sets. |
| Freshness behavior | How output freshness is updated. |
| Budget class | Frame, background, idle, offline, etc. |
| Priority | Importance/relevance. |
| Affinity | Worker/main/GPU/background. |
| Determinism class | Authoritative, deterministic, visual-only, nondeterministic allowed. |
| Failure policy | Retry, failed-preserved, reject, fallback. |
| Diagnostics | Structured issue output. |

## Dataflow rules

1. Product jobs declare inputs and outputs.
2. Input generation changes invalidate dependent outputs.
3. Jobs may be scheduled immediately, budgeted, lazy, idle, or offline.
4. Output publication updates generation/freshness atomically at a barrier.
5. Failed jobs preserve prior valid product only when policy allows.
6. Product dataflow does not replace ECS live state.

---

# Runtime Executor Model

The runtime executor is engine-owned.

## Responsibilities

- execute scheduler plans
- run serial fallback
- dispatch parallel waves
- manage worker threads
- enforce main-thread-only constraints
- enforce GPU-submit-only constraints
- maintain runtime job handles
- merge deferred command buffers
- enforce budgets
- collect metrics
- handle panics/errors
- expose diagnostics

## Executor modes

| Mode | Use |
|---|---|
| Serial | Deterministic debugging, tests, early implementation. |
| Parallel wave | Conflict-free ECS/system wave execution. |
| Background job | Long-running product formation/rebuild. |
| Main-thread lane | Windowing/backend/editor-sensitive work. |
| GPU-submit lane | Render backend submission. |
| Replay deterministic | Strict replay validation. |
| Server authoritative | Deterministic network/server simulation. |

## Executor invariants

1. Domain crates do not depend on executor implementation.
2. Serial fallback is always available.
3. Runtime metrics are inspectable.
4. Executor errors are surfaced as diagnostics.
5. Panics do not silently corrupt world state.
6. Job outputs are not published until barriers allow it.

---

# Determinism, Replay, and Multiplayer Authority

## Deterministic classes

| Class | Meaning |
|---|---|
| Authoritative deterministic | Must match across server/replay. |
| Deterministic local | Stable locally, not necessarily replicated. |
| Visual-only nondeterministic allowed | Can vary; not gameplay authoritative. |
| Background nondeterministic allowed | Allowed only for non-authoritative cache/previews. |
| Offline deterministic preferred | Used for reproducible product formation. |

## Replay rules

Replay-relevant execution must record:

- tick/frame
- command inputs
- schedule/product generations
- authoritative system outputs
- product generation changes where relevant
- diagnostic failures where relevant

## Multiplayer rules

Replicate:

- authoritative commands
- authoritative state transitions
- product generations where required
- dirty regions and world operations
- relevant simulation outputs

Do not replicate:

- render caches
- visual-only product jobs
- local diagnostics overlays
- local VFX buffers
- worker scheduling details

---

# Diagnostics Model

Diagnostics must be central.

## Scheduler diagnostics

- cycle detected
- unsatisfied dependency
- invalid barrier
- access conflict
- main-thread-only bottleneck
- product write conflict
- non-deterministic job in authoritative plan
- parallelism blocked reason

## ECS diagnostics

- missing entity during command apply
- component/resource missing
- stale query snapshot
- invalid deferred query
- structural mutation in readonly window
- command rejected by authority
- command apply failed

## Product job diagnostics

- missing input product
- stale input rejected
- fallback unavailable
- rebuild budget exhausted
- failed-preserved output
- output generation conflict
- product publication failed
- background job cancelled

## Runtime diagnostics

- worker panic
- executor budget exceeded
- main-thread queue overloaded
- GPU-submit barrier violation
- job handle leaked/uncompleted
- command buffer merge failure

## Network/replay diagnostics

- nondeterministic authoritative job
- replay mismatch
- missing generation record
- client/server authority mismatch
- out-of-order command frame
- stale replicated product generation

---

# Inspection Surfaces

The editor/debug layer should expose:

## Schedule Plan View

Shows:

- phases
- waves
- nodes
- barriers
- conflicts
- blocked parallelism
- main-thread-only nodes
- product jobs
- apply-deferred points

## Access Conflict View

Shows:

- systems/jobs
- read/write sets
- conflict edges
- suggested separation/barrier
- product write conflicts

## Deferred Command View

Shows:

- queued command counts
- source node
- merge order
- apply results
- failures

## Product Job View

Shows:

- job inputs
- outputs
- freshness
- budget
- status
- diagnostics
- generation updates

## Runtime Metrics View

Shows:

- serial/parallel execution time
- worker utilization
- blocked time
- budget misses
- background queue depth
- main-thread bottlenecks

---

# Relationship to Field Products

The execution fabric is the runtime coordination layer for product jobs.

Examples:

```text
field product changed
  -> product dependency invalidates render/collision/influence products
  -> product jobs scheduled
  -> scheduler places jobs according to access/budget
  -> runtime executes jobs
  -> product freshness/generation updates
```

This connects directly to:

- Adaptive Field Product System
- SDF Product Renderer Architecture
- Open World Product Streaming
- Field Product Diagnostics
- Procgen Field Product System
- SDF Physics Collision
- Field Influence AI

---

# Relationship to Renderer

Rendering should be split:

## RenderPrepare

Can use product jobs:

- select visible products
- prepare render product sets
- update GPU residency requests
- prepare diagnostics
- create offscreen view work

## RenderSubmit

Stricter:

- backend/GPU submission
- main-thread/backend constraints
- no live ECS extraction
- consumes prepared frame
- publishes renderer diagnostics

RenderSubmit is not a general job phase.

---

# Relationship to AI and Physics

## Physics

Physics can use:

- parallel broadphase jobs
- contact generation jobs
- strict collision query products
- apply barrier for movement/contact results
- deterministic authoritative execution when gameplay-relevant

## AI

AI can use:

- influence product jobs
- query snapshot products
- deferred query responses
- background planning jobs
- deterministic authoritative decisions where multiplayer/replay needs it

---

# Relationship to Procgen and Streaming

Procgen and streaming use background/budgeted product jobs.

Examples:

- generate terrain product
- place prefabs
- generate vegetation density
- form water masks
- build collision products
- update product residency
- prepare fallback summaries

They should publish formed products through product barriers, not mutate runtime state arbitrarily.

---

# Phased Implementation Plan

## Phase 1: Plan-Only Parallel Readiness

No actual multithreading yet.

Deliver:

- explicit parallelizable waves or wave annotations
- access conflict diagnostics
- barrier model
- apply-deferred node representation
- schedule plan inspection
- blocked-parallelism diagnostics

Acceptance:

- current serial runtime can execute the new plan shape
- plan can explain which systems could run in parallel and which cannot

## Phase 2: Deferred Mutation Hardening

Deliver:

- explicit command buffer model
- per-node command buffers
- deterministic merge order
- fallible apply diagnostics
- entity lifecycle race handling
- apply barrier inspection

Acceptance:

- deferred command failures produce structured diagnostics
- merge/apply order is deterministic

## Phase 3: Runtime Executor Abstraction

Deliver:

- engine-owned executor interface/resource
- serial executor implementation
- parallel-ready executor API
- main-thread lane
- background lane
- metrics
- error/panic policy

Acceptance:

- runtime can execute the same plan through serial executor
- executor is isolated from domain crates

## Phase 4: Parallel ECS Waves

Deliver:

- readonly execution window
- parallel execution of conflict-free waves
- worker-local command buffers
- deterministic merge/apply
- runtime metrics
- serial fallback parity tests

Acceptance:

- parallel and serial modes produce equivalent authoritative results

## Phase 5: Deferred Query and Snapshot Products

Deliver:

- query snapshot descriptor
- generation tracking
- stale query diagnostics
- request/response query queue
- strict/fallback query policy

Acceptance:

- render/AI/diagnostic jobs can consume stable query snapshots

## Phase 6: Product Job/Dataflow Integration

Deliver:

- product job descriptors
- product dependency scheduling
- freshness/generation update barriers
- budgeted background jobs
- failed-preserved policy
- product job diagnostics

Acceptance:

- field/render/procgen/collision/influence jobs can be scheduled as product jobs

## Phase 7: Network/Replay Determinism

Deliver:

- deterministic job class enforcement
- replay generation records
- network authority diagnostics
- visual-only nondeterminism policy
- authoritative plan validation

Acceptance:

- authoritative schedules reject nondeterministic jobs
- replay can validate key generation/state transitions

---

# Open Questions

## Scheduler

1. Does `ExecutionStage` evolve into `ExecutionWave`, or does the current name remain with clearer semantics?
2. Are apply-deferred barriers user-authored, compiler-inserted, scheduler-inferred, or mixed?
3. Are product jobs scheduled in the same `ExecutionPlan` as ECS systems or through linked product plans?
4. How are long-running background jobs represented relative to per-frame schedules?
5. How do budget classes affect ordering and starvation prevention?

## ECS

1. Are deferred command buffers per-node, per-system, per-worker, or hybrid?
2. What is the deterministic merge key?
3. Can commands enqueue more commands during apply?
4. Are entity lifecycle races warnings, errors, or command-specific outcomes?
5. Which query snapshots are valid across frames?

## Product Jobs

1. Who owns product dataflow dependency records?
2. Are product freshness updates applied at ECS barriers, product barriers, or both?
3. What is the minimal product job descriptor?
4. How are failed product jobs retried or preserved?
5. How are stale product inputs rejected per consumer?

## Runtime

1. Should the first executor use Rayon, a custom executor, or an internal abstraction with serial-first implementation?
2. How are main-thread-only jobs represented?
3. How are GPU-submit-only jobs represented?
4. What panic policy is acceptable for worker jobs?
5. What metrics are required in the first implementation?

## Network and Replay

1. Which jobs are authoritative?
2. Which products require generation records in replay?
3. How are visual-only nondeterministic jobs marked?
4. How does network relevance interact with product job scheduling?
5. How are command-frame boundaries represented in the execution plan?

---

# Design Decisions

1. The system is an execution fabric, not an ECS-only parallelization.
2. Domain crates describe work; engine runtime executes work.
3. Scheduler owns deterministic planning, not worker threads.
4. ECS remains live world truth.
5. Product dataflow is for derived products, not a replacement for ECS.
6. Deferred mutation is explicit and barriered.
7. Deferred queries require generation/freshness metadata.
8. Product jobs declare inputs, outputs, access, budgets, and determinism.
9. Serial fallback is mandatory.
10. Diagnostics are required for every blocked, failed, stale, or unsafe execution path.
11. Multiplayer/replay authority must be explicit per job/product.
12. Visual-only nondeterminism is allowed only when marked.

---

# Acceptance Criteria

This design is accepted when:

1. It cleanly maps current ECS and scheduler code into a future execution model.
2. It preserves domain/runtime ownership boundaries.
3. It supports multithreaded ECS execution through access-safe waves.
4. It supports staged/deferred mutation with deterministic apply.
5. It supports query snapshots and deferred query results.
6. It supports product jobs and dataflow freshness updates.
7. It supports renderer preparation, procgen, physics, AI, streaming, and diagnostics jobs.
8. It has a serial fallback implementation path.
9. It defines replay/network determinism rules.
10. It exposes actionable diagnostics and inspection surfaces.
