---
title: SDF-First Design Diagram Index
description: Accepted PlantUML diagram index for SDF-first field-product architecture.
status: accepted
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-12
related_designs:
  - ./sdf-first-field-world-platform-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
  - ./sdf-first-production-capability-map.md
---

# SDF-First Design Diagram Index

## Status

Accepted diagram source index.

The PlantUML files are canonical diagram sources. If the docs site later needs
rendered SVGs, keep these `.puml` files as the source of truth.

## Diagrams

| Diagram | File | Purpose |
|---|---|---|
| Design priority and capability map | `./diagrams/runenwerk-design-priority-map.puml` | Shows accepted foundational docs and long-term capability tracks. |
| Field product flow | `./diagrams/adaptive-field-product-system-product-flow.puml` | Shows authored/generated/simulated inputs becoming formed products and consumers. |
| Field product lifecycle | `./diagrams/adaptive-field-product-system-lifecycle.puml` | Shows product freshness and residency lifecycle states. |
| SDF renderer product pipeline | `./diagrams/sdf-product-renderer-pipeline.puml` | Shows renderer consumption, GPU residency, producers, flows, and surfaces. |
| Execution fabric waves | `./diagrams/ecs-scheduler-execution-fabric-waves.puml` | Shows phases, waves, barriers, deferred apply, product jobs, render prepare, and submit. |
| Open-world product residency | `./diagrams/open-world-product-streaming-residency.puml` | Shows near/mid/far/summary/outside product residency. |
| SDF-first production capability map | `./diagrams/sdf-first-production-capability-system-map.puml` | Shows long-term SDF-first production capability groups and product interactions. |

## Diagram Rules

- Keep diagrams focused; do not create one giant architecture diagram.
- Diagrams must reflect accepted architecture, not draft-only terminology.
- Diagrams must not imply that future capability tracks are already
  implemented.
- If a diagram conflicts with the accepted ADR or design docs, the text docs
  win and the diagram should be updated.
