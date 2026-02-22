# Backlog

Purpose: capture implementation tracks that are agreed but not yet complete, with clear scope and sequence.

## Active

### ECS + WGPU Compute World Rendering
- Status: `in_progress` (started February 22, 2026)
- Goal: render the world scene from ECS simulation data through a compute pipeline, then composite UI overlay scenes on top.

Scope:
- Add a scheduler stage that extracts render-ready world data from ECS (`world_render_extract`). ✅
- Introduce a world compute renderer that:
  - consumes extracted ECS world render data,
  - writes an offscreen world texture via compute shader,
  - composites that texture into the frame before UI rendering. ✅
- Keep scene layering explicit: world scene render first, overlay UI scene render second. ✅
- Add frame graph orchestration for compute+render pass ordering with dependency validation. ✅
- Add pipeline registry and runtime switching path per pass slot. ✅

Acceptance criteria:
- World rendering no longer depends on immediate UI draw commands.
- Active world scene data (agents/positions/state-derived visual signals) is visible through compute output.
- Overlay UI continues to render over world output with existing scene stack behavior.
- Freeze/pause state remains visible in rendered world feedback.

Next increments:
1. Add ECS render component set (`RenderGlyph`, `RenderMaterial`, `RenderLayer`) so render extraction is generic and not tied to gameplay agent structs.
2. Move color/radius/team style values into data files under `assets/gameplay/`.
3. Add camera component/resource extraction (position, zoom, bounds) for world-to-screen mapping control.
4. Split compute into culling/binning and shading passes as entity count grows.
5. Add profiling counters (agent count, extraction ms, compute ms, compose ms).

## Planned

### Scene Registry + Descriptor Construction
- Decouple scene construction from hardcoded match paths.
- Load scene descriptors/manifests from data and build world/overlay runtimes through a registry.

### UI Batch Pipeline Modularization
- Finish splitting `ui_build_batches_system` into focused builders for console, logs, input, and diagnostics.
- Consolidate duplicated scroll viewport math into shared helper contracts.
