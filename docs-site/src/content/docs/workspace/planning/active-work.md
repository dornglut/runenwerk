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

ID: `PT-UI-COMPONENT-PLATFORM-010`

Title: UI Component Platform Render Surface / Output

State: implementation pending local validation

Owner: `ui_render_data` owns renderer-neutral output evidence contracts; `ui_controls` owns the control-facing bridge; `ui_runtime` owns evidence generation from emitted runtime frames; engine render owns submission proof without UI semantics.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/implementation-routine.md`, `DOMAIN_MAP.md`, `CRATES.md`, `DEPENDENCY_RULES.md`, `TESTING.md`, crate inventory, authority model, programming principles, Phase 10 design, UI runtime rendering roadmap, and UI architecture docs.

Validation expectation: run focused format, check, test, and diff validation for `ui_render_data`, `ui_controls`, `ui_runtime`, and `engine`.

Known blockers: Connector command execution is unavailable. This full P10 branch needs local validation before it can be recorded in completed-work.

Next action: Run the full P10 validation gate. If green, merge the P10 completion PR and record PT-UI-COMPONENT-PLATFORM-010 as completed. Then open PT-UI-COMPONENT-PLATFORM-011 Base Control Packages.

Evidence: P10 implementation covers the full owner chain: `ui_render_data` renderer-neutral evidence vocabulary, `ui_controls` bridge, `ui_runtime` evidence generation from emitted `UiFrame` output, and engine render submission proof that consumes evidence without owning UI semantics.

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
