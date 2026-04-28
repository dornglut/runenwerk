---
title: "engine_net_quic"
description: "Documentation for engine_net_quic."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---

# engine_net_quic

`engine_net_quic` is the Quinn-based QUIC adapter for `engine_net` runtime contracts.

It implements client/server endpoint bootstrap, handshake framing, live datagram loops, runtime handle channels, and admission/trust policies.

## Responsibilities

- Create QUIC server/client endpoints
- Enforce trust policy (roots/pinned fingerprint)
- Perform handshake stream framing (`Hello`, `JoinRequest`, `JoinAccepted`, `JoinRejected`)
- Run post-handshake live datagram loops
- Bridge runtime command/event channels to async QUIC tasks
- Handle reconnect/backoff and server admission flow

## Module Layout

- `src/config/`
  - `client.rs`, `server.rs`, `transport.rs`
- `src/transport/`
  - `endpoint_factory.rs`, `trust.rs`, `certificates.rs`, `framing.rs`, `lanes.rs`
- `src/runtime/`
  - `command_bus.rs`, `connection.rs`, `event_bus.rs`, `event_dispatch.rs`,
    `handles.rs`, `join_rejection.rs`, `message_transport.rs`, `reconnect.rs`,
    `reconnect_backoff.rs`, `routing.rs`
- `src/client/`
  - `bootstrap.rs`, `policy.rs`, `runtime.rs`
- `src/server/`
  - `admission.rs`, `policy.rs`, `accept.rs`, `peer.rs`, `runtime.rs`
- `src/driver/`
  - `net_loop.rs`

## Runtime Model

- Handshake path: QUIC bidirectional streams + framed envelopes
- Live path: QUIC datagrams for runtime session traffic after admission

## Key Public API

- Transport/runtime: `QuicTransport`, `QuicTransportConfig`, `QuicServerEndpoint`
- Trust/admission: `QuicTrustPolicy`, `QuicServerJoinVerifier`, `QuicJoinVerificationError`
- Runtime handles: `QuicRuntimeClientHandle`, `QuicRuntimeServerHandle`
- Handshake framing: `write_message`, `read_message`
- Utility: `default_client_bind_addr`, `certificate_fingerprint_sha256`

## Relationship to engine_net

- Consumes `engine_net` protocol/session contracts
- Emits/consumes `SessionRuntimeEvent` and `SessionRuntimeCommand`
- Remains gameplay-agnostic; it does not own replication semantics
