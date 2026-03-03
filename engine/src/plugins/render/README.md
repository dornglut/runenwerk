# Render Plugin Decoupling Plan

## Purpose

Render plugin is moving toward a pure orchestration role so feature plugins can provide concrete rendering behavior without core edits.

## Usage

- Plugin: `RenderPlugin`
- Scheduler nodes: `frame_render_prepare`, `frame_render_submit`
- Consumes:
  - render graph registrations
  - executor registrations
  - shader registry updates
  - feature/plugin-provided frame data

## Ownership Boundaries

- Owns graph compilation, pass ordering, execution dispatch, and submission lifecycle.
- Owns render diagnostics/timing orchestration.
- Does not own feature/world payload schemas, builtin-default gameplay/UI behavior, or feature-specific pass logic.

## Extension Points

- Register feature-owned frame graphs through render graph registry APIs.
- Register feature-owned pass executors through executor registry APIs.
- Register ECS frame resources through `EngineData::register_render_frame_resource::<T>()` in feature plugin setup so submit includes them in `RenderFrameDataRegistry`.
- Schedule feature extract/publish systems to run before `frame_render_prepare` so frame resources are finalized before submit.
- Register shader roots/assets and consume shader handles in feature plugins.
- Add feature-owned renderer plugins/examples without editing core render orchestration.
- Add feature-owned render authoring schemas that compile into runtime graph/executor/pipeline registrations.

## Additional Details

## Goal

Turn the render plugin into pure orchestration/tooling so feature plugins can build any renderer (for example, SDF-style) without editing core render files.

## Target Architecture

Core render keeps only:

- graph compilation and validation
- pass scheduling and execution order
- resource lifetime/orchestration
- command encoder/submission lifecycle
- diagnostics and profiling hooks

Core render must not own:

- built-in pass behavior (`builtin_compute`, `builtin_compose`, `builtin_ui_composite`, etc.)
- default frame graph passes from `renderer.rs`
- shader-specific loading/hot-reload behavior
- text rendering logic (`text.rs`)
- world-specific frame data coupling (any mandatory concrete world payload struct)

## Required Extractions

1. Builtins to plugin:
- Move builtin executor behavior into a separate compatibility/default plugin.
- Keep `render` core executor registry as id -> custom executor only.

2. Defaults to plugin:
- Remove default graph/fallback pass wiring from `renderer.rs`.
- Core should fail clearly when no feature graph is registered.

3. Shader manager to asset service:
- Extract `shader_manager.rs` responsibilities into a shared shader asset/hot-reload plugin.
- Core should consume shader handles/artifacts, not file paths.
- Keep shader load + hot-reload behavior, but make it data-driven:
  - shader roots/folders come from plugin registration or config
  - shaders are auto-discovered from configured assets folders
  - no hardcoded shader ids/path constants in core render

4. Text renderer to UI plugin:
- Move `text.rs` into UI plugin ownership.
- UI plugin registers its own resources/pipelines/passes/executors.

5. Render frame decoupling:
- Replace hard dependency on concrete world frame structs with plugin-defined resources/contexts.
- Core pass execution uses ids + opaque data handles.

## Current Shader Registry Behavior

- Shader assets are ECS components (`ShaderAssetComponent`) stored in `ShaderRegistryResource`.
- Shader files are auto-discovered by scanning configured roots (default: `assets/shaders`).
- Shader ids are derived from relative file paths; no enum or hardcoded global key list is required.
- Hot reload emits ECS events (`ShaderRegistryEvent`) and updates revision counters.
- Runtime systems consume shader handles and events, not hardcoded shader enum variants.

## Tooling for Custom Renderers

Provide a renderer authoring surface that lets features define complete pipelines without core edits:

- typed ids: feature/resource/pipeline/pass/executor
- declarative graph builder + validation
- plugin registration hooks for pipelines and executors
- plugin/config registration for shader asset roots that support auto-load and hot-reload
- debugging tools:
  - active graph dump
  - unresolved ids report
  - pass timings per owner/plugin

Authoring-specific requirements for this tooling:

- dependency-aware reload for graph assets and referenced shader inputs
- atomic application of compiled render-graph bundles
- source-aware diagnostics for unresolved resource/pipeline/executor/shader references

## Migration Phases

1. Phase A: Add final APIs alongside legacy behavior.
2. Phase B: Move default world/UI render behavior into standalone plugins.
3. Phase C: Remove builtins/default fallback behavior from core render.
4. Phase D: Remove compatibility plugin once all examples/features use plugin-owned paths.

## Acceptance Criteria

- `render` core has no hardcoded default passes/pipelines/text/shader behavior.
- SDF example runs entirely through plugin-owned graph + executors.
- A second custom renderer can be added without touching core render files.
- Missing plugin registrations fail fast with actionable diagnostics.
- Render authoring reload never leaves mixed-generation graph state active.

## ECS-First Targets (Current Audit)

Areas that still use non-ECS-owned registries/state and should migrate next:

Progress completed in this pass:

- `shader_manager.rs` now uses ECS component indexing for shader id lookup (no plugin-local id map resource).
- `render_graph_registry.rs` now stores owner registrations as ECS components with indexed owner lookup.
- `render_executor_registry.rs` now stores executor bindings in ECS components and emits ECS registry events.
- `render_executor_registry.rs` no longer auto-seeds builtin executor ids in `Default`; builtin aliases must be registered by plugins/features.
- `renderer.rs` no longer injects default/fallback frame graph passes; it compiles only ECS-registered feature graphs and reports `no_registered_passes` diagnostics when empty.
- UI plugin now owns UI rect shader selection via ECS resource (`UiRenderShaderConfig`), and render submit resolves it to a `ShaderHandle` before invoking core renderer.
- `text.rs` moved from `render/domain` into `ui/domain`; font atlas + glyph pipeline ownership now lives in UI plugin.
- `RenderPassPrepareContext` / `RenderPassEncodeContext` now use typed frame-data lookup (`frame_data::<T>()`) instead of hardcoding concrete world frame types.
- `EngineData` render state now lives in ECS (`render_resources`) and feature plugins register their own typed frame resources.
- `gfx.render` / `renderer.prepare_packet` / `renderer.render` now accept a generic `RenderFrameDataRegistry` input.
- Render pass prepare/encode contexts now consume only caller-provided frame data from `RenderFrameDataRegistry`; renderer no longer synthesizes packet-local payload entries.
- Render submit now populates frame data from ECS through typed `RenderFrameResourceBindings`; feature plugins opt-in by registering resource types, instead of hardcoding submit payload types.
- Feature data producers now explicitly target the render-prep stage (`... -> frame_render_prepare -> frame_render_submit`) instead of only depending on submit.
- Core `renderer.rs` no longer performs world/mesh preparation from feature payloads; `builtin_mesh_overlay` is intentionally no-op in core so feature plugins must own that behavior.
- Legacy `model_manager` coupling has been removed from the active core render domain module.
- Scene compatibility render payload module was removed; feature plugins now own their frame payload schemas.

1. Render frame coupling:
- Render submit no longer adapts from a separate world-frame type; it collects only registered ECS frame resources into `RenderFrameDataRegistry`.
- Runtime `world_render` compatibility helpers were removed.
- Target: migrate feature plugins from coarse compatibility resources to smaller plugin-owned ECS resources per executor.

ECS gaps discovered during this migration are tracked in:

- `ecs/requests.md` (currently `Open requests: none.`)

Render-plugin architecture requests are tracked in:

- `engine/src/plugins/render/requests.md`
- `engine/src/plugins/render/ecs-first-proposal.md` (active direction)
