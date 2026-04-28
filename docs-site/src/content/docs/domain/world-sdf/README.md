---
title: World SDF
description: Current crate documentation for the world_sdf domain crate.
status: active
owner: world-sdf
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# World SDF

`world_sdf` owns chunk/page SDF payload records, cave summary data, hierarchy
summaries, and collision query contracts for world-scale SDF data.

## Purpose

Use this crate when a system needs domain-level SDF storage or collision-ready
summaries for chunked world data.

## Public Surface

- `SdfChunkStore`, `SdfChunkPayload`, `SdfPageRecord`, `SdfBrickRecord`.
- `SDF_PAGE_EDGE_BRICKS`, `SDF_BRICK_EDGE_SAMPLES`, and brick/page metadata.
- `CaveSectorStore`, `CavePortalGraph`, cave light/volume scope summaries.
- `ChunkHierarchyNode` and `ChunkHierarchySummary`.
- `CollisionQueryService`, `SphereSweep`, `CollisionHit`, and readiness/result
  types.

## Ownership Boundary

`world_sdf` owns SDF payload structure and domain collision query contracts. It
does not own edit operation logs, streaming decisions, renderer resource upload,
or app/editor commands.

## Related Crates

- `sdf` owns analytic field math and sampling contracts.
- `world_ops` owns edit invalidation and build queues.
- `spatial` supplies chunk/page coordinate vocabulary.
