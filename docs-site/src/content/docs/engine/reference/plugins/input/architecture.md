---
title: "Input Plugin Architecture"
description: "Runenwerk integration boundary for standalone RunenInput."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# Input Plugin Architecture

## Authority

Reusable device-input observation and deterministic confirmed-state semantics are owned
by `dornglut/runen-input`.

Runenwerk consumes exact accepted revision `2751e19fa42255b86e786e7cd837198c917b7b25`. The former
`engine/src/plugins/input/neutral.rs` predecessor implementation is deleted; there is no
forwarding module or duplicate reducer authority.

## Module layout

- `engine/src/plugins/input/state.rs` — Runenwerk ECS/integration and frame projections.
- `engine/src/plugins/input/actions_and_bindings.rs` — Runenwerk product actions/bindings.
- `engine/src/plugins/input/app_ext.rs` — Runenwerk product-action App composition convenience.
- `engine/src/plugins/input/mod.rs` — Input plugin installation, private integration activation, and Runenwerk-owned exports.
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
  borrowed `runen_input::InputState::admit(&group)`.
- Keyboard and pointer product edges are projected by comparing confirmed state before
  and after semantic admission; Runenwerk does not recreate predecessor
  `DigitalAdmission`, `ControlId`, or `DigitalTransition`.
- Product consumers use Runenwerk `ActionState` when they need actions and use
  RunenInput values directly when they need reusable device semantics.
- `InputFinalizePlugin` alone installs the private Runenwerk Input integration activation
  fact. `AppActionBindingsExt::add_input_bindings` admits against that fact rather than
  inferring plugin selection from public `InputState` / `ActionState` resources.
- Packages that name RunenInput values declare a direct `runen-input.workspace = true`
  dependency rather than relying on Engine re-export.
- Native tablet capability booleans remain acquisition facts. At the RunenInput
  boundary, `true` maps to `CapabilityKnowledge::Supported` and `false` maps to
  `CapabilityKnowledge::Unknown`; `Unsupported` requires explicit negative evidence.
- Draw keeps its UI-owned boolean capability model and projects only `Supported` to
  `true`; `Unknown` and `Unsupported` project to `false`, subject to its existing
  measurement-projectability checks.

## Non-owners

Runenwerk input integration does not own text/IME semantics, RunenUI routing/focus,
native backend health/calibration acquisition, Draw stroke semantics, camera policy,
replay/network input formats, or speculative device families.
