---
title: Multiplayer Design Proposal
description: Multiplayer architecture proposal for Runenwerk networking, replication, and runtime integration.
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---
# Multiplayer Integration Design

## Goal

Make multiplayer easy to implement in the engine without forcing every game feature to hand-write:

- snapshot structs
- delta structs
- codec plumbing
- replication drivers
- inbox/outbox glue
- prediction replay logic
- per-feature network boilerplate

The engine should provide a **generic, composable multiplayer model** that is:

- data-oriented
- ECS-friendly
- prediction-capable
- authority-aware
- reusable across many gameplay styles
- customizable without forcing a full custom networking stack

---

## Core Design Principle

Do **not** make the ECS event system the multiplayer system.

Instead, use **three separate layers**:

1. **Replicated state**
   - components that synchronize across the network
2. **Input streams**
   - typed command/input flows from clients to authority
3. **Runtime transport bridge**
   - the layer that talks to the actual network runtime

This keeps the architecture clean:

- ECS systems stay focused on simulation
- replication stays focused on synchronized state
- transport/runtime stays focused on delivery

---

## Architectural Model

### 1. Replicated Components

Any component can opt into replication with metadata.

Replication metadata should describe:

- authority model
- sync mode
- prediction mode
- interest/relevancy
- reliability/profile

#### Example

```rust
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Component)]
#[replicated(
    authority = Server,
    sync = Delta,
    prediction = Owner,
    interest = Global,
    profile = PredictedMovement
)]
pub struct Transform2D {
    pub x: f32,
    pub y: f32,
}
```

This must be generic. It should work for:

- transforms
- health
- inventory state
- cooldowns
- animation state
- quest state
- status effects
- any serializable ECS component

#### Design intent

The component remains a normal ECS component.

The replication metadata is only a declaration that the networking layer can use to:

- include it in snapshots/deltas
- determine who owns truth
- determine who may predict it
- determine who should receive it

### 2. Input Streams

Inputs are not events. Inputs are typed command streams.

Any game-defined input type can be registered.

#### Example

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveInput {
    pub x: f32,
    pub y: f32,
}
```

The engine should expose a generic registration API:

```rust
app.register_input_stream::<MoveInput>(InputStreamConfig::predicted());
```

Other possible input streams:

```rust
app.register_input_stream::<BuildCommand>(InputStreamConfig::authoritative());
app.register_input_stream::<FireWeaponInput>(InputStreamConfig::predicted());
app.register_input_stream::<MenuAction>(InputStreamConfig::reliable());
```

#### Design intent

The engine owns:

- buffering local input
- shipping it to the authority
- replaying pending local predicted input after correction
- ordering by simulation tick

The game owns:

- what the input means
- which systems consume it
- how it mutates ECS state

### 3. Ownership Routing

The engine needs a generic way to map a network connection to the entities it controls.

#### Example

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub struct NetworkOwner {
    pub connection_id: ConnectionId,
}
```

Registration:

```rust
app.register_network_owner::<NetworkOwner>();
```

#### Design intent

Ownership routing should be reusable for many gameplay models:

- one player controls one avatar
- one player controls multiple units
- spectator controls nothing
- RTS-style many-unit control
- possession/switching avatars
- vehicle ownership
- server-owned NPCs

The engine should not assume "one connection = one character controller".

### 4. Generic Simulation Systems

The engine should not hardcode gameplay meaning.

The user writes ordinary ECS systems that read typed inputs and mutate state.

#### Example

```rust
fn apply_move_input_system(
    input: InputView<MoveInput>,
    mut query: Query<(&NetworkOwner, &mut Transform2D)>,
) {
    for (owner, transform) in query.iter_mut() {
        if let Some(command) = input.latest_for(owner.connection_id) {
            transform.x += command.x;
            transform.y += command.y;
        }
    }
}
```

This is the correct level of customization:

- input type is generic
- ownership component is generic
- target component is generic
- application rules are game-defined

The engine handles the multiplayer machinery under the surface.

## Public API Surface

### Replication Declaration

#### Attribute / derive

```rust
#[replicated(
    authority = Server,
    sync = Delta,
    prediction = Owner,
    interest = Global,
    profile = PredictedMovement
)]
```

#### Meaning

- `authority`: who owns truth (examples: `Server`, `Client`, `Peer`)
- `sync`: snapshot policy (examples: `Full`, `Delta`, `FullThenDelta`)
- `prediction`: who may predict locally (examples: `None`, `Owner`)
- `interest`: who receives this data (examples: `Global`, `OwnerOnly`, `Spatial`, `Team`, `Distance`)
- `profile`: transport/reliability preset
  - `PredictedMovement`
  - `ReliableState`
  - `SparseEvent`
  - `Cosmetic`

### Registration API

```rust
app.replicate_component::<Transform2D>();
app.replicate_component::<Velocity>();
app.replicate_component::<Health>();

app.register_input_stream::<MoveInput>(InputStreamConfig::predicted());
app.register_input_stream::<FireWeaponInput>(InputStreamConfig::predicted());
app.register_input_stream::<MenuAction>(InputStreamConfig::reliable());

app.register_network_owner::<NetworkOwner>();
```

### Runtime Params

#### Local input producer

```rust
fn gather_input(mut input: LocalInput<MoveInput>) {
    input.push(MoveInput { x: 1.0, y: 0.0 });
}
```

#### Input consumer

```rust
fn apply_input_system(
    input: InputView<MoveInput>,
    mut query: Query<(&NetworkOwner, &mut Transform2D)>,
) {
    for (owner, transform) in query.iter_mut() {
        if let Some(command) = input.latest_for(owner.connection_id) {
            transform.x += command.x;
            transform.y += command.y;
        }
    }
}
```

### Optional Helper Params

Potential useful helpers:

- `OwnedEntities<TOwner>`
- `LocalConnection`
- `Predicted<T>`
- `Authoritative<T>`
- `CorrectionEvents<T>` for optional local feedback
- `ReplicationDiagnosticsView`

## Engine Responsibilities

The engine should handle these automatically once the types are registered.

### Input Stream Pipeline

For each registered input stream:

- collect local input
- timestamp it with simulation tick
- store pending predicted frames on client
- encode and send to authority
- receive and decode on server
- expose typed input view to simulation systems
- replay unacked predicted inputs after correction

### Replicated Component Pipeline

For each registered replicated component:

- include it in snapshot capture
- include it in delta generation
- apply authoritative updates on clients
- support owner prediction where configured
- respect interest filtering
- use appropriate transport profile

### Ownership Routing

For a registered ownership component:

- determine which entities belong to which connection
- determine which entities can be predicted locally
- determine which inputs apply to which entities

### Correction / Replay

For predicted streams and predicted components:

- detect authoritative correction
- apply correction
- replay pending local inputs newer than the corrected tick

## What the Game Must Still Define

The engine should not try to infer game rules.

The game still defines:

- the input types
- the replicated components
- the ownership component
- the actual simulation systems

That is the right boundary.

## What This Replaces

This design is meant to replace the normal need to hand-write a custom `ReplicationDriver` for common multiplayer gameplay.

The low-level driver API should remain as an escape hatch, not as the default workflow.

### Escape hatch use cases

Keep custom low-level replication drivers for:

- aggregate snapshots not directly represented by ECS components
- custom compression strategies
- unusual delta formats
- replication of external/non-ECS data
- very large-scale world streaming
- rollback-specific optimized state packing
- specialized MMO-style replication

Default path should not require it.

## Example: Generic Synced Movement

### Component declarations

```rust
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Component)]
#[replicated(
    authority = Server,
    sync = Delta,
    prediction = Owner,
    interest = Global,
    profile = PredictedMovement
)]
pub struct Transform2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub struct NetworkOwner {
    pub connection_id: ConnectionId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveInput {
    pub x: f32,
    pub y: f32,
}
```

### App setup

#### Server

```rust
app.add_plugin(NetPlugin::<AutoNet>::server());

app.register_network_owner::<NetworkOwner>();
app.register_input_stream::<MoveInput>(InputStreamConfig::predicted());

app.replicate_component::<Transform2D>();
```

#### Client

```rust
app.add_plugin(NetPlugin::<AutoNet>::client());

app.register_network_owner::<NetworkOwner>();
app.register_input_stream::<MoveInput>(InputStreamConfig::predicted());

app.replicate_component::<Transform2D>();
```

### Local input production

```rust
fn movement_input_system(mut input: LocalInput<MoveInput>) {
    input.push(MoveInput { x: 1.0, y: 0.0 });
}
```

### Simulation application rule

```rust
fn apply_move_input_system(
    input: InputView<MoveInput>,
    mut query: Query<(&NetworkOwner, &mut Transform2D)>,
) {
    for (owner, transform) in query.iter_mut() {
        if let Some(command) = input.latest_for(owner.connection_id) {
            transform.x += command.x;
            transform.y += command.y;
        }
    }
}
```

This is generic enough for many use cases:

- 2D movement
- 3D movement
- vehicle steering
- camera control
- RTS unit commands
- ability targeting
- weapon input
- building placement commands

The engine remains generic because it never hardcodes the gameplay meaning.

## Events

The ECS event system should remain local-first.

Use events for:

- local reactions
- UI
- audio
- VFX
- editor/runtime notifications
- optional sparse replicated signals

Do not use ECS events as the primary multiplayer truth path.

Multiplayer truth should flow through:

- input streams
- authoritative state
- replicated components
- snapshots/deltas
- corrections/replay

## Schedules and Ordering

The engine should continue to own schedule ordering for multiplayer internals.

Desired order:

1. runtime receive
2. inbox processing / decode
3. local input gathering
4. simulation
5. replication build
6. outbox flush
7. frame/tick finalization

The game should only register gameplay systems into the appropriate phases.

## Proposed High-Level Types

### Attributes / macros

- `#[replicated(...)]`
- optional `#[net_entity]`

### App methods

- `replicate_component::<T>()`
- `register_input_stream::<T>(config)`
- `register_network_owner::<T>()`

### Runtime params

- `LocalInput<T>`
- `InputView<T>`

### Optional helpers

- `OwnedEntities<TOwner>`
- `LocalConnection`
- `CorrectionStream<T>`
- `ReplicationStateView<T>`

### Escape hatch

- `ReplicationDriver`
- `InputDriver`
- `SnapshotApplyDriver`

## Recommended Defaults

### For movement-like state

- authority: `Server`
- sync: `Delta`
- prediction: `Owner`
- interest: `Global` or `Spatial`
- profile: `PredictedMovement`

### For gameplay state like health/inventory

- authority: `Server`
- sync: `Delta`
- prediction: `None`
- interest: `Global`, `OwnerOnly`, or `Team`
- profile: `ReliableState`

### For sparse one-off replicated signals

- profile: `SparseEvent`

## Design Benefits

This model gives:

- low boilerplate
- high reuse
- generic input/state flow
- explicit ownership
- strong customization
- prediction support
- no forced character-controller abstraction
- clean separation from transport/runtime
- compatibility with the existing low-level engine net stack

## Final Recommendation

Default multiplayer workflow should be:

1. define serializable replicated components
2. annotate them with replication metadata
3. define serializable input types
4. register ownership mapping
5. write ordinary ECS systems that read typed input views and mutate state
6. let the engine handle transport, prediction, replay, snapshots, and correction

This should be the standard path.

Custom replication drivers remain available for advanced cases, but they should no longer be required for normal multiplayer gameplay.
