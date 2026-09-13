---
title: Multiplayer Design Proposal
description: Historical multiplayer proposal retained for context; superseded by current networking architecture and migration designs.
status: superseded
owner: net
layer: net
canonical: false
last_reviewed: 2026-09-13
replaced_by:
  - net-architecture.md
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-plugin-runtime-bridge.md
  - ../design/active/ecs-net-replication-boundary.md
  - multiplayer-replication-implementation-roadmap.md
---

# Multiplayer Design Proposal

This page is historical. It captured the desired multiplayer direction
before the current networking design package was split by boundary.

Do not use this page as current implementation guidance.

Current guidance lives in:

- [Networking architecture](net-architecture.md)
- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)
- [Engine net plugin runtime bridge](../design/active/net-plugin-runtime-bridge.md)
- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Implementation roadmap](multiplayer-replication-implementation-roadmap.md)

Reusable prediction/reconciliation semantics now belong to standalone RunenNet. The
former Runenwerk prediction-boundary and component-registration authoring designs are
archived as historical evidence rather than current successor authority.

## Historical Value

The proposal established useful doctrine that remains valid:

- ECS events should not become the multiplayer system.
- Replicated state, input streams, and runtime transport bridge are
  separate flows.
- Gameplay systems define input meaning and state mutation.
- Transport/runtime code must not own gameplay semantics.
- Low-level replication drivers remain useful as escape hatches.

## Superseded Claims

The original proposal mixed target architecture with future API sketches.
The following ideas are not current implementation truth:

- a zero-boilerplate default workflow replacing custom drivers;
- generated snapshot, delta, and apply code for every component;
- `#[replicated(...)]` as the current macro name;
- automatic ownership routing for all gameplay models;
- generic smoothing/correction presentation policy in engine code.

The current code provides metadata macros, protocol contracts, engine
bridge systems, tick-buffered input flow, prediction replay hooks, and
driver escape hatches. The remaining implementation sequence is tracked
in the roadmap.
