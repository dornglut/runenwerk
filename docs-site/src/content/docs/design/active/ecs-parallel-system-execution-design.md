---
title: ECS Parallel System Execution Design
description: Active design for future ECS schedule parallelism, deterministic command merging, world access sharding, and blocked-parallelism diagnostics.
status: active
owner: domain/ecs
layer: domain
canonical: true
last_reviewed: 2026-05-15
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
related_roadmaps:
  - ../../domain/ecs/features.md
  - ../../net/ecs-runtime-prioritized-roadmap.md
---

# ECS Parallel System Execution Design

## Status

Active design only. ECS execution remains serial while rendered-world V1 lands. Product jobs remain the active multithreading path for field/import/render-product work.

## Existing Groundwork

The scheduler already computes dependency waves and conflict diagnostics:

- `domain/scheduler/src/plan.rs::ExecutionScheduler::run_schedule`
- `domain/ecs/src/system/runtime.rs::Runtime::run_schedule`
- `domain/scheduler/src/plan.rs::SerialWaveMirrorsStage`

Current execution is serial by wave and serial within each wave. Deferred commands are not thread-safe and are flushed after each wave.

## Future Parallel Contract

Parallel ECS implementation must define:

- `Send + Sync` constraints for systems and resources that may run in parallel;
- explicit read/write access metadata for world sharding;
- per-wave command queues;
- deterministic command merge order;
- diagnostics for systems blocked from parallel execution;
- a serial fallback with identical observable results.

## Non Goals

- No parallel ECS implementation in the rendered-world V1 slice.
- No public ECS parallel APIs before the design is accepted with fitness tests.
- No hidden global mutable state to bypass scheduler access contracts.

## Implementation Sequence

1. Keep `domain/ecs/src/system/runtime.rs::Runtime::run_schedule` serial.
2. Add blocked-parallelism reporting from existing wave/conflict metadata.
3. Introduce thread-safe deferred command buffers behind an internal feature gate.
4. Add deterministic merge tests.
5. Add parallel wave execution only after serial/parallel equivalence tests exist.

## Tests

Required future coverage:

- serial and parallel schedules produce identical world state;
- command buffers merge deterministically;
- blocked systems report exact access conflicts;
- non-`Send + Sync` systems remain serial with diagnostics;
- scheduler wave diagnostics remain stable.
