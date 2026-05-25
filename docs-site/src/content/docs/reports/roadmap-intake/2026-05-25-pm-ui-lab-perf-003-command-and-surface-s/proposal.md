---
title: Roadmap Intake WR-107
description: Roadmap intake proposal for PM-UI-LAB-PERF-003 command and surface source-truth closure.
status: active
owner: editor
layer: workspace
canonical: false
last_reviewed: 2026-05-25
---

# Roadmap Intake WR-107

Idea: `PM-UI-LAB-PERF-003 Command And Surface Source Truth Closure`

Suggested title: `UI Lab command and surface source-truth closure`

Initial planning state: `ready_next`

## Evidence

- Accepted design:
  `docs-site/src/content/docs/design/accepted/ui-lab-command-surface-source-truth-closure-design.md`
- Completed prerequisite:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-perf-002-runtime-evidence-platform-closure/closeout.md`
- Roadmap dependency:
  `WR-105 -> WR-107`

## Governance Notes

- `apps/runenwerk_editor` owns concrete command descriptors, command dispatch,
  dynamic availability, app diagnostics, provider execution, and provider ids.
- `domain/editor/editor_shell` owns app-neutral surface metadata, tool-suite
  registry validation, and shell projection contracts.
- `domain/ui/ui_definition` remains behavior-free and does not own editor
  command semantics, provider families, or surface execution.
- No ADR is required while implementation preserves those boundaries.

## Bounded Write Scope

The implementation row is bounded to command catalog/projection, surface
registry/provider support, legacy adapters, focused tests, planning metadata,
and closeout evidence. It must not start direct-manipulation UX, persistence/API
ergonomics, or final certification work.

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-25-pm-ui-lab-perf-003-command-and-surface-s/proposal.yaml
```
