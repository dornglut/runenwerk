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
- World edit ingestion is centralized through world plugin ingress APIs, not ad hoc game-side dirty-map writes.
- World authority revision advances from integrated build outputs.
- Collision authority now has explicit missing-payload behavior; missing chunk payload data does not silently bypass world authority.

## Related Docs

- Render plugin architecture: `engine/docs/reference/plugins/render/architecture.md`
- Migration roadmap: `engine/docs/roadmaps/world-runtime-final-architecture-migration.md`
