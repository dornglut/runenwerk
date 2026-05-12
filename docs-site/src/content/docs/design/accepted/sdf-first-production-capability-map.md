---
title: SDF-First Production Capability Map
description: Accepted long-term capability map for SDF-first world, rendering, simulation, gameplay, and tooling systems.
status: accepted
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-05-12
related_adrs:
  - ../../adr/accepted/0008-adopt-sdf-first-field-product-architecture.md
related_designs:
  - ./sdf-first-field-world-platform-design.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./sdf-product-renderer-and-gpu-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
supersedes:
  - ../deferred/sdf-world-production-slice-design.md
---

# SDF-First Production Capability Map

## Status

Accepted long-term capability map.

This document is not an MVP, prototype, or limited first-slice plan. It records
the accepted long-term system map and the gates each capability must pass before
implementation.

## Purpose

Runenwerk's production target is an SDF-first field world with product-driven
rendering, simulation, tooling, diagnostics, and future multiplayer authority.
Capability planning should preserve long-term architecture without pretending
every track is implemented or ready for immediate code work.

## Capability Groups

### World Substrate

Accepted direction:

- editable SDF terrain, caves, interiors, and large worlds;
- material, density, wetness, support, and provenance fields;
- chunk, region, clipmap, sector, portal, basin, and view scopes;
- deterministic generation plus authored operations and simulation mutation
  requests.

Implementation gate:

- owning domain product contracts;
- operation/invalidation path;
- ratification and diagnostics;
- streaming/residency policy.

### Rendering And Product Surfaces

Accepted direction:

- SDF terrain and field products as primary renderer input;
- hybrid GPU representation through sparse sampled SDF products and analytic or
  graph SDF products;
- dynamic product targets, prepared views, target aliases, history, and
  diagnostics.

Implementation gate:

- product selection contract;
- GPU residency and generation tracking;
- backend-neutral UI binding;
- stale/fallback/ghost diagnostics.

### Prefabs And Characters

Accepted direction:

- SDF-first prefabs for trees, rocks, ruins, props, interactables, and authored
  field emitters;
- SDF-first character definitions as the preferred long-term character path;
- mesh-derived or imported products allowed only when explicitly classified.

Implementation gate:

- prefab and character descriptor contracts;
- graph or composition substrate decision;
- rig/pose product contract for characters;
- render/collision product separation;
- diagnostics and bounds ratification.

### Vegetation, Water, Atmosphere, And VFX

Accepted direction:

- vegetation as deterministic field products, not authored per-blade state;
- water and wetness as scoped products, not renderer-only planes;
- day/night as explicit time, celestial, atmosphere, material-response, and
  schedule products;
- VFX as products with visual-only versus gameplay-relevant authority classes.

Implementation gate:

- field product descriptors for each family;
- renderer handoff and product diagnostics;
- explicit interaction and query products where gameplay-relevant;
- no hidden renderer authority.

### Physics, Collision, AI, And Gameplay Influence

Accepted direction:

- strict collision/query products separate from visual products;
- character movement, prefab collision, water queries, and gameplay interaction
  through strict products;
- influence fields for threat, scent, sound, visibility, navigation cost,
  magic/corruption, and debug heatmaps.

Implementation gate:

- strict query product formats;
- fallback certification policy;
- active-body residency/pinning;
- AI query contracts and authority classification;
- diagnostics for visual-only misuse.

### Procgen And World Processes

Accepted direction:

- procedural generation as a product producer with deterministic lineage;
- generated bases plus authored operations and simulation state form current
  products;
- future fluids, snow, erosion, sediment, ash, thermal, and material transport
  as product-driven process domains.

Implementation gate:

- generator descriptors and versioning;
- cache keys and lineage;
- mutation candidate format;
- domain ratification path;
- background/offline job policy;
- multiplayer authority classification.

## Design Gates

Every future capability track must define:

- owning domain or crate boundary;
- authoritative descriptions and derived products;
- product descriptor fields and query policy;
- strict versus visual product behavior;
- mutation and invalidation path;
- diagnostics and inspection DTOs;
- renderer/runtime handoff;
- validation tests;
- relationship to multiplayer/replay authority when relevant.

## Deferred Detail Drafts

Detailed future-system drafts remain useful as deferred design seeds, but they
are not implementation instructions until their gates are resolved:

- `../deferred/sdf-prefab-composition-system-design.md`
- `../deferred/sdf-character-animation-system-design.md`
- `../deferred/field-vegetation-system-design.md`
- `../deferred/day-night-atmosphere-system-design.md`
- `../deferred/water-wetness-field-system-design.md`
- `../deferred/sdf-physics-collision-system-design.md`
- `../deferred/field-influence-ai-system-design.md`
- `../deferred/procgen-field-product-system-design.md`
- `../deferred/field-vfx-particles-system-design.md`
- `../deferred/fluid-snow-erosion-world-processes-system-design.md`

## Invariants

- Long-term architecture is not reduced to a limited MVP.
- Capability acceptance does not bypass owner/domain design gates.
- No future track may use renderer caches, debug overlays, or local visual
  products as hidden authority.
- SDF-first remains the primary production direction while derived mesh products
  stay available where explicitly classified.
