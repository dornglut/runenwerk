---
title: Commands
description: Engine-agnostic guide for deferred ecs commands.
status: active
owner: runen-ecs
layer: domain
canonical: true
last_reviewed: 2026-09-10
---

# ECS Commands

Commands are deferred structural mutations collected during system execution and applied at ECS deferred-publication frontiers.

## Purpose

- Queue structural world mutations safely during system execution.
- Preserve deterministic command collection and application order in the serial reference executor.
- Avoid query/structure aliasing while a system is executing.

## Key Concepts

- `Commands`: ordinary transfer-safe per-system deferred command queue.
- `BatchCommands`: ordinary transfer-safe grouped command list applied in deterministic order.
- `LocalCommands` / `LocalBatchCommands`: explicit invoker-thread-only capabilities for local deferred effects.
- **DeferredPublicationFrontier**: ECS-owned visibility point reported only after the corresponding queued commands have been applied successfully.

## API Notes

- Helpers: `spawn`, `despawn`, `insert`, `remove`, `queue`, and `batch`.
- `commands.apply(world)` applies queued commands immediately when using manual world commands outside runtime-managed system execution.
- Runtime-managed command params are scope-bound to the system execution and collected by the runtime.
- Ordinary `Commands` are transfer-safe when their effects and system parameter proof support
  transferable registration. Use explicit `LocalCommands` with `.on_invoker_thread()` only for
  genuinely local deferred effects.
- Current planner stages may determine where the serial reference executor performs a flush, but planner-stage identity is not the public deferred-visibility contract.

## Invariants

- Runtime-deferred structural changes become visible only after the applicable ECS deferred-publication frontier.
- Deferred queues are staged only for successful system runs; failed schedule execution does not replay discarded queues later.
- Command queues are applied in deterministic reference-execution order.
- `BatchCommands` preserves command order and stops on the first error; earlier successful mutations remain applied according to the documented batch failure contract.
- Access conflicts alone do not introduce additional deferred-visibility boundaries.
