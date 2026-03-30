---
title: "Scheduler Crate"
description: "Documentation for Scheduler Crate."
---

# Scheduler Crate

## Purpose

Provides dependency-aware scheduling and node execution ordering.

## Usage

- Crate: `scheduler`
- Legacy DAG path: register `Node`s and dependency edges, then build a `Scheduler`.
- Typed path: register `RegisteredSystem`s in an `ExecutionScheduler` and run them by `ScheduleLabel`.

## Ownership Boundaries

- Owns graph validation, ordering, and execution orchestration.
- Does not own domain-specific plugin/system logic.

## Extension Points

- Add scheduling diagnostics and execution controls.
- Extend ordering/validation behavior while preserving deterministic execution.
- Use `SystemAccess` and `ExecutionPlan` to prepare for future parallel execution.
