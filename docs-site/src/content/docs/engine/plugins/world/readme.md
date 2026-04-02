---
title: "World Plugin"
description: "Documentation for World Plugin."
---

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

- World plugin is active and wired into Cavern Hunt as the single authoritative runtime world path.
- Runtime gameplay/worldgen/snapshot paths no longer initialize, mutate, or query legacy `CavernCollisionField`/`CavernGeometryGraph` authority resources.
- Snapshot restore is world-checkpoint-authoritative and no longer inserts legacy runtime geometry resources.
- Active net snapshot wire contract is `CavernRunSnapshotV2`/`CavernRunDeltaV2`; legacy V1 payloads are explicitly rejected.
- Runtime worldgen geometry edits feed only world ingress/build integration authority paths.
- Material GI scaffold now tracks authority revision only from world runtime state (`WorldAuthorityState`) without geometry-graph fallback.
- Legacy `domain/world/collision_field` has been removed from Cavern domain world surface.
- Cavern resource marker wiring no longer registers legacy geometry/collision authority resources as ECS runtime resource components.
- Worldgen runtime init now uses topology bounds for initial world ingress seed stamping and shape-based ingress edits for extraction seal add/remove.
- Build payload dispatch now uses deterministic op-window-derived chunk payload generation for the active runtime path (replacing placeholder payload generation).
- Fixed-step gameplay systems in Cavern Hunt are explicitly ordered after world integration and before replication.

## Runtime Authority Notes

- World runtime mode is synchronized from simulation authority (`Local`/`Server`/`Peer` => `ServerAuthoritative`, `Client` => `ClientReplica`) and mirrored by authority-role configuration entry points.
- World edit ingress enforces runtime authority mode: `submit_world_operation(...)` returns no-op in `ClientReplica` mode and does not mutate op-log/dirty state.
- Partition ownership is centralized in `WorldPartitionConfig`, including fixed-point quantization scale; `WorldRuntimeConfig` now focuses on runtime mode only.
- Integration semantics:
  - accepted output increments world revision
  - stale/mismatched outputs are dropped without revision advancement
  - dirty reasons that arrive during rebuild are preserved for follow-up dispatch
  - integration rejects malformed payload contracts (chunk-id/revision/generation mismatch) before publishing authoritative state
  - op-window payload application preserves topology across non-topology edits (`Smooth`, `MaterialFieldEdit`, `DensityFieldDeform`) and treats `Stamp` as additive authority content
- Collision authority semantics:
  - missing chunk payload is explicit readiness failure
  - sweep readiness validates all chunk payloads required by the swept bounds, not only the end chunk
  - world collision query service exposes a single authoritative sweep outcome contract (`MissingPayload`/`Hit`/`Clear`) so gameplay does not split readiness vs hit logic ad hoc
  - gameplay uses fail-safe blocking behavior on missing authority payload
  - metrics track collision queries and authority misses
- Render invalidation bridge semantics:
  - ingress and build integration publish world->render invalidation records
  - records include chunk-bounds and derived chunk/region sets
  - stale marking dedupes deterministically by chunk during bridge flush
  - bridge queues remain durable while render cache resources are absent and flush once resources return
- Typed world contribution identity semantics:
  - world feature chunk/residency payloads use typed `ChunkId` identities
  - draw-batch linkage for world chunks is represented by typed refs instead of string labels
  - world prepare no longer builds string chunk labels for core world feature identity
- Streaming/replication runtime semantics:
  - world runtime rebuilds replication header/content/op-window/residency state each fixed step
  - world runtime projects deterministic region/chunk invalidation deltas from a world-owned region journal each fixed step
  - downstream snapshot adapters read this resource instead of reconstructing world deltas ad hoc
  - per-connection streaming interest is synchronized from active server sessions + authoritative runtime chunks
  - net replication snapshot send/ack paths now advance world-owned streaming cursors and resync flags
  - per-connection shaping consumes acknowledged region sequence coverage and emits selective checkpoint chunk/op payloads from relevant chunk sets
  - disconnect paths remove stale per-connection streaming entries from world-owned state
- Lightweight world observability:
  - world runtime inspector snapshots now include streaming connections/resync counts, max cursor lag, max region-sequence lag, and region-journal latest sequence + retained count
  - debug overlay surfaces these values directly for rapid authority/streaming troubleshooting
