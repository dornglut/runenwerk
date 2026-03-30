---
title: "Cavern Hunt Playtest Guide"
description: "Documentation for Cavern Hunt Playtest Guide."
---

# Cavern Hunt Playtest Guide

Updated: 2026-03-05

## Goal

This is the current friend-test path for `Cavern Hunt`.

The local/dev flow does not require Axiom. It uses:

- one local dedicated server
- 1 to 4 local clients
- the dev QUIC certificate written by the server
- prebuilt binaries (party script builds once, then launches directly)

Primary local target:

- smooth 2-local clients + dedicated server on one machine
- 4-local is best-effort and expected to degrade on lower-end laptops

The optional Axiom-backed path is still supported for advanced testing.

## Default Local Flow

### 1. Start the dedicated server

```bash
games/cavern_hunt/scripts/run_cavern_server.sh
```

Expected behavior:

- the server binds to `127.0.0.1:7000`
- it writes a dev certificate to `var/dev/server-cert.der`
- it keeps running until `Ctrl-C`

### 2. Start one client

```bash
games/cavern_hunt/scripts/run_cavern_client.sh
```

### 3. Start multiple clients

Run the client script once per client, or use:

```bash
games/cavern_hunt/scripts/run_cavern_party_local.sh
```

That starts:

- one local server
- two local clients by default

Use explicit 4-client mode when needed:

```bash
games/cavern_hunt/scripts/run_cavern_party_local.sh 4
```

PowerShell equivalents:

```powershell
pwsh -File games/cavern_hunt/scripts/run_cavern_server.ps1
pwsh -File games/cavern_hunt/scripts/run_cavern_client.ps1
pwsh -File games/cavern_hunt/scripts/run_cavern_party_local.ps1 -ClientCount 4
```

Client and server logs are written to:

- `/tmp/cavern_hunt_server.log`
- `/tmp/cavern_hunt_client_1.log`
- `/tmp/cavern_hunt_client_2.log`
- `/tmp/cavern_hunt_client_3.log`
- `/tmp/cavern_hunt_client_4.log`

## Networking Config Assets

Network and multiplayer settings now come from `.ron` assets loaded at process startup.

Default paths:

- client: `games/cavern_hunt/assets/networking/client/local_dev.ron`
- server: `games/cavern_hunt/assets/networking/server/local_dev.ron`

Built-in profiles:

- `local_dev`
- `two_local_balanced`
- `four_local_conservative`

Script profile selection:

- `CAVERN_NET_PROFILE=two_local_balanced games/cavern_hunt/scripts/run_cavern_party_local.sh`
- `CAVERN_NET_PROFILE=four_local_conservative games/cavern_hunt/scripts/run_cavern_party_local.sh 4`

Explicit config override:

- `CAVERN_CLIENT_CONFIG_PATH=... games/cavern_hunt/scripts/run_cavern_client.sh`
- `CAVERN_SERVER_CONFIG_PATH=... games/cavern_hunt/scripts/run_cavern_server.sh`

Direct binary usage:

- `target/debug/grotto_client --config games/cavern_hunt/assets/networking/client/local_dev.ron`
- `target/debug/grotto_server --config games/cavern_hunt/assets/networking/server/local_dev.ron`

Hot reload:

- config hot reload is controlled per file via `hot_reload.enabled` and `hot_reload.poll_interval_seconds`
- only safe runtime fields are applied live (cadence/budgets/interpolation/diagnostics/mode)

## Optional Environment Variables

These remain useful for local rendering/dev workflow:

- `CAVERN_RENDER_MODE`
  - `legacy` or `material_graph`
  - default: material profile setting (normally `material_graph`)
- `CAVERN_MATERIAL_PROFILE`
  - `performance`, `balanced`, or `quality`
  - default: `performance` in local party script
- `CAVERN_GI_MODE`
  - `off`, `ao`, `probes`
- `CAVERN_GI_QUALITY`
  - `low`, `medium`, `high`
- `CAVERN_RELEASE`
  - default: `1` for local party script (release binaries)
- `CAVERN_CLIENT_START_STAGGER_SECONDS`
  - default: `0.15`
  - staggers local client startup to reduce burst contention

## Optional Axiom-Backed Flow

Enable these fields in your client/server `.ron` profiles:

- client:
  - `use_axiom_handoff = true`
  - `axiom_api_base_url`
  - `axiom_lobby_id`
  - `axiom_access_token` or `axiom_refresh_token`
  - `axiom_device_id`
- server:
  - `use_axiom_verifier = true`
  - `axiom_api_base_url`
  - `dedicated_server_shared_secret`

If these are unset, local fallback join remains active.

## Optional Axiom Operator Console (Runtime Control)

Server profiles now include an `axiom_operator` block (disabled by default):

- `enabled`
- `ws_url`
- `runtime_token`
- `heartbeat_seconds`
- `snapshot_interval_ticks`
- `max_buffered_events`

Bridge endpoints (Axiom):

- runtime bridge: `/v2/operator/runtime/ws`
- fleet bridge: `/v2/operator/fleet/ws`

You can still hardcode values in `.ron` config, but local/dev is easier with env overrides:

- `CAVERN_AXIOM_OPERATOR_ENABLED`
- `CAVERN_AXIOM_OPERATOR_WS_URL`
- `CAVERN_AXIOM_OPERATOR_RUNTIME_TOKEN`
- `GROTTO_FLEET_AXIOM_ENABLED`
- `GROTTO_FLEET_AXIOM_WS_URL`
- `GROTTO_FLEET_AXIOM_COMMAND_TOKEN`
- `GROTTO_FLEET_AXIOM_SERVICE_ID`

Helper to issue bridge tokens from Axiom and print export commands:

```bash
AXIOM_OPERATOR_PASSWORD=... scripts/axiom_issue_operator_bridge_tokens.sh
```

Runtime bridge launch:

```bash
games/cavern_hunt/scripts/run_cavern_server.sh
```

Phase 2 lifecycle/log control process:

- `cargo run -p grotto_fleet_control -- --config ops/fleet/kubernetes.ron`

When operator/fleet bridge env vars are unset, local scripts and non-Axiom flow remain unchanged.

### Kubernetes Quickstart (Local kind/k3d)

From `grotto-quest` root:

```bash
games/cavern_hunt/scripts/k8s/bootstrap_local_stack.sh
games/cavern_hunt/scripts/k8s/bootstrap_operator_bridges.sh
games/cavern_hunt/scripts/k8s/smoke_operator_flow.sh
```

Then port-forward the in-cluster gateway and open operator console:

```bash
kubectl -n multiplayer-dev port-forward svc/multiplayer-stack-caddy 8081:80
```

Browse `http://127.0.0.1:8081` and log in with local operator credentials.

## What a Successful Run Looks Like

For the current vertical slice, a successful run is:

1. the party spawns into the same procedural cavern
2. players move, dash, shoot, and clear cavern rooms
3. the `NestGuardian` elite is killed
4. extraction becomes active
5. at least one living player reaches extraction and completes it
6. every connected client converges on `Success`
7. each successful client receives local `CavernMarks` from extracted scrap

## Current Controls

- `WASD`: move
- mouse: aim
- left mouse: fire
- right mouse or `Space`: dash
- `E`: interact / open chest

## Current Known Limitations

- this is still a vertical slice, not a full alpha
- UI/HUD is minimal and focused on objective clarity
- progression is local-file based, not backend-persisted
- the run uses one biome and one player archetype
- reconnect assumes the same logical server stays alive
- the local/dev playtest path is the primary supported path right now, not a full menu/lobby flow
- occasional render graph warning may appear for UI composite binding during startup

## Troubleshooting

### Materials still look grey

1. Ensure the client runs with:
   - `CAVERN_RENDER_MODE=material_graph`
   - `CAVERN_MATERIAL_PROFILE=balanced`
2. Check `logs/engine.log` for:
   - `cavern material runtime ready`
   - non-zero `graph_count` and `class_program_count`
3. Fix any `cavern material diagnostic` parse errors in material assets.
