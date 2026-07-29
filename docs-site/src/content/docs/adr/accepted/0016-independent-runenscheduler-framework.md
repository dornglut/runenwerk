---
title: Establish RunenScheduler as an Independent Framework
description: Accepted ownership and dependency decision establishing RunenScheduler as the neutral dependency and readiness planner used by RunenECS and non-ECS hosts.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-29
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runenscheduler-design-canvas.md
  - ../../design/active/runenscheduler-core-semantics.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../reports/investigations/runenscheduler-ownership-investigation.md
related_roadmaps:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# ADR 0016: Establish RunenScheduler as an Independent Framework

## Decision

Establish **RunenScheduler** as an independently useful framework for deterministic
in-process dependency and readiness planning.

Target repository identity:

```text
product       repository                         package          crate
RunenScheduler target dornglut/runen-scheduler   runen-scheduler  runen_scheduler
```

The accepted dependency direction is:

```text
RunenScheduler
    +--> RunenECS scheduling adapter
    |        -> RunenECS system execution
    |             -> Runenwerk ECS integration
    |
    +--> Runenwerk non-ECS adapters
             -> product, asset, procedural, editor, and offline execution
```

RunenScheduler does not depend on RunenECS or Runenwerk. RunenECS and Runenwerk may
depend on exact accepted RunenScheduler revisions after the clean-cutover gate.

Repository creation and source transfer do not occur merely because this architecture
is accepted. The corrected neutral implementation is first proven inside Runenwerk
through RunenECS and one genuine non-ECS consumer. One implementation authority is
then transferred, consumers are cut over, and the original Runenwerk scheduler package
is deleted.

## Context

Runenwerk currently contains one package named `scheduler` with two competing
execution authorities:

- a callback-owning generic DAG scheduler;
- an ECS-oriented system scheduler with access metadata, phases, waves, barriers, and
  execution.

The package also mixes:

- ECS component/resource/message and deferred-command semantics;
- Runenwerk lifecycle, product, rendering, generation, replay, and networking policy;
- process-global telemetry and wall-clock behavior;
- filesystem and stdout output;
- executable callbacks and mutable host context.

That source is implementation evidence, not a reusable framework boundary.

RunenECS requires access-aware deterministic planning, but the same neutral capability
also has independent value for non-ECS product and derivation pipelines above
Runenwerk's existing execution infrastructure. Keeping the planner inside RunenECS
would force non-ECS consumers through ECS vocabulary or create duplicate planning
authority.

## RunenScheduler ownership

RunenScheduler owns:

- checked schedule-local task and schedule identities;
- stable diagnostic task keys and origin records;
- explicit dependencies and normalized neutral grouping relationships;
- opaque access and host-placement keys;
- shared and exclusive access claims;
- deterministic normalization and tie-breaking;
- explicit ambiguity policy;
- cycle, unknown-reference, identity, and constraint validation;
- immutable prepared readiness DAGs;
- predecessor and successor relationships;
- causal-edge and access-exclusion provenance;
- canonical serial interpretation;
- structured planning diagnostics and inspection products.

The core prepared plan is inert. It contains identities and scheduling facts, not
required callbacks, ECS worlds, thread pools, futures, or GPU objects.

## Excluded ownership

RunenScheduler does not own:

- entity, component, resource, query, or ECS world semantics;
- `SystemParam` extraction or unsafe borrowing contracts;
- deferred ECS command production and application;
- application, startup, shutdown, frame, fixed-step, or rendering phases;
- product generations, stale-result rejection, or publication;
- production worker pools, work stealing, affinity, parking, shutdown, or panic
  containment in V1;
- async I/O polling, sockets, timers, or future wake scheduling;
- GPU resources, hazards, states, barriers, queues, submissions, or device generations;
- physics-engine internal simulation graphs;
- durable or distributed workflows;
- calendar or cron scheduling;
- global telemetry, filesystem export, or diagnostics presentation.

## RunenECS consequence

RunenECS owns the adapter from ECS metadata into the neutral contract.

RunenECS retains:

- system registration and callback execution;
- access derivation from queries and `SystemParam`;
- world and resource borrowing safety;
- component, resource, broadcast, queue, tick-buffer, drain, orphaned-component, and
  structural semantics;
- deferred command buffers and explicit command application;
- ECS parameter and execution diagnostics.

Illustrative neutral mapping:

```text
ECS read claim    -> Shared opaque AccessKey
ECS write claim   -> Exclusive opaque AccessKey
ECS order         -> explicit dependency
ECS set relation  -> deterministic adapter expansion
command buffers   -> ECS-owned task outputs
command apply     -> explicit ECS-owned task or boundary
```

No `World`, query, `SystemParam`, `Commands`, or ECS-specific access-domain type enters
the RunenScheduler public contract.

## Runenwerk consequence

Runenwerk retains:

- application and engine lifecycle;
- physical serial, worker-pool, and work-stealing executors;
- queue capacity and backpressure;
- generations and stale-result handling;
- product and query-snapshot publication;
- editor, rendering, networking, replay, and diagnostics presentation policy;
- cross-framework adapters and exact-revision compatibility evidence.

The preferred non-ECS proof places neutral product dependency/access planning above
the existing `RuntimeJobExecutorResource`. RunenScheduler determines legal readiness;
Runenwerk's executor performs work; Runenwerk validates and publishes outcomes.

## GPU and physics consequence

GPU and physics are later integration evidence, not V1 core ownership.

RunenScheduler may later orchestrate CPU preparation, opaque GPU submission,
completion, and CPU continuation. RunenGPU retains all device hazard and submission
semantics.

Third-party physics engines may share host worker resources through adapters while
retaining their tested internal dynamic job graphs. Physics islands and solver stages
do not become neutral V1 concepts.

## Determinism and readiness consequence

The accepted plan model is a readiness DAG, not a mandatory sequence of global waves.

A task becomes causally ready when its own predecessors complete. Diagnostic
topological layers do not block unrelated successors.

The same normalized definition, planner contract revision, and explicit policy must
produce equivalent task order, dependencies, exclusions, diagnostics, cycle evidence,
provenance, and canonical serial order.

Access incompatibility and semantic order remain distinct. The planner does not
silently invent meaningful order merely to serialize conflicting tasks.

Every generated edge or boundary is inspectable and records why it exists.

## Implementation consequence

Dornglut owns the public semantic contract but does not require custom implementation
of established graph algorithms or worker infrastructure.

The first implementation program must compare private backends such as:

- Dagga for resource-aware DAG planning;
- stable topological sorting;
- Petgraph or Daggy graph storage and algorithms.

No third-party graph types become public API. A private backend is accepted only when
it satisfies deterministic conformance, structured diagnostics, provenance, identity,
and replaceability without a maintained fork or semantic workaround.

V1 does not create a new production thread pool or async runtime.

## Proof requirements

Before external extraction:

1. RunenECS constructs and consumes neutral plans through an explicit adapter.
2. A genuine non-ECS product or derivation pipeline constructs and consumes the same
   neutral contract above an existing host executor.
3. Both pass deterministic planning, inspection, error, and canonical serial
   conformance.
4. Serial and concurrent non-ECS execution produce equivalent accepted outputs and
   deterministic publication evidence.
5. No framework public API requires Runenwerk or ECS domain types.

Re-exports, lifecycle labels, hypothetical GPU use, an ordinary worker queue, or a
test-only fake consumer do not satisfy the second-consumer gate.

## Clean cutover

The RunenScheduler program proceeds:

1. inventory current source and every consumer;
2. accept the neutral canvas, core semantics, alternatives review, and move/delete map;
3. delete or isolate the legacy generic DAG and separate ECS/Runenwerk policy;
4. implement one minimal neutral planning authority inside Runenwerk;
5. prove RunenECS and non-ECS consumers;
6. authorize and bootstrap `dornglut/runen-scheduler`;
7. transfer one accepted implementation authority;
8. pin consumers to exact accepted revisions;
9. migrate all active consumers;
10. delete `domain/scheduler`, broad engine re-exports, and temporary seams;
11. close provenance, validation, release, and duplicate-authority evidence.

Temporary duplication may exist only on an unmerged transfer branch. Compatibility
packages, forwarding namespaces, aliases, mirrors, submodules, source includes, and
branch dependencies do not survive cutover.

## Initial package rule

RunenScheduler begins with one public package:

```text
runen-scheduler
```

Do not initially create core, executor, async, macros, GPU, physics, testing, facade,
or compatibility packages. A later package requires a proven independent dependency,
release, platform, MSRV, backend, or externally consumed conformance boundary.

## Consequences

### Positive

- RunenECS no longer owns a supposedly general planner.
- Non-ECS consumers avoid ECS vocabulary and duplicate DAG authority.
- Host executors remain replaceable and independently testable.
- Determinism, ambiguity, provenance, and serial conformance become explicit product
  guarantees.
- GPU and physics can integrate later without contaminating the neutral hazard model.
- The existing Runenwerk executor is reused rather than duplicated.

### Costs

- an additional framework repository and dependency edge must eventually be governed;
- RunenECS requires an adapter instead of directly owning all scheduling types;
- internal proof and two-consumer conformance precede extraction;
- shared repository-family and cutover changes require explicit serialization;
- public semantics must remain smaller than the explored dynamic/runtime feature set.

## Superseded direction

This ADR supersedes active language that places a neutral `runen_schedule` package
inside the future RunenECS repository or leaves scheduler ownership undecided between
ECS-local and independent forms.

ADR 0014 remains authoritative for repository independence, Runenwerk integration,
clean cutover, provenance, and removal of duplicate source authority. This ADR amends
the scheduler ownership portion of that architecture.
