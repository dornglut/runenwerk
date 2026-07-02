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

Title: Overlay / Popup / Layering full implementation

State: active implementation on PR #44 using the full canonical overlay/popup/layering design scope

Lifecycle state: `active-implementation`

Owner: `ui_controls` owns reusable overlay declarations, ergonomic builders, base-control lowering, package descriptors, package validation, catalog projection, and inspection projection. `ui_input` owns normalized input facts only. `ui_runtime` owns package-backed runtime overlay intent formation, overlay session/stack state, placement, layer/focus/dismissal evidence, deterministic replay/report proof, renderer-neutral proof data, proof-frame projection, and no-bypass assertions through `ui_runtime::overlay`, not through `ui_runtime::input`. `ui_static_mount` owns static frame validation.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `ARCHITECTURE.md`, `DEPENDENCY_RULES.md`, `DOMAIN_MAP.md`, `TESTING.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/completed-work.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, and `docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md`.

Write scope: Continue PR #44 into the full Phase 13 implementation. Implement package-backed reusable overlay / popup / layering: control vocabulary, base-control lowering, package descriptors, package validation, catalog projection, inspection projection, runtime package-backed replay/report/proof, proof-frame projection, and static mount validation. Runtime overlay files must live under `domain/ui/ui_runtime/src/overlay/`. Do not implement UI Gallery, UI Designer, authored UI editing, command execution, product/editor/game mutation, full text editing, generic plugin framework, `foundation/meta`, shared plugin primitives, broad Workbench/provider redesign, backend renderer behavior, or world-space overlays.

Validation expectation: Run the full Phase 13 gate from the canonical overlay design when a local checkout is available, including `cargo fmt --all --check`, focused `cargo check`, package/catalog/inspection overlay tests, runtime package-backed overlay tests, static mount tests, docs validation, and `git diff --check`. Connector-only work must report command validation as unavailable and name manual validation performed.

Known blockers: Stop and revise scope if implementation requires files outside the canonical expected file list, editor shell surface registration, Workbench provider redesign, command execution in generic UI, product/editor/game mutation in generic UI, app-specific modal lifecycle in generic UI, full text editing, dynamic plugin framework, `foundation/meta`, generic plugin primitives, phase-shaped public API names, compatibility-only aliases/shims, or overlay behavior below `ui_runtime::input`.

Next action: Validate the implementation locally, fix compile/test/docs issues, and only then record closeout evidence. Do not mark the phase completed until implementation and validation evidence are recorded.

Evidence: PR #43 is closed and merged into `main` at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User correction on 2026-07-01 requires PR #44 to close the full Phase 13, not merge as design-only or substrate-only.

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
