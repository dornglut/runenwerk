---
title: "World Plugin Docs"
description: "Documentation for World Plugin Docs."
---

# World Plugin Docs

This section tracks the final world/runtime architecture migration for `engine/src/plugins/world`.

## Scope

- Authoritative world model:
  - chunked + region-partitioned
  - revisioned
  - operation-log driven
  - sparse hierarchical SDF storage
- Build pipeline:
  - dirty invalidation
  - async rebuild scheduling
  - completed snapshot integration
- Render handoff:
  - prepare-only contribution resources
  - renderer runtime cache ownership boundary
- Multiplayer:
  - server-authoritative chunk revisions/op windows
  - per-connection streaming interest cursors

## Key Source Modules

- Plugin root: `engine/src/plugins/world/mod.rs`
- Plugin wiring: `engine/src/plugins/world/plugin.rs`
- Lifecycle: `engine/src/plugins/world/chunks/lifecycle.rs`
- Operation log: `engine/src/plugins/world/edits/log.rs`
- Edit ingress: `engine/src/plugins/world/edits/ingress.rs`
- Build dispatch/integration:
  - `engine/src/plugins/world/build/jobs.rs`
  - `engine/src/plugins/world/build/integration.rs`
- Prepare contributions: `engine/src/plugins/world/prepare/contributions.rs`

## Current Guarantees

- Dirty chunk IDs can no longer stall because of missing runtime chunk records.
- World maintenance runs in fixed-step simulation scheduling and is ordered before replication.
- World maintenance phase sets are public and can be used by gameplay plugins for explicit ordering against world integration.
- World runtime mode now tracks simulation authority role (`Local`/`Server`/`Peer` => authoritative, `Client` => replica).
- World edit ingestion is centralized through world plugin ingress APIs, not ad hoc game-side dirty-map writes.
- World ingress rejects non-authoritative (`ClientReplica`) mutations, preventing op-log/dirty-map drift on replica clients.
- Build payload dispatch no longer uses placeholder payload generation in the active runtime path.
- Dirty reasons that arrive during rebuild are preserved and drive follow-up rebuild after integration.
- World authority revision advances from integrated build outputs.
- Collision authority now has explicit missing-payload behavior; missing chunk payload data does not silently bypass world authority.
- Collision sweep authority exposes one world-owned outcome contract (`MissingPayload`/`Hit`/`Clear`) to keep gameplay fail-safe behavior deterministic.
- World metrics expose ingress/invalidations/collision authority miss counts and last integrated world revision.
- World->render invalidation bridge is record-driven for ingress and integration, including chunk-bounds and chunk/region sets.
- Bridge flush dedupes invalidations deterministically by chunk and preserves queued records when render cache resources are missing.
- Core world contribution identity in prepared payloads is now typed (`ChunkId` + typed draw-batch refs) instead of string chunk IDs.
- World replication deltas/op-windows/residency hints are now produced by world runtime systems and exposed via `WorldReplicationStateResource`.
- World-owned streaming interest now tracks active server connections, authoritative chunk relevance, per-connection cursor progression, and disconnect cleanup.
- World now emits deterministic region/chunk invalidation journal records from ingress and integration, and replication exposes those records as region-level invalidation deltas.
- Connection-aware checkpoint shaping now uses acknowledged cursors + relevant chunk sets so incremental payloads stay selective instead of defaulting to full world payload snapshots.
- World runtime inspector/debug overlay now exposes streaming connection/resync counts, cursor/region lag, and region-journal sequence/retained-size diagnostics.
- Cavern runtime paths are hard-cut to world authority: legacy collision-field authority modules are removed, runtime geometry graph authority surfaces are retired, and active snapshot wire contracts are `CavernRunSnapshotV2` / `CavernRunDeltaV2` with explicit V1 decode rejection.

## Related Docs

- Render plugin architecture: `engine/docs/reference/plugins/render/architecture.md`
- Migration roadmap: `engine/docs/roadmaps/world-runtime-final-architecture-migration.md`
