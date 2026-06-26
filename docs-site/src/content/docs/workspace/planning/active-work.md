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

ID: `PT-UI-COMPONENT-PLATFORM-009`

Title: UI Component Platform Layout / Container / Virtualization

State: implementation pending local validation

Owner: ui

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-layout-container-virtualization-design.md
docs-site/src/content/docs/domain/ui/roadmap.md
```

Write scope:

```text
domain/ui/ui_layout/src/contracts.rs
domain/ui/ui_layout/src/lib.rs
domain/ui/ui_layout/Cargo.toml
domain/ui/ui_layout/tests/layout_contract.rs
domain/ui/ui_controls/src/layout.rs
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/catalog/layout.rs
domain/ui/ui_controls/src/catalog/mod.rs
domain/ui/ui_controls/Cargo.toml
domain/ui/ui_controls/tests/control_layout_contract.rs
domain/ui/ui_controls/tests/control_layout_catalog_contract.rs
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-layout-container-virtualization-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_layout
cargo check -p ui_controls
cargo test -p ui_layout layout
cargo test -p ui_controls control_layout
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
Connector command execution is unavailable. Phase 9 needs local validation before completion.
```

Next action:

```text
Run the Phase 9 validation gate locally. If green, close Phase 9 and open Phase 10 Render Surface / Output planning.
```

Evidence:

```text
009A ownership realignment was accepted by user direction on 2026-06-26.
009B ui_layout layout foundation implementation exists on this branch.
009C ui_controls control layout bridge implementation exists on this branch.
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
