# Grotto Quest Roadmap (Priority-Sorted)

## Priority 0 - Foundation
- Stabilize ECS core (archetypes, query/query_mut, lifecycle correctness).
- Stabilize scheduler core (deterministic ordering, cycle detection, dependency validation).
- Establish game runtime skeleton (fixed-step loop, stage boundaries, diagnostics).

## Priority 1 - Playable MVP Slice
- Character creation (name, appearance, starter traits).
- Core combat (attack, dodge, basic combos, 2 starter skills/profession).
- Procedural dungeon (3-5 rooms, simple encounters, 1 mini-boss).
- Loot and progression basics (XP, level, simple items).
- Hub (shop + storage).

## Priority 2 - Depth Systems
- ECS skill composition framework with runtime modifiers.
- Scheduler-driven game stage pipeline.
- Stronger encounter variation and anti-spam combat incentives.

## Priority 3 - Meta Loop
- Expanded party behavior and tactical controls.
- Deeper hub services (crafting, loadouts, planning).
- Prestige reset loop with new system unlock layers.

## Priority 4 - Scale and Polish
- Expand content breadth (skills, enemies, biomes, bosses).
- Telemetry-informed balancing.
- Visual/audio/game-feel polish.

## Priority 0.5 - Engine UI Foundation (Very High)
- Establish SDF/MSDF UI architecture integrated with ECS + scheduler + wgpu renderer.
- Implement first UI stage pipeline (`ui_input`, `ui_layout`, `ui_build_batches`, `ui_render_extract`, `ui_render_submit`).
- Deliver MVP SDF widgets (panels/buttons/health bar) and verify deterministic execution.
- Define material/shader extension path for UI nodes.
- Immediate milestone: retained ECS console window with typed input + clickable confirm button using one unified submit path.

### Priority 0.5 Exit Criteria
- Console panel, input text, and confirm button are ECS-driven and rendered through UI pipeline stages.
- `Enter` and confirm button click trigger the same submit path and produce identical behavior.
- UI stages run in deterministic order and are covered by at least one scheduler-level ordering validation.
- UI frame path has no per-frame unbounded allocations in hot loops (basic profiling/smoke validation).

### Priority 0.5 Defer List
- Text selection/copy/paste and IME composition.
- Advanced text wrapping/overflow behaviors.
- Theming/animation polish beyond minimal interaction feedback.
- Rich widget set beyond panel/input/button needed for console milestone.
