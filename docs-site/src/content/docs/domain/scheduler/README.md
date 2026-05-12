---
title: "Scheduler Crate"
description: "Documentation for Scheduler Crate."
status: active
owner: scheduler
layer: domain
canonical: true
last_reviewed: 2026-05-13
related_designs:
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
related_roadmaps:
  - ../../workspace/sdf-first-execution-roadmap.md
---

# Scheduler Crate

## Purpose

Provides dependency-aware scheduling and node execution ordering.

## Usage

- Crate: `scheduler`
- Legacy DAG path: register `Node`s and dependency edges, then build a `Scheduler`.
- Typed path: register `RegisteredSystem`s in an `ExecutionScheduler` and run them by `ScheduleLabel`.

## Ownership Boundaries

- Owns graph validation, deterministic planning, execution waves, barriers, and
  plan diagnostics.
- Does not own runtime worker implementation, product truth, domain-specific
  plugin/system logic, or renderer/backend behavior.

## Extension Points

- Add scheduling diagnostics and execution controls.
- Extend ordering/validation behavior while preserving deterministic execution.
- Use `SystemAccess` and `ExecutionPlan` to prepare for future parallel execution.

## Accepted Direction

The accepted SDF-first execution fabric keeps the scheduler as the owner of
deterministic planning, not domain behavior or runtime worker implementation.

Near-term scheduler work should evolve the current typed path toward:

- phases and waves that remain serial-compatible;
- explicit barriers for deferred apply, product publication, and query snapshot
  publication after each serial wave, with render submit, generation
  finalization, and replay/network capture deferred to later phases;
- diagnostics for access conflicts, cycles, blocked parallelism, missing
  barriers, and invalid authority classes;
- product-job planning inputs without making the scheduler own product truth.

Serial fallback is mandatory. Future parallel execution must preserve the same
authoritative result as serial execution.

The SDF-first open-world substrate roadmap sequences scheduler work before
procgen code: product publication barriers and query snapshot publication must
be deterministic plan concepts before product-domain work relies on them.
