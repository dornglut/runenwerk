# Grotto Quest Workspace

This workspace contains the current Grotto Quest runtime, networking, replay, and game-host binaries.

## Current State

As of 2026-03-05, the active runtime baseline is:

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
- Dedicated-authority session bootstrap, live QUIC runtime tasks, reconnect handling, and staged hybrid replication (`keyframe.v2` + `patch.v2`)
- Optional Axiom-backed join-grant issue/consume integration for client admission and server verification
- Optional Axiom operator runtime bridge (`/v2/operator/runtime/ws`) and fleet lifecycle/log bridge (`/v2/operator/fleet/ws`)
- `Cavern Hunt` as the current game vertical slice, including:
  - deterministic cavern generation
  - fixed-camera SDF 3D rendering
  - asset-driven material graph shading (`RON`) with triplanar procedural surfaces
  - PBR-lite lighting and staged GI modes (`off`, `AO+bent normal`, probe scaffold)
  - client/server tracing enabled by default with material diagnostics in `logs/engine.log`
  - 1-4 player dedicated-authority live sessions
  - AI fill companions
  - elite -> extraction run flow
  - local meta reward persistence
  - local/dev playtest scripts in `scripts/`

## Main Gaps

- The replicated gameplay state is still a vertical-slice subset, not a complete production combat model
- Hybrid replication V2 currently patches players in-place and still uses coarse vector channels for enemies/projectiles/pickups
- `RollbackSession`, `DeterministicLockstep`, and `HighThroughputAuthority` are planned profiles, not implemented production profiles
- `grotto_client`/`grotto_server` now bootstrap from `.ron` network assets, but still lack a full in-game lobby/menu flow

## Entry Points

- Project status: [docs/current-state.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/current-state.md)
- Engine overview: [engine/README.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/engine/README.md)
- Documentation index: [docs/index.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/index.md)
- Local Kubernetes operator runbook: [docs/kubernetes-local-operator-demo.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/kubernetes-local-operator-demo.md)
- Cavern Hunt GDD: [game/CAVERN_HUNT_GDD.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_GDD.md)
- Cavern Hunt playtest guide: [game/CAVERN_HUNT_PLAYTEST.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/game/CAVERN_HUNT_PLAYTEST.md)
- Operator/fleet architecture map: [docs/axiom-operator-console.md](/Users/joshua/Projekte/multiplayer_workspace/grotto-quest/docs/axiom-operator-console.md)
