---
title: Execution Fabric and Product Jobs Design
description: Accepted execution architecture for ECS schedule invocation, product jobs, query snapshots, and deterministic product barriers.
status: accepted
owner: engine
layer: domain / engine-runtime
canonical: true
last_reviewed: 2026-08-25
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ./runenecs-extraction-boundary-design.md
  - ./runenecs-boundary-repair-execution-plan.md
  - ./sdf-first-field-world-platform-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
related_roadmaps:
  - ../../engine/roadmaps/runtime-product-job-executor-roadmap.md
supersedes:
  - ../superseded/ecs-scheduler-execution-fabric-design.md
---

# Execution Fabric and Product Jobs Design

## Status

Accepted execution architecture.

This design defines the Runenwerk application/product execution layer without
creating a universal scheduler. It does not own ECS-internal scheduling semantics.
The accepted
[RunenECS Extraction Boundary Design](./runenecs-extraction-boundary-design.md)
owns the ECS boundary, and the accepted
[RunenECS Boundary Repair Execution Plan](./runenecs-boundary-repair-execution-plan.md)
owns the repair sequence.

Issue #198 established the durable split:

```text
RunenECS
  ECS system identity and access facts
  explicit ECS ordering and sets
  ECS schedule validation
  deferred ECS command boundaries
  deterministic serial ECS reference execution

Runenwerk
  application/frame/fixed/render lifecycle
  product-job planning and execution
  host/main-thread/backend constraints
  barriers around ECS schedule invocation and product publication
  cross-framework composition and application policy

RunenNet
  networking protocol/session/replication/delivery/recovery semantics
```

There is no accepted external RunenScheduler dependency or `runen_schedule`
package. The
[Issue 198 current-main census](../../reports/investigations/runenecs-issue-198-current-main-census.md)
records the source evidence used for that reconciliation. No implementation or
source transfer is authorized by this clarification alone.

Implementation sequence for Runenwerk product jobs is tracked by the
[Runtime Product Job Executor Roadmap](../../engine/roadmaps/runtime-product-job-executor-roadmap.md).

## Purpose

Runenwerk needs to coordinate independently owned ECS schedules and product work:
field-product formation, render preparation, procgen, physics integration, AI
products, streaming, VFX, diagnostics, network integration, replay/capture, and
background jobs.

The execution fabric coordinates these without collapsing ownership:

```text
ECS remains live runtime state.
RunenECS owns ECS schedule semantics and reference execution.
Field products remain formed product state.
Runenwerk owns product-job planning, host execution, and publication policy.
RunenNet owns reusable networking semantics.
Graph owns neutral graph structure.
Diagnostics explain failures and provenance.
```

## Ownership

### RunenECS

RunenECS owns:

- entity/component/resource storage contracts;
- query and system-parameter access facts;
- ECS system identity and registration;
- explicit system ordering and sets;
- access-conflict classification and schedule validation;
- deferred ECS command semantics and application boundaries;
- deterministic standalone serial execution as the correctness/reference model;
- ECS plan diagnostics and explainability.

Runenwerk may inspect or present these facts through public RunenECS APIs. It does
not reconstruct or redefine them in a second scheduler.

### Runenwerk

Runenwerk owns:

- application/frame/fixed/render/startup/shutdown policy;
- product-job descriptions and dependency planning;
- execution of product jobs;
- invocation of accepted RunenECS schedules at application-owned lifecycle points;
- worker threads and future product-job parallel lanes;
- main-thread/backend affinity and host constraints;
- product publication, query-snapshot publication, and application barriers;
- cross-framework failure/recovery policy, runtime metrics, and plugin composition;
- host/application networking integration and archival replay/capture policy.

A Runenwerk product plan may contain a node that invokes a prepared RunenECS
schedule, but that node is an integration boundary. The product planner does not
reinterpret the schedule's internal order, access conflicts, or deferred-command
semantics.

### RunenNet

RunenNet retains protocol/schema identity, replication consistency,
session/authority semantics, delivery, acknowledgements/resynchronization,
recovery, transport-independent networking contracts, and separately accepted
prediction/interest semantics. Runenwerk supplies application policy and adapts
simulation state; it does not duplicate those network contracts.

### Graph

`domain/graph` owns neutral graph structure and validation where independently
justified. It does not become ECS or product execution authority.

## Two-level plan model

Runenwerk does not expose one universal execution-plan type for all work.

```text
RunenECS Schedule / PreparedSchedule
  explicit ECS order
  access compatibility
  deferred-command boundaries
  canonical serial execution
  ECS diagnostics

Runenwerk Product Plan
  product-job dependencies
  lifecycle placement
  product publication barriers
  host/backend affinity
  invocation of prepared ECS schedules
  product diagnostics
```

These plans may be composed by adapters, but each owner preserves its semantics.
A resource hazard does not silently create semantic order, and product lifecycle
policy does not leak into the ECS schedule contract.

## Product Job Descriptor

`ProductJobDescriptor` is the target description of formed product work:

- stable job identity;
- job kind;
- input products or source dependencies;
- output product identities;
- scope and scale band;
- read/write access to product-owned state;
- freshness and generation behavior;
- budget class and priority;
- affinity such as worker, main thread, background, or GPU-adjacent prepare;
- determinism class;
- authority class;
- failure policy;
- diagnostics output.

Product jobs update products through Runenwerk-owned publication barriers. They do
not replace live ECS state and do not mutate authoritative ECS state except
through accepted RunenECS APIs or explicit application-owned requests.

## ECS deferred mutation boundary

Deferred ECS mutation is a RunenECS semantic contract, not a Runenwerk product
scheduler contract. RunenECS defines command buffering, deterministic application,
failure behavior, and the points at which a prepared ECS schedule applies deferred
structural changes.

Runenwerk may place application/product barriers before or after an ECS schedule
invocation. It must not merge, reorder, or partially apply ECS command buffers
outside the RunenECS contract.

## Query snapshots

Deferred product work must not read live ECS state later without generation and
freshness context.

Runenwerk integration may form query-derived products through public RunenECS
queries and explicit extraction boundaries. Accepted product query modes include:

- immediate ECS query during RunenECS system execution;
- snapshot/query-derived product with source generation, scope, freshness,
  consumer class, and invalidation policy;
- deferred application request/response with requested freshness, fallback
  policy, response generation, and diagnostics.

Render, AI, diagnostics, editor inspection, background planning, and procgen may
consume query-derived products. Strict consumers can reject stale or fallback
results.

## Determinism and authority

Accepted Runenwerk product-job determinism classes include:

- authoritative deterministic;
- deterministic local;
- visual-only nondeterministic allowed;
- background nondeterministic allowed for non-authoritative caches;
- offline deterministic preferred.

These classes do not redefine RunenECS schedule correctness or RunenNet protocol
semantics.

Application integration that participates in networking or replay records the
application-owned inputs, tick/frame placement, mappings, product-generation
changes, and diagnostic failures required by the owning RunenNet/replay contract.
Visual-only jobs must not be accepted into authoritative product plans.

## Renderer relationship

Render preparation can resolve products, prepare render product selections,
request GPU residency, and collect diagnostics. Render submit consumes prepared
frames, performs backend submission, and must not perform live ECS extraction to
discover product state.

GPU-submit-only and main-thread-only work are Runenwerk/runtime constraints, not
RunenECS scheduling concepts.

## Diagnostics

Required inspection surfaces include:

- RunenECS schedule/explain view consumed through its public diagnostics;
- product-plan and product-job views;
- ECS deferred-command diagnostics from RunenECS;
- query-derived product freshness/provenance;
- runtime metrics;
- host networking/replay integration diagnostics.

Diagnostics must preserve the owner of each failure rather than flattening ECS,
network, rendering, or product semantics into one execution-fabric error model.

## Validation expectations

Future implementation work should prove:

- RunenECS serial execution defines ECS correctness and any optimized executor is
  observationally equivalent under accepted ECS semantics;
- access conflicts prevent unsafe ECS overlap without inventing semantic order;
- Runenwerk product publications happen only at explicit product barriers;
- Runenwerk invokes RunenECS schedules without reinterpreting their internal plan;
- stale query-derived products are diagnosable;
- authoritative product plans reject visual-only nondeterministic jobs;
- networking integration consumes RunenNet contracts rather than duplicating
  session, replication, or recovery semantics.
