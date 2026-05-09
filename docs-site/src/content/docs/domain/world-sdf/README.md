---
title: World SDF
description: Current crate documentation for the world_sdf domain crate.
status: active
owner: world-sdf
layer: domain
canonical: true
last_reviewed: 2026-05-09
---

# World SDF

`world_sdf` owns chunk/page SDF payload records, CPU field-preview product
payload DTOs, cave summary data, hierarchy summaries, and collision query
contracts for world-scale SDF data.

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
- `FieldPreviewGrid`, `FieldPreviewPayload`, `FieldPreviewProduct`, and
  `ratify_field_preview_product` for CPU-formed scalar distance, vector
  gradient, occupancy, and material-channel preview products.

## Ownership Boundary

`world_sdf` owns SDF payload structure, field-preview payload structure, and
domain collision query contracts. It does not own edit operation logs, streaming
decisions, renderer resource upload, GPU texture handles, or app/editor
commands.

## Related Crates

- `sdf` owns analytic field math and sampling contracts.
- `world_ops` owns edit invalidation and build queues.
- `spatial` supplies chunk/page coordinate vocabulary.
