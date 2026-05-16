---
title: Animated SDF Authoring Graphs Lower Before Runtime
description: Decision that animated SDF authoring graphs must lower through ratified products before runtime hot paths consume them.
status: accepted
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
  - 0004-separate-description-from-execution.md
  - 0005-projections-are-derived-state.md
  - 0008-adopt-sdf-first-field-product-architecture.md
preserves_context_from:
  - ../proposed/animated-sdf-lowering-and-purpose-specific-products.md
---

# ADR: Animated SDF Authoring Graphs Lower Before Runtime

## Status

Accepted.

This ADR accepts only the durable execution-boundary invariant from the broader
Animated SDF proposal. The broader product-family context remains preserved in
the proposed ADR and active design.

## Context

Runenwerk is SDF-first and field-product-driven. The accepted architecture
separates authoritative domain descriptions, ratified formed products, derived
runtime caches, and expression products.

Animated SDF assets add authoring graphs, semantic part graphs, rig/control
graphs, motion graphs, deformation graphs, and material or interaction layers.
Executing editable graphs directly in runtime hot paths would make validation,
scheduling, caching, determinism, product authority, and diagnostics difficult
to enforce.

## Decision

Animated SDF assets must not execute directly from editable authoring graphs in
runtime hot paths.

Before runtime systems consume animated SDF behavior, authored graphs must lower
through:

- validated semantic IR;
- runtime field plans or equivalent formed execution plans with explicit lineage
  and dependencies;
- scheduler-visible product jobs;
- purpose-specific products or proxies for the consuming domain.

Render products may be approximate. Strict consumers such as physics, gameplay,
server authority, and validation must consume only products explicitly certified
for their correctness requirements by an owning product contract.

Runtime caches, render proxies, editor overlays, and debug views remain derived
state unless an owning product contract explicitly certifies them as strict
consumer truth.

## Non-Decisions

This ADR does not decide:

- semantic region ID shape;
- field correctness levels;
- runtime field plan crate ownership;
- cache invalidation doctrine;
- physics proxy conservativeness;
- asset composition policy;
- scheduler sync point details;
- the first concrete validation scenario.

Those topics remain in the active design and proposed ADR until each is promoted
through its own accepted decision or owning design.

## Rejected Alternatives

Execute authoring graphs directly. Rejected because it hides validation,
scheduling, cache invalidation, and consumer authority behind editable graph
behavior.

Use one universal animated field. Rejected because rendering, physics, gameplay,
AI/navigation, VFX, and editor diagnostics need different correctness,
performance, freshness, and fallback rules.

Let runtime caches become source of truth. Rejected because it would bypass
accepted SDF-first product lineage and make debugging, replay, multiplayer
authority, and editor inspection fragile.

## Consequences

Future animated SDF implementation must define semantic IR, product jobs,
product descriptors, and diagnostics before runtime execution.

Animation, SDF, physics, rendering, gameplay, VFX, editor, and runtime systems
consume purpose-specific products instead of private graph state.

Cache invalidation and publication must be scheduler-visible.
