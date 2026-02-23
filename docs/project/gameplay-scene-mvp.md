# Gameplay Scene MVP Spec

## Purpose
Define the next concrete implementation target: an autonomous ECS gameplay simulation running as the world scene, with overlay UI composited on top.

## Scope
- One active world scene kind: `Gameplay`.
- One active overlay UI scene (HUD/console/debug), with stack push/pop for modal overlays (pause, inventory later).
- Autonomous simulation only for MVP validation: no direct player control required yet.

## Runtime Contract
- World simulation executes in scene-local ECS world only.
- Overlay UI does not read world ECS directly.
- Cross-scene data flow uses typed scene channels (`WorldToOverlayMessage`).
- Scene transitions remain queued and applied only at the transition stage boundary.

## ECS Components (MVP)
- `Transform2 { x, y }`
- `Velocity2 { x, y }`
- `Team` (`Player`, `Enemy`)
- `Health { current, max }`
- `AttackTimer { cooldown_s, elapsed_s }`
- `AggroTarget { entity }` (optional component)
- `MoveIntent { x, y }` (system output, cleared each tick)
- `CombatStats { attack_range, attack_damage }`

## ECS Resources (MVP)
- `GameplayClock` (fixed-step tick counter and dt)
- `GameplayBounds` (arena extents for movement clamp/bounce)
- `SpawnConfig` (initial spawn counts and spacing)
- `RngState` (deterministic seed for repeatable tests)

## Scheduler Stages (World Scene)
1. `gameplay_sense`
2. `gameplay_decide`
3. `gameplay_move`
4. `gameplay_combat`
5. `gameplay_resolve`
6. `gameplay_emit_ui`

### Stage Intent
- `sense`: acquire nearest target and update `AggroTarget`.
- `decide`: compute `MoveIntent` for chase/kite behavior.
- `move`: integrate velocity/position and clamp to bounds.
- `combat`: apply attack cooldown + range checks, enqueue damage events.
- `resolve`: apply damage, mark deaths, respawn or despawn policy.
- `emit_ui`: publish typed summary messages (`Combat`, `Loot`, `Quest`, `Tick`) to overlay channel.

## Bootstrapping
- On scene enter, spawn:
  - one player agent,
  - three enemy agents with small stat variance.
- Keep spawn deterministic using a fixed seed.
- Arena is a bounded 2D simulation surface for now (render-independent).

## Overlay Integration
- Overlay displays:
  - current world scene kind,
  - simulation tick,
  - alive counts by team,
  - latest combat events in colored scrollback.
- Existing category colors stay authoritative:
  - `[world]`, `[combat]`, `[loot]`, `[quest]`.

## Commands (MVP)
- `set_world gameplay` switches active world scene to `Gameplay`.
- `set_world hub` switches to non-combat stub scene.
- `push_overlay pause` opens pause overlay and pauses world scene.
- `pop_overlay` closes top overlay and resumes world if no pausing overlays remain.

## Test Plan
- Deterministic stage-order test: same seed yields same combat outcomes.
- Transition test: pushing pause overlay halts world tick advancement.
- Overlay flow test: world combat events appear in overlay scrollback with expected category tags.
- Lifecycle test: replacing world scene resets gameplay ECS world cleanly.

## Definition of Done
- Gameplay scene runs every frame through all gameplay stages without panics.
- Player/enemy autonomous combat loop is visible through overlay diagnostics/messages.
- World and overlay scene switching works at runtime without restart.
- All new gameplay and transition behavior is covered by tests in `engine_v2`.
