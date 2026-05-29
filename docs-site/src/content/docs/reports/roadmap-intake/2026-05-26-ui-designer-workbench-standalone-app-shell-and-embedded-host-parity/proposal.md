---
title: Roadmap Intake WR-122
description: Ready-next roadmap intake for PM-UI-DESIGNER-WB-003 standalone UI Designer host parity.
status: accepted
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-26
---

# Roadmap Intake WR-122

Idea: UI Designer Workbench standalone app shell and embedded host parity
Suggested title: Standalone App Shell And Embedded Host Parity
Initial planning state: `ready_next`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- Does the future implementation need a new ADR? Current planning says no while ownership and dependency direction stay unchanged.
- Which current-candidate switch, if any, does task production:plan require before WR-122 implementation?

## Accepted Evidence

- Accepted design:
  `docs-site/src/content/docs/design/accepted/ui-designer-workbench-product-design.md`.
- PM001 governance closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-001-governance-code-truth-and-track-activation/closeout.md`.
- PM002 V1 model closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-ui-designer-wb-002-v1-package-document-session-and-evidence-model/closeout.md`.
- WR-122 contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-122-standalone-app-shell-and-embedded-host-parity/plan.md`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
