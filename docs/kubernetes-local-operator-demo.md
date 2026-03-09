# Kubernetes Local Operator Demo

This runbook points to the current local operator/bootstrap scripts used by Cavern Hunt.

## Prerequisites

- Local Kubernetes cluster (for example: `kind`, `k3d`, or Docker Desktop Kubernetes)
- `kubectl`
- `bash`

## Main Scripts

- Base local stack bootstrap:
  - [games/cavern_hunt/scripts/k8s/bootstrap_local_stack.sh](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/scripts/k8s/bootstrap_local_stack.sh)
- Operator bridge bootstrap:
  - [games/cavern_hunt/scripts/k8s/bootstrap_operator_bridges.sh](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/scripts/k8s/bootstrap_operator_bridges.sh)
- End-to-end smoke flow:
  - [games/cavern_hunt/scripts/k8s/smoke_operator_flow.sh](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/games/cavern_hunt/scripts/k8s/smoke_operator_flow.sh)

## Suggested Order

1. Run `bootstrap_local_stack.sh`.
2. Run `bootstrap_operator_bridges.sh`.
3. Run `smoke_operator_flow.sh`.

For current fleet control service behavior and endpoints, see:
[apps/grotto_fleet_control/README.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/apps/grotto_fleet_control/README.md)
