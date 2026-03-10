# engine_net

`engine_net` is the transport-agnostic multiplayer contract crate.

It defines protocol messages, session state transitions, replication contracts, runtime events/commands, and simulation-facing network markers.

## Responsibilities

- Protocol messages and envelope encoding/decoding
- Client/server session state model and admission flow contracts
- Replication driver interfaces and snapshot cursors
- Runtime command/event contracts for host adapters
- Shared simulation/network identity types
- Transport abstraction (`Transport`, `TransportKind`, `ConnectionId`)

`engine_net` does not implement concrete transport I/O. That is handled by adapters such as `engine_net_quic`.

## Module Layout

- `src/protocol/`
  - `version.rs`, `control.rs`, `snapshot.rs`, `input.rs`, `ack.rs`, `envelope.rs`
- `src/session/`
  - `ids.rs`, `admission.rs`, `handoff.rs`
- `src/replication/`
  - `profile.rs`, `model.rs`, `timeline.rs`, `prediction.rs`, `interest.rs`, `diagnostics.rs`
- `src/runtime/`
  - `events.rs`, `client.rs`, `server.rs`
- `src/simulation/`
  - `mod.rs`, `tick.rs`, `frame.rs`
- `src/transport/`
  - `mod.rs`, `lanes.rs`, `semantics.rs`

## Key Public Contracts

- Protocol: `ClientMessage`, `ServerMessage`, `MessageEnvelope`, `encode_message`, `decode_message`
- Session: `ClientSessionState`, `ServerSessionState`, `SessionPhase`, `SessionRuntimeCommand`, `SessionRuntimeEvent`
- Replication: `ReplicationDriver`, `Replicate`, `Replicated`, `SnapshotCursor`
- Replication metadata: `NetComponentMetadata`, `ReplicationRegistry`, `NetEntityMap`
- Transport: `Transport`, `TransportKind`, `ConnectionId`
- Macros (re-exported): `#[net_component(...)]`, `#[net_entity]`

## Relationship to Other Crates

- `engine_sim`: shared simulation types re-exported by `engine_net`
- `engine_net_quic`: concrete QUIC runtime adapter implementing transport/runtime behavior
- `engine`: consumes session/runtime contracts and drives replication systems
