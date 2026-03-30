---
title: Commands
description: Engine-agnostic guide for deferred ecs commands.
---

# ECS Commands

Commands are deferred structural mutations applied at stage boundaries.

## Purpose
- Insert, remove, or modify components safely during system execution.
- Preserve deterministic execution order.
- Enable atomic batch operations for multiple mutations.

## Key Concepts
- Commands – Queued operations applied after system run.
- DeferredCommand – Structural changes delayed until stage flush.
- BatchCommands – Apply multiple operations atomically.
- ConditionalCommands – Apply only if query conditions hold.

## Usage Examples
- Spawning entities with a bundle of components.
- Inserting components into existing entities.
- Removing components safely without conflicting with queries.

## Invariants & Rules
- Commands are visible only after stage flush.
- Multiple systems may queue commands simultaneously; ordering is deterministic.
- Integration with engine adapters may expose commands to runtime tooling.