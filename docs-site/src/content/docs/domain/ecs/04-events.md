---
title: Events
description: Engine-agnostic guide for ecs events and reactive systems.
---

# ECS Events

Events are transient signals that propagate between systems.

## Purpose
- Decouple systems through message passing.
- Track frame-specific or persistent events.
- Support multi-consumer and filtered observation.

## Key Concepts
- emit_event<T> – Send an event of type T.
- EventReader<T> / EventWriter<T> – System param interfaces.
- ObserverTrigger – Control when observers fire (OnEmit, OnDrain, EndOfFrame).
- EventChannelConfig – Configure capacity, overflow, and lifetime.

## Usage Examples
- Frame-specific signals for gameplay actions.
- Persistent notifications for editor tooling.
- Observers reacting to specific event types.

## Invariants & Rules
- FrameTransient events cleared at frame end.
- Observers respect configured trigger boundaries.
- Overflow policies must be considered (DropOldest, DropNewest, Panic).