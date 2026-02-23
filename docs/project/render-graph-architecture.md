# Render Graph Architecture

## Purpose
Define the engine_v2 frame-graph path used to schedule compute and render passes with explicit dependencies and switchable pipelines.

## Current Implementation
- `engine_v2/src/render/frame_graph.rs`
  - `FrameGraph` builder for compute/render passes.
  - Pass descriptors include `reads`, `writes`, and explicit `depends_on` handles.
  - Execution order is computed via topological sort with cycle detection.
  - Resource hazards (`write->read`, `write->write`, `read/write->later write`) infer dependencies automatically.
- `engine_v2/src/render/pipeline_registry.rs`
  - `PassSlot` (`world_compute`, `world_compose`, `ui_composite`).
  - `PipelineKey` values per slot.
  - Runtime validation for slot/key compatibility.
- `engine_v2/src/render/renderer.rs`
  - Builds a frame graph each frame from descriptor data:
    - `assets/render/frame_graph.ron`
    - `assets/render/frame_graph_overlays.ron` (scene-aware appended passes by world/overlay labels)
    - fallback to built-in defaults if config is missing/invalid.
  - Default graph stages:
    1. `world_compute` (compute)
    2. `world_compose` (render)
    3. `mesh_overlay` (render)
    4. `ui_composite` (render)
  - Executes passes in graph order through `FramePassExecutor` dispatch by pass name.
  - Reuses mesh MSAA/depth targets by surface size/format (rebuild only on resize/format changes).

## Runtime Ergonomics
- Pipeline inspection and switching are exposed through console commands:
  - `pipelines`
  - `set_pipeline <world_compute|world_compose|ui> <pipeline_key>`
- Current world compute slot supports:
  - `world_compute_basic`
  - `world_compute_high_contrast`
- File-backed WGSL shader loading + hot reload:
  - `assets/shaders/ui_rect.wgsl`
  - `assets/shaders/world_compute_basic.wgsl`
  - `assets/shaders/world_compute_high_contrast.wgsl`
  - `assets/shaders/world_compose_fullscreen.wgsl`
- Shader control commands:
  - `reload_shaders`
  - `shader_watch <on|off>`
  - `shader_status`
- Model hot reload for SDF/raymarch integration:
  - Blender `.glb` files in `assets/models/`
  - Imported as SDF proxy volumes (per-primitive bounds)
  - Commands: `models`, `reload_models`, `model_watch <on|off>`, `model_status`

## Why This Model
- Keeps pass ordering explicit and data-driven.
- Allows compute and render passes to coexist in one scheduling model.
- Enables runtime pipeline switching without hardcoding pass internals in gameplay systems.

## Current Limitations
- `engine_v2/src/render/renderer.rs` still centralizes graph build, executor routing, and multiple pass-specific policies in one large module.
- Scene-specific graph contribution selection is label-driven; descriptor/manifest-driven scene contribution is not complete yet.
- Resource lifetime policy is still localized per pass/runtime type (no shared transient allocator or aliasing policy yet).
- Frame diagnostics exist but do not yet expose aggregate per-pass prepare/encode metrics or cache hit/miss summaries.

## Target Architecture
1. Keep `FrameGraph` as ordering and hazard model.
2. Add `FramePassExecutor` registry keyed by pass slot/name.
3. Build a `RenderPacket` first, then let executors perform `prepare` + `encode`.
4. Move render-target lifetime to a small resource cache owned by render runtime.
5. Let world/overlay scene descriptors append graph nodes/resources through a contribution API.

## Migration Status
1. Executor MVP: complete.
   - `world_compute` and `world_compose` execute through `FramePassExecutor`.
2. Mesh/UI executor migration: complete.
   - `mesh_overlay` and `ui_composite` now dispatch through executor registration.
3. Resource policy: partial.
   - Mesh MSAA/depth target reuse by `(surface_format, width, height)` is in place.
   - Cache metrics and broader transient resource policy are still pending.
4. Scene contribution descriptors: in progress.
   - Overlay pass append works via config; manifest-driven scene descriptors are still pending.

## Next Steps
1. Move scene overlay selection from simple label match to scene descriptor assets (`assets/scenes/*.ron`) so graph contributions come from scene manifests.
2. Add transient resource allocator and aliasing rules after descriptor path stabilizes.
3. Add frame diagnostics for graph build time, executor prepare/encode time, and active pipelines.
