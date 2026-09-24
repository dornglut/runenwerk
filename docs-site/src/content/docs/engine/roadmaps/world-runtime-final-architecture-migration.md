---
title: "World Runtime Final Architecture Migration"
description: "Documentation for World Runtime Final Architecture Migration."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# World Runtime Final Architecture Migration

Owner: `engine/src/plugins/world`

Status: completed (hard-cut): world runtime is authoritative in Cavern Hunt runtime paths; legacy collision-field authority is removed and legacy V1 snapshot/delta payloads are explicitly rejected.

## Phases

1. World plugin skeleton + ownership boundary cut.
2. Typed op-log + dirty invalidation with dual-write from legacy geometry edits.
3. Build queue/graph and completed-output integration.
4. Render world/detail/cave prepared feature payload handoff and runtime cache wiring.
5. Per-connection replication snapshot capture + world chunk/op streaming schema.
6. Gameplay query migration from legacy geometry/collision to world query services.
7. Legacy authority removal and guard tests.

## Current Delivered

- `WorldPlugin` registered and initialized from Cavern Hunt plugin wiring.
- New world subdomain modules landed (`ids`, `chunks`, `sdf`, `edits`, `build`, `streaming`, `caves`, `prepare`, `debug`).
- Geometry edit path and worldgen init now dual-write into `WorldOperationLog` and mark dirty chunks.
- Net replication driver now supports `capture_snapshot_for_connection(...)` and server state keeps per-connection snapshot history.
- Render feature contracts expanded with world/cave/detail/procedural/wind contribution resources.
- Fixed-step world maintenance cadence and explicit world build pipeline ordering are active.
- Dirty-chunk lifecycle now bootstraps missing runtime records from dirty map state.
- World op ingress is now centralized through world plugin edit-ingress APIs.
- World runtime mode now tracks simulation authority role and ingress rejects non-authoritative (`ClientReplica`) mutation submissions.

## Active Correctness Gate (Current Focus)

1. Replace placeholder build payload dispatch with deterministic op-window-derived payload generation.
2. Keep authoritative collision behavior explicit when chunk payload is unavailable:
   - no silent gameplay "free pass" on missing chunk payload data
   - explicit readiness/fail-safe contract and metrics for authority misses
3. Ensure world authority revision progresses only from accepted integrated world build outputs.
4. Expand world runtime tests from source-cutoff guards to end-to-end behavior checks.

## Correctness Gate v2 Progress (Implemented)

- Build dispatch now derives chunk payload/region summary from op-window operations affecting the chunk, rather than placeholder payload generation.
- Dirty reasons merged while a chunk is rebuilding are preserved and force follow-up rebuild after current integration completes.
- World authority revision now advances only when a completed output is accepted by integration.
- World runtime counters now expose ingress operations, invalidated chunk count, collision query count, collision authority misses, and last world revision.
- World maintenance phase sets are public and Cavern Hunt fixed-step gameplay systems are explicitly ordered relative to `WorldRuntimeSet::BuildIntegrate`.
- World->render invalidation now publishes record contracts from both ingress and integration, carrying chunk-bounds + chunk/region sets and deterministic chunk-level dedupe on flush.
- Missing render-cache resources no longer risk invalidation loss: bridge records stay queued until cache resources are restored and then flush cleanly.
- World/render contribution identity flow is now typed for chunk-linked world payloads (`ChunkId` and typed draw-batch refs), replacing stringly chunk identity in the core handoff path.
- World runtime now rebuilds `WorldReplicationStateResource` each fixed step from authoritative chunk/op-log/store state; Cavern world checkpoint capture consumes this world-owned state instead of rebuilding deltas ad hoc.
- World streaming interest is now synchronized from active server connections and authoritative chunk runtime state each fixed step.
- Net replication now feeds per-connection cursor/resync progression into world-owned streaming interest state, including disconnect cleanup.
- World now maintains a deterministic region invalidation journal sourced by ingress + integration, and replication exposes region/chunk invalidation deltas directly from that world-owned journal.
- Connection shaping now consumes acknowledged region-invalidation progression from world-owned streaming interest and emits selective checkpoint chunk/op payloads only for relevant chunk sets.
- Lightweight world debug/inspect surfaces now include streaming/journal progression counters (connections, resync count, cursor lag, region-sequence lag, journal sequence/size) for runtime diagnostics.
- Cavern snapshot restore no longer rebuilds/synchronizes legacy `CavernCollisionField` authority from topology; checkpoint restore remains world-runtime-authoritative.
- Cavern runtime wiring no longer initializes legacy `CavernCollisionField`/`CavernGeometryGraph` authority resources; no runtime compatibility graph insertion path remains.
- Runtime geometry edit application no longer mutates legacy `CavernCollisionField`; world ingress + world runtime integration remain the sole authority path.
- Material GI scaffold now reads revision only from `WorldAuthorityState` and no longer falls back to `CavernGeometryGraph` revision.
- Legacy `CavernCollisionField` is no longer registered as a runtime ECS resource component in Cavern Hunt.
- Worldgen bootstrap no longer queries `CavernGeometryGraph` resources for seed-bounds or extraction-seal ID capture; init now uses topology bounds and edit-apply outcomes for compatibility metadata.
- Cavern runtime worldgen/edit systems no longer mutate/query `CavernGeometryGraph`; runtime edit ingress now mirrors only world-authoritative operations.
- Cavern snapshot/restore active wire contract is now `CavernRunSnapshotV2`/`CavernRunDeltaV2` with world-only authority fields.
- Net replication decode path now explicitly rejects legacy `V1` snapshot/delta payloads with deterministic unsupported-version errors.
- Snapshot restore no longer inserts legacy runtime geometry resources (`CavernGeometryGraph` / `CavernGeometryRuntimeState`).
- Legacy `domain/world/collision_field` module has been removed from the world domain surface.

## Deferred Next-Track Work

- Extend world-owned streaming policy beyond baseline connection+chunk shaping (interest windows, prioritization/rate policy).
- Expand region-centric planning surfaces for SAG-facing workflows on top of the hardened world runtime substrate.
- Add richer optional world visual debugging/editor-grade surfaces without reintroducing runtime authority coupling.
