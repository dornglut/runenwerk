---
title: Spatial
description: Current crate documentation for the spatial domain crate.
status: active
owner: spatial
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# Spatial

`spatial` owns engine-agnostic world-space coordinate contracts and mapping
helpers for chunked, ring-buffered, and clipmap-oriented world systems.

## Purpose

Use this crate when code needs stable vocabulary for:

- world identity and position value types;
- camera-relative frame construction;
- chunk, region, and hierarchy coordinates;
- clipmap level/window mapping;
- ring-buffer slot mapping for reusable spatial storage.

## Public Surface

- `frames`: `WorldFrame`, `CameraRelativeFrame`, `build_camera_relative_frame`.
- `positions`: `WorldPosition`, `WorldLocalPosition`.
- `ids`: `WorldId`.
- `grid`: `ChunkCoord3`, `RegionCoord3`, ids, partition config, and hierarchy config.
- `clipmap`: `ClipmapConfig`, `ClipmapLevel`, `ClipmapCoord3`, `ClipmapWindow`.
- `ring`: `RingBufferConfig`, `RingSlot3`, `slot_for_coord`.

## Ownership Boundary

`spatial` owns coordinate meaning and deterministic mapping rules. It does not
own residency decisions, streaming policy, runtime storage, ECS resources, SDF
payloads, rendering, or editor behavior.

## Related Crates

- `spatial_index` stores/query objects by spatial keys.
- `chunking` decides desired chunk residency around a focus.
- `world_ops` tracks edits, dirty regions, build queues, and replication deltas.
- `world_sdf` stores chunk/page SDF payloads and collision-ready summaries.
