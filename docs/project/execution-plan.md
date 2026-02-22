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
- World scene now emits scene-scoped notifications and overlay applies them via dedicated stage (`scene_overlay_messages`).
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
- Frame graph baseline added for mixed compute/render execution (`world_compute -> world_compose -> ui_composite`) with cycle checks and inferred resource dependencies.
- Pipeline registry added with slot/key validation plus runtime pipeline switching commands (`pipelines`, `set_pipeline ...`).

## Next (Active)
- Abstraction/generalization pass to reduce coupling and improve maintainability.
- Generalize world render extraction from gameplay-specific components to reusable ECS render components (`RenderGlyph`, `RenderMaterial`, `RenderLayer`).
- Add camera extraction/resource path so compute rendering uses explicit scene camera state.
- Let scenes contribute graph nodes/resources so world scene type controls render graph composition.

## Recommended Breakdown For Next
1. Split `engine_v2/src/runtime/scene.rs` into focused modules (`manager`, `lifecycle`, `gameplay`, `config`) while preserving behavior.
2. Add scene registry/descriptor layer so world/overlay construction is data-driven and not hardcoded in `SceneManager`.
3. Decouple lifecycle event generation from UI log formatting (typed lifecycle channel consumer remains optional/debug-only).
4. Split `ui_input_system` and `ui_build_batches_system` into smaller systems with stable contracts (text input, scroll routing, logs rendering, diagnostics rendering).
5. Consolidate shared UI utilities (scrollback clamping + viewport helpers) to remove duplication across scene/command systems.
6. Add focused tests around scene registry transitions, lifecycle ordering, and refactored UI subsystem boundaries.

## Later
- Data-driven scene manifest (`assets/scenes/*.ron`) mapping scene IDs to templates/bootstrap assets.
- Persistence/knowledge layer evaluation after gameplay loops stabilize (save schema and telemetry shape informed by real simulation).
- Template diagnostics and reload feedback tooling.
- UI diff/rebuild and batching performance pass.
- Scene stack diagnostics panel in editor/debug overlay.

## Definition Of Done For Current Phase
- Scene manager/runtime code is modularized without behavior regressions.
- Scene construction and switching are descriptor-driven for world/overlay kinds.
- UI input/build paths are split into smaller systems with deterministic ordering.
- Existing scene, gameplay, and UI tests remain green with additional coverage for new abstraction seams.
