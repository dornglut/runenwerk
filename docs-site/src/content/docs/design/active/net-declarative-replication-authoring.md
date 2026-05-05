---
title: "Net Declarative Replication Authoring Design"
description: "Design for macro-based replication metadata and future low-boilerplate gameplay authoring."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Declarative Replication Authoring Design

## Purpose

This design defines the intended low-boilerplate authoring model for
normal gameplay replication while preserving low-level driver escape
hatches for specialized systems.

## Implemented Substrate

Implemented now:

- `#[net_entity]` macro implementing `NetEntity`.
- `#[net_component(...)]` macro implementing `NetComponentMetadata`.
- `ReplicatedComponentDescriptor`,
  `ReplicatedEntityDescriptor`, and `ReplicatedResourceDescriptor`.
- `ReplicationRegistry` registration and descriptor lookup.
- Profiles, interest policies, authority, direction, reliability,
  prediction, priority, frequency, and owner-prediction metadata.
- Compile-time rejection of unsupported macro arguments.

## Partial Contracts

Partial now:

- Macros generate metadata only.
- Macros do not generate snapshot encoding, delta generation, patch
  application, or runtime registration systems.
- The normal gameplay path still often requires a custom
  `ReplicationDriver`.
- Resource and entity metadata descriptors exist, but runtime extraction
  is not yet standardized.

## Target Authoring Model

Normal game code should eventually be able to:

1. mark networked entity marker types with `#[net_entity]`;
2. mark serializable replicated components with `#[net_component(...)]`;
3. register input stream types;
4. register ownership routing;
5. write ordinary ECS systems that consume typed inputs and mutate
   authoritative state;
6. rely on engine/net integration for extraction, snapshots, deltas,
   application, ACKs, diagnostics, and replay.

Custom drivers remain supported for:

- aggregate snapshots;
- custom compression;
- external non-ECS state;
- large-scale world streaming;
- rollback-optimized packing;
- unusual delta formats.

## Boundary Rules

- Metadata declarations describe replication intent.
- Metadata declarations do not make clients authoritative.
- Gameplay semantics remain in gameplay/app modules.
- Transport adapters never inspect gameplay metadata directly.
- Generated code must target public `engine_net::replication` contracts.

## Future Work

Future work:

1. Add a registration-driven extraction/apply bridge for common ECS
   components.
2. Add generated or derived component codec hooks where they fit public
   API ergonomics.
3. Add resource replication authoring if use cases justify it.
4. Add examples that use declarative metadata without hand-written
   transport glue.
5. Keep the low-level driver path documented as an advanced escape hatch.

## Validation Plan

Required validation:

- macro compile tests;
- descriptor registry tests;
- docs examples kept in sync with actual macro names and arguments;
- workspace check after public macro/API changes.
