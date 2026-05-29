---
title: Roadmap Intake WR-123
description: Ready-next roadmap intake for PM-UI-DESIGNER-WB-004 UI Designer product catalog, hierarchy, canvas, inspector, diagnostics, and diff/review surfaces.
status: accepted
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-26
---

# Roadmap Intake WR-123

Idea: UI Designer Workbench catalog hierarchy canvas inspector V1
Suggested title: Catalog Hierarchy Canvas Inspector V1
Initial planning state: `ready_next`

## Governance Notes

- Architecture governance kickoff was run for PM-UI-DESIGNER-WB-004 before implementation planning.
- `domain/ui` owns generic recipe, package, UI document, Canonical UI IR, token, target-profile, diagnostics, and evidence descriptor truth.
- `domain/editor` owns UI Designer app-neutral view models, catalog row projection, selection parity, hierarchy/canvas/inspector surface contracts, and retained composition.
- `apps/runenwerk_editor` owns concrete provider execution, shell session filters and selection state, command bridging, fixtures, and future native/runtime evidence.
- No ADR is required while WR-123 preserves current ownership and dependency direction.

## Open Questions

- Which current-candidate promotion, if any, does `task production:plan` require before WR-123 implementation?
- Which PM004 product surface should provide the first runtime evidence artifact after promotion?

## Accepted Evidence

- Accepted product design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- Accepted recipe/catalog design:
  `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`.
- Accepted visual layout and composition design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- PM003 host-parity closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-003-standalone-app-shell-and-embedded-host-parity/closeout.md`.
- WR-123 contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-123-catalog-hierarchy-canvas-inspector-v1/plan.md`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
