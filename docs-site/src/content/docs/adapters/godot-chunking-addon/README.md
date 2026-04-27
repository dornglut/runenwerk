---
title: Godot Chunking Addon
description: Current documentation for the godot_chunking_addon adapter crate.
status: active
owner: adapter
layer: adapter/tool
canonical: true
last_reviewed: 2026-04-28
---

# Godot Chunking Addon

`godot_chunking_addon` is a Godot extension adapter for Runenwerk chunk streaming.

It bridges Godot-facing node APIs to the engine-agnostic `chunking` and `spatial` domain crates.

## Ownership

This crate belongs to the adapter/tool layer.

It may translate Godot types and lifecycle calls into Runenwerk domain contracts, but it must not own chunking invariants, spatial partitioning rules, world streaming policy, or simulation authority.

## Public Godot Node

The crate exposes a `ChunkStreamingNode`.

The node owns a `ChunkStreamer` instance and provides Godot-callable functions for configuring chunk partitioning and updating the streaming focus.

## Configuration Methods

The current node supports:

- `set_chunk_edge_meters(value)`
- `get_chunk_edge_meters()`
- `set_region_chunk_dims(x, y, z)`
- `set_fixed_point_scale(value)`
- `set_load_radii(load_radius_chunks, unload_radius_chunks, vertical_load_radius_chunks, vertical_unload_radius_chunks)`
- `set_planar_xz_mode()`
- `set_volume_3d_mode()`
- `clear_active_chunks()`
- `active_chunk_count()`
- `describe_config()`

## Streaming Focus

`update_focus_from_vector3(position)` converts a Godot `Vector3` into Runenwerk meter coordinates and updates the underlying `ChunkStreamer`.

The node emits signals for chunk-set changes:

- `chunk_entered(x, y, z)`
- `chunk_exited(x, y, z)`
- `active_chunk_count_changed(count)`

## Adapter Boundary

In scope:

- Godot `Vector3` to Runenwerk meter-coordinate conversion;
- Godot node configuration functions;
- signal emission for entered/exited chunks;
- construction of `GridPartitionConfig` and `ChunkStreamingConfig` from Godot-facing values.

Out of scope:

- owning chunk streaming invariants;
- owning spatial coordinate rules;
- engine runtime scheduling;
- persistence;
- networking;
- editor policy.

## Validation

Run:

```text
cargo check -p godot_chunking_addon
cargo check --workspace
```
