---
title: SDF Product Renderer Architecture
description: Superseded draft for the SDF-first renderer as a product execution layer over field products.
status: superseded
owner: workspace
layer: engine/runtime
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
---

# SDF Product Renderer Architecture

## Status

Superseded draft.

Replaced by `../accepted/sdf-product-renderer-and-gpu-residency-design.md`.

This document defines how the Runenwerk renderer should evolve to support an SDF-first, field-product-driven world.

The renderer must not become the source of world truth. It should execute render flows over prepared SDF and field products supplied by domain/runtime product systems.

The design assumes the Adaptive Field Product System provides product identity, scope, scale band, lineage, freshness, residency, query contracts, and diagnostics.

---

# Purpose

Runenwerk needs a renderer that can consume and present SDF-first production systems:

- SDF terrain and caves
- SDF prefabs
- SDF characters and enemies
- grass and vegetation fields
- water and wetness fields
- material and substance fields
- day/night atmosphere products
- lighting/radiance products
- fog, mist, smoke, and VFX fields
- diagnostics and editor overlays

The renderer should remain an execution layer:

```text
field products / runtime product selection
  -> prepared render products
  -> GPU residency and upload
  -> render flow execution
  -> product surfaces / game surface / editor viewport
```

It should not own authored world state, simulation truth, ECS entity semantics, prefab semantics, gameplay authority, or editor workflow policy.

---

# Design Goals

1. Keep the renderer SDF-first.
2. Keep render execution separate from product truth.
3. Allow one render flow to render many prepared views and product surfaces.
4. Support dynamic product surfaces, offscreen targets, and history resources.
5. Add GPU residency for SDF and field products.
6. Support SDF terrain, SDF prefabs, SDF characters, vegetation, water, and atmosphere.
7. Support renderer diagnostics for stale, fallback, missing, or ghost products.
8. Avoid mesh-centric renderer assumptions.
9. Avoid live ECS extraction during submit.
10. Preserve current render-flow direction instead of replacing it with a monolithic renderer.

---

# Non-Goals

This design is not:

- a mesh renderer design
- a physics system
- a world-authoring system
- a prefab authoring system
- a material graph design
- a fluid solver
- an animation graph design
- an editor viewport ownership model
- a multiplayer replication model

Those systems produce or describe products. The renderer consumes prepared render products.

---

# Core Doctrine

## Renderer consumes products

The renderer receives prepared render product sets.

Examples:

| Product | Renderer Usage |
|---|---|
| SDF terrain products | world raymarch / field sampling |
| material field products | shading and surface classification |
| SDF prefab products | trees, rocks, ruins, props |
| SDF character products | player, enemies, creatures |
| vegetation products | grass/reeds/clumps |
| water products | rivers, lakes, wetness, foam, mist |
| atmosphere products | sky, sun, moon, fog, exposure |
| diagnostic products | overlays, inspectors, stale/fallback views |

The renderer should never treat these as authored truth.

## Runtime cache is derived

GPU buffers, atlases, page tables, target caches, temporal histories, and acceleration structures are derived.

They must track product generations and must be invalidated or refreshed when upstream products change.

## Product surfaces are first-class

The renderer should be able to render to:

- main surface
- editor viewport surface
- offscreen product surface
- debug product surface
- material/field preview surface
- history/temporal target

A flow should not need to be cloned per viewport or per product.

## SDF first, mesh optional

Meshes may exist as derived fallback, debug, import, or export products.

The main renderer architecture must not require mesh assets for terrain, characters, vegetation, rocks, trees, enemies, or water.

---

# High-Level Architecture

```text
Adaptive Field Product System
  -> Product Registry
  -> Product Residency
  -> Product Resolver
  -> Render Product Resolver
  -> Prepared Render Product Set
  -> SDF GPU Residency
  -> Render Producers
  -> Render Flow Execution
  -> Product Surfaces / Main Surface / Viewports
```

## Main renderer subsystems

| Subsystem | Responsibility |
|---|---|
| Render Product Resolver | Chooses best renderable products for a view. |
| SDF GPU Residency | Uploads active SDF/field products to GPU resources. |
| Prepared Product Set | Per-view/per-frame render input snapshot. |
| SDF World Producer | Prepares terrain/cave/world SDF inputs. |
| SDF Prefab Producer | Prepares SDF prefab instances. |
| SDF Character Producer | Prepares animated SDF characters/enemies. |
| Vegetation Producer | Prepares grass/reed/plant field inputs. |
| Water/Wetness Producer | Prepares water surface, flow, wetness, foam, mist. |
| Atmosphere Producer | Prepares day/night, sky, fog, exposure, sun/moon. |
| Diagnostics Producer | Prepares overlays and inspection products. |
| Render Flow Executor | Executes prepared flows and target aliases. |

---

# Render Product Resolver

The render product resolver selects products for a specific view.

Inputs:

- camera/view state
- viewport dimensions
- product registry
- product residency table
- product freshness state
- scale-band policy
- fallback policy
- debug flags
- visibility/culling context

Outputs:

- selected SDF terrain products
- selected material field products
- selected SDF prefab instances
- selected SDF character products
- selected vegetation products
- selected water products
- selected atmosphere products
- selected diagnostic overlays
- fallback/ghost indicators

The resolver should answer:

```text
What is the best currently usable render product for this scope, band, and consumer?
```

It should not answer:

```text
What is the true world state?
```

---

# Prepared Render Product Set

A prepared render product set is the per-view snapshot consumed by render producers and flows.

It should include:

- view identity
- camera data
- selected product IDs
- product generations
- target descriptors
- selected scale bands
- selected fallback states
- debug/diagnostic toggles
- history target references
- GPU residency handles, if already resolved

This keeps rendering deterministic for the frame.

---

# SDF GPU Residency

SDF GPU residency owns GPU packing and upload.

It does not own product truth.

## Required structures

| Structure | Purpose |
|---|---|
| SDF Brick Atlas | Stores sampled distance/material data for chunks/regions. |
| SDF Page Table | Maps world scopes to atlas pages and scale bands. |
| Material Field Atlas | Stores material, biome, wetness, substance channels. |
| SDF Instance Buffer | Stores prefab instances and transforms. |
| Animated SDF Pose Buffer | Stores character/enemy pose-driven field transforms. |
| Vegetation Field Buffer | Stores grass density, wind, bend, trample fields. |
| Water Field Buffer | Stores water mask, flow, wetness, foam/mist fields. |
| Atmosphere Buffer | Stores sun/moon/sky/fog/exposure state. |
| Diagnostic Buffer | Stores stale/fallback/ghost/debug states. |

## Residency invariants

1. Every uploaded resource is tied to a product generation.
2. Stale product data is visibly marked or invalidated.
3. Missing products use explicit fallbacks.
4. Ghost summaries are never treated as full-authority render data.
5. GPU allocation failure produces diagnostics.
6. Renderer-owned resources do not leak backend details into domain crates.

---

# Render Producers

## SDF World Producer

Prepares:

- terrain SDF chunks
- cave/world SDF regions
- material fields
- near/mid/far scale bands
- fallback summaries
- debug state

First production use:

- endless SDF field terrain
- later caves and interiors

## SDF Prefab Producer

Prepares:

- SDF trees
- SDF rocks
- SDF ruins
- SDF props
- field composition instances
- material overrides
- bounds
- LOD/fade state

Prefabs are SDF compositions, not mesh instances.

## SDF Character Producer

Prepares:

- animated SDF player
- animated SDF enemies
- pose products
- SDF body-part transforms
- material masks
- local bounds
- interaction markers

No mesh skinning should be required for the core path.

## Vegetation Producer

Prepares:

- grass density fields
- species rules
- deterministic seeds
- wind inputs
- trample/bend fields
- recovery state
- LOD fade state

Near vegetation may be explicit procedural SDF blades/clumps. Far vegetation may be density/material contribution.

## Water/Wetness Producer

Prepares:

- water masks
- level-set or surface fields
- flow direction
- shoreline wetness
- foam/mist
- reflection/refraction approximation data
- water diagnostics

The first production slice can use structured simple water products. Full fluid simulation can feed the same producer later.

## Atmosphere Producer

Prepares:

- normalized time-of-day
- sun direction/color/intensity
- moon direction/color/intensity
- sky gradient
- fog color/density
- exposure
- ambient term
- night glow influence

Day/night is a product source, not just a shader color lerp.

## Diagnostics Producer

Prepares overlays for:

- product residency
- stale products
- fallback usage
- ghost summaries
- missing products
- invalid scale-band selection
- rebuild failures
- consumer/product usage

---

# Render Flows

The renderer should keep reusable render flows and add SDF-first flows.

## Required flow families

| Flow | Purpose |
|---|---|
| `sdf.world.primary` | Render SDF terrain/world fields. |
| `sdf.prefab.instances` | Render SDF prefab instances. |
| `sdf.character.animated` | Render animated SDF characters/enemies. |
| `field.vegetation.grass` | Render procedural field vegetation. |
| `field.water.surface` | Render rivers/lakes/wetness/foam/mist. |
| `world.atmosphere.day-night` | Render sky/fog/atmosphere lighting state. |
| `debug.field-products` | Render product diagnostics and overlays. |

Flows may be compute, fullscreen, graphics, or hybrid depending on backend maturity.

---

# Culling and Product Selection

Renderer culling should combine:

- frustum culling
- product residency
- scale-band selection
- distance thresholds
- screen-space error
- occlusion where available
- cave/sector/portal visibility later
- fallback availability
- debug mode overrides

Render distance is product selection, not global world loading.

---

# LOD and Artifact Control

The renderer must support artifact-safe transitions:

- cross-fade product bands
- geomorph or field-blend where applicable
- dithered vegetation transitions
- hysteresis on LOD changes
- stable procedural seeds
- temporal smoothing for lighting/atmosphere
- pinned collision products handled outside render path
- diagnostics when fallback causes visible quality loss

Hard popping should be considered a renderer/product integration defect.

---

# Day/Night Integration

The atmosphere producer updates:

- sky
- sun/moon lighting
- fog
- exposure
- material response parameters
- water appearance
- vegetation glow/visibility parameters
- enemy/night hooks through non-render systems

The renderer should consume day/night state through prepared atmosphere products.

It should not directly mutate gameplay state.

---

# Editor and Product Surfaces

The renderer must support:

- game surface
- editor viewport
- field preview
- volume slice preview
- material preview
- product diagnostics view
- offscreen history targets

Editor surfaces should consume renderer products and diagnostics. They should not own renderer internals.

---

# Open Questions

1. What is the canonical GPU representation for sampled SDF chunks: dense bricks, sparse bricks, clipmap pages, or a hybrid?
2. How much SDF evaluation should be shader-side analytic versus uploaded sampled fields?
3. How are animated SDF character graphs encoded for GPU evaluation?
4. How many product generations can remain resident before eviction?
5. Should atmosphere be part of render runtime only, or have a domain-level time/day-night contract?
6. What is the first supported material-field channel set for the production slice?
7. How should renderer inspection expose product lineage without pulling domain internals into renderer code?
8. What fallback is allowed for missing SDF terrain in the main game surface?
9. Do field-product diagnostics render as overlays, panels, or both?
10. How strict should the renderer be when asked to render stale products?

---

# Design Decisions

1. The renderer remains an execution layer, not a world owner.
2. SDF/field products are the primary renderer input.
3. Meshes are not the core render path.
4. Runtime GPU resources are derived state.
5. Product surfaces and prepared views remain central.
6. Render producers prepare typed feature contributions.
7. Render flows stay reusable and view/product agnostic.
8. Day/night is handled through atmosphere products.
9. Diagnostics are part of the renderer contract.
10. Artifact-free LOD transitions are a quality requirement, not polish.

---

# Implementation Phases

## Phase 1: Product-Surface Closeout

Deliver:

- dynamic target cache allocation
- target alias execution
- prepared offscreen view execution
- history target allocation/invalidation
- render inspection for dynamic targets/history
- no live ECS extraction at submit

## Phase 2: SDF Field Preview Rendering

Deliver:

- render one SDF field product
- product surface output
- viewport display
- freshness/fallback diagnostics

## Phase 3: SDF GPU Residency

Deliver:

- SDF brick/page-table prototype
- product generation tracking
- fallback resource handling
- residency diagnostics

## Phase 4: SDF World Producer

Deliver:

- terrain/world SDF product selection
- near/mid/far render bands
- material field binding
- debug overlays

## Phase 5: Atmosphere and Day/Night

Deliver:

- time-of-day product
- sun/moon/fog/exposure buffers
- visible render response
- diagnostic controls

## Phase 6: SDF Characters and Prefabs

Deliver:

- SDF prefab instance rendering
- simple animated SDF character rendering
- character pose product handoff

## Phase 7: Vegetation and Water

Deliver:

- grass density/wind/trample rendering
- water/wetness/foam/mist rendering
- artifact-safe LOD transitions

---

# Acceptance Criteria

This architecture is accepted when:

1. The renderer can render SDF products without mesh assumptions.
2. Product surfaces can be rendered through prepared views.
3. SDF product freshness/residency is visible in diagnostics.
4. The renderer can consume terrain, material, prefab, character, vegetation, water, and atmosphere products.
5. Missing/stale/fallback products are explicit.
6. Day/night is represented as a product-driven render input.
7. The first production slice can be built without a renderer redesign.
