---
title: Open World Product Streaming System Design
description: Superseded draft for product-based streaming, render distance, residency, and future cave-sector support.
status: superseded
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ../accepted/sdf-first-field-world-platform-design.md
---

# Open World Product Streaming System Design

## Status

Superseded draft.

Replaced by `../accepted/field-product-contracts-diagnostics-and-residency-design.md`
and `../accepted/sdf-first-field-world-platform-design.md`.

This document defines product-based streaming for an infinite SDF-first world.

The world may be infinite in address space, but only a finite working set of products is resident.

---

# Purpose

Runenwerk needs open-world streaming that supports:

- endless SDF terrain
- later endless cave networks
- grass and vegetation fields
- SDF prefabs
- SDF characters and enemies
- rivers/lakes/wetness
- day/night and atmosphere products
- AI influence fields
- physics/collision products
- editor diagnostics
- future multiplayer relevance

Streaming should load and evict typed products, not whole levels.

---

# Core Concept

```text
infinite addressable world
  -> deterministic generation and authored operations
  -> scoped products
  -> finite resident product working set
```

Every streamed product has:

- identity
- scope
- scale band
- product family
- generation
- freshness
- residency state
- fallback rule
- consumer class
- diagnostics

---

# Design Goals

1. Stream field products, not monolithic levels.
2. Support infinite coordinate space with finite memory.
3. Support product-specific render distance.
4. Keep rendering, physics, AI, water, and diagnostics on different residency rules.
5. Support future cave sector/portal streaming.
6. Support fallback and ghost summaries safely.
7. Support deterministic generation plus persistent edits.
8. Support multiplayer relevance and generation checks later.
9. Make residency visible through diagnostics.
10. Avoid artifact-heavy LOD transitions.

---

# Non-Goals

This design is not:

- a renderer
- a terrain generator
- a network replication system
- a cave generator
- a fluid simulation
- a product payload format

It defines product streaming and residency rules.

---

# Addressing Model

## World scope

Baseline open-world scopes:

```text
WorldId
  -> RegionId
    -> ChunkId
      -> ProductId
```

Products may also use:

- clipmap window
- view scope
- water basin
- biome region
- gameplay relevance region
- diagnostic scope

## Future cave scope

Caves need topology-aware scopes:

```text
CaveSystemId
  -> CaveSectorId
    -> PortalId
    -> Chamber/Tunnel ProductId
```

Distance is not enough for caves. Cave streaming depends on connectivity, visibility, portals, gameplay relevance, and path-back safety.

---

# Product Residency States

| State | Meaning |
|---|---|
| Resident | Payload is loaded and usable. |
| Non-Resident | Known product is not loaded. |
| Pending Load | Product has been requested for load. |
| Pending Unload | Product is scheduled for unload. |
| Rebuilding | Product is being regenerated or updated. |
| Stale | Product exists but is outdated. |
| Potentially Stale | Product may still be usable but upstream changed. |
| Fallback Resident | Lower-quality fallback is resident. |
| Ghost Summary | Lightweight non-authoritative continuity summary. |
| Missing | No usable product exists. |
| Failed Preserved | Prior valid product is retained after failure. |

---

# Render Distance as Product Selection

Render distance is not one number.

It is a set of product selection policies:

| Policy | Example |
|---|---|
| Visual distance | how far visual products render |
| Physics distance | how far collision products stay strict |
| AI distance | how far influence/navigation products stay active |
| Audio distance | how far acoustic/sound fields matter |
| Water distance | how far water simulation/render products stay active |
| Lighting distance | how far lighting/radiance products update |
| Prefetch distance | how far ahead products are requested |
| Multiplayer relevance | what state matters to clients/server |

The renderer asks for the best visual products. Physics asks for strict collision products. AI asks for influence/navigation products. They should not share one global distance.

---

# Scale Bands

Baseline bands:

| Band | Meaning |
|---|---|
| Near | High-detail active area around player/camera/focus. |
| Mid | Medium-detail context. |
| Far | Coarse visual/simulation context. |
| Summary | Very coarse regional/world-level representation. |
| Preview | Editor/authoring product. |
| Collision | Physics-safe product band. |
| Offline | High-quality non-realtime product. |

Scale bands are product-specific. Rendering LOD and physics LOD do not have to match.

---

# Product Families and Streaming Rules

## Terrain/SDF products

Near:

- high-res SDF chunks
- material fields
- collision products

Mid:

- medium SDF or summary products
- visual materials
- simplified collision only if needed

Far:

- coarse terrain summaries
- no exact collision
- visual-only fallback

## Grass/vegetation products

Near:

- density fields
- procedural blade/clump representation
- trample/bend fields

Mid:

- clump fields
- lower detail wind response

Far:

- material/normal/shimmer contribution only

## Prefab products

Near:

- full SDF prefab instance products
- collision/query products if interactive

Mid:

- simplified SDF or summary

Far:

- silhouette/summary/fallback product

## Character/enemy products

Player:

- always high priority

Nearby enemies:

- animation/render/collision/AI products resident

Distant enemies:

- simulation summaries or dormant products

## Water products

Near:

- water surface field
- wetness field
- flow field
- buoyancy/query product if gameplay-relevant

Mid/Far:

- visual summaries
- coarse flow/wetness
- no strict buoyancy unless gameplay requires it

## Atmosphere/day-night products

Global but lightweight:

- resident by default
- may have view-specific render products
- should not require terrain product residency

## Diagnostics products

Editor-selected diagnostics should override normal streaming priority where safe.

---

# Streaming Priority

Product priority should consider:

- camera distance
- player distance
- frustum visibility
- screen-space size
- gameplay importance
- physics relevance
- AI relevance
- water/flow activity
- editor selection
- multiplayer relevance
- predicted movement
- cave connectivity later
- portal visibility later
- fallback quality
- product freshness
- rebuild cost

Priority must be budgeted.

---

# Budgets

Streaming should honor budgets:

- maximum resident memory
- maximum GPU upload per frame
- maximum CPU product formation time
- maximum rebuild jobs
- maximum unloads per frame
- maximum high-priority misses
- background rebuild budget
- editor diagnostic override budget

Budget failures should create diagnostics.

---

# Fallback and Ghost Summary Rules

## Fallback

A fallback is an explicitly chosen lower-quality product.

Examples:

```text
near terrain SDF -> mid SDF -> far summary -> missing
```

Fallback is allowed for visual products, but strict products require tighter rules.

## Ghost summary

A ghost summary is lightweight continuity data.

Allowed:

- distant visual continuity
- far lighting continuity
- rough influence continuity
- editor streaming visualization

Not allowed unless explicitly designed:

- precise collision
- authoritative water simulation
- final gameplay interaction
- exact physics correction
- authoritative enemy decision state

Ghost usage must be diagnosable.

---

# Deterministic Generation and Persistence

Base world:

```text
world seed + coordinates + generation rules = generated base product
```

Current world:

```text
generated base product + authored edits + simulation state = formed product
```

Persist:

- world seed
- generation rules
- operation logs
- edited chunks/sectors
- important simulation state
- selected cached products
- product generations

Do not persist the entire infinite world.

---

# Multiplayer Model

Multiplayer should replicate authoritative operations and generations, not local caches.

Replicate:

- world operations
- terrain/cave edits
- gameplay-relevant simulation deltas
- product generations
- dirty regions
- authoritative influence/water/physics state where needed

Do not replicate:

- local render caches
- GPU resources
- local GI history
- editor-only overlays
- temporary visual LOD products

Authority is per product family.

---

# Future Cave Streaming

Caves require a topology-aware extension.

Cave-specific products:

- cave sector product
- cave portal graph
- cave SDF product
- cave collision product
- cave navigation product
- cave lighting/darkness product
- cave acoustic product
- cave airflow product
- cave influence product

Rules:

1. Do not unload geometry visible through a portal.
2. Keep path-back sectors partially resident.
3. Preserve portal silhouettes.
4. Stream connected sectors, not only nearest chunks.
5. Use summaries for deep branches.
6. Diagnostics must show sector residency.

---

# Runtime Flow

```text
player/camera moves
  -> determine active scopes
  -> evaluate product policies
  -> score product priority
  -> update residency targets
  -> schedule loads/rebuilds/unloads
  -> renderer/physics/AI query best products
  -> diagnostics report stale/fallback/missing states
```

---

# Open Questions

1. What are the canonical chunk and region sizes for the first production slice?
2. How many scale bands are required before implementation begins?
3. What products are allowed to use ghost summaries?
4. Which products may be stale but still render?
5. What is the first memory budget target?
6. How should editor selection override streaming policy?
7. What is the first multiplayer-relevant product set?
8. How much prefetch is required for fast traversal?
9. How should caves map onto existing chunk/region scopes?
10. Should water basins be independent streaming scopes from regions?

---

# Design Decisions

1. Streaming is product-based, not level-based.
2. Infinite world means infinite addressability, not infinite loaded data.
3. Render distance is product selection.
4. Physics and gameplay products use stricter fallback rules than visual products.
5. Ghost summaries are non-authoritative.
6. Caves will require sector/portal scope.
7. Multiplayer replication will target operations and generations, not derived render caches.
8. Product residency must be diagnosable.
9. Streaming budgets are mandatory.
10. Deterministic generation plus edit logs is the persistence model.

---

# Implementation Phases

## Phase 1: Product Residency Contracts

Deliver:

- residency states
- product streaming policy descriptors
- fallback/ghost rules
- diagnostics vocabulary

## Phase 2: Chunk/Region Product Streaming

Deliver:

- product residency table
- chunk/region resolver
- near/mid/far product selection
- basic budgets

## Phase 3: Render Product Streaming

Deliver:

- visual product selection
- render distance policy
- GPU upload scheduling
- fallback summaries

## Phase 4: Strict Product Streaming

Deliver:

- collision product policy
- gameplay-relevant product rules
- no ghost authority rules

## Phase 5: Diagnostics

Deliver:

- residency overlay
- stale/fallback/ghost views
- missing product diagnostics
- budget diagnostics

## Phase 6: Cave Scope Design

Deliver:

- cave sector/portal scope
- connected-sector streaming rules
- cave product priorities

## Phase 7: Multiplayer Relevance

Deliver:

- generation checks
- authoritative product classes
- replicated operation/relevance rules

---

# Acceptance Criteria

This design is accepted when:

1. Products can be loaded and evicted by scope and family.
2. Renderer can select visual products without loading full world truth.
3. Physics can demand strict products and reject visual fallbacks.
4. Product residency state is inspectable.
5. Fallback and ghost usage are explicit.
6. Infinite terrain can be represented as finite resident products.
7. The model leaves room for infinite cave networks.
8. Multiplayer authority is not blocked by local caching.
