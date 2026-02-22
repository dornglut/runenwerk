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
  - Builds a frame graph each frame:
    1. `world_compute` (compute)
    2. `world_compose` (render)
    3. `ui_composite` (render)
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

## Next Steps
1. Move pass/resource identifiers to data-backed descriptors for scene-specific graph contributions.
2. Add transient resource allocator and aliasing rules.
3. Support multiple world pipelines (lighting/post/culling) through graph node composition.
4. Add frame diagnostics (graph build time, pass execution timing, active pipelines).
