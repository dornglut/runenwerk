---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-28
related_docs:
  - ../workflow-lifecycle.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-011`

Title: UI Component Platform Base Control Packages implementation

State: review; local validation green

Lifecycle state: `review`

Owner: `ui_controls` owns base control package descriptors, UI-local contribution declarations, preset/lowering code, catalog projection, and read-only inspection facts. `ui_layout`, `ui_render_data`, `ui_input`, `ui_theme`, and accessibility/focus contracts remain owner crates that base controls reference. Full runtime interaction behavior remains Phase 12 scope.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/routines/code-refactor-routine.md`, `DOMAIN_MAP.md`, `CRATES.md`, `DEPENDENCY_RULES.md`, `TESTING.md`, crate inventory, authority model, programming principles, Phase 9 ownership realignment design, Phase 10 render surface/output design, typed app-composition/plugin framework direction, and the Phase 11 base control packages design.

Write scope: rework Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView as UI-local control contributions lowered through `BaseControlsPlugin`, `UiControls`, `ControlContribution`, `ControlDef`, control presets, field groups, theme groups, `ControlCompiler`, `ControlCatalog`, and `ControlInspection`. Do not implement pointer/keyboard runtime behavior, hover/pressed/selected runtime state machines, overlays, text editing, runtime mount eligibility, authored-file rendering, backend renderer behavior, or app/editor/game-specific reusable-control behavior.

Validation expectation: local checkout validation has run green for the Phase 11 gate. Keep Phase 11 in review until the implementation patch is accepted or merged.

Known blockers: Code review and merge are pending. Phase 11 completion has not been recorded in completed work.

Next action: Review and merge the Phase 11 implementation patch, then record completion only after the accepted patch remains green. Do not start Phase 12 until Phase 11 is completed.

Evidence: Phase 10 is complete through PR #34 merged into `main`, and the user reported the full P10 validation gate green on 2026-06-26. On 2026-06-28, the Phase 11 implementation patch locally passed `cargo fmt --all --check`, `cargo check -p ui_controls`, focused `ui_controls` package/catalog/layout/render/base_control tests, docs validation, and `git diff --check`.

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
