---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-30
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

Title: Overlay / Popup / Layering substrate design intake

State: active planning / design intake after PR #43 merge

Lifecycle state: `active-planning`

Owner: `ui_controls` may declare reusable overlay/open intent requirements only. `ui_input` owns normalized input facts only. `ui_runtime` may own runtime overlay intent formation, layer/focus/dismissal evidence, deterministic replay/report proof, and renderer-neutral proof data. `ui_static_mount` owns static frame validation. `runenwerk_editor` may contain only narrow proof/test adapters if a later accepted implementation scope requires one.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/planning/active-work.md`, `docs-site/src/content/docs/workspace/planning/roadmap.md`, `docs-site/src/content/docs/workspace/planning/production-tracks.md`, `docs-site/src/content/docs/workspace/planning/completed-work.md`, `docs-site/src/content/docs/workspace/planning/decision-register.md`, `docs-site/src/content/docs/design/active/ui-component-platform-generic-interaction-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/editor-ui-runtime-v2-and-interaction-formation-design.md`, `docs-site/src/content/docs/design/implemented/editor-self-authoring-and-final-ui-design.md`, and `docs-site/src/content/docs/design/active/ui-component-platform-overlay-popup-layering-design.md`.

Write scope: Documentation/planning only. First close out Phase 12 and 12A from merged PR #43 evidence, then revise the active overlay/popup/layering design. Do not implement Rust, UI Gallery, UI Designer, authored UI editing, command execution, product/editor/game mutation, full text editing, generic plugin framework, `foundation/meta`, shared plugin primitives, or broad Workbench/provider redesign in this planning pass.

Validation expectation: For this docs-only intake, run `python tools/docs/validate_docs.py` and `git diff --check` when a local checkout is available. The Phase 13 design must also include the future Rust implementation gate with `cargo fmt --all --check`, focused `cargo check`, focused tests for `ui_controls`, `ui_input`, `ui_runtime`, `ui_static_mount`, any editor proof adapter, docs validation, and `git diff --check` before implementation can start.

Known blockers: No Rust implementation is authorized until the overlay/popup/layering design records exact owner crates/files, exact non-goals, proof scenarios, negative scenarios, expected evidence contract, no-bypass assertions, validation commands, and stop conditions. Product-facing UI Gallery and full UI Designer work require separate accepted plans.

Next action: Review and accept, revise, or reject `ui-component-platform-overlay-popup-layering-design.md`. After acceptance, create an implementation scope before changing Rust.

Evidence: PR #43 is closed and merged into `main` at merge commit `c8b73dfa95fc335fd2b33c9137cac03a0f35060f`. User start condition reports PR #43 validated and merged. The PR body records Phase 12 lower-tier reusable interaction proof and Phase 12A executable interaction story proof with durable base-controls names, while explicitly excluding overlays, popups, dropdowns, tooltips, modals, layering, product state mutation, command execution, text editing, dynamic plugin loading, and UI Gallery product exposure.

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
