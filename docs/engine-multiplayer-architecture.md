# Engine and Multiplayer Architecture Map

## High-level architecture

1. `engine` provides the ECS app runtime and schedule pipeline (`PreUpdate -> FixedUpdate -> Update -> Render*`).
2. `engine_net` defines multiplayer protocol/session abstractions.
3. `engine_net_quic` runs transport runtime tasks (connect/accept/send/receive/reconnect).
4. `cavern_hunt` is the game layer (authoritative simulation, replication schema, correction/prediction).
5. `grotto_client` and `grotto_server` wire everything together from configuration.

## Recommended reading order

1. `README.md`
2. `docs/current-state.md`
3. `engine/src/app/mod.rs`
4. `engine/src/runtime/schedules.rs`
5. `engine/src/plugins/net.rs`
6. `engine_net/src/protocol.rs`
7. `engine_net/src/session.rs`
8. `engine_net_quic/src/lib.rs`
9. `cavern_hunt/src/plugins/net_sync.rs`
10. `cavern_hunt/src/domain/snapshot.rs`

## File ownership map (what does what)

- Runtime/bootstrap:
  - `engine/src/lib.rs`
  - `engine/src/app/mod.rs`
- Schedule ordering:
  - `engine/src/runtime/schedules.rs`
- Engine networking bridge (ECS <-> runtime):
  - `engine/src/plugins/net.rs`
- Protocol message types:
  - `engine_net/src/protocol.rs`
- Session state machine:
  - `engine_net/src/session.rs`
- QUIC runtime and reconnect behavior:
  - `engine_net_quic/src/lib.rs`
- Game-level multiplayer sync and correction:
  - `cavern_hunt/src/plugins/net_sync.rs`
- Replicated snapshot/delta schema:
  - `cavern_hunt/src/domain/snapshot.rs`
- Networking tuning/profile schema:
  - `cavern_hunt/src/domain/net_config.rs`
- Server entrypoint/wiring:
  - `grotto_server/src/main.rs`
- Client entrypoint/wiring:
  - `grotto_client/src/main.rs`

## Operator control plane map

- Operator WebSocket protocol and runtime bridge:
  - `grotto_online/src/operator.rs`
- Server-side operator command intake and snapshot/event emission:
  - `grotto_server/src/operator_control.rs`
- Runtime command types for operator actions:
  - `engine_net/src/session.rs`
- QUIC runtime command behavior (drain + targeted disconnect):
  - `engine_net_quic/src/lib.rs`
- Operator config schema in `.ron` assets:
  - `cavern_hunt/src/domain/net_config.rs`
- Out-of-process lifecycle/log control service:
  - `grotto_fleet_control/src/lib.rs`
  - `grotto_fleet_control/src/main.rs`
  - `grotto_fleet_control/src/service.rs`
  - `grotto_fleet_control/src/provider.rs`
  - `grotto_fleet_control/src/kubernetes.rs`

## Replication debugging starting points

If replication is not behaving correctly, inspect in this order:

1. `cavern_hunt/src/plugins/net_sync.rs`
2. `engine/src/plugins/net.rs`
3. `cavern_hunt/src/domain/snapshot.rs`

This sequence covers game-side replication logic, engine bridge behavior, and payload schema compatibility.
