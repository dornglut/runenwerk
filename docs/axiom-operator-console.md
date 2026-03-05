# Axiom Operator Console Architecture

Updated: 2026-03-05

## Scope

This document maps the runtime and fleet control paths used by Axiom operator console.

## Phase 1 (implemented now): runtime control + observability

Runtime control is handled in-process by `grotto_server` through a persistent WebSocket bridge.

- Bridge transport and operator protocol:
  - `grotto_online/src/operator.rs`
- Server-side command intake, dedupe, snapshots, runtime event emission:
  - `grotto_server/src/operator_control.rs`
- Server boot wiring:
  - `grotto_server/src/main.rs`
- Runtime command surface:
  - `engine_net/src/session.rs`
- QUIC runtime command behavior:
  - `engine_net_quic/src/lib.rs`
- Operator config schema:
  - `cavern_hunt/src/domain/net_config.rs`

Supported runtime commands:

- `set_drain_mode`
- `disconnect_connection`
- `shutdown`
- `snapshot_now`

Notes:

- drain mode rejects new join requests with `ServerShuttingDown`
- targeted disconnect resolves by `ConnectionId`
- command dedupe is handled by `command_id` in server operator control state
- outbound messages include snapshots, structured events, and per-command results

## Phase 2 (implemented service baseline): lifecycle + log inspection

Lifecycle and indexed log inspection are handled out-of-process by `grotto_fleet_control`.

- Fleet service crate:
  - `grotto_fleet_control/src/lib.rs`
  - `grotto_fleet_control/src/main.rs`
- Fleet provider trait and contracts:
  - `grotto_fleet_control/src/provider.rs`
- Command router and auth gating:
  - `grotto_fleet_control/src/router.rs`
- Fleet WebSocket runtime loop:
  - `grotto_fleet_control/src/service.rs`
- Kubernetes provider:
  - `grotto_fleet_control/src/kubernetes.rs`
- Fleet config model:
  - `grotto_fleet_control/src/config.rs`
- Example fleet config:
  - `ops/fleet/kubernetes.ron`

Supported command families (routed in fleet control):

- `start_server(server_id)`
- `stop_server(server_id, graceful_timeout)`
- `inspect_logs(server_id, from_ts, to_ts, level, limit, cursor)`

`stop_server` execution order in fleet service:

1. dispatch runtime commands through Axiom:
   - `set_drain_mode(enabled=true)`
   - `shutdown(grace_ms=...)`
2. wait graceful timeout window
3. execute Kubernetes fallback stop (scale to zero) with force-stop timeout

Service bootstrap command:

- `cargo run -p grotto_fleet_control -- --config ops/fleet/kubernetes.ron`

## Defaults and safety

- all operator features are opt-in and disabled by default
- server profiles include an `axiom_operator` block with `enabled = false`
- local non-Axiom workflows remain unchanged when operator config is off

## Axiom bridge endpoints and token flow

Bridge WebSocket endpoints:

- runtime bridge: `/v2/operator/runtime/ws`
- fleet bridge: `/v2/operator/fleet/ws`

Bridge token issue endpoint:

- `POST /v2/admin/operator/bridge-tokens` with:
  - `bridge_kind`: `runtime_bridge` or `fleet_bridge`
  - `bridge_id`: runtime `server_id` or fleet `service_id`

Local helper script:

- `scripts/axiom_issue_operator_bridge_tokens.sh`
  - logs in with `/v2/auth/operator/login`
  - issues runtime + fleet bridge tokens
  - prints env exports consumed by `grotto_server` and `grotto_fleet_control`

## Where to read first

1. `docs/engine-multiplayer-architecture.md`
2. `cavern_hunt/src/domain/net_config.rs`
3. `grotto_server/src/operator_control.rs`
4. `grotto_online/src/operator.rs`
5. `engine_net/src/session.rs`
6. `engine_net_quic/src/lib.rs`
7. `grotto_fleet_control/src/provider.rs`
8. `grotto_fleet_control/src/kubernetes.rs`
9. `grotto_fleet_control/src/service.rs`
