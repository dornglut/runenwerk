---
title: "ECS Runtime Prioritized Roadmap (Superseded)"
description: "Historical pre-C6/C7/C8 ECS/runtime/network convergence roadmap; not current sequencing authority."
status: superseded
owner: net
layer: net
canonical: false
last_reviewed: 2026-09-15
replaced_by: ./multiplayer-replication-implementation-roadmap.md
---

# ECS Runtime Prioritized Roadmap (Superseded)

This roadmap described the pre-C6/C7/C8 convergence model and is no longer current sequencing authority.

Its former priorities assumed generic ECS `Broadcast*`, `WorkQueue*`, and `TickBuffer*` messaging, scheduler-owned access domains, application-shaped barriers, and a live standalone `domain/scheduler` package. Accepted RunenECS repair work subsequently removed or reassigned those responsibilities.

Use current authority instead:

- [Standalone RunenECS architecture](https://github.com/dornglut/runen-ecs/blob/main/ARCHITECTURE.md) for reusable ECS semantics and conformance;
- [Repository Family Extraction Boundaries](../adr/accepted/0014-repository-family-extraction-boundaries.md) for Runenwerk/framework ownership;
- [Current multiplayer replication implementation roadmap](./multiplayer-replication-implementation-roadmap.md) for retained networking work.

The original roadmap remains available through repository history for provenance. It must not be used to reactivate removed messaging/scheduler surfaces or to infer current priorities.
