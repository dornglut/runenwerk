---
title: UI Component Platform Generic Interaction Design
description: Implemented Runenwerk-local generic interaction semantics across package declarations, normalized input, runtime formation, visible proof, and static mount evidence.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../reports/closeouts/pt-ui-component-platform-012-generic-interaction-closeout.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./ui-component-platform-executable-interaction-story-design.md
  - ./ui-component-platform-overlay-popup-layering-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
  - ./ui-component-platform-base-control-packages-design.md
---

# UI Component Platform Generic Interaction Design

## Status

Implemented through the current Runenwerk-local base-controls interaction path.

## Implemented flow

```text
ui_controls interaction declarations
  -> ui_input normalized facts
  -> ui_runtime descriptor-backed interaction formation
  -> InteractionVisualProof
  -> InteractionProofRenderFrame / UiFrame
  -> ui_static_mount validation
```

`ui_controls` provides package-backed interaction requirements, states, triggers, outcomes, support summaries, catalog projection, and inspection projection. `ui_input` supplies normalized pointer/keyboard/focus/semantic/text-intent facts. `ui_runtime` owns mounted fixtures, target/focus resolution, replay/session state, reports, proof formation, and deterministic frame projection.

## Base-control semantics

The proof covers inert display, pointer hover/press/release/capture, focus/focus-visible, keyboard activation, action intent, list/tree/table navigation intent, text-intent probes, disabled/read-only suppression, no-target evidence, and release-outside cancellation behavior.

These outcomes are reusable semantic intents and evidence. They do not execute product commands or mutate product selection/data.

## Landed naming

Current public APIs and proof helpers use durable domain names such as `base_controls_*`, `InteractionStorySession`, `InteractionFormationReport`, `InteractionVisualProof`, and `BaseControlsInteractionProofHost`. Phase-shaped aliases are not current authority.

## Boundary assertions

Reusable generic interaction must keep these categories outside the proof path:

```text
host command execution
product/editor/game mutation
overlay-specific behavior unless delegated to the implemented overlay owner
full editable-text transactions
backend renderer behavior
```

The runtime proof records no-bypass counters/evidence for these boundaries.

## Evidence

Current `ui_controls` interaction declarations, `ui_input` normalized facts, `ui_runtime/src/input/generic_interaction/`, editor proof-host integration, focused interaction replay/parity tests, and static-mount tests establish implementation parity. Retained closeouts preserve delivery history.
