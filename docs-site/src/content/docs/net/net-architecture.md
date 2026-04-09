---
title: "Networking Architecture"
description: "Runtime architecture of the Runenwerk multiplayer stack."
---

# Runenwerk Networking Architecture

This document describes the **runtime architecture of the Runenwerk
multiplayer stack**.

It explains how networking flows through the following domains:

    engine_sim
    engine_net
    engine_net_quic
    games/*/src/net

The goal is to maintain a **clean separation between simulation,
replication, and transport** while supporting a **server-authoritative
ECS architecture**.

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

    Game Domain
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

## Game Domains

Located in:

    games/*/src/net

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

Game domains declare **what replicates**, not **how replication works**.

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

------------------------------------------------------------------------

# Snapshot System

Replication uses **snapshot-based synchronization**.

Snapshots contain:

    spawn messages
    component patches
    despawn messages

Snapshot types:

    FullSnapshot
    DeltaSnapshot
    ComponentPatch

Snapshots are versioned using:

    SnapshotTick

Clients maintain baselines to apply deltas.

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

------------------------------------------------------------------------

# Network Identity

Networked entities use a stable identifier:

    NetEntityId

This identifier:

-   is stable across clients
-   survives reconnection
-   maps to ECS entities locally

ECS entity IDs are never sent over the network.

------------------------------------------------------------------------

# Transport Lanes

Replication traffic is routed through **transport lanes**.

Examples:

    Reliable
    Unreliable
    UnreliableSequenced
    InputStream

`InputStream` in this section is the transport-lane identifier (`TransportLane::InputStream`), not the ECS runtime `InputStream<T>`/design `TickBuffer<T>` primitive name.

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

The engine provides:

-   replication registration
-   snapshot encoding
-   delta replication
-   transport routing
-   prediction
-   reconciliation

------------------------------------------------------------------------

# Final Goal

When complete:

-   gameplay code remains **declarative**
-   networking stays **deterministic**
-   transport stays **replaceable**
-   replication remains **scalable**
