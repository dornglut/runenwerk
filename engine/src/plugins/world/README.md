# World Plugin

`engine/src/plugins/world` owns the authoritative world runtime model used by server and client ECS/domain layers.

## Ownership

- World truth contract: chunked/regioned revisioned SDF state + typed operation log.
- ECS/domain owns:
  - chunk lifecycle state
  - dirty propagation
  - rebuild queue/graph
  - operation-log replay and invalidation
  - prepared world feature payload publication
- Renderer owns:
  - runtime residency and GPU caches
  - world/detial/cave runtime cache materialization from prepared payloads only

## Submodules

- `ids/`: stable chunk/region/revision/op identifiers.
- `frames/`: planet-local and camera-relative frame resources.
- `chunks/`: partitioning, lifecycle, dirty reason tracking.
- `sdf/`: authoritative sparse brick/page payload types and hierarchy summaries.
- `edits/`: typed op schema, append-only op-log, replay/invalidation entry points.
- `build/`: dirty queue, phase graph, job dispatch, completed output integration.
- `queries/`: gameplay query service contracts over authoritative/derived caches.
- `streaming/`: per-connection interest and replication payload contracts.
- `caves/`: sector/portal/light-scope summary resources.
- `prepare/`: world payload export into render prepared resources.
- `debug/`: counters and diagnostics resources for inspect surfaces.

## Current Migration Status

- World plugin is active and wired into Cavern Hunt in dual-write mode.
- Legacy `CavernGeometryGraph` / `CavernCollisionField` remain for compatibility while gameplay and snapshot callsites are migrated to world query/checkpoint APIs.
