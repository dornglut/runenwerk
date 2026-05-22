---
title: Roadmap Intake WR-059
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-059

Idea: Boids Hybrid SDF/Mesh Procedural Sprite Rewrite
Suggested title: Boids Hybrid SDF/Mesh Procedural Sprite Rewrite
Initial planning state: `completed`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- None after the design-first contract.

## Resolved Decisions

- Promotion evidence is the accepted renderer GPU evidence design, completed `WR-058` procedural API closeout, architecture-governance kickoff, and the implementation contract at `docs-site/src/content/docs/reports/implementation-plans/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/plan.md`.
- The row depends on `WR-058`.
- Implementation is bounded to `engine/examples/boids_render_flow`, the boids compute/compose shaders, renderer public docs, the accepted GPU evidence design, this intake folder, and the WR-059 implementation and closeout documents.
- The canonical visual defaults to local 2D SDF impostors through `RenderFlow::procedural_pass(...)` and `ProceduralPassDescriptor::local_sdf_2d_impostors(...)`.
- The boids rewrite must remove fullscreen-per-boid rendering, the fragment loop over all boids, and history copies unless a real trail/history consumer is implemented and tested.
- WR-059 does not set a numeric GPU timing threshold; numeric runtime budgets and stress thresholds belong to `WR-060`.
- Closeout evidence is recorded at `docs-site/src/content/docs/reports/closeouts/wr-059-boids-hybrid-sdf-mesh-procedural-sprite-rewrite/closeout.md`.

## Completion Notes

- Completion quality is `bounded_contract`.
- The canonical example now uses storage-backed compute, publish-to-render-buffer compute, and public local SDF impostor procedural rendering.
- `WR-060` still owns production runtime evidence, finite timing artifacts, numeric budgets, and stress thresholds.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
