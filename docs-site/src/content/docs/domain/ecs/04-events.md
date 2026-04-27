---
title: Events
description: Engine-agnostic guide for ecs events and reactive systems.
status: active
owner: ecs
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# ECS Events

Events are typed signals exchanged between systems through world-managed channels.

## Purpose

- Decouple systems with typed message passing.
- Support frame-transient and retained channel lifetimes.
- Enable per-system incremental reads with cursor-based channels.

## Key Concepts

- World APIs: `publish_broadcast<T>`, `read_broadcast<T>`, `drain_broadcast_admin<T>`, `clear_broadcast_admin<T>`.
- Param APIs: `BroadcastReader<T>`, `BroadcastWriter<T>`.
- Channel config: `BroadcastStreamConfig` (`capacity`, `overflow`, `lifetime`, `tracing`).
- Observers: `observe_events` with `ObserverTrigger::{OnEmit, OnDrain, EndOfFrame}`.

## API Notes

- `BroadcastReader<T>::iter_all()` reads all pending events.
- `BroadcastReader<T>::iter_new()` reads only unseen events for that system param state.
- `finalize_frame_boundary()` applies frame-lifetime cleanup and end-of-frame observer triggers.

## Invariants

- `FrameTransient` channels are cleared at end-of-frame processing.
- Overflow policy is enforced per channel (`DropOldest`, `DropNewest`, `Panic`).
- Observer notifications are generated only on configured trigger boundaries.

## Current Constraints

The current event channel model is intentionally lightweight and currently combines
multiple messaging roles that should be separated for long-term multiplayer/runtime
stability:

- broadcast-style fan-out notifications,
- queue-like destructive workflows,
- runtime/network bridge traffic.

Also note that `finalize_frame_boundary()` is currently a world-level API and must be
called by the runtime lifecycle to enforce frame cleanup boundaries.

For current repository-grounded status and the planned redesign sequence, see:

- [../../net/ecs-runtime-feature-inventory.md](../../net/ecs-runtime-feature-inventory.md)
- [../../net/ecs-runtime-gap-summary.md](../../net/ecs-runtime-gap-summary.md)
- [../../net/ecs-runtime-prioritized-roadmap.md](../../net/ecs-runtime-prioritized-roadmap.md)
