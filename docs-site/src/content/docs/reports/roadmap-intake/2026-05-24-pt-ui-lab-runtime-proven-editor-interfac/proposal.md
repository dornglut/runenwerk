---
title: Roadmap Intake WR-094
description: Roadmap intake proposal for PM-UI-LAB-002 command catalog and surface registry cleanup.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-24
---

# Roadmap Intake WR-094

Idea: PM-UI-LAB-002 command catalog and surface registry source-of-truth cleanup for the runtime-proven Editor Interface Lab track.
Suggested title: UI Lab command catalog and surface registry source of truth
Initial planning state: `ready_next`

Source design:
`docs-site/src/content/docs/design/accepted/ui-lab-command-catalog-and-surface-registry-design.md`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- Which current candidate must be switched or completed before WR-094 can enter implementation?
- Which runtime evidence harness should PM-UI-LAB-002 use for catalog and registry proof before later visual-lab milestones?
- Does promotion preflight require metadata repair before current-candidate selection?

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
