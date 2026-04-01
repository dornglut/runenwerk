---
title: "World Overview"
description: "Overview and documentation map for World."
---

# World Module

`engine/src/plugins/world` owns authoritative runtime world state and world-to-render/world-to-net integration contracts.

## Purpose

- Maintain chunked, revisioned world runtime state in ECS resources.
- Convert world edits into dirty invalidation, build scheduling, and integrated chunk payloads.
- Publish prepared world contributions for render without submit-time authority reads.
- Host replication/streaming-facing state for chunk and op-window synchronization.

## Runtime Contract

- Canonical world maintenance runs on fixed-step simulation cadence.
- Dirty chunk map entries must bootstrap runtime chunk records before build scheduling.
- Build dispatch/integration is explicit and ordered:
  - lifecycle advance
  - build queue/dispatch
  - completed-output integration
- Collision/query services must treat missing authoritative chunk payloads explicitly; gameplay must not silently bypass world authority.

## Key Resources

- `WorldChunkRuntimeMapResource`: per-chunk lifecycle/revision/generation/runtime flags.
- `WorldDirtyChunkMapResource`: dirty reason sets keyed by `ChunkId`.
- `WorldOperationLog`: append-only typed op log.
- `WorldSdfChunkStoreResource`: authoritative per-chunk payload + per-region summary.
- `WorldBuildQueueResource` / `WorldBuildGraphResource` / `WorldCompletedBuildQueueResource`: build runtime pipeline state.
- `WorldStreamingInterestResource` / `WorldReplicationStateResource`: replication and per-connection streaming state.
- `WorldDebugMetricsResource`: diagnostics and queue/build counters.

- [Module Readme](./readme)
