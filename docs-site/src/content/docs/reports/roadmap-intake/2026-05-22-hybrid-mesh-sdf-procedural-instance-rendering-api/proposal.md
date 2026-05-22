---
title: Roadmap Intake WR-058
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-058

Idea: Hybrid Mesh/SDF Procedural Instance Rendering API
Suggested title: Hybrid Mesh/SDF Procedural Instance Rendering API
Initial planning state: `completed`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- None after the design-first contract.

## Resolved Decisions

- Promotion evidence is the accepted renderer GPU evidence design, accepted product/render and SDF boundary designs, completed `WR-057` pass-shape closeout, architecture-governance kickoff, and the implementation contract at `docs-site/src/content/docs/reports/implementation-plans/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/plan.md`.
- The row depends on `WR-057`.
- Implementation is bounded to `engine/src/plugins/render`, `engine/examples`, `engine/tests`, renderer public docs, the accepted GPU evidence design, this intake folder, and the WR-058 implementation contract.
- The first public SDF impostor API is local 2D only; 3D SDF raymarch and sparse residency hooks remain out of scope.
- Primitive, blend, depth, cull, target, generated quad/local mesh, and instance layout policy must be explicit in v1.

## Closeout

Completed evidence is recorded at `docs-site/src/content/docs/reports/closeouts/wr-058-hybrid-mesh-sdf-procedural-instance-rendering-api/closeout.md`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
