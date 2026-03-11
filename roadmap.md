# Hard-Cutover Net API + Multi-Client Runtime Roadmap

## Summary
Deliver a full multi-client networking re-architecture with a hard cutover to a clean game-facing API (`engine::net::prelude::*`), while keeping compile-green checkpoints across phases.

Target outcome: per-connection replication correctness, targeted server delivery, sender-aware input handling, and one ergonomic import surface for game code.

## Implementation Changes

### Phase 1A: Runtime Contract Break in `engine_net` (multi-client semantics)
- Update `SessionRuntimeCommand` in `net/engine_net/src/session/ids.rs` to replace broadcast-only server command with explicit targeted + broadcast variants.
- Keep `SessionRuntimeEvent` connection-aware for all server-ingress relevant paths.
- Add `net/engine_net/src/prelude.rs` and re-export it from `net/engine_net/src/lib.rs` so macro + runtime contracts have one canonical prelude.

### Phase 1B: Driver Trait Extraction in `engine_net`
- Move replication driver traits out of `net/engine_net/src/replication/prediction.rs` into a dedicated module (for example `net/engine_net/src/replication/driver.rs`).
- Update `ReplicationDriver::receive_remote_input` to include `ConnectionId` so authoritative gameplay knows who sent inputs.
- Re-export the driver trait surface from `net/engine_net/src/replication/mod.rs` so callsites use stable paths.
- Update callsites in `net/engine_net/src/runtime/server.rs` to use per-connection ACK/baseline semantics consistently with targeted dispatch and sender-aware input handling.

### Phase 2: Transport Adapter Alignment in `engine_net_quic`
- Update `run_server_runtime_task` in `net/engine_net_quic/src/server/runtime.rs` to route targeted server messages to one connection and keep explicit broadcast behavior separate.
- Update backpressure handling in `run_server_runtime_task` so `try_send` full is not treated as disconnect.
- Update `accept_incoming_connection` in `net/engine_net_quic/src/server/accept.rs` to avoid resetting shared session state for unrelated active peers during new admissions.
- Update event delivery helpers in `net/engine_net_quic/src/runtime/helpers.rs` (`send_runtime_event`, `send_peer_event`) to stop silent loss of critical state transitions.
- Update peer loop handling in `net/engine_net_quic/src/server/peer.rs` to preserve accurate close reason propagation and connection lifecycle events.

### Phase 3: Engine Net Plugin Multi-Client Refactor
- Refactor role/queue state in `engine/src/plugins/net/resources.rs` by splitting server/client replication state and keying server replication by `ConnectionId`.
- Add an explicit per-connection baseline checkpoint model in `engine/src/plugins/net/resources.rs` (or submodule) with at least:
  - `last_ack_cursor`
  - `last_sent_cursor`
  - `last_full_snapshot_cursor`
  - `last_full_snapshot_tick`
  - `needs_full_resync`
- Refactor runtime bridge in `engine/src/plugins/net/runtime_io.rs`:
  - `network_runtime_receive_system` keeps connection identity through server ingress.
  - `server_receive_system` applies ACK/input per connection and updates ECS-facing status without clobbering active peers.
- Refactor replication/prediction logic in `engine/src/plugins/net/prediction.rs`:
  - `replication_step_system` emits snapshots/deltas per connection baseline.
  - `apply_authoritative_delta` validates base cursor and uses driver delta apply path.
- Add explicit fixed-step ordering constraints in `engine/src/plugins/net/resources.rs` configure functions so behavior is not registration-order dependent.
  per-connection checkpoint state is server-owned runtime state, not long-term plugin-owned semantic state

### Phase 4: Hard Cutover Ergonomics (`engine::net::prelude`)
- Add `engine/src/net/mod.rs` and `engine/src/net/prelude.rs` as canonical game-facing API.
- Update `engine/src/lib.rs` to export `pub mod net;`.
- Update `engine/src/plugins/net/plugin.rs` to add `NetPlugin::<Driver>::new(NetRole)` while keeping role constructors as aliases.
- Hard-cutover imports to `engine::net::prelude::*` across existing usage:
  - `apps/grotto_client/src/main.rs`
  - `apps/grotto_server/src/main.rs`
  - `games/cavern_hunt/tests/live_multiplayer/helpers.rs`
  - `games/cavern_hunt/src/net/replication_intent/components.rs`
  - `engine/tests/network_plugins.rs`
- Remove old public net import path expectations from broad plugin exports (module surface cleanup in `engine/src/plugins/mod.rs` and related re-exports) so new code path is unambiguous.

### Phase 5: Game/Integration Migration and Documentation
- Update cavern_hunt net-facing modules to consume new prelude and new runtime contracts (notably connection-aware input/replication points in its net domain modules).
- Update docs to reflect final API and multi-client behavior:
  - `engine/src/plugins/net/README.md`
  - `engine/src/plugins/net/NETWORK_RUNTIME_FLOW.md`
  - `net/engine_net/README.md`
  - `net/engine_net/REPLICATION_PIPELINE.md`
- update NET_PLUGIN.md 
- update NETWORK_USAGE.md

## Test Plan
- Contract/unit level:
  - `cargo test -p engine_net`
  - `cargo test -p engine_net_quic`
  - Add/adjust tests for per-connection ACK divergence, targeted send, and delta-base validation.
- Engine integration:
  - `cargo test -p engine --test network_plugins`
  - Add scenarios for two clients with different baselines and rejection/reconnect while other clients stay active.
- End-to-end acceptance gate:
  - `cargo test -p cavern_hunt --test live_multiplayer`
  - Both existing tests must pass:
    - `two_live_clients_share_one_cavern_hunt_run`
    - `four_live_clients_complete_run_and_reconnect_one_client`
- Phase checkpoint rule:
  - Each phase lands compile-green with all above suites green before moving to next phase.

## Clarification: Hard Cutover + Compile-Green Per Phase
- Hard cutover means no long-lived compatibility shim APIs are kept.
- Compile-green per phase means each phase includes all required callsite migrations for that phase's API break before merge.
- Temporary breakage is allowed only inside an in-progress branch, never at phase boundaries.

## Assumptions and Defaults
- This is a hard cutover: existing callsites are migrated, not left dual-path.
- `NetPlugin::<Driver>::new(NetRole)` is added; role constructors remain as convenience aliases.
- No new `games/minimal_game` crate is added in this stream; effort is focused on re-architecture + migration of existing code.
- Multi-client correctness is defined as per-connection ACK/baseline/input identity and targeted server replication behavior.
