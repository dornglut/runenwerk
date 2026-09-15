---
title: "ECS Runtime Gap Summary (May 2026, Superseded)"
description: "Historical May 2026 ECS/runtime/multiplayer gap audit; not current repository authority."
status: superseded
owner: net
layer: net
canonical: false
last_reviewed: 2026-09-15
replaced_by: ./multiplayer-replication-implementation-roadmap.md
---

# ECS Runtime Gap Summary (May 2026, Superseded)

This capability audit is a historical May 2026 snapshot and no longer describes current RunenECS or networking ownership.

Later accepted RunenECS C6-C8 work removed or reassigned several capabilities that this audit called current, including generic ECS messaging channels, gameplay ownership/lifecycle policy, scheduler messaging access domains, and the standalone `domain/scheduler` package.

Current authority is split deliberately:

- [Standalone RunenECS architecture](https://github.com/dornglut/runen-ecs/blob/main/ARCHITECTURE.md) owns current reusable ECS semantics and conformance;
- [Repository Family Extraction Boundaries](../adr/accepted/0014-repository-family-extraction-boundaries.md) owns Runenwerk's durable framework/integration boundary;
- [Multiplayer replication implementation roadmap](./multiplayer-replication-implementation-roadmap.md) owns current retained networking work.

Do not infer current API support from the historical `Broadcast*`, `WorkQueue*`, `TickBuffer*`, ownership, frame/tick finalization, or scheduler-barrier entries in the former audit. The original snapshot remains available through repository history.
