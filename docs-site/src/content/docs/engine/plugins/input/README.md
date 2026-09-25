---
title: "Input Plugin"
description: "Runenwerk integration for standalone RunenInput and product action projection."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Input Plugin

## Purpose

Runenwerk's Input Plugin integrates the standalone
[`dornglut/runen-input`](https://github.com/dornglut/runen-input) framework with
Runenwerk runtime, ECS, frame-local compatibility projections, and product actions.

Reusable backend-neutral device observation and deterministic confirmed-state semantics
belong to RunenInput. Runenwerk consumes the exact accepted revision:

```text
ba87e7c80a9626239a011038cec97c30010379a8
```

Runenwerk does not keep a second neutral reducer or compatibility forwarding namespace.

## Ownership

**RunenInput owns:**

- source/device/tool/contact identity and semantic device observations;
- physical/logical keyboard evidence, pointer buttons/motion, scroll, contacts, and
  demonstrated tablet/stylus observations;
- measurement/source-time/evidence/history/origin semantics;
- grouped validation/admission and deterministic confirmed-state reduction;
- confirmed key/button, pointer-position, and contact queries.

**Runenwerk owns:**

- `InputState` as the ECS/integration shell around `runen_input::InputState`;
- frame-local keyboard press evidence, text, mouse/touch histories, scalar compatibility
  projections, and product edge pulses;
- `ActionState`, bindings, action ids/defaults, rebinding, and product-action projection;
- winit/native acquisition, App/Host/window lifecycle, and product adapter policy.

RunenUI focus/routing/text semantics, Draw behavior, camera policy, and backend health or
calibration acquisition remain with their existing owners.

## Runtime model

Window-scoped input:

```text
winit evidence
  -> runtime winit adapter
    -> runen_input semantic values
      -> PlatformEvent
        -> Runenwerk InputState integration
          -> runen_input::InputState::admit(...)
            -> Runenwerk frame/product projections
              -> ActionState
```

Targetless relative motion follows the same semantic admission path without inventing a
window target.

Native tablet acquisition remains in `native_tablet_input`. That adapter constructs
RunenInput observations directly; Draw consumes those observations directly from
`runen_input` while Runenwerk retains only its integration resource/projection layer.

## Behavioral laws retained by integration

- repeat does not fabricate a second product press edge;
- backend reconciliation may correct confirmed held state without becoming an ordinary
  source press;
- aggregate held state remains true while any source/device context still holds;
- absolute cursor and relative motion stay distinct;
- absent scroll axes remain absent rather than measured zero;
- unknown measurement domains are not upgraded to invented precision;
- predicted/estimated tablet evidence does not become confirmed state;
- text/IME remains separate from physical key identity.

## Product actions

Product actions remain Runenwerk-owned:

```rust
use engine::plugins::input::domain::{action, ActionState, KeyChord};
use engine::InputState;
use runen_input::PhysicalKeyIdentity;

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

`action_pressed` is frame-local product policy; `action_down` is derived from
RunenInput-confirmed physical state plus current bindings.

## Extension rules

- Extend reusable device semantics in `dornglut/runen-input`, not in Runenwerk.
- Extend Runenwerk action vocabulary/bindings in the product-owned action module.
- Keep winit/native translation at adapter boundaries.
- Do not add forwarding re-exports of RunenInput values through
  `engine::plugins::input`.

## Guides

- Usage: [../../reference/plugins/input/usage-guide.md](../../reference/plugins/input/usage-guide.md)
- Advanced: [../../reference/plugins/input/advanced-guide.md](../../reference/plugins/input/advanced-guide.md)
- Architecture: [../../reference/plugins/input/architecture.md](../../reference/plugins/input/architecture.md)
