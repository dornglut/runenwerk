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
  - ../../design/active/ui-component-platform-gallery-visible-interaction-story-implementation-scope.md
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-012B`

Title: UI Component Platform Gallery Visible Interaction Story

State: implementation scope accepted / active implementation

Lifecycle state: `active-implementation`

Owner: `ui_runtime` remains the reusable interaction proof/session authority through `InteractionStorySession` and `Phase12aInteractionProofHost`. `runenwerk_editor` owns only the concrete Workbench/UI Lab exposure: compiled-in tool-suite surface metadata, provider-family bridge, per-mounted surface session storage, visible retained UI projection, and focused validation. `editor_shell` remains the app-neutral Workbench contract owner. Product/editor/app mutation, overlay behavior, full text editing, and legacy `ToolSurfaceKind` identity are out of scope.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/documentation-structure.md`, `docs-site/src/content/docs/workspace/authority-model.md`, `docs-site/src/content/docs/workspace/workflow-lifecycle.md`, `docs-site/src/content/docs/workspace/planning/README.md`, `docs-site/src/content/docs/guidelines/programming-principles.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-design.md`, `docs-site/src/content/docs/design/active/ui-component-platform-executable-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/ui-component-platform-gallery-visible-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/editor-tool-suite-registry-and-workbench-host-design.md`, and `docs-site/src/content/docs/design/active/runenwerk-capability-workbench-target-architecture.md`.

Write scope: Implement only the files named as authorized write files in `ui-component-platform-gallery-visible-interaction-story-implementation-scope.md`: `apps/runenwerk_editor/src/shell/tool_suites/mod.rs`, `apps/runenwerk_editor/src/shell/tool_suites/ui_lab_tool_suite.rs`, `apps/runenwerk_editor/src/shell/workbench_host.rs`, `apps/runenwerk_editor/src/shell/surface_session.rs`, `apps/runenwerk_editor/src/shell/providers/m6_workspace.rs`, `apps/runenwerk_editor/tests/phase12b_ui_lab_interaction_story_surface.rs`, `docs-site/src/content/docs/design/active/ui-component-platform-gallery-visible-interaction-story-implementation-scope.md`, `docs-site/src/content/docs/design/active/README.md`, and `docs-site/src/content/docs/workspace/planning/active-work.md`.

Validation expectation: Run and record focused validation from the implementation scope: `cargo fmt --all --check`, `cargo check -p runenwerk_editor`, `cargo test -p runenwerk_editor phase12b_ui_lab_interaction_story_surface`, `cargo test -p runenwerk_editor phase12a_interaction_proof_host`, `cargo test -p ui_runtime executable_interaction_story`, `cargo test -p ui_static_mount phase12_executable_interaction_story`, `python tools/docs/validate_docs.py`, and `git diff --check`. If Windows MSVC PDB limits block plain workspace tests, run `CARGO_PROFILE_DEV_DEBUG=0 cargo test --workspace` and record that mapping.

Known blockers: The visible retained UI projection is implemented through the existing concrete provider bridge because the current provider registry still uses numeric runtime provider ids. Numeric provider ids remain hidden current-code plumbing, not durable identity. A future Workbench/provider-registry cleanup should replace numeric provider handles with stable provider identity or derived runtime indices.

Next action: Validate the Phase 12B UI Lab stable-key surface locally, then update PR #43 body and closeout language with the visible proof status and the remaining provider-id ergonomics follow-up.

Evidence: User requested closing the Phase 12 visible proof gap and challenged legacy/numeric-provider ergonomics. The implementation scope now records the stable-key UI Lab route and explicitly forbids legacy `ToolSurfaceKind` identity, fake panel-owned interaction state, product mutation, overlays, text editing, dynamic plugins, and treating numeric provider ids as durable user-facing identity.

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
