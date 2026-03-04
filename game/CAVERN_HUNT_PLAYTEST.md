# Cavern Hunt Playtest Guide

Updated: 2026-03-04

## Goal

This is the current friend-test path for `Cavern Hunt`.

The local/dev flow does not require Axiom. It uses:

- one local dedicated server
- 1 to 4 local clients
- the dev QUIC certificate written by the server

The optional Axiom-backed path is still supported for advanced testing.

## Default Local Flow

### 1. Start the dedicated server

```bash
scripts/run_cavern_server.sh
```

Expected behavior:

- the server binds to `127.0.0.1:7000`
- it writes a dev certificate to `var/dev/server-cert.der`
- it keeps running until `Ctrl-C`

### 2. Start one client

```bash
scripts/run_cavern_client.sh
```

### 3. Start multiple clients

Run the client script once per client, or use:

```bash
scripts/run_cavern_party_local.sh 4
```

That starts:

- one local server
- four local clients

Client and server logs are written to:

- `/tmp/cavern_hunt_server.log`
- `/tmp/cavern_hunt_client_1.log`
- `/tmp/cavern_hunt_client_2.log`
- `/tmp/cavern_hunt_client_3.log`
- `/tmp/cavern_hunt_client_4.log`

## Local/Dev Environment Variables

These are optional for the local fallback path:

- `GROTTO_SERVER_ID`
  - default: `srv-local`
- `GROTTO_SERVER_NAME`
  - default: `localhost`
- `GROTTO_SERVER_ENDPOINT`
  - default: `127.0.0.1:7000`
- `GROTTO_JOIN_TICKET`
  - default: `local-ticket`
- `GROTTO_SERVER_CERT_PATH`
  - default: `var/dev/server-cert.der`
- `AXIOM_API_BASE_URL`
  - default: `http://api.localhost`
- `AXIOM_DEVICE_ID`
  - client-only device identifier for local/Axiom testing

## Optional Axiom-Backed Flow

The client can request a live join grant instead of using the local fallback target when these are set:

- `AXIOM_API_BASE_URL`
- `AXIOM_LOBBY_ID`
- `AXIOM_ACCESS_TOKEN` or `AXIOM_REFRESH_TOKEN`
- `AXIOM_DEVICE_ID`

The server can verify join tickets against Axiom when this is set:

- `DEDICATED_SERVER_SHARED_SECRET`

If those are missing, the local fallback path stays active.

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
