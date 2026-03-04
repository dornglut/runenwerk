# engine_net

`engine_net` contains the transport-agnostic networking layer shared by the engine and runtime hosts.

## Purpose

- Define client/server protocol messages
- Define session state machines and runtime events
- Define replication markers and cursors
- Define simulation-facing command/state types used by the dedicated-authority profile

## Current Scope

Implemented now:

- protocol encode/decode
- join handshake and admission messages
- session bootstrap state
- runtime command/event channels
- snapshot and delta message envelopes
- transport-independent session/admission data

Still narrow:

- replication payloads are currently scene-specific
- the broader gameplay entity replication model is not finished yet
