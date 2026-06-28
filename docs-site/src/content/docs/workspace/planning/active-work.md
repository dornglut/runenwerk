---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-012-PLANNING`

Title: UI Component Platform Generic Interaction design intake

State: active planning

Lifecycle state: `active-planning`

Owner: `ui_controls` may own reusable control interaction declarations, semantic interaction states needed by reusable controls, control-level interaction requirements, descriptor/catalog/inspection facts for interaction support, and control kernel hooks only as declarations/contracts. `ui_input` owns normalized input packet vocabulary, device/gesture facts, pointer/key/focus/text-intent facts as reusable input data, and runtime input sample formation. `ui_runtime` owns runtime interaction formation over emitted or mounted UI, normalized input resolution against runtime UI structure, interaction facts/events for reusable controls, and runtime frame/session evidence. Hosts, apps, editor, and game layers own OS/window input collection, routing policy, command execution, app/editor/game mutation, game/world input policy, and product-specific behavior.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-input-gesture-device-design.md`, `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, and the Phase 11 closeout report.

Write scope: planning and design intake only. Define owner boundaries, acceptance criteria, validation gate, stop conditions, and the exact later implementation PR envelope for generic reusable interaction. Preserve the focus, keyboard, and text-intent seams needed by later editable text controls. Do not change product code, do not implement interaction runtime behavior, do not add overlays/popups/layering, do not add full text editing behavior, do not add app/editor/game-specific mutation, do not extract a shared plugin framework, and do not introduce `foundation/meta`.

Validation expectation: for this planning patch, run `python3 tools/docs/validate_docs.py` and `git diff --check` when a local checkout is available. For the later implementation PR, use the validation gate recorded in the Phase 12 design intake.

Known blockers: Phase 12 is not implementation-authorized. The Phase 12 design intake must be reviewed and accepted before Rust write scope is opened. Existing Phase 5 input declarations and editor Interaction V2 runtime design are inputs, but they are not sufficient by themselves because Phase 12 must define component-platform reusable interaction boundaries across `ui_controls`, `ui_input`, `ui_runtime`, and hosts.

Next action: Review the Phase 12 Generic Interaction design intake and decide whether to authorize a narrow implementation PR. Keep overlays/popups/layering in Phase 13. Keep full text editing in a later phase, but require it to consume the focus, keyboard, text-intent, and runtime interaction substrate shaped by Phase 12.

Evidence: Phase 11 completed through merged PR #37 on 2026-06-28. The merged base-control proof covers Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView through `BaseControlsPlugin`, `UiControls`, `ControlContribution`, `ControlDef`, control presets, field groups, theme groups, `ControlCompiler`, `ControlCatalog`, and `ControlInspection`. Reported validation was green for the Phase 11 cargo/docs/diff gate.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, and stop conditions are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
