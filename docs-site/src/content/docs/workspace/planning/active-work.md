---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-26
---

# Active Work

This file names the current planning focus for scriptless workflow.

## Current focus

ID: `PT-UI-COMPONENT-PLATFORM-008`

Title: UI Component Platform Accessibility / Focus / Inspection

State: implementation pending local validation

Owner: ui

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/design/active/ui-component-platform-accessibility-focus-inspection-design.md
docs-site/src/content/docs/design/active/ui-component-platform-theme-state-style-design.md
docs-site/src/content/docs/domain/ui/roadmap.md
```

Write scope:

```text
domain/ui/ui_controls/src/accessibility.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/tests/control_accessibility_contract.rs
domain/ui/ui_controls/tests/control_accessibility_catalog_contract.rs
docs-site/src/content/docs/design/active/ui-component-platform-accessibility-focus-inspection-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
cargo test -p ui_controls control_accessibility
cargo test -p ui_controls control_theme
cargo test -p ui_controls control_state
cargo test -p ui_controls control_input
cargo test -p ui_controls control_catalog
cargo test -p ui_controls control_package
cargo test -p ui_controls control_registry
cargo test -p ui_controls control_story_proof
cargo test -p ui_controls control_authoring
cargo test -p ui_artifacts control_package
cargo test -p ui_program route
git diff --check
```

Known blockers:

```text
Phase 8 needs local validation before completion.
```

Next action:

```text
Run the Phase 8 validation gate locally.
```

Evidence:

```text
Phase 7 Theme / State / Style passed local validation by user report on 2026-06-26.
Phase 8 design authority was accepted by user report on 2026-06-26.
```

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If legacy generated views disagree, report them as stale mirrors.

## Update shape

```text
ID:
Title:
State:
Owner:
Authority files:
Write scope:
Validation expectation:
Known blockers:
Next action:
Evidence:
```
