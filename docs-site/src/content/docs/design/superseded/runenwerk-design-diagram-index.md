---
title: Runenwerk Design Diagram Index
description: Superseded PlantUML diagram index for the draft SDF-first design set.
status: superseded
owner: workspace
layer: cross-domain
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/sdf-first-design-diagram-index.md
---

# Runenwerk Design Diagram Index

## Status

Superseded diagram index.

Replaced by `../accepted/sdf-first-design-diagram-index.md`. The diagram
source files were promoted and renamed where needed under
`../accepted/diagrams/`; the table below preserves the historical draft index
for review only.

This page defines the initial PlantUML diagram set for the SDF-first Runenwerk production architecture drafts.

The diagrams are source diagrams. If the docs-site does not yet render `.puml` files directly, keep these files as canonical diagram sources and let Codex later add rendering/export support.

---

# Purpose

The current draft design set introduces many related systems:

- Adaptive Field Product System
- SDF Product Renderer Architecture
- SDF World Production Slice
- Open World Product Streaming
- Field Product Diagnostics
- SDF Prefabs
- SDF Characters
- Field Vegetation
- Day/Night Atmosphere
- Water/Wetness Fields
- SDF Physics/Collision
- Field Influence AI
- Procgen Field Products
- Field VFX/Particles
- Fluid/Snow/Erosion World Processes
- ECS Scheduler Execution Fabric

This diagram index gives visual entry points for the system relationships, priorities, blockers, and runtime flows.

---

# Diagram Files

| Diagram | File | Purpose |
|---|---|---|
| Design priority/blocker map | `../accepted/diagrams/runenwerk-design-priority-map.puml` | Shows P0/P1/P2/P3 dependencies between design docs. |
| Field product architecture | `../accepted/diagrams/adaptive-field-product-system-product-flow.puml` | Shows authored/generated/simulated inputs becoming formed products and consumers. |
| Field product lifecycle | `../accepted/diagrams/adaptive-field-product-system-lifecycle.puml` | Shows product freshness/residency lifecycle states. |
| Renderer product pipeline | `../accepted/diagrams/sdf-product-renderer-pipeline.puml` | Shows product resolver, GPU residency, producers, flows, and surfaces. |
| Execution fabric waves | `../accepted/diagrams/ecs-scheduler-execution-fabric-waves.puml` | Shows phases, parallel waves, barriers, deferred apply, product jobs, render prepare/submit. |
| Open-world streaming residency | `../accepted/diagrams/open-world-product-streaming-residency.puml` | Shows near/mid/far/summary/outside product residency. |
| First production slice map | `../accepted/diagrams/sdf-first-production-capability-system-map.puml` | Historical draft view; accepted replacement frames this as long-term SDF-first capability planning. |

---

# Recommended Usage

Use each diagram to answer one question.

| Question | Diagram |
|---|---|
| What should be designed first? | Design priority/blocker map |
| How do field products flow through the engine? | Field product architecture |
| What states can a product be in? | Field product lifecycle |
| How does the renderer consume products? | Renderer product pipeline |
| How do ECS, scheduler, jobs, and barriers fit together? | Execution fabric waves |
| What is loaded near/far in an open world? | Open-world streaming residency |
| What systems make up the first production slice? | First production slice map |

Avoid one giant mega-diagram. The architecture should stay understandable by splitting ownership, flow, lifecycle, scheduling, and spatial residency into separate views.

---

# Priority and Blocker Summary

| Priority | Design Area | Blocked By | Blocks |
|---|---|---|---|
| P0 | Adaptive Field Product System | none | renderer, streaming, diagnostics, product jobs, all field systems |
| P0 | ECS Scheduler Execution Fabric | ECS/scheduler code-truth alignment | renderer jobs, physics, AI, procgen, product jobs |
| P0 | SDF Product Renderer Architecture | field-product model, execution fabric | production slice, characters, vegetation, water, diagnostics surfaces |
| P0 | Open World Product Streaming | field-product model | production slice, procgen, water, caves later |
| P0 | Field Product Diagnostics | field-product model | all debugging and promotion |
| P1 | SDF World Production Slice | renderer, streaming, diagnostics | first integration target |
| P1 | SDF Prefab Composition | field products, renderer, streaming | characters, trees, rocks, enemies |
| P1 | SDF Character Animation | prefab composition, renderer, execution fabric | physics, enemies |
| P1 | Field Vegetation | renderer, streaming, water/day-night inputs | grass interaction |
| P1 | Day/Night Atmosphere | renderer, product model | vegetation, water, enemies |
| P1 | Water/Wetness Fields | renderer, streaming, product model | physics, vegetation, fluids later |
| P2 | SDF Physics/Collision | character, prefab, streaming, execution fabric | AI/enemies/gameplay authority |
| P2 | Field Influence AI | physics, character, scheduler | enemies/gameplay |
| P2 | Procgen Field Products | field products, streaming, prefab, vegetation | infinite world generation |
| P2 | Field VFX/Particles | renderer, atmosphere, water, vegetation | mood/VFX integration |
| P3 | Fluid/Snow/Erosion World Processes | water, procgen, physics, mutation authority | later world simulation |

---

# Rendering Note

PlantUML files are intentionally kept as `.puml` source files.

Possible later Codex tasks:

1. Verify whether the Astro docs-site already renders PlantUML.
2. If not, add a docs build/export step for `.puml` diagrams.
3. Generate committed SVG outputs only if the docs pipeline needs static assets.
4. Keep `.puml` files as canonical diagram sources.
5. Add docs validation for broken diagram references.
