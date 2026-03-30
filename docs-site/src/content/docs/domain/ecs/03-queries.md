---
title: Queries
description: Engine-agnostic guide for ecs queries.
---

# ECS Queries

Queries retrieve entities and components according to specific constraints.

## Purpose
- Select subsets of entities based on component composition.
- Support both read-only and mutable access.
- Enable filters like `With`, `Without`, `Added`, `Changed`.

## Key Concepts
- Query – Defines which entities/components a system can access.
- QueryState – Detached reusable query object.
- Filters – Include or exclude entities based on components or state.
- Changed / Added – Track per-query modifications since last run.

## Implementation / API
Detached query example:

    let query = world.query_state::<(&mut Position, &Velocity), ()>();
    for (pos, vel) in query.iter(&mut world) {
        pos.x += vel.x;
        pos.y += vel.y;
    }

Filters:
- With<T> – Include only entities with component T.
- Without<T> – Exclude entities with component T.
- Added<T> – Entities where T was just added.
- Changed<T> – Entities where T changed since last query.

## Invariants & Rules
- Queries do not see deferred command changes until stage flush.
- Changed<T> / Added<T> are per-query instance stateful.
- Avoid mutable aliasing conflicts in the same query.

## Usage Examples
- Iterating all players with Position and Velocity.
- Filtering only newly added components for initialization.