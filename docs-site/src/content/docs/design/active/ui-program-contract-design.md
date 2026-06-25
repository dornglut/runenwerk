---
title: UI Program Contract Design
description: Current truth contract for the bounded UiProgram proof-slice evidence and the remaining concrete architecture implementation gap.
status: active
owner: ui
layer: domain
canonical: false
last_reviewed: 2026-05-31
related:
  - ./runenwerk-domain-workbench-north-star.md
  - ./ui-program-architecture.md
  - ./ui-program-proof-slice-plan.md
  - ../../reports/audits/pt-ui-program-truth-claim-correction.md
---

# UI Program Contract Design

## Status

This is the current truth and supersession contract for the completed
`PT-UI-PROGRAM` proof-slice track.

It is not retroactive evidence for `PM-UI-PROGRAM-002`, does not rewrite
historical closeouts, and does not authorize product code, crate creation,
MaterialProgram work, or shared `foundation/meta` extraction.

## What The Completed Track Proves

`PT-UI-PROGRAM` proves bounded retained-compatible UI proof slices:

- `Label` structural `UiFrame` text proof;
- `Button` route/event/host-command proof;
- `InspectorField` binding/state proof;
- `ColorPicker` wheel-plus-triangle rich control proof;
- world-space anchored prompt host-boundary proof;
- headless fixture, diagnostics, source-map, artifact, and reproducibility
  aggregation evidence.

Those slices are valid runtime/test evidence for the bounded surfaces they
touch. They are not evidence that the final long-term UiProgram architecture
exists in code.

## What Remains Unproven

The following long-term architecture contracts remain design targets, not
implemented architecture truth:

- `UiProgram` as a durable program contract;
- `ControlGraph`, `LayoutGraph`, `StateGraph`, `StyleGraph`,
  `InteractionGraph`, `BindingGraph`, `VisualGraph`,
  `AccessibilityGraph`, and `InspectionGraph` as concrete graph families;
- `UiCompiler` as the source-to-runtime-artifact compiler;
- `UiRuntimeArtifactManifest` as durable inspectable artifact metadata;
- `UiRuntimeArtifactTables` as optimized runtime tables;
- `UiEvaluator` as the frame/input/tick evaluator over runtime artifacts;
- final editor, game, world-space, and headless host contracts around those
  artifacts.

The current code remains a retained-compatible proof path. It contains useful
slice evidence, but it is not the final `domain/ui/ui_program`,
`domain/ui/ui_compiler`, `domain/ui/ui_artifacts`, or
`domain/ui/ui_evaluator` architecture.

## Required Future Architecture Truth

An `architecture_runtime_proven` claim for UiProgram requires a separate
accepted implementation track, expected to be named
`PT-UI-PROGRAM-ARCHITECTURE` unless a later accepted decision chooses a better
name.

That future track must prove:

- exact domain-owned module or crate boundaries for the final UI program
  architecture;
- concrete public contracts for `UiProgram`, `UiCompiler`,
  `UiRuntimeArtifactManifest`, `UiRuntimeArtifactTables`, and `UiEvaluator`;
- graph-family ownership and migration behavior;
- runtime artifact generation and evaluation through executable tests;
- source maps, diagnostics, fixtures, and compatibility evidence;
- retained UI migration rules and rollback behavior.

## Downstream Gate

MaterialProgram remains the required second-domain platform proof, but it is
blocked by default until the UiProgram architecture truth gap is resolved or a
later accepted ADR explicitly changes the gate.

Bounded UI proof slices may inform MaterialProgram design. They do not, by
themselves, authorize MaterialProgram proof planning, implementation, crate
creation, or shared platform extraction.

<!-- BEGIN RUNENWERK:UI_COMPONENT_PLATFORM:uiprogram-alignment -->
## Relationship to UI Component Platform

`PT-UI-COMPONENT-PLATFORM` aligns with the UiProgram owner map. `ui_controls`, `ui_program`, `ui_artifacts`, `ui_evaluator`, `ui_binding`, `ui_state`, `ui_hosts`, `ui_accessibility`, `ui_geometry`, `ui_text`, `ui_theme`, `ui_input`, `ui_layout`, `ui_render_data`, and `ui_story` remain bounded owners. Component Platform docs define reusable package contracts and proof requirements; UiProgram proof slices remain bounded and must not become product-specific bypasses.
<!-- END RUNENWERK:UI_COMPONENT_PLATFORM:uiprogram-alignment -->
