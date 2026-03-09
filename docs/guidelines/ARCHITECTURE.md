# Architecture

This document defines the active crate/domain boundaries in `grotto-quest` and where new code belongs.

## Top-Level Domains

- `foundation/`: low-level reusable runtime primitives (`ecs`, `ecs_macros`, `grid`, `scheduler`)
- `engine/`: runtime loop, plugin system, rendering, input, scene, time, UI integration
- `net/`: transport/session/replication/replay infrastructure (`engine_net`, `engine_net_quic`, `engine_sim`, `engine_replay`)
- `games/`: game-owned simulation/content logic (`cavern_hunt`)
- `apps/`: runnable binaries and service integration (`grotto_client`, `grotto_server`, `grotto_online`, `grotto_fleet_control`)
- `assets/`: data assets consumed by engine/games/apps
- `docs/`: architecture and operational documentation
- `ops/`: deployment/runtime ops definitions (Docker, fleet, Helm)

## Dependency Direction

Keep dependency flow unidirectional:

- `foundation` -> no project-internal dependency on higher domains
- `engine` -> `foundation`
- `net` -> `foundation` (and self-contained net crates)
- `games` -> `engine` + `foundation` + `net` contracts as needed
- `apps` -> `games` + `engine` + `net` + `foundation`

Avoid sideways coupling between app crates or between game crates via private internals.

## Ownership Boundaries

- `foundation/*` owns core data structures and execution primitives.
- `engine` owns runtime composition and plugin integration points.
- `net/*` owns protocol/session/transport contracts and replay storage/runtime integration.
- `games/cavern_hunt` owns gameplay rules, replication mapping, and game content behavior.
- `apps/*` owns process wiring, config loading, and external system integration.

If logic is game-specific, keep it in `games/`. If it is engine-generic, keep it in `engine` or `foundation`.

## Placement Rules

When adding code:

1. Choose the owning domain first (`foundation`, `engine`, `net`, `games`, or `apps`).
2. Reuse local helpers in that domain before adding new abstractions.
3. Expose narrow public interfaces instead of reaching into internals across crates.
4. Add or update local `README.md` and `requests.md` when behavior or scope changes.

## Architecture Guardrails

- Prefer explicit types, deterministic control flow, and clear ownership.
- Do not add silent failure paths or broad catch-all error handling.
- Do not move code across domains unless the ownership boundary itself is changing.
- Keep docs and crate boundaries aligned with `Cargo.toml` workspace members.

See also:

- `AGENTS.md` for agent behavior rules.
- `DOMAIN_MAP.md` for crate-level ownership and dependency summary.
- `CODE_PATTERNS.md` for implementation patterns used across domains.
