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

ID: `PT-UI-COMPONENT-PLATFORM-010B-010D`

Title: UI Component Platform Render Output Evidence Foundation / Control Bridge

State: implementation pending local validation

Owner: `ui_render_data` owns renderer-neutral output evidence contracts. `ui_controls` owns the bridge that references those contracts. `ui_runtime` and `engine/src/plugins/render` are not implementation owners in this slice.

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
DOMAIN_MAP.md
CRATES.md
docs-site/src/content/docs/workspace/crate-inventory.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/guidelines/programming-principles.md
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-render-surface-output-design.md
docs-site/src/content/docs/design/active/ui-runtime-rendering-pipeline-roadmap.md
docs-site/src/content/docs/domain/ui/architecture.md
docs-site/src/content/docs/domain/ui/roadmap.md
```

Write scope:

```text
domain/ui/ui_render_data/Cargo.toml
domain/ui/ui_render_data/src/lib.rs
domain/ui/ui_render_data/src/output_evidence.rs
domain/ui/ui_render_data/tests/render_output_evidence.rs
domain/ui/ui_controls/Cargo.toml
domain/ui/ui_controls/src/lib.rs
domain/ui/ui_controls/src/package.rs
domain/ui/ui_controls/src/render.rs
domain/ui/ui_controls/src/catalog/mod.rs
domain/ui/ui_controls/src/catalog/render.rs
domain/ui/ui_controls/tests/control_render_contract.rs
domain/ui/ui_controls/tests/control_render_catalog_contract.rs
docs-site/src/content/docs/workspace/planning/active-work.md
```

Validation expectation:

```text
cargo fmt --all --check
cargo check -p ui_render_data
cargo check -p ui_controls
cargo test -p ui_render_data render_output
cargo test -p ui_controls control_render
cargo test -p ui_controls control_layout
cargo test -p ui_controls control_catalog
git diff --check
```

Known blockers:

```text
Connector command execution is unavailable. This implementation slice needs local validation before completion.
```

Next action:

```text
Run focused local validation. If green, close 010B/010D as the renderer-neutral output evidence foundation and control bridge. Do not proceed to ui_runtime output evidence or backend proof until this slice is validated.
```

Evidence:

```text
010B adds renderer-neutral primitive-family, expected-count, frame/surface summary, provenance, diagnostic, and output evidence contracts in ui_render_data.
010D adds a ui_controls ControlRenderDescriptor and read-only catalog bridge that reference ui_render_data vocabulary directly.
No backend renderer behavior, runtime output generation, mount eligibility change, or generic render/output vocabulary in ui_controls is introduced.
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
