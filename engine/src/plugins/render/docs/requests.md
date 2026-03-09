# Render Plugin Requests

## Open Requests

### Fully ECS-First Render Core Migration

Status: `in_progress`  
Requested: `2026-02-24`

Problem:

- Render entry paths still depended on concrete frame payload parameters.
- Feature plugins should be able to provide arbitrary typed frame data without render API changes.

Current progress:

- `gfx.render` now accepts `RenderFrameDataRegistry` instead of a concrete frame payload argument.
- `renderer.prepare_packet` and `renderer.render` now accept `RenderFrameDataRegistry` inputs.
- Render pass prepare/encode contexts now consume only caller-provided frame data from `RenderFrameDataRegistry`; renderer no longer inserts packet-local synthetic frame payloads.
- Render submit now builds `RenderFrameDataRegistry` from ECS `RenderFrameResourceBindings`; feature plugins register resource types via setup instead of hardcoded submit payload wiring.
- Core `renderer.rs` no longer performs mesh/world preparation from `RenderFrameData`; builtin `builtin_mesh_overlay` dispatch is now no-op in core and must be provided by feature plugins.
- Legacy model-manager wiring has been removed from the core render domain module.
- Scene-owned compatibility payload module (`scene::domain::render_data`) was removed; feature plugins now own and register their own frame resources (for example `SdfWorldState`).
- Feature producers now wire explicitly to render prep (`grid_prepare`, `debug_metrics_overlay`, `sdf_renderer_example_update` all run before `frame_render_prepare`).

Next steps:

- Continue splitting feature-owned frame resources into narrower per-pass payloads where useful.
- Keep render core orchestration generic while moving any remaining feature-specific convenience helpers out of shared runtime APIs.

### Typed Frame Data Providers (Render-Agnostic Core)

Status: `superseded`  
Requested: `2026-02-24`
Superseded by: `engine/src/plugins/render/docs/ecs-first-proposal.md`

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
- `engine/src/plugins/render/docs/ecs-first-proposal.md`
