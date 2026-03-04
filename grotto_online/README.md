# grotto_online

`grotto_online` bridges the game runtime to Axiom control-plane services.

## Purpose

- request join grants
- refresh or restore auth/session data
- consume join tickets on the dedicated server path
- validate join-grant metadata before runtime admission
- map consumed Axiom handoff data into runtime-facing admission state

## Current State

Implemented now:

- `JoinGrant` validation and conversion into runtime session targets
- HTTP client for auth refresh and join-grant issue/consume
- client-side grant provider integration
- server-side ticket verification integration

Not yet complete:

- full client control-plane boot flow
- websocket resync
- richer live control-plane state management inside the runtime
