---
title: UI Component Platform Input Gesture And Device Design
description: Implemented Runenwerk-local per-control input, gesture, device, pointer, keyboard, wheel, text-input, and semantic-action declaration contract.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-catalog-discovery-inspection-design.md
  - ./ui-component-platform-state-binding-host-intent-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Input Gesture And Device Design

## Status

Implemented as the current Runenwerk-local control-facing declaration model. Runtime normalized input formation and host device collection remain separate owners.

## Implemented contract

`domain/ui/ui_controls/src/input.rs` provides:

- `ControlInputMode` and `ControlInputModeSet`;
- `ControlGestureKind` and `ControlGestureRequirement`;
- `ControlDeviceKind` and `ControlDeviceRequirement`;
- pointer, keyboard, wheel, and text-input requirement records;
- `ControlSemanticActionRequirement`;
- `ControlInputDescriptor`;
- deterministic capability/inspection summaries.

Implemented input modes include pointer, wheel, keyboard, semantic action, text input, touch-ready, controller, and stylus/tablet. Gesture declarations include hover, press, drag, marquee selection, multi-click, cancel, commit, rollback, pointer capture, and lost capture. Device facts include pressure, tilt, twist, tangential pressure, eraser, barrel button, coalesced samples, and predicted samples.

## Ownership

`ui_controls` owns per-control capability declarations. `ui_input` owns normalized runtime input facts/samples. App/editor/game/platform layers own OS/window event collection, product input policy, command routing, and mutation.

The still-active ownership-realignment work may further normalize generic vocabulary ownership; this implemented classification records the current contract without claiming that broader framework extraction has already happened.

## Boundary rules

- Declarations are package facts, not runtime event handlers.
- Gesture requirements are not recognizers or command executors.
- Device facts are normalized requirements, not raw OS packet ownership.
- Pointer-capture requirements do not themselves perform capture.
- Product/game/world input policy remains host-owned.
- No input declaration grants runtime mount eligibility.

## Evidence

Current `ui_controls/src/input.rs`, base-control lowering/projection, catalog/inspection projection, and focused input/catalog tests establish implementation parity with the planned capability set.
