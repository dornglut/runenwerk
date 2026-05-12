---
title: SDF-First Field World Platform Design
description: Accepted parent architecture for Runenwerk's SDF-first field world, product ownership, and layer boundaries.
status: accepted
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-12
related_adrs:
  - ../../adr/accepted/0004-separate-description-from-execution.md
  - ../../adr/accepted/0005-projections-are-derived-state.md
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
  - ./sdf-first-production-capability-map.md
supersedes:
  - ../superseded/workspace-field-world-and-simulation-platform-design.md
  - ../superseded/adaptive-field-product-system-design.md
---

# SDF-First Field World Platform Design

## Status

Accepted parent architecture.

This document defines the long-term architecture. It is not an MVP plan and
does not limit the platform to a small first slice.

## Purpose

Runenwerk's world architecture is SDF-first and product-driven. The platform is
built around editable field truth, formed products, explicit product lineage,
strict consumer policy, and domain-owned validation.

The architecture must support:

- editable SDF terrain, caves, interiors, prefabs, and field-driven worlds;
- material, wetness, vegetation, influence, radiance, collision, and diagnostic
  products;
- rendering and UI expression products as derived consumer outputs;
- physics, AI, procgen, VFX, water, atmosphere, fluids, snow, and erosion as
  long-term product-capable domains;
- open-world streaming through finite resident product working sets;
- multiplayer and replay authority through operations, generations, and
  explicitly authoritative products.

## SDF-First Doctrine

SDF-first means SDF and field products are the primary production substrate for
world geometry, editable spatial truth, collision/query formation, renderer
inputs, and product lineage.

SDF-first does not mean SDF-only. Meshes are allowed as derived, imported,
exported, debug, preview, fallback, or interoperability products. A mesh may be
authoritative only when an owning domain design explicitly accepts that role for
that product family.

The central rule is:

```text
Authoritative domain state -> ratified formed products -> derived runtime caches and expression products.
```

No renderer cache, UI projection, editor overlay, or debug product becomes
authoritative world truth by convenience.

## Ownership

Foundation owns reusable low-level vocabulary such as typed identity,
diagnostics, ratification, schema, and command contracts. Foundation must not
own world-field policy, rendering policy, simulation behavior, product
formation, or runtime execution.

Domain crates own engine-agnostic product contracts, descriptors, ratifiers,
query contracts, mutation requests, and invariants. Current owners remain:

- `domain/sdf`: analytic SDF math, primitives, composition, sampling, and core
  SDF queries.
- `domain/spatial`: world, chunk, region, clipmap, ring, and coordinate
  vocabulary.
- `domain/chunking`: desired residency planning around focus points.
- `domain/world_ops`: operation logs, dirty regions, build queues,
  invalidation, and replication deltas.
- `domain/world_sdf`: current SDF-world payloads, SDF field products, collision
  query contracts, previews, ratification, and cave summaries.
- future domain crates may own broader product families after accepted
  crate-level designs.

The target generic product contract owner is a domain-level field/product
contract boundary. It must not be foundation and must not be engine runtime.
Until that boundary exists in code, existing `world_sdf` product types remain
the current SDF-world specialization, not a universal product registry.

Engine runtime owns execution: scheduling integration, product job execution,
runtime resources, GPU residency, renderer submission, streaming execution,
plugin composition, cache formation, diagnostics projection, and serial or
parallel executors.

Apps and editor tooling own workflows, command routing, panels, projections,
and presentation. They consume products and diagnostics; they do not define
field-world invariants.

## Field World Model

The field world platform has four cooperating layers:

1. Authored and generated descriptions: edit operations, generated bases,
   imported sources, authored rules, and simulation source events.
2. Normalized and ratified domain state: validated descriptions and accepted
   mutations owned by their domain.
3. Formed products: typed products with scope, scale band, lineage, freshness,
   residency, consumer class, authority, diagnostics, and query policy.
4. Runtime caches and expression products: GPU buffers, atlases, page tables,
   history targets, scene color, picking ids, debug overlays, and UI products.

Only the first two layers may be authoritative. Formed products can be strict
consumer truth when their owning domain certifies them. Runtime caches and
expression products are derived.

## Scope And Scale

Accepted scope vocabulary:

- chunk and region for chunked world products;
- clipmap window for camera/focus-relative multiscale products;
- view for viewport and product-surface outputs;
- sector and portal for caves, interiors, and connected spaces;
- basin and river segment for water systems;
- non-spatial for global metadata, time/celestial products, producer metadata,
  and diagnostics.

Accepted scale-band vocabulary:

- near, mid, far, summary;
- preview;
- collision or strict-query;
- offline;
- family-specific bands when an owning design defines them.

Scale bands are product policy, not a global quality slider. Render, collision,
AI, water, and diagnostics may select different bands for the same scope.

## Mutation And Invalidation

World-affecting changes must pass through explicit operation, command, import,
generation, or simulation-mutation paths owned by a domain.

Invalidation records must identify:

- cause;
- affected scopes and scale bands;
- affected product families and consumers;
- generation or lineage change;
- rebuild priority and budget class;
- fallback permission;
- diagnostics.

Dirty flags are allowed as implementation details only when they are tied to
typed invalidation, lineage, and rebuild policy.

## Long-Term Capability Tracks

Terrain, caves, prefabs, characters, vegetation, water, atmosphere, physics,
AI influence, procgen, VFX, fluids, snow, erosion, and world processes are all
accepted long-term capability tracks for the SDF-first platform.

Acceptance of the architecture does not imply those systems are implemented or
that their future crate boundaries are already active. Each track must still
enter implementation through an owning design, domain boundary, product
contract, and validation plan.

## Invariants

- SDF/field products are the primary production substrate for world geometry
  and field-world truth.
- Meshes are allowed as derived or specialized products, but not as hidden
  replacements for SDF-first truth.
- Consumers use query contracts or prepared product selections, not private
  storage internals.
- Strict products and visual products are separate unless a domain certifies one
  product for both roles.
- Renderer, UI, diagnostics, and editor outputs are derived unless explicitly
  documented otherwise.
- Multiplayer-relevant authority is explicit per product family.
- Product failures, stale use, fallback use, and ghost summaries are
  diagnosable.

## Validation Expectations

Future implementation work should add tests for:

- product descriptors and ratification;
- invalidation and generation compatibility;
- strict consumer fallback rejection;
- renderer derived-cache invalidation;
- diagnostics for missing, stale, fallback, and unauthorized products;
- serialization or schema compatibility for accepted product descriptions.
