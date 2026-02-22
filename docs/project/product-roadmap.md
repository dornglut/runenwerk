# Product Roadmap

## Priority 0: Foundation
- Stabilize ECS core (archetypes, query/query_mut, lifecycle correctness).
- Stabilize scheduler core (deterministic ordering, cycle detection, dependency validation).
- Harden runtime skeleton (frame loop, stage boundaries, diagnostics).

## Priority 0.5: Engine UI Foundation (High)
- Retained ECS UI architecture integrated with scheduler + wgpu.
- UI stage graph: `ui_input -> ui_layout -> ui_build_batches -> ui_render_extract -> ui_render_submit`.
- Console milestone with shared submit behavior (`Enter` and confirm button).
- Establish extensible material/shader path.
- Scene foundation for world + overlay composition (`SceneManager`, transition queue, layered execution).
- ECS-to-GPU world rendering bridge (extract world scene ECS data and render via wgpu compute before UI overlay).
- Frame-graph render orchestration with dynamic pass-pipeline selection.

### 0.5 Exit Criteria
- ECS-driven panel/input/button rendered through UI stage pipeline.
- Enter and button click produce identical submit behavior.
- Deterministic scheduler ordering for UI stages.
- No obvious per-frame unbounded allocation in hot UI path.
- Runtime scene switching works without engine restart and supports UI overlay on top of world scene.

### 0.5 Deferred
- IME composition and full text-editing parity.
- Rich theme/animation polish.
- Large widget library beyond console needs.

## Priority 1: Playable MVP Slice
- Character creation (starter traits/appearance).
- Core combat loop (attack/dodge/basic skills).
- Procedural dungeon (3-5 rooms + mini-boss).
- Basic loot/progression and minimal hub (shop/storage).

## Priority 2: Depth Systems
- ECS skill composition framework with runtime modifiers.
- Scheduler-driven gameplay stage pipeline expansion.
- Encounter variation and anti-spam combat incentives.

## Priority 3: Meta Loop
- Deeper party control and tactical commands.
- Hub services (crafting/loadout planning).
- Prestige/reset loop and layered unlock systems.

## Priority 4: Scale and Polish
- Expanded content breadth (skills/enemies/biomes/bosses).
- Balancing with telemetry.
- Visual/audio/game-feel polish.
