---
title: "Input Plugin Architecture"
description: "Documentation for Input Plugin Architecture."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-24
---

# Input Plugin Architecture

## Ownership Boundary

- The internal neutral input owner owns backend-neutral observation semantics, physical-control identity correlation, deterministic admission/reduction, and confirmed held/contact state.
- `InputState` is the Runenwerk integration/projection shell around that owner; it retains non-action compatibility projections and accepted frame-local keyboard press evidence but no independent physical-control interner or reducer.
- `ActionState` owns Runenwerk product action ids, bindings/defaults, rebinding, and held/pressed action projection.
- Product binding semantic types use `PhysicalKeyIdentity`; they do not own or expose winit key types.
- Gameplay, editor, scene, RunenUI semantics, window lifecycle, and native-tablet policy remain outside this ownership boundary.

## Module Layout

- Primary module: engine/src/plugins/input/mod.rs
- Neutral semantic/reducer owner: engine/src/plugins/input/neutral.rs
- Runenwerk physical/device integration and compatibility projections: engine/src/plugins/input/state.rs
- Product action/binding projection: engine/src/plugins/input/actions_and_bindings.rs
- Entry surface: InputFinalizePlugin
- Runtime schedule touchpoints: `PreUpdate` (`CoreSet::Input`) and `FrameEnd` (`CoreSet::FrameEnd`)

## Runtime Coupling

- Backend/platform adapters normalize observations before the `InputState` integration shell forwards them to the self-contained neutral owner.
- The neutral owner, not `InputState`, correlates normalized keyboard/pointer controls with confirmed digital-control state.
- Runenwerk-only compatibility loss remains explicit through reusable unspecified domains rather than predecessor-named neutral semantics.
- `ActionState` derives from `InputState`; it is not a second writable physical-state reducer.
- Product action consumers depend on `ActionState`, while pointer/text/touch consumers continue to use their owning `InputState` projections.
- Cross-plugin coupling remains data-oriented through typed resources and schedule ordering.
- Architecture changes should stay narrow and avoid broad app, RunenUI, tablet, or repository extraction redesign.
