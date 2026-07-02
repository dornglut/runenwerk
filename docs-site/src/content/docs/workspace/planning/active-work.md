---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-014`

Title: Text Editing / Editable Text Behavior

State: review after local Phase 14 implementation validation

Lifecycle state: `review`

Owner: `ui_controls` owns package-backed editable-text declarations, base-control text-editing lowering, package descriptors, package validation, catalog projection, and inspection projection. `ui_input` owns normalized keyboard, text, composition, focus, and selection facts only. `ui_runtime` owns renderer-neutral text-editing replay/report/caret/selection/composition/edit-intent/validation/suppression/focus/no-bypass evidence and proof-frame projection. `ui_static_mount` owns static frame validation. Host/product/editor/game layers own actual persistence, domain mutation, command routing, authored UI editing, app-specific editor policy, and product undo stacks.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-text-editing-design.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`.

Write scope: Review the local Phase 14 implementation branch. The implementation scope is package-backed editable-text declaration vocabulary, descriptor wiring, package validation, base-control lowering for semantically editable `InspectorField`, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/proof-frame evidence, static mount validation, focused tests, and planning closeout. Do not add product/editor/game mutation, command execution, authored UI editing, UI Gallery, UI Designer, Workbench/provider redesign, rich text editor, code editor, app-specific undo/redo, dynamic plugin framework, `foundation/meta`, shared plugin primitives, overlay runtime changes unrelated to text editing, compatibility-only aliases/shims, or phase-shaped public API names.

Validation expectation: The implementation gate is recorded in `ui-component-platform-text-editing-design.md` and includes focused checks/tests for `ui_controls`, `ui_input`, `ui_runtime`, `ui_static_mount`, `ui_story`, `runenwerk_editor`, text-editing package/catalog/inspection/runtime/static proof, docs validation, and `git diff --check`.

Known blockers: No local implementation blocker remains. Completion remains blocked until the Phase 14 branch is accepted or merged and completion truth is recorded. Product/editor/game mutation, command execution, authored UI editing, UI Gallery, UI Designer, Workbench/provider redesign, rich text editor behavior, code editor behavior, dynamic plugin framework, `foundation/meta`, shared plugin primitives, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Next action: Review the Phase 14 implementation branch. After acceptance or merge, record completion truth before opening Phase 15.

Evidence: Main was inspected after PR #44 merge and is identical to merge commit `6f2d3827f315191d7aeaf68a64f523627197cad8`. Phase 13 completed evidence is recorded in completed work, roadmap, production track, decision register, and the overlay design. The 2026-07-02 user handoff supplied exact owner files, validation envelope, evidence expectations, and stop conditions for local implementation. The local branch implements package-backed editable text through `ui_controls`, `ui_input`, `ui_runtime::text_editing`, and `ui_static_mount` without product/editor/game mutation.

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
