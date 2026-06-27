---
title: Closeout Reports
description: Historical completion evidence for Runenwerk phases, slices, migrations, and proof gates.
status: active
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-06-27
related_docs:
  - ../../workspace/workflow-lifecycle.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/routines/phase-completion-drift-check-routine.md
---

# Closeout Reports

Use this folder for detailed historical completion evidence.

`completed-work.md` remains the short completion index. Closeout reports keep larger evidence records out of planning files.

## Use when

Create a closeout report when a completed phase, migration, proof gate, or cleanup pass needs more detail than a short completed-work entry should carry.

Examples:

```text
phase validation detail
changed-file evidence
known gap audit
migration map
proof report summary
stale mirror report
follow-up risk inventory
```

## Report shape

```text
ID:
Title:
Completed on:
Owner:
Scope promised:
Scope delivered:
Files changed:
Validation run:
Validation unavailable:
Known gaps:
Drift found:
Follow-up:
Evidence links:
```

## Rules

- Closeout reports are historical evidence.
- Closeout reports do not own current planning state.
- Link closeout reports from `completed-work.md` when they exist.
- Do not move active work or roadmap state into closeout reports.
- Preserve evidence when moving detail from planning into a closeout report.
- Use kebab-case filenames.

## Naming

Prefer:

```text
<track-id>-<short-title>-closeout.md
```

Examples:

```text
pt-ui-component-platform-010-render-surface-output-closeout.md
pt-ecs-006-short-title-closeout.md
```
