---
title: Renderer Scale Residency And GPU Driven Visibility Platform
description: Active design for finite renderer working sets, GPU residency budgets, culling, LOD, indirect drawing, and millions-scale evidence.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: true
last_reviewed: 2026-05-22
related_designs:
  - ../accepted/render-product-graph-platform-design.md
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ./renderer-gpu-evidence-and-procedural-visuals-design.md
---

# Renderer Scale Residency And GPU Driven Visibility Platform

## Decision

Renderer scale is a finite-working-set problem. Unbounded world and product
space may exist in owning domains, but renderer submission consumes bounded
resident, visible, and submitted GPU working sets per frame.

The renderer owns derived execution structures: GPU buffers, page or cluster
registries, visibility buffers, indirect command buffers, timing, memory
pressure diagnostics, and inspection DTOs. Product truth, product selection,
freshness, authority, fallback legality, rebuild policy, and residency intent
remain outside the renderer.

## Scope

This track covers:

- renderer-facing chunk, page, cluster, and instance registries;
- finite resident and visible working-set contracts;
- GPU memory class budgets and upload-byte budgets;
- visibility compaction, frustum culling, screen-size LOD, and later occlusion;
- indirect draw and dispatch command generation;
- scale evidence for large addressable populations without per-entity CPU
  submission.

It does not define authored world truth, gameplay entity semantics, product
freshness policy, or domain-owned streaming decisions.

## Scale Doctrine

The renderer must not claim infinity by attempting to render infinite things.
The accepted model is:

```text
unbounded product/world space
  -> bounded selected products
  -> bounded resident GPU resources
  -> bounded visible candidates
  -> bounded submitted draw/dispatch work
  -> measured frame budget
```

Millions-scale evidence must distinguish:

- addressable product or instance records;
- resident GPU records;
- visible candidates after culling and LOD;
- submitted draw, dispatch, and indirect command counts.

## Evidence

Runtime evidence must expose memory pressure, upload pressure, visible counts,
culled counts, LOD bands, indirect command counts, GPU timings, and fallback or
over-budget diagnostics. Any hardware-dependent target must include unsupported
or degraded-mode diagnostics instead of silent success.
