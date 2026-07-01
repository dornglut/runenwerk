---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-01
related_docs:
  - ../workflow-lifecycle.md
  - ../../design/active/ui-component-platform-generic-interaction-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-design.md
  - ../../design/active/ui-component-platform-executable-interaction-story-implementation-scope.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering substrate implementation

State: active implementation on PR #44 after accepted overlay/popup/layering design scope

Lifecycle state: `active-implementation`

Owner: `ui_controls` may declare reusable overlay/open intent requirements and ergonomic descriptor builders only. `ui_input` owns normalized input facts only. `ui_runtime` may own runtime overlay intent formation, overlay session/stack state, placement, layer/focus/dismissal evidence, deterministic replay/report proof, renderer-neutral proof data, and no-bypass assertions. `ui_static_mount` owns static frame validation. `runenwerk_editor` may contain only narrow proof/test adapters if implementation inspection proves one is required.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/completed-work.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, `docs-site/src/content/docs/design/implemented/editor-self-authoring-and-final-ui-design.md`, and `docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md`.

Write scope: Continue PR #44 from accepted design intake into implementation. Implement the reusable overlay / popup / layering substrate only. Do not implement UI Gallery, UI Designer, authored UI editing, command execution, product/editor/game mutation, full text editing, generic plugin framework, `foundation/meta`, shared plugin primitives, broad Workbench/provider redesign, backend renderer behavior, or world-space overlays.

Validation expectation: Run the accepted Phase 13 gate when a local checkout is available: `cargo fmt --all --check`, focused `cargo check` for `ui_controls`, `ui_input`, `ui_runtime`, `ui_static_mount`, `ui_story`, `runenwerk_editor`, focused overlay/input/runtime/static/story/editor tests, `python tools/docs/validate_docs.py`, and `git diff --check`. Connector-only work must report command validation as unavailable and name manual validation performed.

Known blockers: Stop and revise scope if implementation requires files outside the accepted expected file list, editor shell surface registration, Workbench provider redesign, command execution in generic UI, product/editor/game mutation in generic UI, app-specific modal lifecycle in generic UI, full text editing, dynamic plugin framework, `foundation/meta`, generic plugin primitives, phase-shaped public API names, or compatibility-only aliases/shims.

Next action: Implement the scoped owner-crate overlay declarations, normalized input facts, runtime overlay session/stack/report/proof frame, static mount evidence, tests, and planning closeout evidence on PR #44. Do not mark the phase completed until implementation and validation evidence are recorded.

Evidence: PR #43 is closed and merged into `main` at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User start condition reports PR #43 validated and merged. PR #44 records the Phase 13 design intake and the follow-up ergonomics/flexibility/extensibility pass. User correction on 2026-07-01 requires PR #44 to close the full Phase 13, not merge as design-only.

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
