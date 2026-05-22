---
title: Roadmap Intake WR-057
description: Roadmap intake proposal generated from a new idea.
status: active
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-057

Idea: Render Flow Pass Shape And Instance Contract Guards
Suggested title: Render Flow Pass Shape And Instance Contract Guards
Current planning state: `completed`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- Completion evidence is recorded in `docs-site/src/content/docs/reports/closeouts/wr-057-render-flow-pass-shape-and-instance-contract-guards/closeout.md`.
- Dependency `WR-056` is completed with GPU timing closeout evidence.
- Remaining quality gaps are carried by the later procedural API and boids proof rows.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
