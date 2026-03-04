# engine_net_quic

`engine_net_quic` is the QUIC transport adapter for the current dedicated-authority runtime path.

## Purpose

- Create QUIC client/server runtimes
- Handle join handshake transport
- Maintain live session tasks
- Expose runtime command/event channels back to the engine
- Support reconnect and trust-policy driven admission

## Current State

Implemented now:

- server/client runtime handles
- self-signed local/dev certificate generation
- certificate fingerprint support
- trust-policy input for client admission
- live control-path handshake
- post-handshake datagram exchange
- reconnect support in the runtime task layer

The current engine and host binaries use this crate for the live `DedicatedAuthority` profile.
