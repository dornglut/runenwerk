---
title: engine_net_macros
description: Current documentation for the retained engine_net_macros RN8 migration surface.
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-13
---

# engine_net_macros

`engine_net_macros` provides attribute macros for the retained Runenwerk replication-metadata surface.

The crate is a proc-macro crate. It generates implementations against `engine_net::replication` contracts, but it does not own the replication runtime itself. Under the current RN8 boundary it is migration residue, not the accepted future ordinary multiplayer authoring model.

Current scope note: these macros generate metadata implementations. They
do not generate snapshot encoding, delta generation, ECS extraction, ECS
apply logic, or transport sends.

## Macros

### `#[net_entity]`

`#[net_entity]` implements:

```text
engine_net::replication::NetEntity
```

for the annotated struct.

### `#[net_component(...)]`

`#[net_component(...)]` implements:

```text
engine_net::replication::NetComponentMetadata
```

and generates a `ReplicatedComponentDescriptor`.

Supported arguments:

- `authority`
- `direction`
- `reliability`
- `prediction`
- `priority`
- `frequency_hz`
- `profile`
- `interest`
- `owner_prediction`

Unsupported arguments are rejected at macro parse time.

## Default Metadata

When omitted, the macro uses these defaults:

- `authority = Server`
- `profile = ReliableState`
- `interest = Global`
- `owner_prediction = false`
- optional `direction`, `reliability`, `prediction`, `priority`, and `frequency_hz` remain unset unless provided.

## Example

```rust
use engine_net_macros::{net_component, net_entity};

#[net_entity]
pub struct Player;

#[net_component(
    authority = Server,
    direction = ServerToClient,
    reliability = Reliable,
    prediction = Disabled,
    priority = High,
    frequency_hz = 30,
    profile = ReliableState,
    interest = Global,
    owner_prediction = false
)]
pub struct PlayerTransform;
```

## Ownership Boundaries

In scope:

- retained replication metadata macros;
- generated implementations for `engine_net::replication` traits;
- compile-time validation of supported macro arguments.

Out of scope:

- transport behavior;
- session runtime;
- replication scheduling;
- snapshot delta generation;
- game-specific networking policy;
- definition of the future ordinary multiplayer authoring syntax.

Current architecture and migration authority:

- [Runenwerk networking architecture](../net-architecture.md)
- [Multiplayer replication implementation roadmap](../multiplayer-replication-implementation-roadmap.md)

The former declarative component-registration target design is archived as historical evidence. Do not infer a future `.replicate::<T>()`, attribute, Replicated View API, audience vocabulary, or other authoring syntax from this retained crate.

## Validation

Run:

```text
cargo test -p engine_net_macros
cargo check --workspace
```
