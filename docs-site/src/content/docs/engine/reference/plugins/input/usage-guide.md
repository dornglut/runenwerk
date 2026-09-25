---
title: "Input Plugin Usage Guide"
description: "Documentation for Input Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
publication: primary
---

# Input Plugin Usage Guide

## Purpose

Maintains backend-neutral physical/device input state and a separate Runenwerk product action/binding projection with frame-local action pulses.

## Entry Points

- Module: engine/src/plugins/input/mod.rs
- Entry: InputFinalizePlugin
- App product-action composition: AppActionBindingsExt
- Physical/device state: InputState
- Product action/binding state: ActionState
- Local README: engine/src/plugins/input/README.md

## Minimal Setup

```rust
use engine::plugins::input::InputFinalizePlugin;
use engine::prelude::AppActionBindingsExt;
use runen_input::PhysicalKeyIdentity;

app.add_plugin(InputFinalizePlugin);
app.add_input_bindings([(
    "product.open_inventory",
    PhysicalKeyIdentity::code("KeyI"),
)]);
```

## Runtime Contract

- `PreUpdate` / `CoreSet::Input`: derive `ActionState` from confirmed `InputState` plus accepted frame-local keyboard press evidence.
- `FrameEnd` / `CoreSet::FrameEnd`: clear action pulses and physical frame-local compatibility samples without clearing durable confirmed held state.
- `InputState` owns physical/device truth; `ActionState` owns Runenwerk action vocabulary, bindings, rebinding, `action_pressed`, and `action_down`.
- `InputFinalizePlugin` installs one private Runenwerk integration-activation fact. `AppActionBindingsExt` requires that fact; public `InputState` + `ActionState` presence is not plugin activation.
- Gameplay/editor/scene systems consume these resources but do not own the input authorities.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
