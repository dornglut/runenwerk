---
title: Systems
description: Engine-agnostic guide to defining and using ecs systems in the domain layer.
---

# ECS Systems

Systems are functions or processes that operate over components and resources. They define the behavior of the ECS by reading and writing data in a controlled and deterministic way.

## Purpose

- Encapsulate domain logic applied to entities and resources.
- Enable deterministic and efficient execution of gameplay or simulation rules.
- Separate computation from data storage (components/resources).

## Key Concepts

- **System** – A function that queries components/resources and performs updates.
- **System Param** – Typed inputs to systems, such as components, resources, or events.
- **Query** – Filters and retrieves entities with matching components.
- **Command Queue** – Deferred mutations applied after system execution.
- **Stage / Set** – Execution group to define ordering constraints.

## Implementation / API

Systems are added to a `Runtime` with scheduling labels and explicit read/write access. They operate on `World` state via `SystemParam`s.

### System with Query

Query over components:
```rust
    #[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
    struct Position { x: f32, y: f32 }

    #[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
    struct Velocity { x: f32, y: f32 }

    fn movement_system(query: Query<(&mut Position, &Velocity)>) {
        for (pos, vel) in query.iter() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
```
### Deferred Commands

Systems can queue structural changes safely:
```rust
    fn spawn_entity(mut commands: Commands) {
        commands.spawn(Position { x: 0.0, y: 0.0 });
    }
```
### Stage & Set Ordering

Runtime execution can be ordered:
```rust
    use scheduler::ScheduleLabel;

    #[derive(Copy, Clone)]
    struct Update;
    impl ScheduleLabel for Update { fn name() -> &'static str { "Update" } }

    runtime.add_systems::<Update, _, _>(&mut world, tick.in_set(Gameplay));
    runtime.add_systems::<Update, _, _>(&mut world, spawn_entity.in_set(PostGameplay).after(Gameplay));
```
## Invariants & Rules

- Systems must declare **read/write access** explicitly to avoid conflicts.
- Structural changes are **deferred**; queries only observe changes after stage flush.
- Avoid side effects outside system parameters to maintain deterministic execution.
- Use history or telemetry APIs **only for diagnostics**, not core gameplay logic.

## Usage Examples (Domain-Level)

### Example 1: Movement System

Iterates over positions and velocities:
```rust
    fn movement_system(query: Query<(&mut Position, &Velocity)>) {
        for (pos, vel) in query.iter() {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
```
### Example 2: Spawn System

Queues a new entity safely:
```rust
    fn spawn_entity(mut commands: Commands) {
        commands.spawn(Position { x: 0.0, y: 0.0 });
    }
```
## Design Guidelines

- Keep all system logic independent of engine specifics.
- Declare explicit access for components/resources.
- Use queries for gameplay logic; use history or telemetry only for diagnostics.
- Stages and sets should define ordering to ensure deterministic behavior.