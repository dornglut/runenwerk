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

ID: `PT-UI-COMPONENT-PLATFORM-005`

Title: UI Component Platform Input / Gesture / Device

State: implementation pending local validation

Owner: ui

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/design/active/ui-component-platform-input-gesture-device-design.md
docs-site/src/content/docs/design/active/ui-component-platform-catalog-discovery-inspection-design.md
docs-site/src/content/docs/domain/ui/roadmap.md
```

Write scope:

```text
domain/ui/ui_controls/src/input.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/mod.rs
domain/ui/ui_controls/src/catalog/entry.rs
domain/ui/ui_controls/src/catalog/index.rs
domain/ui/ui_controls/src/catalog/inspection.rs
domain/ui/ui_controls/src/catalog/query.rs
domain/ui/ui_controls/tests/control_input_contract.rs
domain/ui/ui_controls/tests/control_input_catalog_contract.rs
docs-site/src/content/docs/workspace/planning/active-work.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_controls
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
Connector-mode command execution is unavailable. Phase 5 cannot be marked complete until the validation gate is run locally and reported green.
```

Next action:

```text
Run the Phase 5 validation gate locally. If it passes, run phase-completion drift check and mark Phase 5 complete.
```

Evidence:

```text
Phase 4 Catalog / Discovery / Inspection passed local validation by user report on 2026-06-26 after rebasing or merging latest main and rerunning the validation gate.
Phase 5 design authority was accepted by user report on 2026-06-26.
A bounded Phase 5 input/gesture/device declaration implementation exists on this branch.
Catalog was split by stable responsibility during Phase 5 to keep the inspection bridge maintainable.
Package façade now re-exports catalog and input declarations without duplicating the input module.
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
