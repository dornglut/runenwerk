---
title: "ECS Runtime Dataflow and Support Systems Design"
description: "Layered model for ECS truth, runtime flow primitives, registries, extractors, diagnostics, history, and observation systems."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---

# ECS Runtime Dataflow and Support Systems Design

## Purpose

This document defines how runtime dataflow systems fit alongside ECS in Runenwerk.

The goal is to keep a clean separation between:

- current world truth
- runtime flow/messaging
- relationship and routing registries
- derived extraction and diagnostics layers
- retained history
- observation and derived views

This avoids forcing every runtime concern into plain components/resources, while also avoiding overusing message-driven patterns where ECS state is the better fit.

## Core Principle

Use this model:

- ECS for current truth
- runtime primitives for transient flow
- registries for stable cross-cutting relationships and metadata
- extractors/logs for derived and retained views
- observation/view systems for filtered subscriptions, projections, inspection, and health-style summaries

In short:

> ECS state + ECS-owned runtime primitives + derived extraction/introspection layers

## Runtime Vocabulary Mapping (Current Code)

Design terms in this document map to current runtime code as follows (repo state: 2026-04-09):

| Design term | Current code term(s) | Status |
| --- | --- | --- |
| `Broadcast<T>` | `BroadcastStream*`, `BroadcastReader`, `BroadcastWriter` in `domain/ecs/src/world/messaging/broadcast.rs` and `domain/ecs/src/system/params.rs` | Aligned (semantic match). |
| `WorkQueue<T>` | `WorkQueue*` in `domain/ecs/src/world/messaging/work_queue.rs`, `domain/ecs/src/system/params.rs`, and net bridge systems | Aligned (hard rename completed). |
| `TickBuffer<T>` | `TickBuffer*` in `domain/ecs/src/world/messaging/tick_buffer.rs` and `domain/ecs/src/system/params.rs` | Aligned (hard rename completed). |

`TickBuffer<T>` is now a generic ECS primitive: provenance is opaque (`TickBufferProvenance`), primitive diagnostics are neutral, and net/prediction counters (`acked`, `lagged`, `replayed`, `corrected`) live in net diagnostics resources.

## Layer Model

### Layer 1: ECS state

This is the authoritative current state of the world.

Includes:

- entities
- components
- resources
- queries
- structural mutations

Use ECS state for data that is true now.

Examples:

- `Transform`
- `Velocity`
- `Health`
- `MatchPhase`
- `ConnectionState`

### Layer 2: ECS-owned runtime primitives

These are world-owned subsystems for runtime flow semantics that are not well modeled as plain state.

Core primitives:

- `Broadcast<T>`
- `WorkQueue<T>`
- `TickBuffer<T>`

Supporting registries:

- `OwnershipRegistry`
- `DiagnosticsRegistry`

These are not plugin-local helpers. They are part of the runtime substrate and should be scheduler-visible where relevant.

### Layer 3: ECS-derived services

These are derived/query layers built on top of ECS state and tracked mutations.

Examples:

- `ChangeExtractor`
- `EventLog<T>`
- replay/debug traces
- replication delta builders

These are not primary state owners. They are computed or retained views.

### Layer 4: observation and derived views

These are filtered or specialized views built on top of ECS state, registries, diagnostics, and change extraction.

Examples:

- `SubscriptionRegistry`
- `Watch<T>`
- `ProjectionView`
- `IndexRegistry`
- `InspectionRegistry`
- `InvariantRegistry`
- `HealthMonitor`
- `TraceLog`

These are usually not foundational runtime primitives. They are layered capabilities built on top of the core systems.

### Layer 5: engine/plugin adapters

These are integration layers that consume ECS state/primitives and connect them to specific runtime domains.

Examples:

- networking transport adapters
- rendering extractors
- editor adapters
- persistence/export bridges

These layers should consume ECS/runtime contracts, not define core semantics themselves.

## Runtime Primitive Vocabulary

### `Broadcast<T>`

#### Purpose

Publish one occurrence so many consumers can observe it.

#### Semantics

- one publish
- many readers
- non-destructive reads
- unread cursor behavior per reader
- suited for fan-out observation

#### Good for

- notifications
- observations
- reactive systems
- transient domain events
- telemetry-like occurrence flow

#### Examples

- `Broadcast<PlayerDamaged>`
- `Broadcast<EntitySpawned>`
- `Broadcast<QuestCompleted>`

#### Not for

- single-consumer work handoff
- tick-bounded simulation command flow

#### ECS fit

`Broadcast<T>` is a world-owned runtime primitive.
Systems access it through explicit system params.
It is not modeled as normal component state.

### `WorkQueue<T>`

#### Purpose

Hand off work items to one draining consumer.

#### Semantics

- FIFO
- destructive drain
- one consumer owns drained items
- bounded capacity/backpressure matters

#### Good for

- outbound network work
- deferred jobs
- save requests
- import jobs
- workflow handoff

#### Examples

- `WorkQueue<OutboundServerMessage>`
- `WorkQueue<AssetImportJob>`
- `WorkQueue<SaveRequest>`

#### Not for

- fan-out notifications
- tick-scoped deterministic input flow

#### ECS fit

`WorkQueue<T>` is a world-owned runtime primitive.
It must not degrade into plugin-local `Vec<T>` resources with ad hoc semantics.

### `TickBuffer<T>`

#### Purpose

Store ordered records against simulation ticks.

#### Semantics

- records are associated with a simulation tick
- ordering within the tick matters
- replay/dedup/ack may matter
- cleanup follows tick lifecycle

#### Good for

- local input
- remote input
- deterministic command frames
- fixed-step simulation control flow

#### Examples

- `TickBuffer<PlayerInput>`
- `TickBuffer<RemoteCommandFrame>`
- `TickBuffer<AICommand>`

#### Not for

- one-off notifications
- arbitrary work queues

#### ECS fit

`TickBuffer<T>` is a world-owned runtime primitive specialized for simulation-time flow.
It is not current truth and should not be modeled as plain resource state.

## Supporting Systems

### `State<T>` / `ResourceState<T>`

#### Purpose

Represent current truth for one global-ish value.

#### Semantics

- current value only
- overwritten/updated over time
- not append-only
- not delivery-oriented

#### Good for

- match phase
- connection state
- active mode
- selected tool
- configuration snapshots

#### ECS fit

This usually maps cleanly to a resource.

#### Difference from `Broadcast<T>`

- `Broadcast<T>` = "something happened"
- `State<T>` = "this is true now"

### `OwnershipRegistry`

#### Purpose

Track who owns or controls which targets.

#### Semantics

- persistent relationship model
- supports routing and filtering
- supports transfer
- should work for entities and resources
- not a transient message flow

#### Good for

- multiplayer authority
- control routing
- interest filtering
- prediction eligibility
- controller-to-target resolution

#### ECS fit

`OwnershipRegistry` is a world-owned registry/subsystem.
It should not be scattered as inconsistent local conventions.

### `ChangeExtractor`

#### Purpose

Produce deterministic "what changed since X?" output.

#### Semantics

- derived from tracked mutations
- can operate over tick or frame windows
- can be filtered
- outputs generic ECS delta batches

#### Good for

- replication
- world streaming
- editor sync
- diagnostics
- export/save diffing

#### ECS fit

`ChangeExtractor` is a derived world service, not primary state.

### `DiagnosticsRegistry`

#### Purpose

Provide stable, queryable runtime counters and snapshots.

#### Semantics

- stable keys
- queryable snapshots
- cross-cutting observability
- not gameplay state

#### Good for

- desync triage
- lag metrics
- replay/ack/correction counters
- queue pressure
- dropped event/input analysis

#### ECS fit

`DiagnosticsRegistry` is a world-owned observability subsystem.

### `EventLog<T>` / `HistoryLog<T>`

#### Purpose

Retain append-only history for replay, audit, or debugging.

#### Semantics

- retained history
- not transient delivery
- inspection-oriented

#### Good for

- replay foundations
- audit traces
- editor history
- determinism debugging

#### ECS fit

`EventLog<T>` is a retained history subsystem, usually world-owned.

## Observation and Derived View Systems

These systems are important, but they are typically layered on top of the foundational runtime systems rather than introduced as base transport primitives.

### `Watch<T>`

#### Purpose

Observe when a state/value changes.

#### Good for

- match phase changes
- selected entity changes
- active tool changes
- config changes

#### Typical foundation

Built on top of:

- `State<T>`
- resource change tracking
- `ChangeExtractor`

#### ECS fit

A lightweight observation layer, not a primary state owner.

### `SubscriptionRegistry`

#### Purpose

Provide general filtered subscriptions over state, ownership, and extracted changes.

#### Good for

- "notify me when `Health` changes for entity set X"
- "notify me when ownership changes for controller C"
- "notify me when anything in region R changes"

#### Typical foundation

Built on top of:

- `ChangeExtractor`
- `OwnershipRegistry`
- `DiagnosticsRegistry`
- `State<T>`

#### ECS fit

A derived observation layer, not a replacement for `Broadcast<T>`.

### `ProjectionView`

#### Purpose

Maintain a materialized derived view of ECS/runtime state.

#### Good for

- replication projections
- minimap projections
- editor inspection views
- UI-facing summaries

#### Typical foundation

Built on top of:

- ECS state
- `ChangeExtractor`
- `OwnershipRegistry`

#### ECS fit

A derived view/cache layer.

### `IndexRegistry`

#### Purpose

Provide fast reverse lookups or alternate access paths derived from ECS truth.

#### Good for

- controller -> owned targets
- tag -> entities
- region -> entities in cell

#### Typical foundation

Built on top of:

- ECS state
- `OwnershipRegistry`
- change tracking

#### ECS fit

A derived indexing subsystem, usually maintained incrementally.

### `InspectionRegistry`

#### Purpose

Expose standardized introspection surfaces for tools, debugging, and runtime inspection.

#### Good for

- stream registry snapshots
- ownership snapshots
- extraction cursor snapshots
- current diagnostics snapshots

#### Typical foundation

Built on top of:

- `DiagnosticsRegistry`
- `OwnershipRegistry`
- `ChangeExtractor`
- runtime registries

#### ECS fit

A tooling/introspection layer.

### `InvariantRegistry`

#### Purpose

Define and query runtime invariants and validation checks.

#### Good for

- "every owned target must exist"
- "no queue exceeds configured capacity"
- "ownership assignment is unique"
- "extraction cursor never moves backward"

#### Typical foundation

Built on top of:

- ECS state
- registries
- diagnostics
- extraction state

#### ECS fit

A validation layer, not a transport primitive.

### `HealthMonitor`

#### Purpose

Provide higher-level health summaries from lower-level diagnostics and invariants.

#### Good for

- replication unhealthy
- input lagging
- correction storm detected
- queue saturation sustained

#### Typical foundation

Built on top of:

- `DiagnosticsRegistry`
- `InvariantRegistry`
- `SubscriptionRegistry`

#### ECS fit

A derived health/status layer.

### `TraceLog`

#### Purpose

Retain structured technical traces for debugging and replay-adjacent analysis.

#### Good for

- scheduler trace
- replication extraction trace
- ownership transfer trace
- input replay trace

#### Typical foundation

Built on top of:

- `EventLog<T>`
- diagnostics
- extraction outputs
- runtime boundary hooks

#### ECS fit

A retained debugging/audit layer.

## Optional Later Systems

### `RequestQueue<T>` and `ResponseQueue<T>`

#### Purpose

Model explicit request/result workflows.

#### Good for

- pathfinding requests/results
- asset load requests/results
- async-like work orchestration

#### ECS fit

Usually a workflow convention built on top of `WorkQueue<T>`.

### `Mailbox<T>`

#### Purpose

Targeted inbox delivery per receiver.

#### Semantics

- messages are addressed to a specific recipient
- recipient can be entity, subsystem, actor, or service
- inbox is receiver-oriented rather than globally drained

#### Examples

- `Mailbox<EntityId, DamageCommand>`
- `Mailbox<AiAgentId, AiMessage>`
- `Mailbox<SystemId, StreamCommand>`

#### Actor-style meaning

Actor-style design means:

- each actor/subsystem owns its state
- other parts of the runtime send it messages
- the actor processes its own inbox

#### Good for

- targeted subsystem delivery
- isolated stateful processors
- actor-like boundaries

#### Risks

- can fight ECS-wide batch processing
- may overcomplicate a data-oriented runtime if introduced too early

#### ECS fit

`Mailbox<T>` is an optional world-owned targeted-delivery subsystem.
It is not part of the core runtime foundation right now.

## What Goes Where

### Use a component when:

- data is per-entity
- data is current truth
- systems naturally query it in batches

Examples:

- `Transform`
- `Velocity`
- `Health`

### Use a resource when:

- data is one current global-ish value
- it represents current truth
- it does not need special queue/stream semantics

Examples:

- `MatchPhase`
- `ConnectionState`
- `TimeState`

### Use a world-owned runtime primitive when:

- data is transient flow
- semantics are not plain current truth
- special lifecycle or access behavior matters
- scheduler visibility matters

Examples:

- `Broadcast<T>`
- `WorkQueue<T>`
- `TickBuffer<T>`

### Use a world-owned registry when:

- it represents a stable cross-cutting relationship or metadata map
- multiple systems depend on a consistent central model

Examples:

- `OwnershipRegistry`
- `DiagnosticsRegistry`

### Use a derived extraction/service layer when:

- output is computed from tracked state/history
- consumers want windows/deltas/queries rather than raw state ownership

Examples:

- `ChangeExtractor`
- replication delta builders
- replay history readers

### Use an observation/view layer when:

- consumers want filtered or specialized observation
- the behavior can be derived from state, registries, or extraction outputs
- the system should not redefine core transport semantics

Examples:

- `SubscriptionRegistry`
- `Watch<T>`
- `ProjectionView`
- `InspectionRegistry`
- `HealthMonitor`

## Decision Guide

Use this checklist.

### Current truth

Ask:

- "What is true right now?"

Use:

- component
- resource
- `State<T>`

### Fan-out occurrence

Ask:

- "Did something happen that many systems may observe?"

Use:

- `Broadcast<T>`

### Single-consumer work handoff

Ask:

- "Should one system drain and own the work?"

Use:

- `WorkQueue<T>`

### Tick-scoped ordered commands

Ask:

- "Must this be applied at a specific simulation tick?"

Use:

- `TickBuffer<T>`

### Ownership / routing / authority

Ask:

- "Who owns or controls this target?"

Use:

- `OwnershipRegistry`

### What changed since X

Ask:

- "What changed since tick/frame/window X?"

Use:

- `ChangeExtractor`

### Runtime health / debugging

Ask:

- "How do I inspect lag, backpressure, replay, drops, or queue pressure?"

Use:

- `DiagnosticsRegistry`

### Retained history

Ask:

- "Do I need retained trace/history instead of transient delivery?"

Use:

- `EventLog<T>` / `HistoryLog<T>`

### Filtered observation

Ask:

- "Do I need a filtered watch/subscription/projection over existing state and changes?"

Use:

- `SubscriptionRegistry`
- `Watch<T>`
- `ProjectionView`

### Validation / health summary

Ask:

- "Do I need invariant checks or high-level runtime health signals?"

Use:

- `InvariantRegistry`
- `HealthMonitor`

### Targeted inbox delivery

Ask:

- "Is this message specifically for one recipient or actor-like processor?"

Use:

- `Mailbox<T>`

## Important Non-Goals

The runtime should not:

- model all flow as plain resources with `Vec<T>`
- use naming-only conventions where structural contracts are needed
- treat current truth and transient flow as the same thing
- let plugins own foundational queue/stream semantics
- use watchers/subscriptions as a vague replacement for explicit flow primitives
- use mailbox/actor patterns as a default replacement for ECS batch processing

## Design Rules

1. ECS is for current truth.
2. Runtime primitives are for flow.
3. Registries are for relationships and stable metadata.
4. Extractors/logs are for derived and retained views.
5. Observation/view systems are layered features, not foundational transport primitives.
6. Use the narrowest semantic tool that matches the problem.
7. Do not invent plugin-local versions of core primitives.
8. Human-readable names are for diagnostics, not canonical runtime identity.
9. Deterministic ordering and lifecycle boundaries are mandatory for core runtime primitives.

## Recommended Standard Vocabulary

### Primary runtime flow primitives

- `Broadcast<T>`
- `WorkQueue<T>`
- `TickBuffer<T>`

### Core supporting systems

- `State<T>`
- `OwnershipRegistry`
- `ChangeExtractor`
- `DiagnosticsRegistry`
- `EventLog<T>`

### Observation and derived view systems

- `Watch<T>`
- `SubscriptionRegistry`
- `ProjectionView`
- `IndexRegistry`
- `InspectionRegistry`
- `InvariantRegistry`
- `HealthMonitor`
- `TraceLog`

### Optional later systems

- `RequestQueue<T>`
- `ResponseQueue<T>`
- `Mailbox<T>`

## Summary

Runenwerk should treat ECS as the authority for current world state, while using explicit world-owned runtime systems for flow, routing, extraction, diagnostics, retained history, and observation layers.

The architecture is:

- ECS state for truth
- runtime primitives for flow
- registries for relationships/metadata
- extractors/logs for derived and retained views
- observation/view systems for filtered and tool-facing access

This keeps semantics explicit, scheduler-visible, deterministic, and reusable across runtime, networking, tooling, and editor layers.
