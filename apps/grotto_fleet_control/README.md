# grotto_fleet_control

`grotto_fleet_control` manages out-of-process server lifecycle and log inspection for Axiom operator console.

## Purpose

- execute lifecycle commands (`start_server`, `stop_server`) against a fleet provider
- execute windowed log inspection queries (`inspect_logs`)
- route Axiom operator lifecycle commands into provider calls

## Usage

- build a provider implementation (current: `KubernetesFleetProvider`)
- call `execute_fleet_command(...)` with:
  - provider instance
  - inbound `AxiomOperatorCommand`
  - `FleetCommandContext` authorization scope
- run the fleet service process:
  - `cargo run -p grotto_fleet_control -- --config ops/fleet/kubernetes.ron`

## Ownership boundaries

- Owns fleet lifecycle/log interfaces and provider implementations
- Owns routing of lifecycle/log commands
- Does not own in-process runtime control (`drain`, runtime disconnect) inside `grotto_server`
- Does not own Axiom join-grant handoff/verification

## Extension points

- add new `FleetProvider` implementations (Nomad, ECS)
- extend routing policies in `execute_fleet_command`
- evolve log paging/filter behavior

## Config

The default config path is `ops/fleet/kubernetes.ron`.

- `kubernetes`: deployment/log query behavior for the Kubernetes provider
- `axiom.enabled`: enables command bridge to Axiom
- `axiom.ws_url`: command WebSocket endpoint (Axiom fleet bridge: `/v2/operator/fleet/ws`)
- `axiom.command_token`: bearer token for command channel auth
- `axiom.runtime_graceful_stop_enabled`: issue runtime drain/shutdown dispatch before force stop
- `axiom.runtime_graceful_default_timeout_ms`: wait budget for runtime graceful stop stage
- `axiom.runtime_force_stop_timeout_ms`: fallback Kubernetes force-stop timeout
- `axiom.allowed_server_ids`: optional allow-list (empty means all)

Runtime overrides are available for local ops automation:

- CLI: `--axiom-enabled`, `--axiom-ws-url`, `--axiom-command-token`, `--axiom-service-id`
- Env: `GROTTO_FLEET_AXIOM_ENABLED`, `GROTTO_FLEET_AXIOM_WS_URL`, `GROTTO_FLEET_AXIOM_COMMAND_TOKEN`, `GROTTO_FLEET_AXIOM_SERVICE_ID`
