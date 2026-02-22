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
  - Executes passes in graph order.

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
- Render loop still dispatches pass work through handle identity checks in `renderer.rs`.
- Graph node metadata does not yet route to an executor interface.
- Resource lifetime is partly outside graph policy (example: mesh MSAA/depth targets recreated in mesh encode path).
- Scene-specific graph composition is still hardcoded in renderer.

## Target Architecture
1. Keep `FrameGraph` as ordering and hazard model.
2. Add `FramePassExecutor` registry keyed by pass slot/name.
3. Build a `RenderPacket` first, then let executors perform `prepare` + `encode`.
4. Move render-target lifetime to a small resource cache owned by render runtime.
5. Let world/overlay scene descriptors append graph nodes/resources through a contribution API.

## Migration Plan
1. Executor MVP:
   - implement executors for `world_compute` and `world_compose`,
   - remove direct handle branching for those two passes.
2. Mesh/UI migration:
   - move `mesh_overlay` and `ui_composite` to executors,
   - keep behavior and shader/pipeline controls unchanged.
3. Resource policy:
   - add persistent MSAA/depth target cache by `(surface_format, width, height)`,
   - expose cache hit/miss metrics in performance logs.
4. Scene contribution:
   - introduce descriptor-driven graph contributions for world scene variants and overlays.

## Next Steps
1. Move scene overlay selection from simple label match to scene descriptor assets (`assets/scenes/*.ron`) so graph contributions come from scene manifests.
2. Add transient resource allocator and aliasing rules after descriptor path stabilizes.
3. Add frame diagnostics for graph build time, executor prepare/encode time, and active pipelines.
