---
title: Roadmap Intake WR-130
description: Accepted design-first roadmap intake proposal for PM-UI-DESIGNER-WB-V1-CLOSURE-004 operation parity closure.
status: active
owner: editor
layer: workspace
canonical: false
last_reviewed: 2026-05-26
---

# Roadmap Intake WR-130

Idea: UI Designer Workbench V1 closure operation diff apply rollback parity
Suggested title: UI Designer Workbench V1 closure operation diff apply rollback parity
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance review confirms no ADR is required while operation parity preserves existing ownership and dependency direction.
- `domain/ui/ui_definition` owns generic visual layout, binding, and recipe primitives.
- `domain/editor/editor_definition` owns editor operation vocabulary, reports, and reducers.
- `domain/editor/editor_shell` owns app-neutral action and view-model projection.
- `apps/runenwerk_editor` owns source-versioned session orchestration, dispatch, persistence, and evidence.

## Open Questions

- Promotion depends on `WR-129` completion and the PM-004 design-first contract.
- Product implementation must not start until `task production:plan -- --milestone PM-UI-DESIGNER-WB-V1-CLOSURE-004 --roadmap WR-130` reports promotable/current-candidate readiness.
- Scenario evidence, performance baselines, final closeout, and concrete game HUD runtime behavior remain out of scope.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
