---
title: Events
description: Engine-agnostic guide for ecs events and reactive systems.
---

# ECS Events

Events are typed signals exchanged between systems through world-managed channels.

## Purpose

- Decouple systems with typed message passing.
- Support frame-transient and retained channel lifetimes.
- Enable per-system incremental reads with cursor-based channels.

## Key Concepts

- World APIs: `emit_event<T>`, `read_events<T>`, `drain_events<T>`, `clear_events<T>`.
- Param APIs: `EventReader<T>`, `EventWriter<T>`, `EventChannel<T>`.
- Channel config: `EventChannelConfig` (`capacity`, `overflow`, `lifetime`, `tracing`).
- Observers: `observe_events` with `ObserverTrigger::{OnEmit, OnDrain, EndOfFrame}`.

## API Notes

- `EventChannel<T>::iter_all()` reads all pending events.
- `EventChannel<T>::iter_new()` reads only unseen events for that system param state.
- `finish_event_frame()` applies frame-lifetime cleanup and end-of-frame observer triggers.

## Invariants

- `FrameTransient` channels are cleared at end-of-frame processing.
- Overflow policy is enforced per channel (`DropOldest`, `DropNewest`, `Panic`).
- Observer notifications are generated only on configured trigger boundaries.
