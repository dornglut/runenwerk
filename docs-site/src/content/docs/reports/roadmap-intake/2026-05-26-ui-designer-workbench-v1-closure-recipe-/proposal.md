---
title: Roadmap Intake WR-129
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-26
---

# Roadmap Intake WR-129

Idea: UI Designer Workbench V1 closure recipe catalog insertion and authoring surface closure
Suggested title: UI Designer Workbench V1 Closure Recipe Catalog Insertion
Initial planning state: `blocked_deferred`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- WR-129 depends on completed WR-123 product-surface projection evidence and WR-128 package/session source-truth evidence.
- The accepted recipe-library and visual-composition designs are the implementation gates for the closure slice.
- The next evidence path is `docs-site/src/content/docs/reports/implementation-plans/wr-129-ui-designer-workbench-v1-closure-recipe-catalog-insertion/plan.md`.

## Bounded Scope

- Searchable compatible recipe catalog rows with disabled reasons, slot/token/state/accessibility metadata, and target-profile compatibility.
- Compatible recipe insertion into the active source-versioned package/document model.
- Synchronized hierarchy, canvas, inspector, diagnostics, and diff projections over the same source version.
- Explicit non-coverage of full operation apply/rollback parity, scenario evidence, performance baselines, final closeout, and concrete game HUD runtime behavior.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
