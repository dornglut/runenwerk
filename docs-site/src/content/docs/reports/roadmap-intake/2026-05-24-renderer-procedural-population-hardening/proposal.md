---
title: Roadmap Intake WR-089
description: Roadmap intake proposal for the renderer procedural population hardening track.
status: active
owner: engine
layer: workspace
canonical: false
last_reviewed: 2026-05-24
---

# Roadmap Intake WR-089

Idea: Renderer procedural population hardening track for indirect draw
validation, reusable GPU primitive shader dispatch, fixed-step graph catch-up
scheduling, and procedural camera projection after
`PT-RENDER-PROCEDURAL-POPULATION` runtime-proven closeout.

Suggested title: Renderer Procedural Population Hardening Doctrine And Track
Activation

Initial planning state: `current_candidate`

## Governance Notes

- Architecture governance review is required before implementation because the
  track changes renderer graph contracts, GPU primitive runtime execution, and
  scheduling semantics.
- Renderer remains the owner of derived execution data; product, gameplay,
  world, and streaming source truth remain outside renderer ownership.
- No ADR is required for track activation while dependency direction and source
  truth remain unchanged.

## Accepted Split

- `WR-089`: doctrine and track activation only.
- `WR-090`: indirect draw contract hardening.
- `WR-091`: reusable GPU primitive shader dispatch.
- `WR-092`: fixed-step graph catch-up scheduling.
- `WR-101`: procedural camera and view projection.
- `WR-093`: evidence, benchmarks, docs, and runtime-proven closeout after
  `WR-101`.

Spatial hash and chunked unbounded populations are represented by a separate
intake proposal and are not part of this hardening track.

Richer procedural behavior authoring and boids split/merge dynamics are also
represented by a separate deferred intake proposal. They require future
ownership and architecture review before promotion.

## Apply Command

```text
task roadmap:apply-intake -- --proposal docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-renderer-procedural-population-hardening/proposal.yaml
```
