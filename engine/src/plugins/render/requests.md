# Render Plugin Requests

## Open Requests

### Typed Frame Data Providers (Render-Agnostic Core)

Status: `open`  
Requested: `2026-02-24`

Problem:

- Render orchestration should not require or own any concrete world/frame struct.
- Multiple plugins/features need to contribute different typed data each frame.
- Current path still includes an adapter from runtime-owned state into render packet data.

Request:

- Add a render-plugin provider pattern that lets plugins register frame data providers.
- Providers run before render submit and publish typed references/handles into a shared per-frame data registry.
- Render core consumes only the generic registry API and never imports feature/world structs.

Proposed render-side API shape:

- provider registration in render plugin state (resource-backed registry in ECS).
- deterministic provider order (priority + stable tie-break).
- typed insert/get APIs with:
  - single value per type
  - optional keyed variants per type for multi-source use cases
- per-frame clear/finalize lifecycle so stale data cannot leak across frames.
- diagnostics:
  - registered providers
  - published types/keys for current frame
  - missing-type reports for executors

Acceptance criteria:

- Render entrypoints no longer accept concrete frame payload structs.
- At least two feature plugins can publish different typed data in the same frame.
- Render executors can read both data types using only typed lookup.
- Missing data yields actionable diagnostics without release-path panics.

Related:

- `ecs/requests.md` (no ECS-core gaps currently required for this request)
