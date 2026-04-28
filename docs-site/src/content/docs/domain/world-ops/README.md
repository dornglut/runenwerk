---
title: World Ops
description: Current crate documentation for the world_ops domain crate.
status: active
owner: world-ops
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# World Ops

`world_ops` owns edit operation records, dirty-region tracking, build queue
contracts, replay windows, and replication deltas for chunked world data.

## Purpose

Use this crate when world changes need to be recorded, invalidated, rebuilt, or
replicated without depending on engine runtime glue.

## Public Surface

- `Operation`, `OperationRecord`, `OperationId`: authored world edit records.
- `QuantizedVec3`, `QuantizedAabb`, `quantize_position`, `quantize_aabb`.
- `OperationLog` and `ReplayWindow`: append/read windows for edit history.
- `DirtyChunkMap`, `DirtyReason`, `DirtyReasonSet`: invalidation state.
- `RegionInvalidationJournal` and related records.
- `BuildGraph`, `BuildQueue`, and build generation/revision types.
- Replication deltas such as `OpWindowDelta`, `ChunkContentDelta`, and
  `RegionInvalidationDelta`.

## Ownership Boundary

`world_ops` owns operation and invalidation semantics. It does not own concrete
SDF brick storage, renderer upload, network transport, editor command UI, or ECS
resource scheduling.

## Related Crates

- `world_sdf` stores and serves SDF chunk/page payloads.
- `chunking` decides desired residency.
- `engine` consumes build/dirty state through plugins and schedules.
