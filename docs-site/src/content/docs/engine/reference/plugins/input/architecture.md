---
title: "Input Plugin Architecture"
description: "Runenwerk integration boundary for standalone RunenInput."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
---

# Input Plugin Architecture

## Authority

Reusable device-input observation and deterministic confirmed-state semantics are owned
by `dornglut/runen-input`.

Runenwerk consumes exact accepted revision `b2bf687e8071d19e124ea5b2c8948c49891cc1de`. The former
`engine/src/plugins/input/neutral.rs` predecessor implementation is deleted; there is no
forwarding module or duplicate reducer authority.

## Module layout

- `engine/src/plugins/input/state.rs` — Runenwerk ECS/integration and frame projections.
- `engine/src/plugins/input/actions_and_bindings.rs` — Runenwerk product actions/bindings.
- `engine/src/plugins/input/mod.rs` — Input plugin installation and Runenwerk-owned exports.
- `engine/src/runtime/winit_input.rs` — winit → RunenInput semantic translation.
- `adapters/native_tablet_input` — native tablet acquisition/translation.
- standalone `runen-input` — reusable semantic observation/reducer authority.

## Dependency direction

```text
winit/native backend adapters
        |
        v
    runen-input
        |
        v
Runenwerk InputState integration
        |
        +--> frame-local compatibility projections
        +--> ActionState product projection
        +--> Draw/Editor adapters
```

RunenInput does not depend on Runenwerk, RunenECS, RunenUI, Draw, or product policy.

## Integration rules

- Runenwerk `InputState` contains a private `runen_input::InputState`.
- All reusable mutation goes through `InputObservationGroup` +
  `runen_input::InputState::admit`.
- Keyboard and pointer product edges are projected by comparing confirmed state before
  and after semantic admission; Runenwerk does not recreate predecessor
  `DigitalAdmission`, `ControlId`, or `DigitalTransition`.
- Product consumers use Runenwerk `ActionState` when they need actions and use
  RunenInput values directly when they need reusable device semantics.
- Packages that name RunenInput values declare a direct `runen-input.workspace = true`
  dependency rather than relying on Engine re-export.

## Non-owners

Runenwerk input integration does not own text/IME semantics, RunenUI routing/focus,
native backend health/calibration acquisition, Draw stroke semantics, camera policy,
replay/network input formats, or speculative device families.
