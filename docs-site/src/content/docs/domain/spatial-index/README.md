---
title: Spatial Index
description: Current crate documentation for the spatial_index domain crate.
status: active
owner: spatial-index
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# Spatial Index

`spatial_index` owns engine-agnostic indexing contracts for storing entries by
spatial keys and querying them by bounds.

## Purpose

Use this crate when a domain system needs mutable spatial registration and
read-only spatial queries without depending on engine runtime state.

## Public Surface

- `SpatialEntry`: stored object id plus indexed bounds.
- `SpatialKey`: stable key used by index implementations.
- `AabbQuery` and `QueryResult`: query input/output contracts.
- `SpatialIndex` and `MutableSpatialIndex`: read/write traits for index users.
- `SpatialHashIndex` and `SpatialHashConfig`: current concrete index.
- `SpatialIndexMapStorage`: map-backed storage used by the hash index.
- `SpatialIndexError`: explicit mutation/query error type.

## Ownership Boundary

`spatial_index` owns indexing behavior and query contracts. It does not own
chunk streaming policy, collision response, rendering culling policy, ECS
resources, or world edit invalidation.

## Related Crates

- `spatial` supplies world/chunk coordinate vocabulary.
- `geometry` supplies explicit bounds such as `Aabb3`.
- `chunking` and `world_ops` may consume index results but own their policies.
