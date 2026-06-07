---
title: World Streaming
description: Current crate documentation for Runenwerk's payload-neutral world streaming lifecycle wrapper.
status: active
owner: world-streaming
layer: domain
canonical: true
last_reviewed: 2026-06-07
---

# World Streaming

`world_streaming` is Runenwerk's domain wrapper around the reusable
`Crystonix/spatial_streaming` request/event lifecycle core.

## Purpose

Use this crate when code needs payload-neutral chunk lifecycle control:

- desired chunk state becomes stream requests;
- hosts perform provider work;
- provider events return to the controller;
- deterministic lifecycle events describe state transitions.

## Public Surface

The crate re-exports the external `world_streaming` core:

- `WorldStreamingController`
- `WorldStreamingConfig`
- `StreamingTick`
- `StreamingTickOutput`
- `StreamRequest`
- `StreamRequestId`
- `StreamRequestKind`
- `ProviderEvent`
- `ProviderEventKind`
- `WorldStreamingEvent`
- `WorldStreamingEventKind`
- `ChunkLifecycleState`
- `ChunkRuntimeRecord`
- `StreamingBudgets`
- `ChunkPriority`

## Ownership Boundary

`world_streaming` owns lifecycle truth only. It does not own:

- SDF payloads;
- ECS spawning or engine resources;
- renderer resources;
- asset catalogs;
- save formats;
- mesh generation;
- procgen documents;
- Godot nodes;
- Runenwerk product semantics.

Runenwerk keeps those meanings in their current owners:

- `world_ops` owns edits, dirty regions, build queues, and replication deltas.
- `world_sdf` owns SDF chunk/page payloads and collision-ready summaries.
- `product` owns formed product descriptors and publication contracts.
- `procgen` owns procedural documents and lowering policy.
- `engine` owns runtime scheduling and resource composition.

## Current Reconnect Scope

This M17 reconnect adds the wrapper and documentation only. It does not replace
`engine/src/plugins/world/chunks/lifecycle.rs` yet.

The engine chunk runtime still tracks Runenwerk-specific dirty/build/revision
state. Future integration must use a strangler migration:

1. route one host/provider proof through `WorldStreamingController`;
2. translate lifecycle events into existing engine runtime records;
3. prove parity for streaming request order and unload cleanup;
4. only then remove or narrow the old engine lifecycle path.

## Related Crates

- `spatial` supplies world and chunk coordinate contracts.
- `chunking` computes desired residency.
- `world_ops` owns dirty/build/replication semantics.
- `world_sdf` owns SDF payload data.
