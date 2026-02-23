# Scene Architecture

## Purpose
Define an ECS-first scene system that supports:
- fast switching between scenes (menu, gameplay, hub, etc.),
- independent UI overlays (HUD, pause, inventory, console),
- deterministic scheduler behavior,
- clear migration path from current single-world runtime to multi-scene runtime.

## Principles
- Scenes are data + systems, not monolithic objects.
- Scene switching is state transition, not engine reconstruction.
- UI overlays are separate scene layers, not hardcoded special cases.
- Rendering and simulation remain scheduler-driven and deterministic.

## Terminology
- `Scene`: a bundle of ECS world state plus a scheduler pipeline.
- `SceneKind`: semantic type (for example `Boot`, `MainMenu`, `Gameplay`, `Hub`).
- `SceneLayer`: priority domain (`World`, `OverlayUi`, optional `DebugOverlay`).
- `SceneHandle`: stable runtime ID for a loaded scene instance.
- `SceneStack`: ordered list of active scene handles.

## Target Runtime Model
1. `SceneRegistry` stores scene descriptors and build/load callbacks.
2. `SceneManager` owns loaded scene instances and the active `SceneStack`.
3. Each scene instance has:
   - `World` (its ECS data),
   - `Scheduler<SceneData>` (its pipeline),
   - metadata (`kind`, `layer`, `paused`, `visible`).
4. Frame update:
   - evaluate pending scene transition commands,
   - run simulation for active non-paused scenes in stack order,
   - extract render data from visible scenes into a composited frame (`world_render_extract`),
   - run world compute rendering, then overlay UI rendering.

## Overlay Strategy
- `World` layer scene drives gameplay simulation and world rendering.
- `OverlayUi` layer scene drives HUD/menus/console.
- Overlay can be swapped independently from the world scene.
- Pause menu behavior:
  - push `OverlayUi::Pause` scene,
  - mark world scene `paused=true`,
  - keep rendering world + pause overlay.

This gives us "game as scene + UI as overlay scene" without coupling UI and gameplay worlds.

## Scheduler Integration
- Keep scheduler deterministic per scene.
- Add top-level orchestration stages in engine runtime:
  1. `scene_transition` (apply queued push/pop/replace operations)
  2. `world_scene_update` (run world-scene scheduler)
  3. `world_render_extract` (copy world-scene ECS render state into renderer frame data)
  4. `scene_overlay_format_messages` (typed world->overlay notifications to formatted lines)
  5. `scene_overlay_apply_messages` (append formatted lines to active overlay scrollback)
  6. `game_command_apply` + `game_command_execute` (consume submit intent and run command side effects)
  7. `overlay_ui_*` stages (`hot_reload`, `input`, `editor`, `layout`, `build_batches`, `render_extract`)
  8. `frame_render_submit` (run frame graph + submit world and UI passes)
- Existing `engine_v2` UI pipeline remains valid inside a UI scene.
- Render submission uses a frame graph so compute and render passes are scheduled via pass/resource dependencies.

## ECS Data Boundaries
- Prefer scene-local ECS worlds for isolation and simpler lifecycle.
- Cross-scene communication via explicit events/messages, not direct entity sharing.
- Shared resources (settings, input snapshot, asset handles) live in engine-level context and are injected into scene data each frame.

## Transition API (Proposed)
- `push_scene(kind, layer)`
- `pop_scene(layer_or_handle)`
- `replace_scene(kind, layer)`
- `set_overlay(kind)` (sugar for replace overlay scene)
- `pause_scene(handle, paused: bool)`
- `set_world(kind)` (replace active world scene runtime)

All transition requests should be queued and applied at `scene_transition` stage boundary.

## Save/Load and Editor Impact
- UI template save/load remains scene-specific (`assets/ui/<scene>.ron`).
- Editor actions should target the currently active UI scene only.
- Future: scene manifest file mapping scene kinds to template/assets.

## Migration Plan
### Phase 1 (Completed)
- SceneManager introduced with queued transition stage.
- Overlay UI runtime moved under scene ownership.
- Overlay stack push/pop preserves suspended runtime state.
- World-scene scheduler bridge added and scene-local ECS world updates each tick.
- Scene-scoped channels introduced for submit/command and world->overlay messages.

### Phase 2 (Active)
- Add `Gameplay` world scene instance with autonomous ECS simulation loop.
- Implement deterministic gameplay stage chain (`sense -> decide -> move -> combat -> resolve -> emit_ui`).
- Keep overlay UI fully scene-local and composited over gameplay scene.
- Extend transition commands for explicit world-scene switching.

### Phase 3
- Multiple world scenes (`Hub`, `Dungeon`, `CombatArena`) with handoff rules.
- Pause and modal overlays as stackable overlays with world pause semantics.
- Scene lifecycle hooks (`on_enter`, `on_exit`, `on_pause`, `on_resume`) as explicit systems/events.

### Phase 4
- Full multi-scene stack including menus and debug overlays.
- Data-driven scene manifests mapping scene kinds to templates/assets/bootstrap data.

## Current Risks
- Increased complexity in scene orchestration.
- Input routing conflicts between layers.
- Debuggability across multiple worlds.

## Mitigations
- Keep transitions stage-bound and logged.
- Enforce layer priorities and input-consume contracts.
- Add diagnostics UI: active stack, scene states, last transition command.

## Next Actions
1. Introduce `SceneRegistry` descriptors so world/overlay runtime creation is decoupled from `SceneManager`.
2. Move lifecycle event handling into dedicated scene lifecycle systems with optional UI/debug subscribers.
3. Continue reducing coupling across existing runtime modules (`manager`, `gameplay`, `config`, `lifecycle`) and remove remaining orchestration spillover in system-layer code.
4. Add scene manifest support to map scene IDs to bootstrap config paths and overlay templates.
