---
title: Roadmap Overview
description: Overview of the engine-agnostic domain roadmaps.
---

## Current Domain Priority

ECS core primitives are now considered usable platform foundation.
Near-term roadmap priority is integration and correctness across engine/world/runtime boundaries,
not additional basic ECS primitives.

## Domain Roadmap (Post-ECS Foundation)

| Priority | Area | Goal | Notes |
| --- | --- | --- | --- |
| **1 (Now)** | World correctness | Stabilize dirty-chunk invalidation, runtime record bootstrap, authoritative build/integration contracts, and collision-readiness semantics. | Engine-facing correctness gate for all downstream work. |
| **1 (Now)** | Runtime alignment | Keep world maintenance on fixed-step simulation cadence with explicit scheduling relative to simulation/replication. | Required for deterministic multiplayer/world sync. |
| **1 (Now)** | Integration contracts | Centralize world edit ingress and bounds-driven invalidation bridges; remove game-level ad hoc dirty map writes. | Reduces duplication and policy drift. |
| **2 (Next)** | World/render boundary | Move from stringly world contribution IDs to typed IDs and explicit residency invalidation intent contracts. | Improves safety and cache coherency. |
| **2 (Next)** | Streaming contracts | Drive chunk headers/content/op-window replication from world runtime state (not only snapshot helpers). | Foundation for large-world region workflows. |
| **3 (Later)** | Region-oriented infrastructure | Promote region-centric change journals and invalidation surfaces to support SAG-style world-region pipelines. | Do not treat spatial hash backend as SAG itself. |

## Explicitly Deferred

- New basic ECS primitives as the main roadmap axis.
- Octree/BVH backend work before world integration gates are closed.
- Editor/product/history/networking expansions that are not required by world correctness and integration milestones.
