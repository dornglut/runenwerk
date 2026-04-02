---
title: Commands
description: Engine-agnostic guide for deferred ecs commands.
---

# ECS Commands

Commands are deferred structural mutations applied at stage boundaries.

## Purpose

- Queue structural world mutations safely during system execution.
- Preserve deterministic mutation order across systems in a stage.
- Avoid query/structure aliasing during a running stage.

## Key Concepts

- `Commands`: per-system deferred command queue.
- `DeferredCommand<T>`: typed command trait.
- `BatchCommands`: grouped command list applied in deterministic order.
- `Runtime` stage flush: queued commands become visible only after stage completion.

## API Notes

- Helpers: `spawn`, `despawn`, `insert`, `remove`, `queue`, `defer`, `batch`.
- `commands.apply(world)` runs queued commands immediately when using manual world commands.
- Runtime-managed `Commands` params are scope-bound to the system execution.

## Invariants

- Deferred structural changes are visible only after stage flush.
- Command queues from failed schedule runs are discarded.
- Batch command order is deterministic and preserved.
