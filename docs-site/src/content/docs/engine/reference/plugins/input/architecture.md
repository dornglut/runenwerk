---
title: "Input Plugin Architecture"
description: "Documentation for Input Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-14
---

# Input Plugin Architecture

## Ownership Boundary

- `InputState` owns confirmed backend-neutral physical/device state plus current non-action compatibility projections and accepted frame-local keyboard press evidence.
- `ActionState` owns Runenwerk product action ids, bindings/defaults, rebinding, and held/pressed action projection.
- Product binding semantic types use `PhysicalKeyIdentity`; they do not own or expose winit key types.
- Gameplay, editor, scene, RunenUI semantics, window lifecycle, and native-tablet policy remain outside this ownership boundary.

## Module Layout

- Primary module: engine/src/plugins/input/mod.rs
- Physical/device state: engine/src/plugins/input/state.rs
- Product action/binding projection: engine/src/plugins/input/actions_and_bindings.rs
- Entry surface: InputFinalizePlugin
- Runtime schedule touchpoints: `PreUpdate` (`CoreSet::Input`) and `FrameEnd` (`CoreSet::FrameEnd`)

## Runtime Coupling

- Backend/platform adapters normalize observations before `InputState` admission.
- `ActionState` derives from `InputState`; it is not a second writable physical-state reducer.
- Product action consumers depend on `ActionState`, while pointer/text/touch consumers continue to use their owning `InputState` projections.
- Cross-plugin coupling remains data-oriented through typed resources and schedule ordering.
- Architecture changes should stay narrow and avoid broad app, RunenUI, tablet, or repository extraction redesign.
