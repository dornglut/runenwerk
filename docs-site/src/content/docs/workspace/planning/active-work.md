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

ID: `PT-UI-COMPONENT-PLATFORM-011-PLANNING`

Title: UI Component Platform Base Control Packages design intake

State: active planning

Owner: `ui_controls` owns base control package descriptors and per-control summaries. `ui_layout`, `ui_render_data`, `ui_input`, `ui_theme`, and accessibility/focus contracts remain owner crates that base controls reference. Full runtime interaction behavior is planned for Phase 12, not Phase 11.

Authority files: `AGENTS.md`, `docs-site/src/content/docs/workspace/start-here.md`, `docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md`, `DOMAIN_MAP.md`, `CRATES.md`, crate inventory, authority model, programming principles, Phase 9 ownership realignment design, Phase 10 render surface/output design, and the Phase 11 base control packages design.

Validation expectation: this is a planning closeout/opening patch. Validate with docs/planning validation and `git diff --check` when a checkout is available.

Known blockers: No code blocker. Phase 11 implementation should not start until the base control package acceptance criteria are reviewed.

Next action: Review and accept the Phase 11 base control packages design. Then implement base control package hardening for Label, Button, InspectorField, ColorPicker, ActionPrompt, ListView, TreeView, and TableView without adding full interaction runtime behavior.

Evidence: Phase 10 is complete through PR #34 merged into `main`, and the user reported the full P10 validation gate green on 2026-06-26.

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
