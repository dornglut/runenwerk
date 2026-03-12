# Input Plugin

## Purpose

Provides action-mapped input state and frame pulse handling, decoupling gameplay/UI systems from concrete key bindings.

## Usage

- Plugin: `InputFinalizePlugin`
- Typed schedule: `FrameEnd`
- Typed set: `CoreSet::FrameEnd`
- Primary state type: `InputState` in `engine/src/plugins/input/domain.rs`

OS input events are consumed through `InputState::handle_window_event` and `InputState::handle_device_event`.
The runtime also feeds normalized platform events through the same `InputState` methods.

## Ownership Boundaries

- Owns action mapping, per-frame action pulses, and key/chord rebinding behavior.
- Does not own scene/render behavior that consumes input.

## Extension Points

- Add new action ids and default bindings in `InputBindings::with_default_bindings()`.
- Add rebinding flows by applying `InputBindingChange` collections.
- Add higher-level input events/resources on top of `InputState`.

## Additional Details

### Goals

- Keep engine/game systems decoupled from concrete keys.
- Allow runtime rebinding (`map_key`, `map_chord`, `unmap_*`) without changing system code.
- Keep action queries and public movement/menu booleans synchronized in `InputState`.

### Core Types

- `InputState` (`engine/src/plugins/input/domain.rs`)
- `InputBindings`
- `InputBindingChange`
- `KeyChord`
- `ModifierRule`
- `action::*` constants (built-in action ids)

### Runtime Model

`InputState` tracks:

- physical key/button state (`keys_down`, mouse buttons)
- action state:
  - `action_pressed(action_id)`: fired this frame
  - `action_down(action_id)`: currently held

Public movement/menu fields are synchronized from the action state each frame so scene/UI systems can read a stable input view.

### Default Action Map

Default bindings are installed by `InputBindings::with_default_bindings()` and used by `InputState::new()`.

Examples:

- `ui.submit`: `Enter`, `NumpadEnter` (Shift forbidden)
- `ui.insert_newline`: `Shift+Enter`, `Shift+NumpadEnter`
- `world.move_left/right/up/down`: `A/D/W/S`
- `system.toggle_pause_menu`: `Escape`
- `scene.next` / `scene.prev`: `F2` / `Shift+F2`
- `scene.overlay_push` / `scene.overlay_pop`: `F5` / `Shift+F5`
- `ui.save_template`: `Ctrl+S` or `Super+S`

### Runtime Remapping

```rust
use engine::plugins::input::domain::{action, KeyChord};
use engine::InputState;
use winit::keyboard::KeyCode;

let mut input = InputState::new();
input.unmap_key(action::WORLD_MOVE_LEFT, KeyCode::KeyA);
input.map_key(action::WORLD_MOVE_LEFT, KeyCode::KeyJ);
input.map_chord(
    "debug.toggle_freecam",
    KeyChord::new(KeyCode::KeyP).with_shift_required(),
);
```

Read action state:

```rust
if input.action_pressed("debug.toggle_freecam") {
    // toggle freecam
}

if input.action_down(action::WORLD_MOVE_LEFT) {
    // held movement
}
```

### Goal API (Resource/Event Friendly)

`InputBindingChange` is designed so remap requests can be passed around as data (for example via ECS resources/events) before being applied:

```rust
use engine::plugins::input::domain::{action, InputBindingChange, KeyChord};
use engine::InputState;
use winit::keyboard::KeyCode;

let mut input = InputState::new();
let applied = input.apply_binding_changes([
    InputBindingChange::UnmapKey {
        action: action::WORLD_MOVE_LEFT.to_string(),
        key: KeyCode::KeyA,
    },
    InputBindingChange::MapChord {
        action: action::WORLD_MOVE_LEFT.to_string(),
        chord: KeyChord::new(KeyCode::ArrowLeft),
    },
]);

assert_eq!(applied, 2);
```

Equivalent changes can also be applied to the world resource:

```rust
let mut input = world.resource_mut::<InputState>()?;
let applied = input.apply_binding_changes(changes);
```

### Frame Lifecycle

- OS events call:
  - `InputState::handle_window_event`
  - `InputState::handle_device_event`
- End-of-frame reset is done by `InputFinalizePlugin`, which calls:
  - `InputState::clear_frame`

`clear_frame` clears per-frame pulses and keeps held actions in sync with current key state.

## Guides

- Usage: [../../../docs/reference/plugins/input/usage-guide.md](../../../docs/reference/plugins/input/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/input/advanced-guide.md](../../../docs/reference/plugins/input/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/input/architecture.md](../../../docs/reference/plugins/input/architecture.md)


