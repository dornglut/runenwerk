---
title: UI Component Platform Executable Interaction Story Design
description: Implemented Tier-5 reusable interaction proof contract with shared replay/live semantics, proof-host evidence, static frame validation, and no-bypass assertions.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
  - ../../reports/closeouts/pt-ui-component-platform-012a-executable-interaction-story-implementation-scope-closeout.md
  - ./ui-component-platform-generic-interaction-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
  - ./ui-component-platform-input-gesture-device-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
---

# UI Component Platform Executable Interaction Story Design

## Status

Implemented through the Phase 12/12A interaction delivery. This is the current proof-design reference; historical branch/phase acceptance details remain in the retained closeout.

## Implemented proof standard

Reusable interaction claims use one semantic path after normalized input:

```text
replay input OR live proof-host input
  -> NormalizedInputSample
  -> InteractionStorySession::apply_sample
  -> InteractionFormationReport
  -> InteractionVisualProof
  -> InteractionProofRenderFrame
  -> UiStaticMountReport
```

Replay and live proof may differ in input source only. Target/focus resolution, state transitions, reusable facts/events/outcomes, suppression/no-target evidence, visual proof, and no-bypass assertions share the same runtime formation path.

## Landed public/runtime concepts

Current durable concepts include:

```text
BASE_CONTROLS_GENERIC_INTERACTION_PROOF_ID
BASE_CONTROLS_EXECUTABLE_INTERACTION_STORY_ID
InteractionStoryExecutionMode
InteractionStorySession
InteractionStoryRunReport
InteractionStoryStepEvidence
InteractionReplayLiveParityReport
BaseControlsInteractionProofHost
```

Durable `base_controls_*` names replaced phase-shaped implementation labels. No compatibility aliases are required.

## Ownership

- `ui_controls` owns reusable interaction declarations.
- `ui_input` owns normalized input facts.
- `ui_runtime` owns session execution, replay/live formation, reports, visual proof, and frame projection.
- `ui_story` owns the broader story/evidence envelope.
- `ui_static_mount` owns static `UiFrame` validation.
- app/editor/game hosts own raw event collection, commands, product mutation, persistence, overlays beyond the owned overlay contract, and product policy.

## Proof boundary

The implemented proof establishes semantic replay/live parity rather than pixel-perfect equality. Wall-clock timing, OS event ids, animation interpolation, and incidental primitive counts are not semantic parity criteria.

No interaction outcome executes a host command or mutates product truth inside the reusable proof path.

## Evidence

Current `ui_runtime` interaction session/replay code, the editor proof-host adapter, parity tests, generic-interaction tests, and static-mount tests are implementation authority. Product-facing Gallery exposure remains separately owned and is not implied by this lifecycle move.
