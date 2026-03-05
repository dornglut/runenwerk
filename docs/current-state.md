# Current Project State

Updated: 2026-03-05

## Summary

The project is now on a single normalized runtime path:

- `ecs` is the runtime ECS
- `engine::App` is the runtime/app entry surface
- the old engine runtime split is gone from active code
- the first real multiplayer profile is `SimulationProfile::DedicatedAuthority`

The codebase is no longer in a migration state. The remaining work is feature expansion and production hardening, not replacing the runtime foundation again.

## Implemented Baseline

### Runtime and engine

- `engine::App`, `engine::Plugin`, typed schedules, typed params, and typed resources are the active runtime API
- scene, UI, render, input, time, grid, debug metrics, replay, and networking plugins all run on that path
- the preserved examples still compile and run on the same API surface

### Simulation and replay

- `engine_sim` owns canonical simulation contracts:
  - `SimulationTick`
  - `SimulationProfile`
  - `AuthorityRole`
  - `DeterminismLevel`
  - `SimulationSeed`
  - `SimulationRng`
- `engine_replay` owns:
  - checkpoints
  - journal frames
  - replay archives
  - replay validation
- the engine replay plugin records and validates authoritative scene simulation state

### Networking

- `engine_net` owns transport-agnostic protocol/session/replication state
- `engine_net_quic` owns the QUIC runtime path
- the current live profile supports:
  - client/server handshake
  - long-lived QUIC runtime tasks
  - reconnect
  - staged hybrid replication (V2):
    - periodic keyframes (`cavern_hunt.keyframe.v2`)
    - in-place patch stream (`cavern_hunt.patch.v2`)
    - adaptive load-shed levels with per-channel budget/cadence controls
  - client-side authoritative apply and adaptive correction/smoothing

### Axiom integration

- `grotto_online` can:
  - request join grants
  - validate join grants
  - consume join tickets
  - map consumed admission data into runtime-facing join state
- `grotto_online` now also provides an operator WebSocket bridge contract for runtime commands, snapshots, and event streaming
- `grotto_server` now supports opt-in operator runtime control:
  - drain mode
  - targeted connection disconnect
  - shutdown orchestration and structured runtime snapshots/events
- `grotto_fleet_control` now runs as a separate lifecycle/log control process with a Kubernetes provider and Axiom WebSocket command bridge
  - `stop_server` issues runtime drain+shutdown dispatch first, then Kubernetes force-stop fallback
  - bridge URLs now target Axiom `/v2/operator/runtime/ws` and `/v2/operator/fleet/ws`
  - runtime/fleet bridge config can be overridden via env/CLI for local ops workflows
- full-stack local Kubernetes operator demo assets now exist:
  - Helm chart: `ops/helm/multiplayer-stack`
  - bootstrap scripts: `scripts/k8s/bootstrap_local_stack.sh`, `scripts/k8s/bootstrap_operator_bridges.sh`
  - smoke script: `scripts/k8s/smoke_operator_flow.sh`
- `grotto_client` and `grotto_server` now load multiplayer/network settings from versioned `.ron` assets
- local/dev fallback paths still exist when Axiom handoff is disabled in config

## Current Runtime State Model

The important public runtime state now includes:

- `SceneRuntimeState`
- `GameplayRuntimeConfig`
- `UiOverlayState`
- `SessionRuntimeState`
- network-facing state such as:
  - `NetworkSessionStatus`
  - `NetworkAdmissionState`
  - `ConnectionHealth`
  - `RoundTripMetrics`

`SessionRuntimeState` is the transport-neutral match/session view of admission data. It is derived from the accepted join state and pushed into the authoritative scene simulation context so replay/reconnect keep the same admitted roster and session settings.

## What Is Solid

- runtime/ECS foundation
- deterministic fixed-step execution
- scene replay/checkpoint capture and restore
- dedicated-authority session bootstrap
- live QUIC runtime handles
- reconnect baseline reset
- V2 keyframe+patch replication path (`NetSyncModeConfig::V2` in client/server networking assets)
- adaptive per-channel patch cadence and op-budget load shedding from server networking profiles
- tunable client reconciliation thresholds from client networking profiles
- Axiom handoff contracts plus optional live HTTP verification
- Axiom operator runtime bridge path for in-process server control and observability (opt-in)
- fleet lifecycle/log control service baseline with Kubernetes-backed implementation
- `Cavern Hunt` friend-test vertical slice:
  - procedural cavern generation
  - fixed-camera SDF 3D rendering
  - asset-driven material graph runtime (`RON` graphs + profile presets)
  - triplanar procedural floor/wall differentiation
  - PBR-lite shading and GI mode switches
  - runtime material diagnostics via tracing (`logs/engine.log`)
  - live multi-client dedicated-authority play
  - reconnect inside active runs
  - AI fill companions
  - elite/extraction success flow
  - local client reward persistence

## What Is Still Narrow or Incomplete

### Gameplay scope

The generic runtime path is still intentionally narrow, but `Cavern Hunt` now owns a game-specific replicated slice on top of it:

- player transforms, aim, dash, and projectile state
- enemy pressure and elite objective state
- loot, extraction, and local reward flow
- admitted roster identity, AI fill, and session-derived spawn policy

This is enough for a friend-testable vertical slice, but not yet enough for a larger production content set.

### Rendering/material maturity gaps

- material graphs are asset-authored only; there is no editor yet
- `ProbeGi` is scaffolded as a mode but does not yet have full probe update/population logic
- normal perturb output exists in graph/schema but is not yet fully integrated into the SDF normal path
- UI composite pass binding warning still appears in some runs (`ui_composite->builtin_ui_composite`) and should be cleaned up in render graph wiring

### Multiplayer maturity gaps

- V2 patching is in-place for player/enemy/projectile/pickup/extraction channels with periodic keyframe recovery; AOI/relevancy is still missing
- no AOI/relevancy model yet
- no shard/zone partitioning
- reconnect still assumes the same logical host rather than reallocation to a new host
- Axiom consume metadata is now retained, but the runtime still needs deeper gameplay/session orchestration on top of it
- lifecycle service deployment/inventory wiring is not yet integrated into production ops automation

### Profile maturity

Implemented first:

- `DedicatedAuthority`

Defined but not yet implemented as full runtime profiles:

- `RollbackSession`
- `DeterministicLockstep`
- `HighThroughputAuthority`

## Recommended Next Steps

1. Polish Cavern Hunt combat feel, HUD clarity, and encounter pacing for friend tests.
2. Keep the 2-local smoothness target green, then expand the 4-local best-effort path under reconnect.
3. Build the full client control-plane flow so `grotto_client` boots from restored Axiom auth/session/lobby state rather than static local config.
4. After Cavern Hunt feels stable, resume broader profile work and richer gameplay replication.

## Verification

The current baseline has been validated with:

- `cargo check --workspace`
- `cargo check -p grotto_client -p grotto_server -p cavern_hunt`
- `cargo test -p grotto_online`
- `cargo test -p engine_net_quic`
- `cargo test -p cavern_hunt net_config`
- `cargo test -p grotto_fleet_control`
- `cargo test -p cavern_hunt --lib`
- `cargo test -p cavern_hunt net_sync -- --nocapture`

All of the above passed on `2026-03-05`.
