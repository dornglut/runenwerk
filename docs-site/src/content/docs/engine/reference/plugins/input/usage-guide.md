---
title: "Input Plugin Usage Guide"
description: "Documentation for Input Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Input Plugin Usage Guide

## Purpose

Maintains backend-neutral physical/device input state and a separate Runenwerk product action/binding projection with frame-local action pulses.

## Entry Points

- Module: engine/src/plugins/input/mod.rs
- Entry: InputFinalizePlugin
- Physical/device state: InputState
- Product action/binding state: ActionState
- Local README: engine/src/plugins/input/README.md

## Minimal Setup

```rust
use engine::plugins::input::InputFinalizePlugin;

app.add_plugin(InputFinalizePlugin);
```

## Runtime Contract

- `PreUpdate` / `CoreSet::Input`: derive `ActionState` from confirmed `InputState` plus accepted frame-local keyboard press evidence.
- `FrameEnd` / `CoreSet::FrameEnd`: clear action pulses and physical frame-local compatibility samples without clearing durable confirmed held state.
- `InputState` owns physical/device truth; `ActionState` owns Runenwerk action vocabulary, bindings, rebinding, `action_pressed`, and `action_down`.
- Gameplay/editor/scene systems consume these resources but do not own the input authorities.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
