# World Runtime Final Architecture Migration

Owner: `engine/src/plugins/world`

Status: in-progress, dual-write enabled in Cavern Hunt.

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

## Remaining Cutover Work

- Replace gameplay collision/pathing callsites with `plugins::world::queries`.
- Snapshot/save/load migration from full graph payloads to checkpoint + op-log windows.
- World chunk streaming payload schema integration into Cavern network snapshot/delta payloads.
- Retire legacy `CavernGeometryGraph` and `CavernCollisionField` as runtime authority.
