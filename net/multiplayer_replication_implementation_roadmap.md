# Multiplayer Replication Implementation Roadmap (Revised)

This roadmap converts the **Multiplayer Replication Model** into an
incremental implementation plan aligned with the current Runenwerk
repository layout.

The focus is:

-   server-authoritative simulation
-   ECS component replication
-   clear separation between networking layers
-   predictable implementation phases
-   minimal early complexity

The roadmap assumes the current crate structure:

``` text
engine_net/
engine_net_quic/
engine_sim/
games/*/src/net
```

------------------------------------------------------------------------

# Architectural Principles

1.  **Game domains declare replication intent**
2.  **engine_net implements replication semantics**
3.  **engine_net_quic handles transport**
4.  **engine_sim owns simulation timing**
5.  **Replication profiles define behavior**
6.  **Interest management limits bandwidth**
7.  **Prediction is opt-in and profile-driven**

------------------------------------------------------------------------

# Layer Responsibilities

  -----------------------------------------------------------------------
Layer                               Responsibility
  ----------------------------------- -----------------------------------
`engine_sim`                        simulation ticks, deterministic
step, command application

`engine_net`                        replication model, snapshots,
prediction

`engine_net_quic`                   QUIC transport runtime

`games/*/net`                       gameplay replication mapping
-----------------------------------------------------------------------

Additional boundary rule:

-   `engine_sim` produces simulation frames and authoritative state
    changes
-   `engine_net` consumes those simulation outputs to build replication
    snapshots

------------------------------------------------------------------------

# Implementation Phases

## Phase 1 --- Replication Vocabulary

**Goal:** define the replication policy model.

Location:

``` text
engine_net/src/replication/
```

Implement:

-   `ReplicationProfile`
-   `ReplicationDirection`
-   `Reliability`
-   `PredictionMode`
-   `BandwidthPriority`

Provide built-in presets:

-   `PredictedMovement`
-   `ReliableState`
-   `SparseEvent`
-   `InputCommand`
-   `Cosmetic`

Files:

``` text
engine_net/src/replication/model.rs
engine_net/src/replication/profile.rs
```

Done when:

-   profiles exist as engine types
-   profile presets exist in one place

------------------------------------------------------------------------

## Phase 2 --- Stable Network Identity

**Goal:** separate ECS identity from network identity.

Implement:

-   `NetEntityId`
-   mapping between ECS entities and `NetEntityId`
-   deterministic spawn/despawn tracking

Locations:

``` text
engine_net/src/replication/model.rs
engine_sim/src/identity.rs
```

Done when:

-   entities never send ECS IDs over the wire
-   spawn/despawn can be serialized

------------------------------------------------------------------------

## Phase 3 --- Command / Input Pipeline

**Goal:** define authoritative input flow.

Implement:

-   client input message types
-   tick tagging for inputs
-   server input ingestion
-   validation hooks

Files:

``` text
engine_net/src/protocol/input.rs
engine_net/src/runtime/server.rs
engine_net/src/runtime/client.rs
```

Done when:

``` text
client input → command → server simulation
```

works reliably.

------------------------------------------------------------------------

## Phase 4 --- Full Snapshot Model

**Goal:** implement authoritative state snapshots.

Implement:

-   full snapshot encoding
-   snapshot ticks
-   spawn/despawn payloads
-   client apply logic

Snapshot support must include:

-   entity spawn
-   entity despawn
-   component creation
-   component removal

Files:

``` text
engine_net/src/protocol/snapshot.rs
engine_net/src/replication/timeline.rs
```

Done when:

-   server sends full world snapshot
-   client reconstructs state correctly

No deltas yet.

------------------------------------------------------------------------

## Phase 5 --- Manual Replication Metadata

Before macros exist, metadata must work manually.

Implement:

-   replication registration
-   component descriptors
-   authority rules
-   interest metadata
-   profile references

Files:

``` text
engine_net/src/replication/model.rs
```

Done when:

-   components can be manually registered as replicated

------------------------------------------------------------------------

## Phase 6 --- Transport Lane Mapping

**Goal:** map replication behavior to transport semantics.

Define lanes such as:

-   `Reliable`
-   `Unreliable`
-   `UnreliableSequenced`
-   `InputStream`

Map profiles to lanes:

-   `PredictedMovement` → `UnreliableSequenced`
-   `ReliableState` → `Reliable`
-   `InputCommand` → `InputStream`
-   `Cosmetic` → `Unreliable`

Files:

``` text
engine_net/src/transport/lanes.rs
engine_net/src/transport/semantics.rs
engine_net_quic/src/transport/lanes.rs
```

Done when:

-   each built-in profile routes through a known lane
-   runtime code can dispatch replication traffic without embedding
    transport policy everywhere

------------------------------------------------------------------------

## Phase 7 --- Server Replication Runtime

Build the authoritative replication pipeline.

Server flow:

``` text
input
→ simulation tick
→ dirty state detection
→ interest filtering
→ snapshot build
→ lane routing
→ transport send
```

Interest filtering is part of the server runtime pipeline, not a
separate late-stage concern.

Files:

``` text
engine_net/src/runtime/server.rs
engine_net/src/replication/timeline.rs
```

------------------------------------------------------------------------

## Phase 8 --- Basic Interest Management

Interest must exist before scaling snapshots.

Implement early:

-   `Global`
-   `OwnerOnly`

Location:

``` text
engine_net/src/replication/interest.rs
```

Done when:

-   owner-only state never leaks
-   global replication works

------------------------------------------------------------------------

## Phase 9 --- Client Apply Runtime

Client receives snapshots and updates ECS.

Flow:

``` text
receive snapshot
→ validate tick
→ resolve NetEntityId
→ apply spawn/update/despawn
→ update baseline
```

Files:

``` text
engine_net/src/runtime/client.rs
```

------------------------------------------------------------------------

## Phase 10 --- Delta Replication

Add efficient state replication.

Implement:

-   baselines
-   delta snapshots
-   patch messages

Files:

``` text
engine_net/src/replication/timeline.rs
engine_net/src/protocol/snapshot.rs
```

Done when:

-   server can send deltas
-   client applies patches correctly

------------------------------------------------------------------------

## Phase 11 --- Recovery / Resync

Handle failure cases.

Implement:

-   full snapshot fallback
-   baseline reset
-   reconnect sync
-   stale snapshot handling
-   baseline mismatch → force full snapshot

Done when:

-   clients recover from packet loss or reconnect

------------------------------------------------------------------------

## Phase 12 --- Prediction and Reconciliation

Implement responsive local control.

Files:

``` text
engine_net/src/replication/prediction.rs
```

Flow:

``` text
input
→ client prediction
→ server snapshot
→ compare states
→ reconcile
→ smoothing
```

Prediction is enabled by profiles such as `PredictedMovement`.

------------------------------------------------------------------------

## Phase 13 --- Advanced Interest Management

Add scalable filtering:

-   Spatial AOI
-   Distance thresholds
-   Team visibility

Files:

``` text
engine_net/src/replication/interest.rs
```

Spatial implementations may include:

-   grid partitions
-   quadtrees
-   region streaming

------------------------------------------------------------------------

## Phase 14 --- Macro Integration

Add declarative ECS integration.

Implement:

``` rust
#[net_component(...)]
#[net_entity]
```

These generate replication metadata automatically.

Important: macros come **after runtime is stable**.

------------------------------------------------------------------------

## Phase 15 --- Game Integration

Games configure replication through components.

Example:

``` rust
#[net_component(
    authority = Server,
    profile = PredictedMovement,
    owner_prediction = true,
    interest = Spatial
)]
struct PlayerState;
```

Game domain responsibilities:

-   smoothing policies
-   prediction tuning
-   profile selection

Location:

``` text
games/*/src/net
```

------------------------------------------------------------------------

## Phase 16 --- Diagnostics and Hardening

Add observability.

Implement:

-   snapshot debug dump
-   lane routing trace
-   `NetEntityId` mapping logs
-   bandwidth metrics
-   replication stats

Add tests for:

-   snapshot encoding
-   delta correctness
-   prediction reconciliation
-   interest filtering

------------------------------------------------------------------------

# Minimum Viable Multiplayer Slice

Implement a vertical slice with:

-   one networked entity
-   one input component
-   one authoritative state component
-   full snapshots only
-   `Global` + `OwnerOnly` interest

Suggested example:

``` text
PlayerInput
PlayerState
Health
```

------------------------------------------------------------------------

# Risks to Avoid

Do NOT:

-   build macros before runtime semantics
-   send ECS entity IDs over the network
-   mix gameplay policy with transport logic
-   implement deltas before full snapshots work
-   build spatial interest too early
-   let QUIC transport know gameplay semantics

------------------------------------------------------------------------

# Milestones

### Milestone A --- Authoritative Replication Core

Includes:

-   profiles
-   `NetEntityId`
-   full snapshots
-   manual metadata
-   transport lane mapping
-   server send / client apply

### Milestone B --- Efficient Replication

Includes:

-   delta snapshots
-   baselines
-   interest filtering
-   recovery/resync

### Milestone C --- Responsive Client

Includes:

-   prediction
-   reconciliation
-   smoothing

### Milestone D --- Developer Ergonomics

Includes:

-   macros
-   automatic metadata
-   simple game integration

------------------------------------------------------------------------

# Final Target

Game code becomes simple:

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

And the engine provides:

-   snapshot generation
-   delta replication
-   interest filtering
-   transport routing
-   prediction
-   reconciliation
