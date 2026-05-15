---
title: Animated SDF Lowering and Purpose-Specific Products
description: Proposed decision that animated SDF authoring graphs lower through semantic IR into scheduled jobs and purpose-specific products instead of executing directly at runtime.
status: draft
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-16
related_designs:
  - ../../design/active/sdf-procedural-animation-and-animated-models-design.md
  - ../../design/accepted/sdf-first-field-world-platform-design.md
  - ../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
related_adrs:
  - ../accepted/0004-separate-description-from-execution.md
  - ../accepted/0005-projections-are-derived-state.md
  - ../accepted/0008-adopt-sdf-first-field-product-architecture.md
---

# ADR: Animated SDF Lowering and Purpose-Specific Products

## Status

Draft / proposed.

## Context

Runenwerk is SDF-first and field-product-driven. The accepted architecture
separates authoritative domain descriptions, ratified formed products, derived
runtime caches, and expression products.

Animated SDF assets add new pressure to that model because authoring graphs,
semantic part graphs, rig/control graphs, motion graphs, deformation graphs,
and material or interaction layers all need to participate in runtime behavior.
Executing those graphs directly in the hot path would make validation,
scheduling, caching, determinism, product authority, and diagnostics difficult
to enforce.

## Decision

Animated SDF assets must not execute directly from editable authoring graphs in
runtime hot paths.

They must lower through:

- validated semantic IR;
- runtime field plans with explicit lineage and dependencies;
- scheduler-visible product jobs;
- purpose-specific products or proxies for render, physics, gameplay,
  navigation, VFX, editor preview, and diagnostics.

Render products may be approximate. Physics and gameplay products must be
certified for their stricter consumer requirements before those consumers use
them. Runtime caches, render proxies, editor overlays, and debug views remain
derived unless an owning product contract explicitly certifies them as strict
consumer truth.

## Rejected Alternatives

### Execute Authoring Graphs Directly

Rejected because it hides validation, scheduling, cache invalidation, and
consumer authority behind editable graph behavior.

### Use One Universal Animated Field

Rejected because rendering, physics, gameplay, AI/navigation, VFX, and editor
diagnostics need different correctness, performance, freshness, and fallback
rules.

### Let Runtime Caches Become Source Of Truth

Rejected because it would bypass accepted SDF-first product lineage and make
debugging, replay, multiplayer authority, and editor inspection fragile.

## Consequences

- Future animated SDF implementation must define semantic IR, product jobs,
  product descriptors, and diagnostics before runtime execution.
- Animation, SDF, physics, rendering, gameplay, VFX, editor, and runtime
  systems consume purpose-specific products instead of private graph state.
- Cache invalidation and publication must be scheduler-visible.
- Detailed follow-up ADRs should cover semantic region IDs, field correctness
  levels, runtime field plan ownership, cache invalidation doctrine, physics
  conservativeness, asset composition, scheduler sync points, and the first
  validation creature.
