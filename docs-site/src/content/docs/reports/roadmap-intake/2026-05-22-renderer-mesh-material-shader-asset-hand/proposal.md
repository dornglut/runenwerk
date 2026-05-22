---
title: Roadmap Intake WR-067
description: Completed roadmap intake proposal for renderer mesh/material/shader handoff.
status: completed
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-22
---

# Roadmap Intake WR-067

Idea: Renderer Mesh Material Shader Asset Handoff
Suggested title: Renderer Mesh Material Shader Asset Handoff
Planning state: `completed`

## Governance Notes

- PM-RENDER-MESH-MATERIAL-001 accepted the renderer mesh/material/lighting
  handoff doctrine and recorded architecture governance evidence.
- WR-067 must preserve Clean Architecture dependency direction: material,
  asset, model, scene, product, and fallback truth stay outside the renderer.
- ADR is required only if implementation changes durable ownership,
  dependency direction, fallback authority, or persisted cross-domain ABI.

## Gate Evidence

- Accepted doctrine:
  `docs-site/src/content/docs/design/accepted/renderer-mesh-material-lighting-and-asset-handoff-design.md`
- Doctrine closeout:
  `docs-site/src/content/docs/reports/closeouts/pm-render-mesh-material-001-mesh-material-lighting-handoff-doctrine/closeout.md`
- Implementation contract:
  `docs-site/src/content/docs/reports/implementation-plans/wr-067-renderer-mesh-material-shader-asset-handoff/plan.md`
- Completed prerequisites:
  `WR-021` material preview product evidence and `WR-058` renderer procedural
  API handoff patterns.
- Completed closeout:
  `docs-site/src/content/docs/reports/closeouts/wr-067-renderer-mesh-material-shader-asset-handoff/closeout.md`

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-22-renderer-mesh-material-shader-asset-hand/proposal.yaml
```
