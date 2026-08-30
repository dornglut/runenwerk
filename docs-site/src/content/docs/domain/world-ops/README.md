---
title: World Ops
description: Current crate documentation for the world_ops domain crate.
status: active
owner: world-ops
layer: domain
canonical: true
last_reviewed: 2026-08-30
---

# World Ops

`world_ops` owns edit operation records, dirty-region tracking, build queue
contracts, replay windows, and replication deltas for chunked world data.

## Purpose

Use this crate when world changes need to be recorded, invalidated, rebuilt, or
replicated without depending on engine runtime glue.

## Public Surface

- `Operation`, `CsgBrushOperation`, `CsgBooleanMode`, `OperationRecord`, and
  `OperationId`: authored world edit records, including accepted P1 CSG brush
  semantics for add, subtract, intersect, and smooth boolean modes.
- `WorldQuantizationScale`, `QuantizedVec3`, `QuantizedAabb`,
  `quantize_position`, and `quantize_aabb`: explicit Runenwerk world-operation
  quantization policy and lowering vocabulary.
- `OperationLog` and `ReplayWindow`: append/read windows for edit history.
- `DirtyChunkMap`, `DirtyReason`, `DirtyReasonSet`, and
  `dirty_reason_for_operation`: invalidation state and operation-kind dirty
  classification.
- `RegionInvalidationJournal` and related records.
- `BuildGraph`, `BuildQueue`, and build generation/revision types.
- Replication deltas such as `OpWindowDelta`, `ChunkContentDelta`, and
  `RegionInvalidationDelta`.

## Ownership Boundary

`world_ops` owns operation, quantization, and invalidation semantics. Reusable
spatial identity, addressing, and partition mechanics come from RunenSpatial.
`world_ops` does not own concrete SDF brick storage, renderer upload, network
transport, editor command UI, or ECS resource scheduling.

## Related Crates

- `runen-spatial` supplies world-qualified spatial identities and checked
  partition/addressing mechanics used by invalidation.
- `world_sdf` stores and serves SDF chunk/page payloads.
- `engine` consumes build/dirty state through plugins and schedules.
