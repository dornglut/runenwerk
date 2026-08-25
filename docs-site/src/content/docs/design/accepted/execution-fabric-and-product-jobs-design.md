---
title: Execution Fabric and Product Jobs Design
description: Accepted execution architecture for scheduler plans, product jobs, deferred mutation, query snapshots, and deterministic barriers.
status: accepted
owner: engine
layer: domain / engine-runtime
canonical: true
last_reviewed: 2026-08-25
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
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

This design defines the long-term target without requiring immediate
multithreading. Serial fallback remains mandatory.

Current-main Issue #198 authority correction: ECS-native system identity, access
facts, explicit ordering/sets, schedule validation, deferred-command boundaries,
and deterministic serial reference execution belong to RunenECS. Runenwerk owns
application/frame/fixed/render/network/replay lifecycle and product barriers.
There is no accepted external scheduler dependency or `runen_schedule` package.
The [Issue 198 current-main census](../../reports/investigations/runenecs-issue-198-current-main-census.md)
records the evidence and exact move/stay/redesign/delete map. The ownership
statements below are read with this correction; no implementation or source
transfer is authorized by this amendment.

Implementation sequence is tracked by the
[Runtime Product Job Executor Roadmap](../../engine/roadmaps/runtime-product-job-executor-roadmap.md).
That roadmap turns this accepted architecture into engine runtime phases while
keeping this document focused on ownership and invariants.

## Purpose

Runenwerk needs an execution model for ECS systems and product jobs,
deferred mutation, query snapshots, field-product formation, render preparation,
procgen, physics, AI, streaming, VFX, diagnostics, network, replay, and
background jobs.

The execution fabric coordinates these systems without collapsing ownership:

```text
ECS remains live runtime state.
Field products remain formed product state.
RunenECS owns deterministic ECS planning; Runenwerk owns product planning.
Runtime owns actual execution.
Graph owns neutral graph structure.
Diagnostics explain failures.
```

## Ownership

`domain/ecs` owns entity/component/resource storage contracts, query metadata,
system interfaces, deferred ECS command descriptors, and ECS-owned diagnostics.

RunenECS owns ECS labels, ordering constraints, access conflict detection,
deterministic stages or waves, deferred-command boundaries, and ECS plan
diagnostics. Runenwerk owns application/product barriers and lifecycle policy.

`domain/graph` owns neutral graph structure and validation. It does not become
the runtime execution authority.

Engine runtime owns executor implementation, serial fallback, worker threads or
future parallel lanes, main-thread/backend constraints, command buffer storage,
product job dispatch, runtime metrics, panic/error policy, and plugin
composition.

## Execution Plan Model

RunenECS can keep a serial-compatible execution-plan shape while evolving
semantics toward validated waves and ECS-owned deferred boundaries. Runenwerk
may add product barriers around that execution. A future plan may
include:

- phases such as update, fixed update, render prepare, render submit, frame end,
  background, editor inspection, and replay validation;
- waves of conflict-free systems or product jobs;
- explicit barriers for deferred apply, product publication, render submit,
  generation finalization, and replay/network capture;
- diagnostics for blocked parallelism, cycles, access conflicts, missing
  barriers, and invalid authority classes.

Implementation may evolve incrementally. The accepted invariant is explicit
planning and barriered publication, not a specific thread-pool API.

## Product Job Descriptor

`ProductJobDescriptor` is the target description of formed product work:

- stable job identity;
- job kind;
- input products or source dependencies;
- output product identities;
- scope and scale band;
- read/write access;
- freshness and generation behavior;
- budget class and priority;
- affinity such as worker, main thread, background, or GPU-adjacent prepare;
- determinism class;
- authority class;
- failure policy;
- diagnostics output.

Product jobs update products through publication barriers. They do not replace
live ECS state and do not mutate authoritative domain truth without a governed
command or mutation request.

## Deferred Mutation

Structural and authoritative mutation must be explicit and deterministic.

Accepted model:

```text
per-node command buffers
  -> optional worker staging
  -> deterministic merge by execution plan order
  -> apply-deferred barrier
  -> diagnostics and generation updates
```

Failed commands produce diagnostics instead of success-shaped failures. Entity
lifecycle races, missing components, stale generations, authority rejection, and
ratification failures must be visible.

## Query Snapshots

Deferred queries must not read live state later without generation context.

Accepted query modes:

- immediate query during system execution with declared access;
- snapshot query product with source generation, scope, freshness, consumer
  class, and invalidation policy;
- deferred query request/response with requested freshness, fallback policy,
  response generation, and diagnostics.

Render, AI, diagnostics, editor inspection, background planning, and procgen may
consume query snapshots. Strict consumers can reject stale or fallback query
results.

## Determinism And Authority

Accepted determinism classes:

- authoritative deterministic;
- deterministic local;
- visual-only nondeterministic allowed;
- background nondeterministic allowed for non-authoritative caches;
- offline deterministic preferred.

Network and replay relevant execution must record command inputs, tick/frame,
authoritative transitions, product generation changes where required, and
diagnostic failures where relevant. Visual-only jobs must not be accepted into
authoritative plans.

## Renderer Relationship

Render prepare can resolve products, prepare render product selections, request
GPU residency, and collect diagnostics. Render submit consumes prepared frames,
performs backend submission, and must not perform live ECS extraction to
discover product state.

GPU-submit-only and main-thread-only work are runtime constraints, not domain
concepts.

## Diagnostics

Required inspection surfaces:

- schedule plan view;
- access conflict view;
- deferred command view;
- product job view;
- runtime metrics view;
- replay/network authority diagnostics.

Diagnostics must explain cycles, blocked parallelism, missing inputs, stale
query snapshots, budget misses, failed product jobs, command apply failures,
worker errors, and authority violations.

## Validation Expectations

Future implementation work should prove:

- serial execution remains equivalent to future parallel execution for
  authoritative results;
- access conflicts prevent unsafe parallel waves;
- command merge/apply order is deterministic;
- product publications happen only at explicit barriers;
- stale query products are diagnosable;
- authoritative plans reject visual-only nondeterministic jobs.
