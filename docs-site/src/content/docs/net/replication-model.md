---
title: "Multiplayer Replication Model"
description: "Documentation for Multiplayer Replication Model."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---

# Multiplayer Replication Model

This document defines the **replication and authority model** used by
the `net/` domain and game domains.

It describes the **attribute-based replication model** used for ECS
components in multiplayer systems.

------------------------------------------------------------------------

# Overview

Multiplayer state replication is driven by **a single attribute macro on
ECS components**.

Example:

``` rust
#[net_component(
    authority = Server,
    owner_prediction = true,
    profile = PredictedMovement,
    interest = Spatial
)]
pub struct PlayerState {
    pub position: Vec3,
    pub velocity: Vec3,
}
```

The macro generates the necessary replication boilerplate:

-   snapshot encoding
-   delta generation
-   replication registration
-   patch application
-   replication metadata

This keeps networking **declarative and component-driven**.

Game developers simply annotate ECS components that should replicate.

------------------------------------------------------------------------

# Replication Attributes

The `net_component` macro defines replication behavior.

Example:

``` rust
#[net_component(
    authority = Server,
    direction = ClientToServer,
    profile = InputCommand,
    interest = OwnerOnly
)]
pub struct InputState {
    pub move_x: f32,
    pub move_y: f32,
    pub fire: bool,
}
```

Supported attributes:

Attribute            Meaning
  -------------------- -----------------------------------
`authority`          Who owns the canonical state
`direction`          Network flow of the component
`profile`            Replication behavior preset
`owner_prediction`   Whether owning client may predict
`interest`           Which clients receive updates

------------------------------------------------------------------------

# Authority Model

Authority defines which side owns the canonical state.

Authority **does not mean ownership**. Ownership refers to the player
controlling an entity.

## Server Authority

``` text
authority = Server
```

Server authority means:

-   server simulation is the source of truth
-   client writes are rejected
-   server snapshots overwrite client state

Typical examples:

-   player state
-   enemy state
-   world objects
-   health
-   physics bodies

------------------------------------------------------------------------

## Client Authority

``` text
authority = Client
```

Client authority is used primarily for **command-like components**.

Typical examples:

-   input buffers
-   UI commands
-   camera state

These values are **validated and consumed by the server**.

------------------------------------------------------------------------

# Replication Direction

Replication direction defines how data flows across the network.

Direction          Meaning
  ------------------ ----------------------------------
`ServerToClient`   Server sends authoritative state
`ClientToServer`   Client sends commands or input
`Bidirectional`    Both sides may send updates

Example:

``` text
direction = ClientToServer
```

Used for input components.

------------------------------------------------------------------------

# Replication Profiles

Profiles define **replication behavior presets**.

Example:

``` text
profile = PredictedMovement
```

Profiles control:

-   update frequency
-   compression strategy
-   reliability model
-   prediction model
-   bandwidth priority

Typical profiles:

Profile               Use Case
  --------------------- -------------------------------
`ReliableState`       important durable state
`PredictedMovement`   high-frequency movement
`SparseEvent`         low-frequency gameplay events
`Cosmetic`            visual-only state
`InputCommand`        client command replication

Profiles prevent repeating networking policy on every component.

------------------------------------------------------------------------

# Replication Profile Reference

Typical engine profile configuration:

  --------------------------------------------------------------------------------------------------
Profile               Direction          Reliability         Frequency    Prediction    Priority
  --------------------- ------------------ ------------------- ------------ ------------- ----------
`PredictedMovement`   `ServerToClient`   `UnreliableDelta`   High         Enabled       High

`ReliableState`       `ServerToClient`   `Reliable`          Medium       Disabled      Medium

`SparseEvent`         `ServerToClient`   `Reliable`          Low          Disabled      Medium

`InputCommand`        `ClientToServer`   `Unreliable`        High         N/A           High

`Cosmetic`            `ServerToClient`   `Unreliable`        Low          Disabled      Low
--------------------------------------------------------------------------------------------------

These presets ensure **consistent replication behavior across the
engine**.

------------------------------------------------------------------------

# Replication Profile Definitions

Profiles are defined in the **engine networking domain** and reused by
game domains through the `profile = ...` attribute.

A minimal profile definition model looks like this:

``` rust
pub enum ReplicationDirection {
    ServerToClient,
    ClientToServer,
    Bidirectional,
}

pub enum Reliability {
    Reliable,
    Unreliable,
    UnreliableDelta,
}

pub enum PredictionMode {
    None,
    OwnerPredicted,
}

pub enum BandwidthPriority {
    High,
    Medium,
    Low,
}

pub struct ReplicationProfile {
    pub direction: ReplicationDirection,
    pub reliability: Reliability,
    pub frequency_hz: u16,
    pub prediction: PredictionMode,
    pub priority: BandwidthPriority,
}
```

Example profile definitions:

## PredictedMovement

``` rust
pub const PREDICTED_MOVEMENT: ReplicationProfile = ReplicationProfile {
    direction: ReplicationDirection::ServerToClient,
    reliability: Reliability::UnreliableDelta,
    frequency_hz: 30,
    prediction: PredictionMode::OwnerPredicted,
    priority: BandwidthPriority::High,
};
```

Used for:

-   player movement
-   character velocity
-   other high-frequency actor state

## ReliableState

``` rust
pub const RELIABLE_STATE: ReplicationProfile = ReplicationProfile {
    direction: ReplicationDirection::ServerToClient,
    reliability: Reliability::Reliable,
    frequency_hz: 10,
    prediction: PredictionMode::None,
    priority: BandwidthPriority::Medium,
};
```

Used for:

-   health
-   inventory
-   match state
-   objective state

## SparseEvent

``` rust
pub const SPARSE_EVENT: ReplicationProfile = ReplicationProfile {
    direction: ReplicationDirection::ServerToClient,
    reliability: Reliability::Reliable,
    frequency_hz: 2,
    prediction: PredictionMode::None,
    priority: BandwidthPriority::Medium,
};
```

Used for:

-   door opened
-   pickup consumed
-   quest or round state changes

## InputCommand

``` rust
pub const INPUT_COMMAND: ReplicationProfile = ReplicationProfile {
    direction: ReplicationDirection::ClientToServer,
    reliability: Reliability::Unreliable,
    frequency_hz: 60,
    prediction: PredictionMode::None,
    priority: BandwidthPriority::High,
};
```

Used for:

-   movement input
-   fire input
-   ability activation requests

## Cosmetic

``` rust
pub const COSMETIC: ReplicationProfile = ReplicationProfile {
    direction: ReplicationDirection::ServerToClient,
    reliability: Reliability::Unreliable,
    frequency_hz: 5,
    prediction: PredictionMode::None,
    priority: BandwidthPriority::Low,
};
```

Used for:

-   muzzle flashes
-   particles
-   cosmetic-only visuals

------------------------------------------------------------------------

# Prediction

Prediction is controlled by replication profiles.

Profiles that support prediction allow the client to:

1.  apply input immediately
2.  run local simulation
3.  reconcile when authoritative snapshots arrive

Prediction is typically used for:

-   player movement
-   character physics
-   vehicles

Prediction should only be used on components that can safely reconcile.

------------------------------------------------------------------------

# Network Entity Identity

Entities replicated across the network must use a **stable network
identity**.

The engine assigns a `NetEntityId` separate from ECS entity IDs.

This ensures:

-   stable entity mapping across clients
-   safe entity creation/destruction
-   reconnection support
-   deterministic snapshot patching

Network identity is never derived from internal ECS entity handles.

------------------------------------------------------------------------

# Entity Replication

Networked entities are marked with a replication attribute.

Example:

``` rust
#[net_entity]
pub struct Player;
```

The engine automatically handles:

-   spawn replication
-   despawn replication
-   entity mapping across peers
-   snapshot state tracking

------------------------------------------------------------------------

# Interest Management

Interest management determines **which entities and components are
replicated to which clients**.

Without interest management, servers would send the **entire world state
to every client**, which is not scalable.

Interest rules allow the networking layer to limit replication to
relevant data only.

Example:

``` text
interest = Spatial
```

Typical interest strategies:

Strategy      Description
  ------------- ---------------------------------------------------------
`Spatial`     Area-of-interest replication using spatial partitioning
`Distance`    Replication based on distance thresholds
`Team`        Only visible to members of the same team
`OwnerOnly`   Replicated only to the owning player
`Global`      Sent to all connected clients

Example owner-only component:

``` rust
#[net_component(
    authority = Server,
    profile = ReliableState,
    interest = OwnerOnly
)]
pub struct Inventory {
    pub items: Vec<ItemId>,
}
```

Spatial interest is typically implemented using:

-   spatial grids
-   quadtrees
-   region-based streaming

Interest management is one of the **primary bandwidth optimization
mechanisms** in large multiplayer worlds.

------------------------------------------------------------------------

# Snapshot Model

Replication uses **snapshot-based synchronization**.

Three snapshot forms exist:

Type             Description
  ---------------- -----------------------------------
Full Snapshot    complete state baseline
Delta Snapshot   difference from previous snapshot
Patch            individual component updates

The engine selects the most efficient encoding depending on the
replication profile.

------------------------------------------------------------------------

# Simulation Tick Model

Networking operates on **simulation ticks**.

Tick Type            Description
  -------------------- ----------------------------------
Simulation Tick      server authoritative update step
Snapshot Tick        snapshot version identifier
Client Render Time   interpolated render time

Typical flow:

``` text
client input -> simulation tick -> snapshot generation
```

Clients maintain a **snapshot buffer** to interpolate between states.

------------------------------------------------------------------------

# Gameplay Component Examples

Below are common gameplay replication patterns.

## Player Movement

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

## Player Input

``` rust
#[net_component(
    authority = Client,
    direction = ClientToServer,
    profile = InputCommand,
    interest = OwnerOnly
)]
pub struct PlayerInput {
    pub move_x: f32,
    pub move_y: f32,
    pub jump: bool,
}
```

## Health State

``` rust
#[net_component(
    authority = Server,
    profile = ReliableState,
    interest = Spatial
)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}
```

## Cosmetic Effect

``` rust
#[net_component(
    authority = Server,
    profile = Cosmetic,
    interest = Spatial
)]
pub struct MuzzleFlash {
    pub duration: f32,
}
```

------------------------------------------------------------------------

# Replication Flow

Typical server-authoritative flow:

Client:

``` text
Input -> Command -> Transport
```

Server:

``` text
Command -> Simulation -> Snapshot
```

Client:

``` text
Snapshot -> Reconciliation -> Interpolation
```

Steps:

1.  Client sends input commands tagged with a simulation tick.
2.  Server executes authoritative simulation.
3.  Server emits snapshots or deltas.
4.  Client applies snapshots.
5.  Prediction corrections are applied if divergence occurred.

------------------------------------------------------------------------

# Bandwidth Priorities

Replication profiles define **bandwidth priority levels** used when
bandwidth is constrained.

Priority   Example Use
  ---------- -----------------
High       player movement
Medium     combat state
Low        cosmetic state

Higher priority components replicate first when bandwidth budgets are
exceeded.

------------------------------------------------------------------------

# Responsibilities by Domain

`engine_net`

-   protocol definitions
-   replication vocabulary
-   snapshot and delta model
-   prediction primitives
-   profile definitions and replication presets

`engine_net_quic`

-   transport runtime
-   QUIC networking
-   connection lifecycle

`engine/src/plugins/net`

-   engine integration
-   scheduling
-   runtime wiring

owning gameplay domain/app networking modules

-   gameplay replication mapping
-   correction and smoothing policies
-   multiplayer gameplay semantics

------------------------------------------------------------------------

# Goals

The replication model aims to provide:

-   declarative networking via attributes
-   server-authoritative simulation
-   deterministic replication vocabulary
-   predictable client reconciliation
-   scalable interest-based replication
-   reusable engine-defined replication profiles
-   clear separation between engine networking and gameplay networking
