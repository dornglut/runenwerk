# Grotto Quest Workspace

This workspace contains the current Grotto Quest runtime, networking, replay, and game-host binaries.

## Current State

As of 2026-03-04, the active runtime baseline is:

- `ecs` as the only ECS crate used by the engine/runtime path
- `engine::App` as the only engine runtime entry surface
- `engine_sim` for shared simulation contracts
- `engine_replay` for checkpoints, journals, and replay archives
- `engine_net` plus `engine_net_quic` for the dedicated-authority networking path
- `grotto_online` for Axiom control-plane and handoff integration
- `grotto_client` and `grotto_server` as the current dedicated client/server binaries

The runtime migration is complete enough that the current path should be treated as the normal engine, not a compatibility branch.

## What Works Now

- Fixed-step runtime with `Startup`, `PreUpdate`, `FixedUpdate`, `Update`, `RenderPrepare`, `RenderSubmit`, and `FrameEnd`
- Scene, UI, render, input, time, grid, debug-metrics, and replay plugins on the active runtime path
- Replay/checkpoint recording and seek/validation against authoritative scene snapshots
- Dedicated-authority session bootstrap, live QUIC runtime tasks, reconnect handling, and scene snapshot replication
- Optional Axiom-backed join-grant issue/consume integration for client admission and server verification
- `Cavern Hunt` as the current game vertical slice, including:
  - deterministic cavern generation
  - fixed-camera SDF 3D rendering
  - asset-driven material graph shading (`RON`) with triplanar procedural surfaces
  - PBR-lite lighting and staged GI modes (`off`, `AO+bent normal`, probe scaffold)
  - 1-4 player dedicated-authority live sessions
  - AI fill companions
  - elite -> extraction run flow
  - local meta reward persistence
  - local/dev playtest scripts in `scripts/`

## Main Gaps

- The replicated gameplay state is still the narrow scene-stub subset, not a full combat/gameplay model
- Delta snapshots exist, but replication granularity is still coarse and scene-specific
- `RollbackSession`, `DeterministicLockstep`, and `HighThroughputAuthority` are planned profiles, not implemented production profiles
- `grotto_client` still relies on env-driven control-plane bootstrap rather than a full in-game lobby/menu flow

## Entry Points

- Project status: [docs/current-state.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/current-state.md)
- Engine overview: [engine/README.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/README.md)
- Documentation index: [docs/index.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/index.md)
- Cavern Hunt GDD: [game/CAVERN_HUNT_GDD.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_GDD.md)
- Cavern Hunt playtest guide: [game/CAVERN_HUNT_PLAYTEST.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_PLAYTEST.md)
