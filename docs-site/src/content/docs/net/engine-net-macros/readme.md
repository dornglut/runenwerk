---
title: "engine_net_macros"
description: "Documentation for engine_net_macros."
---

# engine_net_macros

`engine_net_macros` provides attribute macros for declarative replication metadata.

## Macros

- `#[net_entity]`
  - Implements `engine_net::replication::NetEntity` for the annotated struct.

- `#[net_component(...)]`
  - Implements `engine_net::replication::NetComponentMetadata` and generates a `ReplicatedComponentDescriptor`.
  - Supported arguments:
    - `authority = Server | Client`
    - `direction = ServerToClient | ClientToServer | Bidirectional`
    - `profile = PredictedMovement | ReliableState | SparseEvent | InputCommand | Cosmetic`
    - `interest = Global | OwnerOnly | Spatial | Team | Distance`
    - `owner_prediction = true | false`

When `direction` is omitted, the macro uses the default direction of the selected replication profile.
