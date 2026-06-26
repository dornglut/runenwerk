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

ID: `PT-UI-COMPONENT-PLATFORM-010-PLANNING`

Title: UI Component Platform Render Surface / Output design intake

State: active planning

Owner: `ui_render_data` for renderer-facing output contracts; `ui_runtime` and `engine/src/plugins/render` are adjacent execution owners. `ui_controls` is bridge-only after owner contracts exist.

Authority files:

```text
AGENTS.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
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
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/design/active/ui-component-platform-render-surface-output-design.md
docs-site/src/content/docs/design/active/README.md
```

Validation expectation:

```text
Manual validation through GitHub connector:
- confirm PR #29 is merged into main;
- confirm PR #30 is closed unmerged and superseded;
- confirm Phase 9 owner-first files exist on main;
- confirm 009A/009B/009C evidence is represented in planning;
- confirm Phase 10 planning keeps generic render/output vocabulary out of ui_controls.

Optional local validation when a checkout is available:
- task docs:validate
- task planning:validate
- git diff --check
```

Known blockers:

```text
Phase 10 is planning-only. No Rust implementation, renderer behavior, runtime behavior, or ui_controls render/output vocabulary should be added until the owner-first design is accepted.
```

Next action:

```text
Review and accept the Phase 10 render surface / output design intake. Then split implementation into owner-first slices, starting with renderer-neutral output contracts in ui_render_data before any ui_controls bridge.
```

Evidence:

```text
Phase 9 completed through PR #29 on main.
009A recorded the ownership realignment rule.
009B added generic layout/container/scroll/content/identity/virtualization vocabulary in ui_layout.
009C added the ui_controls control layout bridge over ui_layout.
Catalog inspection exposes read-only layout summaries through Metadata-prefixed layout facts.
Focused ui_layout and ui_controls layout tests exist.
User reported the Phase 9 validation gate green on 2026-06-26.
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
