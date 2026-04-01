---
title: "World Runtime Final Architecture Migration"
description: "Documentation for World Runtime Final Architecture Migration."
---

# World Runtime Final Architecture Migration

Owner: `engine/src/plugins/world`

Status: in-progress, dual-write enabled in Cavern Hunt, now in correctness-hardening phase.

## Phases

1. World plugin skeleton + ownership boundary cut.
2. Typed op-log + dirty invalidation with dual-write from legacy geometry edits.
3. Build queue/graph and completed-output integration.
4. Render world/detail/cave prepared feature payload handoff and runtime cache wiring.
5. Per-connection replication snapshot capture + world chunk/op streaming schema.
6. Gameplay query migration from legacy geometry/collision to world query services.
7. Legacy authority removal and guard tests.

## Current Delivered

- `WorldPlugin` registered and initialized from Cavern Hunt plugin wiring.
- New world subdomain modules landed (`ids`, `chunks`, `sdf`, `edits`, `build`, `streaming`, `caves`, `prepare`, `debug`).
- Geometry edit path and worldgen init now dual-write into `WorldOperationLog` and mark dirty chunks.
- Net replication driver now supports `capture_snapshot_for_connection(...)` and server state keeps per-connection snapshot history.
- Render feature contracts expanded with world/cave/detail/procedural/wind contribution resources.
- Fixed-step world maintenance cadence and explicit world build pipeline ordering are active.
- Dirty-chunk lifecycle now bootstraps missing runtime records from dirty map state.
- World op ingress is now centralized through world plugin edit-ingress APIs.

## Active Correctness Gate (Current Focus)

1. Keep authoritative collision behavior explicit when chunk payload is unavailable:
   - no silent gameplay "free pass" on missing chunk payload data
   - clear behavior once chunk payload exists, even before full SDF semantics are finalized
2. Ensure world authority revision progresses from integrated world build outputs.
3. Expand world runtime tests from source-cutoff guards to end-to-end behavior checks.

## Remaining Cutover Work

- Replace remaining gameplay/query callsites with first-class `plugins::world::queries` contracts.
- Move chunk/op-window replication from snapshot helpers into world runtime-driven streaming systems.
- Build typed world/render contribution identity contracts (replace string IDs in prepared payloads).
- Add bounds-driven world->render runtime-cache invalidation bridge.
- Retire legacy `CavernGeometryGraph` and `CavernCollisionField` as runtime authority after world query parity is complete.
