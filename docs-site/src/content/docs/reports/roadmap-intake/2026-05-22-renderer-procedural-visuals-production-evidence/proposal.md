---
title: Roadmap Intake WR-060
description: Roadmap intake proposal generated from a new idea.
status: completed
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-060

Idea: Renderer Procedural Visuals Production Evidence
Suggested title: Renderer Procedural Visuals Production Evidence
Initial planning state: `ready_next`
Final planning state: `completed`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- None after the design-first production evidence contract.

## Resolved Decisions

- Promotion evidence is completed closeout evidence for `WR-056`, `WR-057`,
  `WR-058`, and `WR-059`, plus the implementation contract at
  `docs-site/src/content/docs/reports/implementation-plans/wr-060-renderer-procedural-visuals-production-evidence/plan.md`.
- Runtime evidence must record backend/capability profile where available,
  timestamp support, timing source, scene size, pass shape, instance count, CPU
  timing, and GPU timing or typed unsupported GPU timing diagnostics.
- The mandatory runtime proof is `engine/examples/boids_render_flow`.
- The mandatory benchmark command is `cargo bench -p engine --bench render_flow_planning`
  unless implementation proves a more specific existing renderer planning
  benchmark is canonical.
- `PM-RENDER-GPU-006` targets `runtime_proven`. `perfectionist_verified` remains
  blocked until `PT-RENDER-PERFECTION` completes the cross-track no-gap audit.
- `WR-060` completed with the closeout at
  `docs-site/src/content/docs/reports/closeouts/wr-060-renderer-procedural-visuals-production-evidence/closeout.md`.
- Remaining benchmark-threshold and finite boids swapchain timing gaps are
  explicit inputs for `PT-RENDER-PERFECTION`.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
