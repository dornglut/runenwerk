# Current Project State

Updated: 2026-03-04

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
  - full snapshot bootstrap
  - real scene delta snapshots
  - client-side authoritative apply and first-pass prediction correction

### Axiom integration

- `grotto_online` can:
  - request join grants
  - validate join grants
  - consume join tickets
  - map consumed admission data into runtime-facing join state
- `grotto_client` and `grotto_server` can use those live hooks when the expected env vars are present
- local/dev fallback paths still exist when Axiom config is absent

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
- scene snapshot replication and delta application
- Axiom handoff contracts plus optional live HTTP verification
- `Cavern Hunt` friend-test vertical slice:
  - procedural cavern generation
  - fixed-camera SDF 3D rendering
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

### Multiplayer maturity gaps

- no full combat replication model yet
- no AOI/relevancy model yet
- no shard/zone partitioning
- reconnect still assumes the same logical host rather than reallocation to a new host
- Axiom consume metadata is now retained, but the runtime still needs deeper gameplay/session orchestration on top of it

### Profile maturity

Implemented first:

- `DedicatedAuthority`

Defined but not yet implemented as full runtime profiles:

- `RollbackSession`
- `DeterministicLockstep`
- `HighThroughputAuthority`

## Recommended Next Steps

1. Polish Cavern Hunt combat feel, HUD clarity, and encounter pacing for friend tests.
2. Expand the live playtest flow to the documented 4-player local/dev path and keep it green under reconnect.
3. Build the full client control-plane flow so `grotto_client` boots from restored Axiom auth/session/lobby state rather than env vars.
4. After Cavern Hunt feels stable, resume broader profile work and richer gameplay replication.

## Verification

The current workspace baseline has been exercised with:

- `cargo fmt`
- `cargo test --workspace`

At the time of this update, the full workspace test suite passed.
