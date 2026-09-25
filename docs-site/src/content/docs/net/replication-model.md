---
title: "Multiplayer Replication Model"
description: "Historical replication model overview retained for context; superseded by current networking architecture and migration designs."
status: superseded
owner: net
layer: net
canonical: false
last_reviewed: 2026-09-13
replaced_by:
  - net-architecture.md
  - ../design/active/net-authoritative-replication-protocol.md
  - ../design/active/net-transport-lanes-delivery.md
  - ../design/active/ecs-net-replication-boundary.md
  - multiplayer-replication-implementation-roadmap.md
---

# Multiplayer Replication Model

This page is historical. It previously described the target replication
model as if all declarative component replication behavior were current.

Do not use this page as current implementation guidance.

Current guidance lives in:

- [Networking architecture](net-architecture.md)
- [Authoritative replication protocol](../design/active/net-authoritative-replication-protocol.md)
- [Transport lanes and delivery](../design/active/net-transport-lanes-delivery.md)
- [ECS/net replication boundary](../design/active/ecs-net-replication-boundary.md)
- [Implementation roadmap](multiplayer-replication-implementation-roadmap.md)

The former declarative component-registration and hard-coded interest-policy target
designs are retained only in the archived design area as historical evidence. They are
not current successor authority.

The implementation inventory below is retained as a point-in-time snapshot from this
document's original 2026-05-05 review. The September 2026 `last_reviewed` date records
successor-link and lifecycle reconciliation; it does not revalidate that historical
inventory as current implementation truth.

## Historical Implementation Snapshot

At that historical point:

- `#[net_component(...)]` and `#[net_entity]` generated metadata implementations;
- `engine_net` carried authority, direction, profile, interest, prediction, and lane vocabulary;
- `engine_net` defined snapshot, delta, ACK, and cursor protocol types;
- runtime helpers and engine plugin systems supported authoritative snapshot/delta flows through driver contracts.

The metadata macros, descriptor registry, profile/interest authoring vocabulary, and related
engine wrapper were later deleted as RN8 migration residue. Current runtime integration retains
only the contracts demonstrated by maintained consumers; future ordinary Replicated View
authoring remains separately evidence-gated.

## Superseded Claims

The original page claimed or implied that the macro generated all replication boilerplate,
including snapshot encoding, delta generation, patch application, and registration. That claim is
historical; the macro/metadata authoring stack itself has since been retired.
