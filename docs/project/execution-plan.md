# Execution Plan

## Objective
Track the active implementation state for the retained ECS console UI foundation and define immediate next work.

## Completed
- Retained ECS UI pipeline for console flow.
- Console panel with scrollback, input field, and confirm button.
- Unified submit bridge: `UiSubmitEvent -> GameCommandEvent -> command execution`.
- Basic command set and command registry extensions (`help`, `clear`, `echo`, `history`, `count`, aliases/usages).
- Text rendering integration with fallback behavior.
- DPI-aware scaling for high-res displays.
- Input clipping and scrollback viewport slicing.
- Editor-buffer-based input model.
- Caret alignment fixes via glyph metrics.
- Hot-reloadable `.ron` template at `assets/ui/console.ron`.
- True renderer-side text clipping/scissor.
- Tiny-window layout hardening for footer controls.
- Component-tree templates with stable node IDs and keyed patch/update.
- Multiline editor mode: wrapping, up/down navigation, viewport behavior.
- Interactive UI editor mode (M1): `F1` toggle, click-to-select node, mouse drag move, arrow-key nudging, save-to-template.
- Scene template switching baseline (`console` / `hud`) with runtime hotkeys.
- Editor visibility controls with persistence in templates (`hide selected`, `restore all`, save/reload).
- `SceneManager` baseline with queued overlay transition commands and deterministic transition stage.
- Overlay UI scheduler stage namespace (`overlay_ui_*`) to isolate scene-layer execution flow.
- Overlay input consume contract (`overlay_consumed`) for future gameplay-scene input routing.
- World scene runtime stub with dedicated scene scheduler and per-scene world container.
- Scheduler bridge stage (`world_scene_update`) that runs world scene after overlay transition stage.
- World scene stub now mutates scene-local ECS component state (`WorldFrameCounter`) each tick.
- Overlay stack commands added (`push`/`pop`) with deterministic queued transitions.
- Overlay runtime ownership moved under `SceneManager` (`overlay_runtime.world` + `overlay_runtime.ui`) instead of top-level engine data.
- World scene stub now updates ECS debug motion (`WorldDebugPosition` + `WorldDebugVelocity`) each tick.
- Editor mode shows live scene diagnostics (world scene state, overlay stack, world tick/debug position).
- Overlay stack now preserves independent runtime state per pushed overlay scene (push suspends current runtime, pop restores it).
- Console submit/command processing now flows through scene-scoped channels (no ECS event entities on the hot command path).
- World scene now emits scene-scoped notifications and overlay applies them via dedicated stages (`scene_overlay_format_messages` and `scene_overlay_apply_messages`).
- Scene channels now use typed message enums for submit, command input, and world->overlay notifications.
- World->overlay notifications now include richer typed payloads (`Tick`, `Combat`, `Loot`, `Quest`) with a dedicated format stage before apply stage.
- Scrollback now supports category-based custom text colors (`world`, `combat`, `loot`, `quest`) in renderer output.
- Separate logs window introduced (distinct from command console) with pause/resume buffering controls.
- Time control commands added for simulation (`freeze_time`, `resume_time`, `toggle_time`).
- Textbox viewports now support wrapped rendering plus vertical and horizontal scrolling.
- Scroll UX upgraded with optional hints and scrollbar indicators (template-controlled).
- Gameplay simulation defaults moved to data (`assets/gameplay/gameplay_stub.ron`) with runtime fallback to defaults.
- Scene lifecycle events added (`enter`, `exit`, `pause`, `resume`) and emitted through scene channels.
- Logs window layout and scroll UX flags moved into UI template layout config.
- World compute rendering foundation added: ECS world data extraction stage (`world_render_extract`) plus compute-to-texture world pass composited before overlay UI.
- Frame graph baseline added for mixed compute/render execution (`world_compute -> world_compose -> mesh_overlay -> ui_composite`) with cycle checks and inferred resource dependencies.
- Pipeline registry added with slot/key validation plus runtime pipeline switching commands (`pipelines`, `set_pipeline ...`).
- Renderer frame flow split into explicit packet stages (`prepare_packet` and `render_packet`) to separate extraction/prepare work from encode/submit.
- Mesh overlay pass now reuses persistent MSAA/depth targets by surface size/format instead of recreating them each frame.
- Added `FramePassExecutor` abstraction and migrated `world_compute`/`world_compose` dispatch to executor-based routing.
- Migrated `mesh_overlay` and `ui_composite` dispatch to executor routing so frame-graph pass execution is now registry-driven.
- Frame graph pass definitions now load from `assets/render/frame_graph.ron` with built-in defaults as fallback when config is missing/invalid.
- Scene-aware frame graph overlays now load from `assets/render/frame_graph_overlays.ron` and append passes by world/overlay scene label.

## Next (Active)
- Scene runtime decomposition (descriptor/registry path and world-update stage split).
- Renderer extraction ownership reduction (move world/mesh/UI extraction out of renderer).

## Architecture Findings (Code Scan)
- `engine_v2/src/render/renderer.rs` remains a monolithic orchestrator (>2k LOC) that still owns asset polling, mesh extraction, camera solve, frame graph build, graph execution, and command submission.
- `prepare_packet`/`render_packet` provides a clean boundary, but extraction still runs inside renderer instead of dedicated extractor systems/services.
- `prepare_mesh_draw` in `engine_v2/src/render/renderer.rs` mixes four responsibilities: model/chunk collection, cache policy, GPU uploads, and draw packet assembly.
- Mesh MSAA/depth targets are now reused by surface size/format, but render-target lifecycle policy still lives inside renderer internals.
- `world_scene_update_system` in `engine_v2/src/systems/scene.rs` combines input mapping, camera controls, gameplay config hot reload, fixed-step stepping, and world->overlay message forwarding.
- Scene construction remains hardcoded across `engine_v2/src/runtime/scene.rs` and `engine_v2/src/runtime/scene/manager.rs` instead of descriptor/registry-driven.
- `engine_v2/src/render/model_manager.rs` bundles discovery, Blender conversion, glTF import, watch state, and status/logging in one module.
- `engine_v2/src/render/world_compute.rs` still combines pipeline build/rebuild, frame upload, and pass encode in one type (`WorldComputeRenderer`).

## Refactor Principles
- Make frame extraction and frame execution separate phases with typed handoff data.
- Keep pass-specific behavior behind per-pass executors; avoid regressing to inline pass-name branching.
- Keep GPU resource lifetime persistent and resize/reload driven.
- Make scene creation data-driven through descriptors and builders.
- Keep hot-reload services generic (same pattern for shaders, models, gameplay configs).

## Execution Program
### Phase A: Render Packet Boundary (`partial`)
1. Keep packetized handoff (`RendererPreparedPacket`) as the rendering boundary.
2. Move world/mesh/UI extraction out of renderer into dedicated extractor systems/services.
3. Keep renderer focused on graph orchestration and encode/submit only.
4. Add deterministic extraction tests for fixed world snapshots.

### Phase B: Pass Executor Registry (`complete`)
1. `FramePassExecutor` dispatch is in place for `world_compute`, `world_compose`, `mesh_overlay`, and `ui_composite`.
2. Frame-graph pass routing now uses executor lookup from graph node metadata.
3. Keep `FrameGraph` responsible for ordering/hazards while executors own pass behavior.

### Phase C: Resource Lifetime and Upload Policy (`partial`)
1. Mesh MSAA/depth target reuse by surface size/format is in place.
2. Split mesh upload path into:
   - static geometry/material cache (rare rebuild),
   - per-frame instance stream (agent transforms/colors).
3. Keep `MeshCacheEntry` internals in a dedicated module; renderer uses a narrow API.
4. Add cache hit/miss and upload-bytes counters per executor.

### Phase D: Scene Runtime Decomposition (`active`)
1. Add `SceneDescriptor` + `SceneRegistry` for world and overlay scene builders.
2. Split `world_scene_update_system` into deterministic stages:
   - `world_input_apply`,
   - `world_camera_apply`,
   - `world_config_hot_reload`,
   - `world_fixed_step_run`,
   - `world_outbox_flush`.
3. Keep lifecycle events typed and consumed by optional debug/UI subscribers.
4. Add transition/lifecycle ordering tests against the registry path.

### Phase E: Unified Hot Reload Services (`planned`)
1. Introduce shared file-watch/reload utility used by shader/model/gameplay config.
2. Normalize status payloads (`revision`, `last_error`, `source_path`, `reload_reason`).
3. Add runtime controls for profiling verbosity levels without recompiling.

## Recommended Next Slice (Low Risk)
1. Implement SceneRegistry bootstrap for existing world/overlay scene kinds without changing scene behavior.
2. Split `world_scene_update_system` into `world_input_apply` and `world_fixed_step_run` first, then preserve existing semantics with explicit stage ordering.
3. Add deterministic ordering tests for `scene_transition -> world_scene_update -> scene_overlay_*` flow.

## Definition of Done for Current Phase
- Renderer no longer owns extraction logic; it consumes packets.
- Frame-graph execution uses pass executors instead of hardcoded handle checks.
- World update path is split into composable systems with fixed ordering.
- Scene construction is descriptor-driven for world and overlay kinds.
- Existing gameplay/UI behavior remains stable with added coverage for the new seams.

## Later
- Data-driven scene manifest (`assets/scenes/*.ron`) mapping scene IDs to templates/bootstrap assets.
- Persistence/knowledge layer evaluation after gameplay loops stabilize (save schema and telemetry shape informed by real simulation).
- Template diagnostics and reload feedback tooling.
- UI diff/rebuild and batching performance pass.
- Scene stack diagnostics panel in editor/debug overlay.
