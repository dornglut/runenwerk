---
title: Roadmap Intake WR-108
description: Roadmap intake proposal for PM-UI-LAB-PERF-004 direct-manipulation Editor Lab UX closure.
status: draft
owner: editor
layer: workspace
canonical: false
last_reviewed: 2026-05-25
related_designs:
  - ../../../design/accepted/ui-lab-direct-manipulation-ux-closure-design.md
  - ../../../design/accepted/ui-lab-perfectionist-audit-design.md
related_reports:
  - ../../closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md
---

# Roadmap Intake WR-108

Idea: `PM-UI-LAB-PERF-004 Direct Manipulation Editor Lab UX Closure`

Suggested title: `UI Lab direct manipulation UX closure`

Initial planning state: `ready_next`

## Evidence

- Accepted design:
  `docs-site/src/content/docs/design/accepted/ui-lab-direct-manipulation-ux-closure-design.md`
- Completed prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-003-command-and-surface-source-truth-closure/closeout.md`
- Supporting operation proof:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`

## Scope

WR-108 is PM004 promotion-planning work only until `task production:plan`
selects a next action and roadmap promotion gates pass. The implementation
scope is direct-manipulation Editor Lab product-surface evidence: hierarchy,
palette, canvas, inspector, diagnostics, operation diff, preview console, undo,
and redo.

It must not start persistence/API ergonomics, project IO, examples, rollback
review, final no-gap certification, or game-runtime UI projection.

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-25-pm-ui-lab-perf-004-direct-manipulation-e/proposal.yaml
```
