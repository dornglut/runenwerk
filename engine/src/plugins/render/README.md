# Render Plugin Decoupling Plan

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
- world-specific frame data coupling (`WorldRenderFrame` as mandatory pass input)

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
- Replace hard dependency on `WorldRenderFrame` with plugin-defined resources/contexts.
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

## ECS-First Targets (Current Audit)

Areas that still use non-ECS-owned registries/state and should migrate next:

Progress completed in this pass:

- `shader_manager.rs` now uses ECS component indexing for shader id lookup (no plugin-local id map resource).
- `render_graph_registry.rs` now stores owner registrations as ECS components with indexed owner lookup.
- `render_executor_registry.rs` now stores executor bindings in ECS components and emits ECS registry events.
- `render_executor_registry.rs` no longer auto-seeds builtin executor ids in `Default`; builtin aliases must be registered by plugins/features.
- `renderer.rs` no longer injects default/fallback frame graph passes; it compiles only ECS-registered feature graphs and reports `no_registered_passes` diagnostics when empty.
- `renderer.rs` UI shader binding now resolves by ECS shader id (`ui_rect`) from auto-discovered assets instead of hardcoding a shader file registration path at render time.
- `text.rs` moved from `render/domain` into `ui/domain`; font atlas + glyph pipeline ownership now lives in UI plugin.
- `model_manager.rs` now stores model assets in ECS components with indexed id lookup and ECS-owned watch/reload config state.
- `RenderPassPrepareContext` / `RenderPassEncodeContext` now use typed frame-data lookup (`frame_data::<T>()`) instead of hardcoding `WorldRenderFrame`.

1. Render frame coupling:
- Core renderer packet build path still sources data from `WorldRenderFrame` directly.
- Target: move packet input to plugin-owned ECS resources/handles and stop requiring world-specific frame struct at renderer entrypoints.

ECS gaps discovered during this migration are tracked in:

- `ecs/requests.md`
