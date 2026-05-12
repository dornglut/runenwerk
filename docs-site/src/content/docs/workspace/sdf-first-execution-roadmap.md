---
title: SDF-First Execution Roadmap
description: Canonical cross-track sequencing roadmap for SDF-first product contracts, execution fabric, product jobs, query snapshots, renderer selection, and product-domain work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-05-12
related_adrs:
  - ../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ../design/accepted/sdf-first-field-world-platform-design.md
  - ../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../design/accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ../design/accepted/sdf-first-production-capability-map.md
related_roadmaps:
  - ./roadmap-index.md
  - ./repo-execution-priority-checklist.md
  - ../apps/runenwerk-editor/roadmap.md
  - ../net/ecs-runtime-prioritized-roadmap.md
  - ../engine/plugins/render/docs/roadmap.md
---

# SDF-First Execution Roadmap

## Purpose

This is the canonical cross-track sequencing roadmap for turning the accepted
SDF-first field-product architecture into implementation order.

The immediate priority is no longer M6.2 procgen. The immediate priority is the
execution substrate that lets procgen, gameplay graph, particles, physics,
animation, world processes, renderer residency, and strict runtime consumers use
formed products without inventing private execution paths.

Owning domain and app roadmaps still own detailed implementation steps. This
page owns cross-track ordering when those roadmaps overlap.

## Current Baseline

Completed or available foundations:

- editor M4 asset and field-product foundations exist;
- M5 runtime preview and reload boundaries exist for the current product
  families;
- M6.0 shared workspace substrate, M6.1 material/texture contracts, and P1 SDF
  modeling core exist;
- accepted SDF-first field-product, execution-fabric, and renderer-residency
  designs now define the target architecture.

Remaining gap:

- product formation, scheduler/ECS execution, query snapshots, renderer product
  selection, and future product-domain work are not yet sequenced around one
  execution fabric.

## Execution Order

Use this order for current implementation planning.

1. Product contract gate.
   - Align current `world_sdf`, material, texture, asset, and editor product
     descriptors with the target contract vocabulary from accepted designs.
   - Treat `ProductJobDescriptor`, `ProductQueryPolicy`, query snapshot products,
     deterministic publication barriers, and `RenderProductSelection` as target
     contracts, not existing complete Rust APIs.
2. Scheduler/ECS execution fabric.
   - Extend scheduler planning toward phases, waves, explicit barriers, plan
     diagnostics, and serial fallback.
   - Keep ECS as live runtime state and scheduler as deterministic planning.
   - Preserve serial equivalence before introducing parallel execution.
3. Product jobs and publication barriers.
   - Route product formation through described jobs with inputs, outputs, scope,
     scale band, access, budget, determinism, authority, failure policy, and
     diagnostics.
   - Publish formed products only at deterministic barriers.
4. Query snapshots and strict consumer policy.
   - Add generation/freshness/consumer policy to deferred query products.
   - Make strict/current-only consumers reject stale, fallback, ghost, and
     visual-only products unless an owning domain certifies the fallback.
5. Renderer product selection and GPU residency.
   - Prepare `RenderProductSelection` from formed products and generations.
   - Keep GPU resources, render targets, and UI samples derived from product
     selections.
6. Resume product-domain tracks.
   - Procgen remains the first product-domain track after the execution gates.
   - Gameplay graph, particles, physics, animation, and world processes follow
     only after their owning contracts can consume product jobs, query snapshots,
     publication barriers, and diagnostics.

## Near-Term Gates

Do not start M6.2 procgen implementation until these gates are satisfied:

- the product contract gate records the current specialization boundaries and
  migration target;
- scheduler/ECS roadmap updates identify the first concrete execution-fabric
  work package;
- product jobs have a documented publication/failure/diagnostics policy;
- query snapshots have a documented freshness and strict-consumer policy;
- renderer work is sequenced against product selection and derived GPU
  residency, not renderer-owned world truth.

Procgen design/domain docs may still be prepared while these gates are being
closed, but procgen implementation should not bypass the execution substrate.

## Roadmap Ownership

- `workspace/sdf-first-execution-roadmap.md` owns cross-track order.
- `apps/runenwerk-editor/roadmap.md` owns editor milestone detail.
- `domain/scheduler/*` owns scheduler contract detail.
- `domain/ecs/*` owns ECS state, query, command, and runtime-bridge detail.
- `net/ecs-runtime-prioritized-roadmap.md` remains a net/runtime convergence
  tracker and feeds this roadmap where ECS runtime work is relevant.
- `engine/plugins/render/docs/roadmap.md` owns render implementation detail, but
  SDF renderer and GPU residency work must follow accepted product selection
  and residency contracts.

## Validation Expectations

Roadmap updates should verify:

- docs validation passes;
- no priority list still says M6.2 procgen is the immediate next
  implementation priority;
- target contracts are described as future contracts until code exists;
- renderer, UI, and debug products remain derived state;
- strict consumers cannot be satisfied by visual-only product paths.
