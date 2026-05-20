---
title: Roadmap Intake WR-030
description: Roadmap intake proposal generated from a new idea.
status: draft
owner: workspace
layer: workspace
canonical: false
last_reviewed: 2026-05-20
---

# Roadmap Intake WR-030

Idea: WR-029 Phase 4 needs a real model/mesh renderable scene contract before GPU pixel proof: current editor viewport scene persistence and extraction only own SDF primitives, while the rendered-world design explicitly excludes general mesh scene extraction. Add an implementation candidate for source-backed foreign mesh renderable fixture or Mesh Preview surface that carries SceneModelMeshMaterialRegionSourceId through app runtime into an actual material-consuming render pass, then use that pass for WR-029 visible pixel proof without weakening WR-028 SDF.
Suggested title: Model Mesh Renderable Scene Contract
Initial planning state: `blocked_deferred`

## Governance Notes

- Run architecture governance review before implementation.
- Confirm Clean Architecture dependency direction and DDD owner.
- Record ADR only if the decision changes durable ownership, dependency direction, or cross-domain contracts.

## Open Questions

- What accepted design, ADR, or closeout evidence justifies promotion?
- Which existing WR items does this depend on?
- Which exact write scopes and validation commands will bound implementation?

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
