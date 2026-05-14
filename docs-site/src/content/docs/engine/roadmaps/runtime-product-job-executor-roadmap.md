---
title: Runtime Product Job Executor Roadmap
description: Implementation sequence for turning the accepted execution fabric and product jobs design into engine runtime execution.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-14
related_designs:
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../design/accepted/sdf-first-field-world-platform-design.md
  - ../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
related_roadmaps:
  - ../../workspace/sdf-first-execution-roadmap.md
---

# Runtime Product Job Executor Roadmap

## Goal

Turn the accepted execution fabric and product jobs design into a reusable
engine runtime subsystem without making scheduler, ECS, product, or drawing
domains own runtime execution.

The long-term program includes runtime product jobs, persistent product caches
and package sidecars, work stealing, GPU jobs, full ECS parallel execution, and
cross-process or distributed jobs. Deferred below means deferred from the first
implementation batch, not excluded from the program.

## Current Baseline

- `domain/product` owns `ProductJobDescriptor`, publication outcomes, query
  snapshots, render selection, diagnostics, and ratification.
- `domain/scheduler` emits execution waves and deterministic barriers.
- `domain/ecs` owns `WorkQueue<T>` as world messaging. It is not the product
  job executor.
- `engine` owns product publication and query snapshot runtime resources, but
  current product work is still formed serially by app/runtime systems before
  those barriers.

## Phase RPJ0 - Roadmap And Status Alignment

Owner: documentation under `docs-site/src/content/docs/engine/roadmaps`.

Status: active.

Requirements:

- link this roadmap from workspace and execution-fabric indexes;
- keep the accepted design as architecture, not a duplicated phase list;
- make clear that later executor families are planned program phases.

Validation:

```text
python tools/docs/validate_docs.py
```

## Phase RPJ1 - Serial Runtime Product Job Executor

Owner: `engine/src/runtime/jobs`.

Status: initial implementation target.

Requirements:

- provide `RuntimeJobExecutorResource`, `RuntimeJob`, typed handles,
  generations, statuses, completions, submission errors, and diagnostics;
- default to serial execution so product-job behavior stays deterministic and
  easy to test;
- install the executor resource from app bootstrap;
- keep product visibility behind existing product publication and query snapshot
  barriers.

Validation:

```text
cargo test -p engine runtime_job
cargo test -p product
cargo test -p scheduler
cargo test -p ecs runtime_phase3
```

## Phase RPJ2 - Worker-Backed CPU Jobs

Owner: `engine/src/runtime/jobs`.

Requirements:

- add a bounded worker backend behind the same executor API;
- use owned input snapshots only;
- reject live `World`, renderer handles, windows, UI state, and backend objects
  from worker jobs by API shape and review;
- support queue capacity, worker count, per-frame completion drain budget,
  stale generation suppression, panic capture, and clean shutdown.

Validation:

```text
cargo test -p engine runtime_job
```

## Phase RPJ3 - Product Publication Integration

Owner: `engine/src/runtime/jobs/product.rs`.

Requirements:

- turn completed runtime product jobs into staged product publications and query
  snapshots;
- preserve deterministic staging order by generation, stage sequence, and
  `ProductJobId`;
- do not add a global product registry.

Validation:

```text
cargo test -p engine runtime_job
cargo test -p product
```

## Phase RPJ4 - Draw Tile Proof

Owner: `apps/runenwerk_draw`.

Requirements:

- move CPU preview and committed ink tile formation off the input hot path;
- keep immediate stroke feedback separate from tile catch-up;
- submit owned drawing document, stroke, and tile snapshots as runtime jobs;
- drain completions on the main thread and stage outputs through existing
  product/query barriers.

Validation:

```text
cargo test -p runenwerk_draw --test app_shell
```

## Phase RPJ5 - Persistent Product Caches And Package Sidecars

Requirements:

- add persistent cache only after product job identity, descriptor generation,
  lineage, and stale-result behavior are stable;
- keep generic cache primitives backend-neutral in engine/runtime;
- keep package and sidecar policy in owning apps or domains;
- include product id, kind, quality or scale band, descriptor generation, source
  revision, formation version, and producer lineage in cache identity.

## Phase RPJ6 - Work-Stealing Executor

Requirements:

- add work stealing as an executor backend, not a new product-job API;
- preserve serial and fixed-worker modes for debugging and deterministic tests;
- prove serial, fixed-worker, and work-stealing execution publish equivalent
  authoritative products.

## Phase RPJ7 - GPU Job Execution

Requirements:

- add GPU jobs only after CPU product jobs and render product selection are
  stable;
- require explicit render-prepare, render-submit, GPU-adjacent prepare, or
  readback affinity;
- publish GPU output through the same product/query barriers;
- never bypass CPU reference or fallback policy where an owning domain requires
  it.

## Phase RPJ8 - Full ECS Parallel Execution

Requirements:

- parallelize ECS waves only after product jobs no longer depend on this step;
- define `Send`/`Sync` policy for system params, resources, components, command
  buffers, and deferred merge order;
- preserve serial fallback and prove serial/parallel equivalence before default
  enablement.

## Phase RPJ9 - Cross-Process And Distributed Jobs

Requirements:

- require serialized job inputs and outputs, version compatibility, process
  lifecycle, trust policy, artifact transfer, timeout/retry policy, and
  preserved last-good products;
- do not start this phase before local job execution, cache identity, and
  failure diagnostics are stable.

## Invariants

- `domain/product` owns product descriptors and ratification, not execution.
- `domain/scheduler` owns plans and barriers, not worker threads.
- `domain/ecs::WorkQueue<T>` remains world messaging, not the executor.
- `engine/src/runtime/jobs` owns execution, queues, workers, completions, and
  runtime diagnostics.
- Product visibility changes only through publication/query barriers.
- Serial fallback remains permanent.
- Failed, stale, panicked, rejected, or timed-out jobs are diagnosable and must
  not silently clear last-good products.

## Stop Conditions

- Stop if a phase requires runtime concepts inside `domain/product`,
  `domain/scheduler`, or another pure domain crate.
- Stop if a worker job needs live `World` access.
- Stop if GPU work requires renderer-private app semantics.
- Stop if cache work needs package-format decisions before cache identity is
  stable.
- Stop if full ECS parallelism becomes a prerequisite for product jobs.
- Stop if distributed execution requires an unaccepted trust, security, or
  artifact format decision.
