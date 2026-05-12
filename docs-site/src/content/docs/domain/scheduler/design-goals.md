---
title: "Scheduler Design Goals"
description: "Documentation for Scheduler Design Goals."
status: active
owner: scheduler
layer: domain
canonical: true
last_reviewed: 2026-05-12
related_designs:
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
related_roadmaps:
  - ../../workspace/sdf-first-execution-roadmap.md
---

# Scheduler Design Goals

## Purpose
Define scheduler behavior for deterministic, validated system execution.

## Core Goals
- Deterministic execution order.
- Explicit dependency semantics.
- Fast and actionable validation failures.
- Low per-node runtime overhead.
- Serial fallback that remains equivalent to future parallel execution for
  authoritative results.

## Graph Invariants
- No self dependencies.
- Edges must reference existing nodes.
- Duplicate edges should not create duplicate execution.
- Cycles are rejected with clear diagnostics.

## API Direction
- Invalid graph configuration returns `Result` errors.
- Builder and graph mutation should avoid hidden side effects.
- Keep scheduler context generic (`Scheduler<C>`).
- Treat `ProductJobDescriptor` inputs as future planning descriptions, not as
  scheduler-owned product truth.

## Observability
- Preserve tracing hooks around node execution and scheduler run.
- Support graph introspection/export through explicit API.
- Expose plan diagnostics for access conflicts, blocked parallelism, missing
  barriers, product-job ordering issues, and authority violations.

## Testing Priorities
- Ordering correctness for dependency chains.
- Cycle detection and unknown-node failure paths.
- Duplicate-name and duplicate-edge behavior.
- Determinism across repeated runs.
- Barrier ordering for deferred apply and product publication.
- Serial/parallel-equivalence tests before enabling any parallel executor.

## Integration Goals
- First-class orchestration for ECS and UI stage pipelines.
- Simple integration with engine runtime contexts.
- SDF-first product jobs, query snapshots, renderer prepare/submit, replay, and
  network capture are sequenced through explicit phases, waves, and barriers.
