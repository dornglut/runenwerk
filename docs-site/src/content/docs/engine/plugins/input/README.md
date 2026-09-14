---
title: "Input Plugin"
description: "Documentation for Input Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Input Plugin

## Purpose

Maintains backend-neutral confirmed physical/device input state separately from Runenwerk-owned product action/binding projection. Ordinary winit input is normalized at the runtime backend edge before it reaches `InputState`; `ActionState` derives product actions from that confirmed state and frame-local accepted keyboard press evidence.

## Usage

- Plugin: `InputFinalizePlugin`
- Action projection: `PreUpdate`, `CoreSet::Input`
- Frame cleanup: `FrameEnd`, `CoreSet::FrameEnd`
- Physical/device state resource: `InputState`
- Product action/binding resource: `ActionState`

For the maintained windowed runtime, `engine/src/runtime/winit_input.rs` translates winit keyboard, scroll, cursor, button, and touch evidence into backend-neutral input values. Window-scoped normalized input continues through the existing `PlatformWindowEventQueueResource`/`PlatformEvent` path before `InputState` admits it to the neutral authority.

Raw `DeviceEvent` relative motion is different: winit supplies no window target, so the runtime normalizes its device/source identity and admits the motion directly to the same `InputState` neutral authority. Primary-window redraw after raw motion is host policy and is not input provenance.

The native-tablet `NativeWindowHook` path remains a specialized backend path pending the separately scheduled tablet-convergence slice; it is not the ordinary winit input authority.

## Ownership Boundaries

- `InputState` owns the backend-neutral confirmed physical/device authority plus current non-action text, pointer, scroll, and touch compatibility projections.
- `InputState` records frame-local accepted physical keyboard press evidence with the modifier snapshot applicable to that press. It does not own product bindings or product action state.
- `ActionState` is the single Runenwerk product-action authority. It owns concrete `action::*` ids, `InputBindings`, `KeyChord`, modifier rules/default presets, runtime binding mutation, and frame-local/current `action_pressed` / `action_down` projection.
- Binding semantic types consume `PhysicalKeyIdentity` directly. They do not contain winit API types or compare debug-formatted backend enums.
- The winit edge adapter owns translation from winit evidence into Runenwerk backend-neutral input values; winit types do not enter the neutral reducer or `PlatformEvent` input payloads.
- `InputContext` preserves runtime/session-scoped source identity separately from optional backend device identity. Window target identity remains Runenwerk integration context, not neutral window lifecycle ownership.
- Legacy mouse/touch histories and scalar scroll remain downstream `InputState` projections for current consumers.
- Does not own scene/render behavior that consumes input, RunenUI routing/focus semantics, window lifecycle, or native-tablet backend policy.

## Extension Points

- Add Runenwerk product action ids and default bindings in the product-owned action/binding module.
- Add rebinding flows by applying `InputBindingChange` collections to `ActionState` with the current `InputState` as the physical-state source.
- Extend neutral device semantics only under the accepted input-boundary design and owning issue; do not add product actions or UI semantics to the neutral reducer.
- Keep backend translation at the backend/platform edge rather than reintroducing winit parsing into semantic binding types.

## Additional Details

### Goals

- Preserve input evidence before backend-specific facts are lost.
- Keep physical/control identity distinct from logical keyboard meaning and committed text.
- Preserve source/device scoping, 2D scroll/domain, raw relative motion, contact lifetime, and optional force evidence truthfully.
- Keep product action policy downstream from physical/device truth.
- Allow runtime rebinding without changing product system code or fabricating historical press edges.
- Keep action/binding semantics backend-neutral while retaining explicit backend/test adapters where needed.

### Core Types

- `InputState`
- `ActionState`
- `InputContext`, `InputSourceId`, `InputDeviceId`
- `KeyboardInput`, `PhysicalKeyIdentity`, `LogicalKey`
- `ScrollInput`, `ContactInput`, `PointerButtonInput`
- `InputBindings`
- `InputBindingChange`
- `KeyChord`
- `ModifierRule`
- `action::*` constants (built-in Runenwerk action ids)

The neutral reducer and its internal control/contact identities remain implementation authority rather than a standalone framework API. Runtime source/device identities are session-scoped and must not be treated as persistent hardware identity.

### Runtime Model

The maintained ordinary path is:

```text
winit event evidence
    -> runtime winit adapter
        -> backend-neutral input values
            -> existing PlatformWindowEvent path when the event has a window target
                -> InputState neutral authority
                    -> frame-local physical press evidence
                        -> ActionState Runenwerk product projection
```

Targetless raw relative motion follows:

```text
winit DeviceEvent + DeviceId
    -> runtime winit adapter
        -> backend-neutral source/device context
            -> InputState neutral authority
```

`InputState` does not own ordinary raw `WindowEvent`/`DeviceEvent` parsing or product action policy. Its direct `KeyCode`/mouse/touch injection helpers remain transitional backend/test conveniences and feed the same neutral authority; `KeyCode` is not the `KeyChord`/`InputBindings` semantic contract.

The neutral reducer is the semantic authority for migrated held physical controls and active contacts. `ActionState` derives product action state from it:

- `action_pressed(action_id)`: fired this frame only from an ordinary accepted aggregate-first press and the modifier snapshot captured for that press;
- `action_down(action_id)`: currently held according to neutral confirmed state and current product bindings.

Repeat metadata cannot create another ordinary action press. Backend-synthetic keyboard reconciliation may change confirmed held truth but cannot create an ordinary press. If the same physical key is held by multiple device contexts, current product action policy remains aggregate. Binding changes may recompute `action_down`; they do not reinterpret already-processed keyboard evidence to fabricate `action_pressed`.

Absolute cursor position and raw relative motion remain independent quantities. Rich scroll observations preserve both axes and their measurement domain; the existing scalar `scroll_delta` is only a downstream legacy projection of the vertical component. Touch/contact state remains multi-contact and source/device scoped; the drawing-facing touch sample stream remains single-primary as a downstream compatibility projection.

Physical keyboard identity, logical key meaning, and committed text are separate. Committed text remains a sibling platform event, not held-state authority.

### Current UI Compatibility Boundary

The editor still performs its existing application-local normalized platform input -> RunenUI translation. I1C changes action/binding ownership only. The historical physical-code-to-logical-character mapping and touch-cancel semantic behavior remain for the later RunenUI-adapter slice; they are not corrected here.

### Default Action Map

Default bindings are product-owned by `InputBindings::with_default_bindings()` and installed by `ActionState::default()`.

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
use engine::plugins::input::domain::{action, ActionState, KeyChord, PhysicalKeyIdentity};
use engine::InputState;

let input = InputState::new();
let mut actions = ActionState::new();
actions.unmap_key(
    &input,
    action::WORLD_MOVE_LEFT,
    &PhysicalKeyIdentity::code("KeyA"),
);
actions.map_key(
    &input,
    action::WORLD_MOVE_LEFT,
    PhysicalKeyIdentity::code("KeyJ"),
);
actions.map_chord(
    &input,
    "debug.toggle_freecam",
    KeyChord::code("KeyP").with_shift_required(),
);
```

Read product action state from `ActionState`:

```rust
if actions.action_pressed("debug.toggle_freecam") {
    // toggle freecam
}

if actions.action_down(action::WORLD_MOVE_LEFT) {
    // held movement
}
```

### Resource-Friendly Binding Changes

`InputBindingChange` carries backend-neutral physical identities and can be passed around as product-owned data before application:

```rust
use engine::plugins::input::domain::{
    action, ActionState, InputBindingChange, KeyChord, PhysicalKeyIdentity,
};
use engine::InputState;

let input = InputState::new();
let mut actions = ActionState::new();
let applied = actions.apply_binding_changes(
    &input,
    [
        InputBindingChange::UnmapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: PhysicalKeyIdentity::code("KeyA"),
        },
        InputBindingChange::MapChord {
            action: action::WORLD_MOVE_LEFT.to_string(),
            chord: KeyChord::code("ArrowLeft"),
        },
    ],
);

assert_eq!(applied, 2);
```

### Frame Lifecycle

- Ordinary window-scoped winit input is normalized at `runtime::winit_input` and applied through `PlatformEvent` before frame schedules.
- Raw relative device motion is normalized at the same edge and admitted without fabricating a window target.
- `InputFinalizePlugin` projects `InputState` into `ActionState` in `PreUpdate` / `CoreSet::Input` before ordinary product consumers.
- End-of-frame cleanup runs in `FrameEnd` / `CoreSet::FrameEnd`: `ActionState` clears frame-local action pulses, then `InputState` clears frame-local text/deltas/sample histories.

Cleanup does not clear durable neutral held-control or active-contact state. Held product actions are recomputed from that confirmed state and the current bindings.

## Guides

- Usage: [../../../docs/reference/plugins/input/usage-guide.md](../../reference/plugins/input/usage-guide.md)
- Advanced: [../../../docs/reference/plugins/input/advanced-guide.md](../../reference/plugins/input/advanced-guide.md)
- Architecture: [../../../docs/reference/plugins/input/architecture.md](../../reference/plugins/input/architecture.md)
