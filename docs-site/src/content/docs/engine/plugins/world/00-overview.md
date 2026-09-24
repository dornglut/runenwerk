---
title: "World Overview"
description: "Overview and documentation map for World."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
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
- Dirty reasons arriving during rebuild are preserved and replayed as follow-up work after integration.
- Build dispatch/integration is explicit and ordered:
  - lifecycle advance
  - build queue/dispatch
  - completed-output integration
- World authority revision progresses only from accepted integrations.
- Collision/query services must treat missing authoritative chunk payloads explicitly; gameplay must not silently bypass world authority.
- World runtime mode is synchronized from simulation authority role; replica clients run in `ClientReplica` mode.
- World edit ingress is authority-gated and must reject mutation submissions when runtime mode is non-authoritative.
- `WorldRuntimeSet` phases are public so gameplay plugins can express explicit ordering relative to world authority updates.
- World->render cache invalidation is record-driven:
  - ingress and build integration publish bounds-derived invalidation records
  - each record carries chunk bounds plus chunk/region membership sets
  - bridge flush dedupes stale marks deterministically by `ChunkId`
  - records stay queued while render cache resources are unavailable
- World region invalidation journal is world-owned and deterministic:
  - ingress + build integration append ordered region/chunk invalidation records
  - journal records are sequence-stamped and bounded for stable replay surfaces
  - replication exports region/chunk invalidation deltas from this journal
- World contribution identity at world->render handoff is typed for chunk-linked payloads (`ChunkId` + typed draw-batch refs), not string chunk labels.
- World replication state is produced by world runtime systems each fixed step and consumed by game snapshot adapters as a read-model.
- World streaming interest is synchronized from active server connections and runtime chunk authority, and net replication updates world-owned per-connection cursor/resync progression.
- Connection checkpoint shaping is interest-aware: acknowledged cursor baselines and relevant chunk sets drive selective chunk/op payload emission.
- Lightweight world debug overlay now includes streaming/journal observability (connection count, resync count, cursor lag, region-sequence lag, and journal head/record count).
- Cavern runtime paths are hard-cut to world authority: legacy collision-field authority is removed, runtime geometry graph authority surfaces are retired, and active snapshot wire contracts are `V2` only.

## Key Resources

- `WorldChunkRuntimeMapResource`: per-chunk lifecycle/revision/generation/runtime flags.
- `WorldDirtyChunkMapResource`: dirty reason sets keyed by `ChunkId`.
- `WorldOperationLog`: append-only typed op log.
- `WorldSdfChunkStoreResource`: authoritative per-chunk payload + per-region summary.
- `WorldBuildQueueResource` / `WorldBuildGraphResource` / `WorldCompletedBuildQueueResource`: build runtime pipeline state.
- `WorldStreamingInterestResource` / `WorldReplicationStateResource`: replication and per-connection streaming state.
- `WorldRegionInvalidationJournalResource`: deterministic region/chunk invalidation journal for downstream consumers.
- `WorldDebugMetricsResource`: diagnostics and queue/build counters.
  - ingress operation count
  - invalidated chunk count
  - collision query count
  - collision authority misses
  - last world revision

- [Module Readme](./README.md)
