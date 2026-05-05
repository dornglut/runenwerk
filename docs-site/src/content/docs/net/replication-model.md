---
title: "Multiplayer Replication Model"
description: "Historical replication model overview retained for context; superseded by active networking design documents."
status: superseded
owner: net
layer: net
canonical: false
last_reviewed: 2026-05-05
replaced_by:
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-declarative-replication-authoring.md
  - ../design/active/net-interest-streaming-design.md
  - ../design/active/net-transport-lanes-delivery.md
---

# Multiplayer Replication Model

This page is historical. It previously described the target replication
model as if all declarative component replication behavior were current.

Do not use this page as current implementation guidance.

Current guidance lives in:

- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)
- [Declarative replication authoring](../design/active/net-declarative-replication-authoring.md)
- [Interest and streaming](../design/active/net-interest-streaming-design.md)
- [Transport lanes and delivery](../design/active/net-transport-lanes-delivery.md)
- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Implementation roadmap](multiplayer-replication-implementation-roadmap.md)

## Current Truth Summary

Implemented now:

- `#[net_component(...)]` and `#[net_entity]` generate metadata
  implementations.
- `engine_net` defines authority, direction, profile, interest,
  prediction, and lane vocabulary.
- `engine_net` defines snapshot, delta, ACK, and cursor protocol types.
- Runtime helpers and engine plugin systems support authoritative
  snapshot/delta flows through driver contracts.

Partial now:

- Metadata macros do not generate snapshot encoding, delta generation, or
  component apply code.
- Generic ECS component extraction and apply are not yet the normal
  gameplay-facing path.
- Prediction replay exists through engine plugin hooks, while smoothing
  and correction presentation stay in gameplay/app modules.

Future work:

- standard extraction/application for common replicated components;
- stronger ACK and lifecycle hardening;
- richer interest diagnostics;
- lower-boilerplate public examples.

## Superseded Claims

The original page claimed or implied that the macro generated all
replication boilerplate, including snapshot encoding, delta generation,
patch application, and registration. That is no longer treated as current
truth. The macro currently generates metadata against `engine_net`
contracts.
