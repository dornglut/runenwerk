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

## What Is Still Narrow or Incomplete

### Gameplay scope

The authoritative replicated state is still the current scene-stub subset:

- debug position/velocity
- scene/world labels
- current gameplay config values
- frame/tick counters
- enemy kill count
- admitted session settings

This is enough for runtime proof and net-path validation, but not enough for the intended co-op action game yet.

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

1. Use `SessionRuntimeState` to configure the live run itself: max player count, AI fill target, and runtime settings application.
2. Expand the authoritative replicated state beyond the current scene-stub fields into actual gameplay entities/components.
3. Build the full client control-plane flow so `grotto_client` boots from restored Axiom auth/session/lobby state rather than env vars.
4. Add richer reconnect recovery and then move on to the next runtime profile work.

## Verification

The current workspace baseline has been exercised with:

- `cargo fmt`
- `cargo test --workspace`

At the time of this update, the full workspace test suite passed.
