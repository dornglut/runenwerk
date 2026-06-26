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

ID: `PT-UI-COMPONENT-PLATFORM-009A`

Title: UI Component Platform Ownership Realignment

State: active planning

Owner: ui

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-layout-container-virtualization-design.md
docs-site/src/content/docs/domain/ui/roadmap.md
```

Write scope:

```text
docs-site/src/content/docs/design/active/ui-component-platform-ownership-realignment-design.md
docs-site/src/content/docs/design/active/ui-component-platform-layout-container-virtualization-design.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
```

Validation expectation:

```text
Manual planning consistency review.
No Rust migration in this pass.
```

Known blockers:

```text
Phase 9 implementation must not start until the ownership realignment and owner-first layout design are accepted.
```

Next action:

```text
Review and accept the ownership realignment and corrected Phase 9 design. Then implement 009B in ui_layout before adding the 009C ui_controls bridge.
```

Evidence:

```text
Phase 8 Accessibility / Focus / Inspection passed local validation and was merged by user report on 2026-06-26.
A manual ownership audit found that phases 5-8 remained declarative but duplicated some generic vocabulary in ui_controls instead of anchoring it in owner crates.
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
