---
title: Chunking
description: Current crate documentation for the chunking domain crate.
status: active
owner: chunking
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# Chunking

`chunking` owns engine-agnostic chunk residency planning around one or more
streaming focuses.

## Purpose

Use this crate to compute which chunks should be loaded, retained, or unloaded
from a current focus and streaming policy.

## Public Surface

- `ChunkStreamingConfig`: radius and budget configuration.
- `StreamingFocus`: position/priority input for desired residency.
- `ChunkStreamingMode` and `ChunkLoadOrder`: policy knobs.
- `ChunkSet`: current or desired chunk membership.
- `ChunkSetDiff`: added/removed/stable chunk differences.
- `ChunkStreamer`: stateful planner that computes chunk set transitions.

## Ownership Boundary

`chunking` owns desired residency math. It does not own IO, asset loading, SDF
generation, ECS scheduling, render submission, or app-specific streaming UX.

## Related Crates

- `spatial` supplies chunk coordinate contracts.
- `world_ops` turns edits and build results into dirty/build work.
- `world_sdf` owns SDF payload storage for resident chunks.
