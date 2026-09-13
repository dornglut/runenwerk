---
title: "Input Plugin"
description: "Documentation for Input Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Input Plugin

## Purpose

Provides action-mapped input state and frame pulse handling, decoupling gameplay/UI systems from concrete key bindings. The plugin now hosts an internal backend-neutral observation/state seam for migrated device facts; product actions and frame-local convenience remain downstream projections.

## Usage

- Plugin: `InputFinalizePlugin`
- Typed schedule: `FrameEnd`
- Typed set: `CoreSet::FrameEnd`
- Primary state type: `InputState` in `engine/src/plugins/input/domain.rs`

OS input events are consumed through `InputState::handle_window_event` and `InputState::handle_device_event`.
The runtime also feeds normalized platform events through the same `InputState` methods.
Full winit/platform-edge normalization remains a later input-boundary migration; current entry points adapt the facts they already receive into the internal neutral seam.

## Ownership Boundaries

- The internal neutral seam owns confirmed held-control and active-contact state for the device facts migrated into it.
- `InputState` owns Runenwerk action mapping, per-frame action pulses, text/frame convenience, and key/chord rebinding behavior above that seam.
- Legacy mouse/touch histories and public movement/menu fields remain downstream projections for current consumers.
- Does not own scene/render behavior that consumes input, RunenUI routing/focus semantics, or native-tablet backend policy.

## Extension Points

- Add new action ids and default bindings in `InputBindings::with_default_bindings()`.
- Add rebinding flows by applying `InputBindingChange` collections.
- Add higher-level input events/resources on top of `InputState`.
- Extend neutral device semantics only under the accepted input-boundary design and owning issue; do not add product actions or UI semantics to the neutral reducer.

## Additional Details

### Goals

- Keep engine/game systems decoupled from concrete keys.
- Allow runtime rebinding (`map_key`, `map_chord`, `unmap_*`) without changing system code.
- Keep action queries and public movement/menu booleans synchronized in `InputState`.
- Keep migrated device facts single-authority while preserving current product-facing behavior during the staged input cleanup.

### Core Types

- `InputState` (`engine/src/plugins/input/domain.rs`)
- `InputBindings`
- `InputBindingChange`
- `KeyChord`
- `ModifierRule`
- `action::*` constants (built-in action ids)

The backend-neutral observation/reducer types are internal implementation authority, not a new public framework API.

### Runtime Model

`InputState` hosts two distinct responsibilities during the staged migration:

- an internal neutral reducer is the semantic authority for migrated held physical controls and active touch contacts;
- product/action state derives from that authority:
  - `action_pressed(action_id)`: fired this frame from an ordinary accepted press edge;
  - `action_down(action_id)`: currently held according to neutral confirmed state and current bindings.

Absolute cursor position, raw relative motion, legacy scalar scroll, button-transition history, and touch sample history remain distinct observations/projections. The existing drawing-facing touch sample projection remains single-primary for compatibility with current consumers, while the neutral authority retains all admitted concurrent touch contacts.

Committed text remains separate from physical held-key authority. Synthetic keyboard reconciliation can change held state without creating ordinary action-pressed edges.

Public movement/menu fields are synchronized from action state so scene/UI systems can read the current product-facing view.

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

`clear_frame` clears frame-local pulses, deltas, and sample-history projections. It does not clear durable neutral held-control or active-contact state; held actions are recomputed from that state and the current bindings.

## Guides

- Usage: [../../../docs/reference/plugins/input/usage-guide.md](../../reference/plugins/input/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/input/advanced-guide.md](../../reference/plugins/input/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/input/architecture.md](../../reference/plugins/input/architecture.md)
