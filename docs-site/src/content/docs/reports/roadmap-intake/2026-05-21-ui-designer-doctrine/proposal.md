---
title: Roadmap Intake WR-046
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-21
---

# Roadmap Intake WR-046

Idea: UI Designer doctrine and target boundary ratification
Suggested title: UI Designer doctrine and target boundary ratification
Initial planning state: `support_only`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.
- Architecture governance for PM-UI-DESIGN-001 found no code, crate,
  dependency-direction, or runtime ownership change in this bounded doctrine
  slice.
- A future ADR or accepted design update is still required before moving
  canonical UI definition ownership out of the existing
  `domain/ui/ui_definition` crate or creating a separate game-runtime UI owner
  crate.

## Open Questions

- None for this support-only doctrine row; implementation remains blocked until
  later PM-UI-DESIGN milestones clear their accepted design gates and WR
  execution rows.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
