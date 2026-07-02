---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-02
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-015`

Title: Generic Text

State: active planning for renderer-neutral reusable text display and layout proof

Lifecycle state: `active-planning`

Owner: `ui_text` owns renderer-neutral text layout contracts, text runs, line metrics, glyph/run evidence, wrapping, alignment, truncation, and overflow vocabulary. `ui_controls` owns package-backed generic-text declarations, package validation, catalog projection, and inspection projection for reusable controls that display text. `ui_runtime` owns renderer-neutral visual proof-frame projection that consumes generic-text evidence without owning renderer backend policy. `ui_static_mount` owns static validation of the proof frame. Host/product/editor/game layers own app-specific copy policy, document buffers, authored UI editing, text editing, code editing, localization policy, font asset provisioning, and renderer backend implementation.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-text-editing-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-text-design.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/workspace/planning/completed-work.md`.

Write scope: Design intake for reusable renderer-neutral text display/layout proof. The planning scope is text runs, inline spans, wrapping, alignment, truncation/ellipsis, line metrics, glyph/run evidence, package/catalog/inspection projection, visual proof, and static mount proof. Prefer docs-only planning until owner files, implementation scope, validation, evidence expectations, and stop conditions are exact.

Validation expectation: Planning docs must pass `python tools/docs/validate_docs.py` and `git diff --check`. A later implementation gate must include focused checks/tests for `ui_text`, `ui_controls`, `ui_runtime`, `ui_static_mount`, `ui_story`, and any package/catalog/inspection/runtime/static proof that Phase 15 changes.

Known blockers: No Phase 14 blocker remains. Phase 15 is not yet implementation-authorized. Text editing, rich text editor behavior, code editor behavior, product document buffers, undo/redo, clipboard, LSP/syntax highlighting, app-specific text rendering policy, dynamic plugin framework, `foundation/meta`, shared plugin primitives, UI Designer, UI Gallery product surface, Workbench/provider redesign, product/editor/game mutation, command execution, authored UI editing, compatibility-only aliases/shims, and phase-shaped public API names remain out of scope.

Next action: Review and refine the Phase 15 Generic Text design intake. Do not start implementation until active planning is promoted with exact owner files, implementation scope, validation, evidence, and stop conditions.

Evidence: Phase 14 is completed through merged PR #46 at merge commit `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Main was inspected after PR #46 merge and is identical to that merge commit. Main contains package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, `ui_runtime::text_editing` replay/report/value/caret/selection/composition/suppression/no-bypass proof, proof-frame projection, static mount validation, and focused tests. Local Phase 14 validation passed on 2026-07-02 with the recorded cargo/docs/diff gate before merge. The Phase 15 scope starts from the completed Phase 14 text-editing substrate but is display/layout-only.

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
