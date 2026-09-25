---
title: Execution Fabric and Product Jobs Design
description: Accepted execution architecture for ECS schedule invocation, product jobs, query snapshots, and deterministic product barriers.
status: accepted
owner: engine
layer: domain / engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: reference
pagefind: false
related_docs:
  - ../../architecture/repository-family-architecture.md
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
related_designs:
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
creating a universal scheduler. It does not own ECS-internal scheduling or
execution semantics.

Current reusable RunenECS semantics, execution capabilities, and conformance are
owned by standalone [`dornglut/runen-ecs`](https://github.com/dornglut/runen-ecs/blob/main/ARCHITECTURE.md).
[ADR 0014](../../adr/accepted/0014-repository-family-extraction-boundaries.md) and
the [framework integration architecture](../../architecture/repository-family-architecture.md)
own the Runenwerk-side framework boundary. Historical Issue #198 evidence remains in
the [current-main census](../../reports/investigations/runenecs-issue-198-current-main-census.md).

The durable split is:

```text
RunenECS
  reusable ECS state/query/system/schedule/deferred-command semantics
  standalone execution capabilities and conformance

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
package.

Implementation sequence for Runenwerk product jobs is tracked by the
[Runtime Product Job Executor Roadmap](../../engine/roadmaps/runtime-product-job-executor-roadmap.md).

## Purpose

Runenwerk needs to coordinate independently owned ECS schedules and product work:
field-product formation, render preparation, procgen, physics integration, AI
products, streaming, VFX, diagnostics, network integration, replay/capture, and
background jobs.

The execution fabric coordinates these without collapsing ownership:

```text
ECS remains live runtime state under standalone RunenECS contracts.
Field products remain formed product state.
Runenwerk owns product-job planning, host execution, and publication policy.
RunenNet owns reusable networking semantics.
Runenwerk `domain/graph` owns authored port-graph structure.
Diagnostics explain failures and provenance.
```

## Ownership

### RunenECS

RunenECS owns its reusable ECS semantics, public execution contract, diagnostics,
and conformance. Runenwerk consumes those facts through the exact accepted public
revision declared in its workspace and must not reconstruct or redefine them in a
second scheduler or local semantic handbook.

### Runenwerk

Runenwerk owns:

- application/frame/fixed/render/startup/shutdown policy;
- product-job descriptions and dependency planning;
- execution of product jobs;
- invocation of accepted RunenECS schedules at application-owned lifecycle points;
- worker threads and product-job parallel lanes owned by the Runenwerk execution fabric;
- main-thread/backend affinity and host constraints;
- product publication, query-snapshot publication, and application barriers;
- cross-framework failure/recovery policy, runtime metrics, and plugin composition;
- host/application networking integration and archival replay/capture policy.

A Runenwerk product plan may contain a node that invokes a RunenECS schedule, but
that node is an integration boundary. The product planner does not reinterpret the
schedule's internal ordering, access, execution, or deferred-command semantics.

### RunenNet

RunenNet retains protocol/schema identity, replication consistency,
session/authority semantics, delivery, acknowledgements/resynchronization,
recovery, transport-independent networking contracts, and separately accepted
prediction/interest semantics. Runenwerk supplies application policy and adapts
simulation state; it does not duplicate those network contracts.

### Graph

`domain/graph` owns Runenwerk-authored port-graph structure and validation where independently
justified. It does not become ECS or product execution authority.

## Two-level plan model

Runenwerk does not expose one universal execution-plan type for all work.

```text
RunenECS schedule contract
  reusable ECS ordering/access/execution semantics
  deferred-command boundaries
  ECS diagnostics

Runenwerk Product Plan
  product-job dependencies
  lifecycle placement
  product publication barriers
  host/backend affinity
  invocation of ECS schedules
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
scheduler contract. Runenwerk consumes the public RunenECS behavior exposed by its
exact dependency revision rather than specifying command buffering, publication,
or failure semantics locally.

Runenwerk may place application/product barriers before or after an ECS schedule
invocation. It must not merge, reorder, or partially reinterpret ECS deferred work
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

- RunenECS schedule/explain information consumed through its public diagnostics;
- product-plan and product-job views;
- ECS deferred-command diagnostics exposed by RunenECS;
- query-derived product freshness/provenance;
- runtime metrics;
- host networking/replay integration diagnostics.

Diagnostics must preserve the owner of each failure rather than flattening ECS,
network, rendering, or product semantics into one execution-fabric error model.

## Validation expectations

Future Runenwerk implementation work should prove:

- Runenwerk product publications happen only at explicit product barriers;
- Runenwerk invokes RunenECS schedules through the accepted public contract without
  reinterpreting framework-internal planning or execution semantics;
- product-job execution configuration does not transfer ECS semantic ownership into
  Runenwerk;
- stale query-derived products are diagnosable;
- authoritative product plans reject visual-only nondeterministic jobs;
- networking integration consumes RunenNet contracts rather than duplicating
  session, replication, or recovery semantics.

Reusable RunenECS executor equivalence, failure behavior, and framework conformance are
validated by the standalone RunenECS repository, not by this Runenwerk design.
