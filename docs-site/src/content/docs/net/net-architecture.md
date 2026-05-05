---
title: "Networking Architecture"
description: "Runtime architecture of the Runenwerk multiplayer stack."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
---

# Runenwerk Networking Architecture

This document is the high-level orientation page for the Runenwerk
multiplayer stack. It summarizes the current architecture and links to
the canonical design package for long-term contracts.

It explains how networking flows through the following domains:

    engine_sim
    engine_net
    engine_net_quic
    gameplay domain/app networking modules

The goal is to maintain a **clean separation between simulation,
replication, and transport** while supporting a **server-authoritative
ECS architecture**.

For implementation order, use [multiplayer-replication-implementation-roadmap.md](multiplayer-replication-implementation-roadmap.md).

For design detail, use:

- [../design/active/net-authoritative-replication-protocol.md](../design/active/net-authoritative-replication-protocol.md)
- [../design/active/net-prediction-reconciliation-boundary.md](../design/active/net-prediction-reconciliation-boundary.md)
- [../design/active/net-plugin-runtime-bridge.md](../design/active/net-plugin-runtime-bridge.md)
- [../design/active/ecs-net-replication-boundary.md](../design/active/ecs-net-replication-boundary.md)
- [../design/active/net-interest-streaming-design.md](../design/active/net-interest-streaming-design.md)
- [../design/active/net-reconnect-history-recovery.md](../design/active/net-reconnect-history-recovery.md)
- [../design/active/net-declarative-replication-authoring.md](../design/active/net-declarative-replication-authoring.md)
- [../design/active/net-transport-lanes-delivery.md](../design/active/net-transport-lanes-delivery.md)
- [../design/active/net-diagnostics-inspection.md](../design/active/net-diagnostics-inspection.md)

------------------------------------------------------------------------

# Core Principles

The networking stack follows these rules:

1.  Server authoritative simulation
2.  Clients send commands, not state
3.  Snapshots replicate authoritative world state
4.  Replication profiles define behavior
5.  Interest management limits bandwidth
6.  Transport is independent of gameplay semantics

------------------------------------------------------------------------

# High Level Architecture

The system is split into **four layers**.

    Gameplay Domain/App Modules
         │
         ▼
    engine_net (Replication + Runtime)
         │
         ▼
    engine_net_quic (Transport)
         │
         ▼
    Network (QUIC)

Simulation runs alongside networking:

    engine_sim

Simulation produces authoritative state which replication transmits.

------------------------------------------------------------------------

# Domain Responsibilities

## engine_sim

Owns **simulation and deterministic game stepping**.

Responsibilities:

-   simulation tick progression
-   command application
-   simulation frame execution
-   deterministic simulation state
-   random number generation

Key files:

    engine_sim/src/command.rs
    engine_sim/src/identity.rs
    engine_sim/src/rng.rs

Networking never directly controls simulation logic.

------------------------------------------------------------------------

## engine_net

Implements **replication semantics and multiplayer runtime logic**.

Responsibilities:

-   replication model
-   snapshot generation
-   delta replication
-   prediction
-   reconciliation
-   interest management
-   replication profiles
-   runtime replication pipeline

Current runtime contracts include:

-   `AuthoritativeServerRuntime` for validated input ingestion,
    per-connection snapshot baseline selection, full/delta snapshot
    construction, monotonic acknowledgement handling, full-resync
    fallback, interest-filtered payload views, lane trace emission, and
    aggregate replication stats.
-   `ClientReplicationRuntime` for authoritative full/delta receive,
    tick and cursor progression checks, strict delta base validation
    against the current client cursor, full-resync requests after failed
    deltas, stale/duplicate snapshot rejection, local `NetEntityId`
    mapping, and operation-plan output for the owning ECS apply layer.
-   `SnapshotTimeline` for authoritative cursor allocation, retained
    full baselines, delta construction, pruning, and merge logic.

Current partial contracts:

-   declarative metadata does not yet generate complete snapshot/delta/apply code;
-   normal gameplay still commonly implements driver traits;
-   interest policies are predicates, not a built-in spatial/team data source;
-   reconnect uses full resync first; history-backed recovery remains future work;
-   diagnostics are aggregate-heavy and need richer per-connection inspection.

Key modules:

    engine_net/src/protocol
    engine_net/src/replication
    engine_net/src/runtime
    engine_net/src/session
    engine_net/src/simulation
    engine_net/src/transport

------------------------------------------------------------------------

## engine_net_quic

Implements **transport and connection runtime**.

Responsibilities:

-   QUIC endpoints
-   connection lifecycle
-   packet IO
-   stream management
-   routing packets to engine_net

Key modules:

    engine_net_quic/src/client
    engine_net_quic/src/server
    engine_net_quic/src/runtime
    engine_net_quic/src/transport

This layer does **not know about gameplay replication**.

------------------------------------------------------------------------

## Gameplay Domain/App Modules

Located in:

    the owning gameplay domain/app crates

Responsibilities:

-   replication policy selection
-   prediction smoothing rules
-   gameplay correction policies
-   profile selection for gameplay components

Example:

``` rust
#[net_component(
    authority = Server,
    profile = PredictedMovement,
    owner_prediction = true,
    interest = Spatial
)]
pub struct PlayerState {
    pub position: Vec3,
    pub velocity: Vec3,
}
```

Gameplay modules declare **what replicates**, not **how replication works**.

------------------------------------------------------------------------

# Runtime Data Flow

The multiplayer system follows a fixed runtime pipeline.

## Client → Server

    Player Input
       │
       ▼
    InputCommand Component
       │
       ▼
    engine_net protocol layer
       │
       ▼
    engine_net_quic transport
       │
       ▼
    Server receives command

Clients only send **intent**, not authoritative state.

------------------------------------------------------------------------

## Server Simulation

Server runtime pipeline:

    receive commands
         │
         ▼
    apply commands to simulation
         │
         ▼
    simulation tick executes
         │
         ▼
    world state updated

Simulation output becomes replication input.

------------------------------------------------------------------------

## Server Replication Pipeline

After simulation:

    detect dirty components
            │
            ▼
    interest filtering
            │
            ▼
    snapshot generation
            │
            ▼
    delta encoding
            │
            ▼
    profile routing
            │
            ▼
    transport lanes
            │
            ▼
    QUIC transport

Key modules:

    engine_net/replication
    engine_net/runtime/server.rs

The current server runtime is transport-agnostic. It chooses full versus
delta snapshots from per-connection acknowledgement state and records a
full-resync request when a baseline is missing or pruned. It does not
make transport lanes responsible for replication policy.

------------------------------------------------------------------------

# Snapshot System

Replication uses **snapshot-based synchronization**.

Snapshots contain:

    spawn messages
    component upserts/removals
    despawn messages

Snapshot types:

    Snapshot
    DeltaSnapshot
    SnapshotPayload

Snapshots are versioned using:

    SimulationTick
    SnapshotCursor

Clients maintain baselines to apply deltas.

Current invariants:

- snapshot cursors must advance monotonically;
- duplicate cursors and stale ticks are rejected;
- delta `base` must equal the client's current cursor;
- missing, mismatched, or malformed deltas request a full resync;
- despawns remove stored component state and suppress late upserts for
  the same `NetEntityId`.

`engine_net` returns operation plans for spawn, despawn, upsert, and
remove actions. The owning gameplay/app integration applies those plans
to ECS state through public ECS/runtime contracts.

------------------------------------------------------------------------

# Interest Management

Interest management determines **which entities replicate to which
clients**.

Without it, the server would replicate the entire world state to every
client.

Strategies include:

    Global
    OwnerOnly
    Spatial
    Distance
    Team

Example:

    interest = Spatial

Interest filtering occurs **before snapshot generation**.

The net crate owns policy vocabulary and predicate evaluation. It does
not own gameplay team membership, spatial partitioning, or distance data;
those inputs are supplied by the owning domain/app layer.

------------------------------------------------------------------------

# Replication Profiles

Replication profiles define **replication behavior presets**.

Profiles control:

-   update frequency
-   reliability
-   prediction
-   priority

Example profiles:

    PredictedMovement
    ReliableState
    SparseEvent
    InputCommand
    Cosmetic

Profiles are defined in:

    engine_net/src/replication

------------------------------------------------------------------------

# Prediction and Reconciliation

Some components support **client-side prediction**.

Typical flow:

    local input
        │
        ▼
    local prediction
        │
        ▼
    server snapshot arrives
        │
        ▼
    compare predicted vs authoritative state
        │
        ▼
    reconcile differences
        │
        ▼
    smooth correction

Prediction is enabled via replication profiles.

Current prediction support is a reusable reconciliation contract, not a
gameplay smoothing implementation. Game/app code owns smoothing,
rollback, and replay policy around authoritative snapshot application.

------------------------------------------------------------------------

# Network Identity

Networked entities use a stable identifier:

    NetEntityId

This identifier:

-   is stable across clients
-   survives reconnection
-   maps to ECS entities locally

ECS entity IDs are never sent over the network.

`NetEntityMap` is the current runtime helper for local ECS/network ID
mapping and emits assignment/removal events for diagnostics.

------------------------------------------------------------------------

# Transport Lanes

Replication traffic is routed through **transport lanes**.

Examples:

    Reliable
    Unreliable
    UnreliableSequenced
    InputStream

`InputStream` in this section is the transport-lane identifier (`TransportLane::InputStream`), not the ECS runtime `TickBuffer<T>` primitive name.

Profiles determine which lane is used.

Example:

    PredictedMovement → UnreliableSequenced
    ReliableState → Reliable
    InputCommand → InputStream

Transport lanes are implemented in:

    engine_net/src/transport
    engine_net_quic/src/transport

------------------------------------------------------------------------

# Server Authoritative Flow Summary

    Client Input
         │
         ▼
    Command
         │
         ▼
    Server Simulation
         │
         ▼
    Replication Pipeline
         │
         ▼
    Snapshot / Delta
         │
         ▼
    Transport
         │
         ▼
    Client Apply
         │
         ▼
    Prediction Reconciliation

------------------------------------------------------------------------

# Development Model

Game developers:

1.  Define networked entities
2.  Annotate replicated components
3.  Choose replication profiles
4.  Implement gameplay systems normally

Example:

``` rust
#[net_entity]
struct Player;

#[net_component(
    authority = Server,
    profile = PredictedMovement,
    owner_prediction = true,
    interest = Spatial
)]
struct PlayerState;

#[net_component(
    authority = Client,
    direction = ClientToServer,
    profile = InputCommand,
    interest = OwnerOnly
)]
struct PlayerInput;
```

Current substrate provides:

-   replication registration
-   snapshot and delta protocol contracts
-   transport routing
-   prediction/reconciliation hooks

Current limitation: these are net-core contracts and helpers. End-to-end
game integration still needs app/domain systems that extract gameplay
payloads and apply operation plans to ECS state without making clients
authoritative over replicated server state.

------------------------------------------------------------------------

# Final Goal

When complete:

-   gameplay code remains **declarative**
-   networking stays **deterministic**
-   transport stays **replaceable**
-   replication remains **scalable**
