---
title: Roadmap Intake WR-102
description: Deferred roadmap intake proposal for procedural population behavior authoring after renderer hardening.
status: active
owner: engine
layer: workspace
canonical: false
last_reviewed: 2026-05-24
---

# Roadmap Intake WR-102

Idea: Renderer procedural population behavior authoring and boids split merge
dynamics after fixed-step and camera correctness.
Suggested title: Procedural Population Behavior Authoring And Boids Dynamics
Initial planning state: `blocked_deferred`

## Governance Notes

- This is a deferred intake only. Do not fold behavior authoring into `WR-089`
  or the renderer hardening implementation slices.
- Run architecture governance before applying or promoting this item because
  semantic flock identity, affinity, goals, wander fields, attractors, and
  split/merge policy may belong outside the renderer.
- Renderer may consume derived behavior evidence, but it must not become the
  source of product, gameplay, or semantic flock truth.
- Record an ADR if a future accepted design changes durable ownership,
  dependency direction, or cross-domain contracts.

## Open Questions

- Which owner holds semantic flock identity, affinity, goals, wander fields,
  attractors, and split/merge policy?
- Which active design should define source truth before this item leaves
  `blocked_deferred`?
- What runtime evidence should prove behavior without making renderer
  procedural APIs own gameplay semantics?

## Deferred Boundary

This proposal depends on `WR-101` because renderer hardening must first prove
fixed-step timing and procedural camera correctness. It should not be applied
or promoted until ownership, design gates, write scopes, and validation commands
are explicit.

## Apply Command

```text
task roadmap:apply-intake -- --proposal <this-folder>/proposal.yaml
```
