# Local Kubernetes Operator Demo (kind/k3d)

Updated: 2026-03-05

This runbook deploys a full local stack in Kubernetes and validates the operator `start_server`, `stop_server`, and `inspect_logs` workflow.

## Scope

The demo deploys these core components into one namespace:

- `core-api`
- `operator-console`
- `postgres`
- `redis`
- `grotto_fleet_control`
- `grotto_server` (default replicas `0`, managed by operator commands)

An in-cluster `caddy` gateway is also deployed for same-origin browser access to `/v2*` and operator UI.

## Prerequisites

- Docker
- `kubectl`
- `helm`
- `curl`
- `jq`
- one local cluster tool:
  - `kind`, or
  - `k3d`

## Quickstart

From `grotto-quest` root:

```bash
scripts/k8s/bootstrap_local_stack.sh
scripts/k8s/bootstrap_operator_bridges.sh
scripts/k8s/smoke_operator_flow.sh
```

Open operator console:

```bash
kubectl -n multiplayer-dev port-forward svc/multiplayer-stack-caddy 8081:80
```

Then browse: `http://127.0.0.1:8081`

Use operator credentials:

- username: `axiom-operator`
- password: `operator-password`

## What Each Script Does

### 1) `bootstrap_local_stack.sh`

- creates `kind`/`k3d` cluster if missing
- builds local images for Axiom + Grotto services
- imports images into cluster
- installs/updates Helm release `multiplayer-stack`
- waits for core deployments to become ready

### 2) `bootstrap_operator_bridges.sh`

- port-forwards API service locally
- logs in as operator
- issues bridge tokens (`runtime_bridge`, `fleet_bridge`)
- updates Kubernetes secrets:
  - `multiplayer-stack-runtime-bridge-env`
  - `multiplayer-stack-fleet-bridge-env`
- restarts bridge consumers
- ensures runtime node `srv-local` exists in Axiom admin resources and marks it active

### 3) `smoke_operator_flow.sh`

- logs in as operator
- resolves runtime node resource by `resource_key=srv-local`
- executes:
  - `start_server`
  - `inspect_logs`
  - `stop_server`
- validates deployment scales `0 -> 1 -> 0`
- validates bridge connection log entries in `core-api`

## Helm Assets

- chart: `ops/helm/multiplayer-stack`
- default namespace: `multiplayer-dev`
- default runtime server id: `srv-local`
- default release: `multiplayer-stack`

Dockerfiles used by bootstrap:

- `ops/docker/grotto_server.Dockerfile`
- `ops/docker/grotto_fleet_control.Dockerfile`

## Known Limitations

- Single-cluster, single-region local setup only.
- `grotto_server` default replicas are `0`; it only runs after `start_server`.
- Bridge routing in Axiom is still in-memory per API instance; this milestone assumes one API replica.
- Secrets are local-dev defaults and are not production-grade.
- No ingress/TLS hardening in this flow; access is via `kubectl port-forward`.
