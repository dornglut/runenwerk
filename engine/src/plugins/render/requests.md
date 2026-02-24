# Render Plugin Requests

## Open Requests

### Typed Frame Data Providers (Render-Agnostic Core)

Status: `superseded`  
Requested: `2026-02-24`
Superseded by: `engine/src/plugins/render/ecs-first-proposal.md`

Problem:

- Render orchestration should not require or own any concrete world/frame struct.
- Multiple plugins/features need to contribute different typed data each frame.
- Current path still includes an adapter from runtime-owned state into render packet data.

Notes:

- This request is retained for history only.
- The active direction is now fully ECS-first:
  - providers are normal ECS systems in render-prep stages
  - render consumes ECS data directly via typed lookup
  - no render-local provider registry abstraction

Related:

- `ecs/requests.md` (no ECS-core gaps currently required for this request)
