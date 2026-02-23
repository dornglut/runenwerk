# Backlog

## Purpose
Capture implementation tracks that are agreed but not yet complete, with clear scope and sequence.

## Active

### Render Simplification Program
- Status: `in_progress` (started February 22, 2026)
- Goal: decouple extraction, pass execution, and GPU resource lifetime to reduce frame-time spikes and simplify feature iteration.

Scope:
- Introduce `RenderPacket` boundary (`world`, `mesh`, `ui`) and remove extraction from renderer hot path.
- Add pass-executor registry to replace hardcoded pass-handle branching in render loop.
- Add persistent render-target cache for MSAA/depth resources by surface size/format.
- Split mesh path into static cache management vs dynamic instance streaming.

Acceptance criteria:
- `Renderer::render` performs orchestration + encode/submit only.
- Pass behavior is dispatched by executor registration, not pass-handle `if` chains.
- No per-frame recreate of mesh MSAA/depth targets on stable resolution.
- Profiling logs show separate extraction vs encode costs.
- Frame graph pass layout can be changed through data (`assets/render/frame_graph.ron`) without code edits.

Next increments:
1. Extract `MeshPacket` and `UiPacket` from current renderer internals. ✅ (`RendererPreparedPacket` + `prepare_packet`/`render_packet`)
2. Introduce `FramePassExecutor` and migrate `world_compute` + `world_compose`. ✅
3. Migrate `mesh_overlay` + `ui_composite` executors. ✅
4. Add resource cache metrics and regression tests (mesh surface target reuse is now in place).
5. Validate scene-contributed frame graph overlays on top of descriptor-driven base graph. ✅ (scene-label overlays wired via `assets/render/frame_graph_overlays.ron`)

### Scene Runtime Decomposition
- Status: `in_progress`
- Goal: make scene orchestration data-driven and reduce coupled logic in world update/system glue.

Scope:
- Add `SceneDescriptor` and `SceneRegistry` for world/overlay builders.
- Split `world_scene_update_system` into narrowly-scoped stages.
- Keep lifecycle events typed and optional for UI/debug consumption.

Acceptance criteria:
- Scene creation/switching no longer hardcoded in manager implementation.
- World update stage responsibilities are isolated and testable.
- Lifecycle ordering remains deterministic under replace/push/pop/pause flows.

Next increments:
1. Add descriptor schema and registry bootstrap.
2. Migrate world/overlay runtime construction to registry.
3. Add transition/lifecycle tests around descriptor-driven manager.

### ECS + WGPU Compute World Rendering
- Status: `baseline_complete`, `optimization_in_progress`
- Goal: retain existing compute + compose pipeline while refactoring around clearer abstractions.

Completed baseline:
- `world_render_extract` scheduler stage.
- Compute world pass + compose pass.
- Frame graph ordering with dependency validation.
- Runtime pipeline switching and shader/model hot reload controls.

Optimization backlog:
1. Add ECS render component set (`RenderGlyph`, `RenderMaterial`, `RenderLayer`) so extraction is not gameplay-struct specific.
2. Move visual style values into data files under `assets/gameplay/`.
3. Split compute into culling/binning and shading passes as world density grows.
4. Add per-pass profiling counters and percentile summaries.

## Planned

### Scene Registry + Descriptor Construction
- Promote descriptor-based scene wiring to data manifests (`assets/scenes/*.ron`).
- Add scene-specific render graph contribution descriptors.

### UI Batch Pipeline Modularization
- Finish splitting `ui_build_batches_system` into focused builders for console, logs, input, and diagnostics.
- Consolidate duplicated scroll viewport math into shared helper contracts.

### Asset Reload Service Unification
- Factor shared file watch/reload logic used by shader/model/gameplay config loaders.
- Standardize status payload shape and runtime diagnostics output.
