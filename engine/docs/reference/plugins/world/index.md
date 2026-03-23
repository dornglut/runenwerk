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
- Build dispatch/integration:
  - `engine/src/plugins/world/build/jobs.rs`
  - `engine/src/plugins/world/build/integration.rs`
- Prepare contributions: `engine/src/plugins/world/prepare/contributions.rs`

## Related Docs

- Render plugin architecture: `engine/docs/reference/plugins/render/architecture.md`
- Migration roadmap: `engine/docs/roadmaps/world-runtime-final-architecture-migration.md`
