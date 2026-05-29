---
title: Roadmap Intake WR-124
description: Ready-next roadmap intake for PM-UI-DESIGNER-WB-005 UI Designer operation diff, apply, rollback, undo/redo, and reload preservation.
status: accepted
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-26
---

# Roadmap Intake WR-124

Idea: UI Designer Workbench operation diff apply rollback
Suggested title: Operation Diff Apply And Rollback
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance kickoff was run for PM-UI-DESIGNER-WB-005 before implementation planning.
- `domain/ui` owns generic visual layout operations, persistence/diff/activation descriptors, and UI-definition diagnostics.
- `domain/editor` owns editor/workbench operation envelopes, operation reports, editor-specific diff families, and app-neutral review view models.
- `apps/runenwerk_editor` owns concrete UI Designer draft mutation, operation history, apply/reject, rollback, reload preservation, and runtime evidence.
- No ADR is required while WR-124 preserves current ownership and dependency direction.

## Open Questions

- Which current-candidate promotion, if any, does `task production:plan` require before WR-124 implementation?
- Which PM005 operation path should provide the first runtime evidence artifact after promotion?

## Accepted Evidence

- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Accepted visual layout and deterministic diff design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted generic persistence, migration, diff, and activation design:
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`.
- Accepted UI Lab operation-driven authoring design:
  `docs-site/src/content/docs/design/accepted/ui-lab-operation-driven-visual-authoring-design.md`.
- Accepted UI Lab persistence/apply/rollback design:
  `docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`.
- PM004 product-surface closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-004-catalog-hierarchy-canvas-inspector-v1/closeout.md`.
- WR-124 contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-124-operation-diff-apply-and-rollback/plan.md`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
