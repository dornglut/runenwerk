---
title: Roadmap Intake WR-086
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-24
---

# Roadmap Intake WR-086

Idea: PM-UI-LAB-005 Editor Lab project IO package persistence diff apply activation report rollback productization
Suggested title: UI Lab project IO diff apply and rollback productization
Initial planning state: `ready_next`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- Confirm whether PM005 ships as one full productization WR or is split after production:plan into package-store and apply-review slices.
- Confirm exact closeout artifact paths in the implementation contract before product code changes.
- Confirm whether reload-last-applied evidence can remain headless runtime evidence until PM006 screenshot/accessibility breadth.

## Accepted Gates

- Accepted design: `docs-site/src/content/docs/design/accepted/ui-lab-persistence-project-io-diff-apply-rollback-design.md`
- Productization design: `docs-site/src/content/docs/design/active/ui-lab-productization-design.md`
- Dependency evidence: `docs-site/src/content/docs/reports/closeouts/pm-ui-lab-004-operation-driven-visual-authoring/closeout.md`

## First Move

```text
task production:plan -- --milestone PM-UI-LAB-005 --roadmap WR-086
```

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
