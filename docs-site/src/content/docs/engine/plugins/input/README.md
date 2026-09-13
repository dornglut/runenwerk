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

Provides action-mapped input state and frame pulse handling above a backend-neutral device-observation/state seam. Ordinary winit input is normalized at the runtime backend edge before it reaches the maintained platform/input path; product actions and frame-local convenience remain downstream projections.

## Usage

- Plugin: `InputFinalizePlugin`
- Typed schedule: `FrameEnd`
- Typed set: `CoreSet::FrameEnd`
- Primary state type: `InputState` in `engine/src/plugins/input/domain.rs`

For the maintained windowed runtime, `engine/src/runtime/winit_input.rs` translates winit keyboard, scroll, cursor, button, and touch evidence into backend-neutral input values. Window-scoped normalized input continues through the existing `PlatformWindowEventQueueResource`/`PlatformEvent` path before `InputState` admits it to the neutral authority.

Raw `DeviceEvent` relative motion is different: winit supplies no window target, so the runtime normalizes its device/source identity and admits the motion directly to the same `InputState` neutral authority. Primary-window redraw after raw motion is host policy and is not input provenance.

The native-tablet `NativeWindowHook` path remains a specialized backend path pending the separately scheduled tablet-convergence slice; it is not the ordinary winit input authority.

## Ownership Boundaries

- The internal neutral seam owns confirmed held-control and active-contact state for migrated device facts.
- The winit edge adapter owns translation from winit evidence into Runenwerk backend-neutral input values; winit types do not enter the neutral reducer or `PlatformEvent` input payloads.
- `InputContext` preserves runtime/session-scoped source identity separately from optional backend device identity. Window target identity remains Runenwerk integration context, not neutral window lifecycle ownership.
- `InputState` owns Runenwerk action mapping, per-frame action pulses, text/frame convenience, and key/chord rebinding behavior above the neutral seam.
- Legacy mouse/touch histories, scalar scroll, and public movement/menu fields remain downstream projections for current consumers.
- Does not own scene/render behavior that consumes input, RunenUI routing/focus semantics, window lifecycle, or native-tablet backend policy.

## Extension Points

- Add new action ids and default bindings in `InputBindings::with_default_bindings()`.
- Add rebinding flows by applying `InputBindingChange` collections.
- Extend neutral device semantics only under the accepted input-boundary design and owning issue; do not add product actions or UI semantics to the neutral reducer.
- Keep backend translation at the backend/platform edge rather than reintroducing winit parsing inside `InputState`.

## Additional Details

### Goals

- Preserve input evidence before backend-specific facts are lost.
- Keep physical/control identity distinct from logical keyboard meaning and committed text.
- Preserve source/device scoping, 2D scroll/domain, raw relative motion, contact lifetime, and optional force evidence truthfully.
- Keep engine/game systems decoupled from concrete keys above the current product binding layer.
- Allow runtime rebinding (`map_key`, `map_chord`, `unmap_*`) without changing system code.
- Keep action queries and public movement/menu booleans synchronized in `InputState`.

### Core Types

- `InputState` (`engine/src/plugins/input/domain.rs`)
- `InputContext`, `InputSourceId`, `InputDeviceId`
- `KeyboardInput`, `PhysicalKeyIdentity`, `LogicalKey`
- `ScrollInput`, `ContactInput`, `PointerButtonInput`
- `InputBindings`
- `InputBindingChange`
- `KeyChord`
- `ModifierRule`
- `action::*` constants (built-in action ids)

The reducer and its internal control/contact identities remain implementation authority rather than a standalone framework API. Runtime source/device identities are session-scoped and must not be treated as persistent hardware identity.

### Runtime Model

The maintained ordinary path is:

```text
winit event evidence
    -> runtime winit adapter
        -> backend-neutral input values
            -> existing PlatformWindowEvent path when the event has a window target
                -> InputState neutral authority
                    -> Runenwerk product/action projections
```

Targetless raw relative motion follows:

```text
winit DeviceEvent + DeviceId
    -> runtime winit adapter
        -> backend-neutral source/device context
            -> InputState neutral authority
```

`InputState` therefore no longer owns ordinary raw `WindowEvent`/`DeviceEvent` parsing. Its direct `KeyCode`/mouse/touch injection helpers remain transitional product/test conveniences and feed the same neutral authority; current `KeyCode` binding policy is scheduled for the later action/binding-separation slice.

The neutral reducer is the semantic authority for migrated held physical controls and active contacts. Product/action state derives from that authority:

- `action_pressed(action_id)`: fired this frame from an ordinary accepted press edge;
- `action_down(action_id)`: currently held according to neutral confirmed state and current bindings.

Absolute cursor position and raw relative motion remain independent quantities. Rich scroll observations preserve both axes and their measurement domain; the existing scalar `scroll_delta` is only a downstream legacy projection of the vertical component. Touch/contact state remains multi-contact and source/device scoped; the drawing-facing touch sample stream remains single-primary as a downstream compatibility projection.

Physical keyboard identity, logical key meaning, and committed text are separate. Repeat metadata is preserved and cannot create a second ordinary pressed edge. Backend-synthetic keyboard reconciliation changes confirmed held state without becoming an ordinary user press/release. Committed text is emitted as a sibling platform event only for ordinary key-press evidence.

### Current UI Compatibility Boundary

The editor still performs its existing application-local `PlatformEvent` -> RunenUI translation. During I1B that adapter is changed only mechanically to consume normalized payloads. In particular, its historical physical-code-to-character mapping and touch-cancel semantic-command behavior are deliberately not treated as corrected here; the later RunenUI-adapter slice owns that semantic cleanup.

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

`InputBindingChange` is designed so remap requests can be passed around as data before being applied:

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

- Ordinary window-scoped winit input is normalized at `runtime::winit_input` and applied through `PlatformEvent`.
- Raw relative device motion is normalized at the same edge and admitted without fabricating a window target.
- End-of-frame reset is done by `InputFinalizePlugin`, which calls `InputState::clear_frame`.

`clear_frame` clears frame-local pulses, deltas, and sample-history projections. It does not clear durable neutral held-control or active-contact state; held actions are recomputed from that state and current bindings.

## Guides

- Usage: [../../../docs/reference/plugins/input/usage-guide.md](../../reference/plugins/input/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/input/advanced-guide.md](../../reference/plugins/input/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/input/architecture.md](../../reference/plugins/input/architecture.md)
