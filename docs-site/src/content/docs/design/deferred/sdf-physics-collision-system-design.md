---
title: SDF Physics Collision System Design
description: Deferred detail draft for SDF-first collision, physics query products, authority, LOD safety, and diagnostics.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
  - ./sdf-prefab-composition-system-design.md
  - ./sdf-character-animation-system-design.md
  - ./water-wetness-field-system-design.md
---

# SDF Physics Collision System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after strict collision product ownership, fallback certification, and
active-body residency policy are accepted.

This document defines physics and collision for an SDF-first engine.

The system must not treat render products as authoritative collision. Collision and physics queries are separate product families with stricter correctness rules.

---

# Purpose

Runenwerk needs a production collision architecture for:

- endless SDF terrain
- caves later
- SDF characters and enemies
- SDF prefabs
- water/wetness query interaction
- grass/vegetation interaction
- gameplay hit/query volumes
- multiplayer authority
- artifact-safe LOD transitions
- editor diagnostics

The first production slice requires a player moving through an SDF field, interacting with grass/water, and encountering enemies.

---

# Core Concept

Physics consumes strict products:

```text
SDF terrain / prefab / character products
  -> collision/query products
  -> physics runtime / gameplay queries
  -> contact, movement, interaction, diagnostics
```

Visual render products may share lineage with collision products, but they are not equivalent.

---

# Design Goals

1. Keep collision SDF-first.
2. Separate visual products from strict physics/query products.
3. Support player and enemy movement over SDF terrain.
4. Support SDF prefab collision.
5. Support SDF character collision/query products.
6. Support water query products without requiring full fluid simulation.
7. Support artifact-safe collision LOD transitions.
8. Support multiplayer authority boundaries.
9. Support diagnostics for missing/stale/fallback products.
10. Avoid renderer-owned collision truth.

---

# Non-Goals

This design is not:

- a full rigid-body solver specification
- a complete character controller implementation
- a full fluid simulation
- a networking protocol
- a renderer design
- an ECS scheduler design

It defines collision/query products, runtime boundaries, and product authority rules.

---

# Product Families

## Collision Field Product

A strict product used for collision and movement queries.

Examples:

- terrain collision SDF
- cave collision SDF
- prefab collision SDF
- character body collision field
- conservative obstacle field

## Contact Query Product

Supports contact and penetration queries.

Examples:

- distance to terrain
- normal at point
- penetration depth
- support point
- slope/walkability classification

## Movement Query Product

Supports character/enemy movement.

Examples:

- ground height
- ground normal
- step eligibility
- walkability
- friction/movement modifier
- water depth
- slope limit

## Interaction Query Product

Supports gameplay and field interactions.

Examples:

- footstep surface class
- wetness at point
- grass/trample eligibility
- hit region
- attack overlap
- pickup/interactable region

## Broadphase Product

Supports efficient candidate lookup.

Examples:

- chunk/region bounds
- prefab bounds
- dynamic character bounds
- active body sets
- spatial hash or AABB tree

## Physics Diagnostic Product

Exposes collision/query state and authority issues.

---

# Strict vs Visual Products

Visual products may be approximate, stale, or fallback if policy allows.

Strict products have stronger rules:

| Product | Visual Use | Strict Physics Use |
|---|---|---|
| render SDF | yes | no, unless certified strict |
| ghost summary | yes for visual continuity | no |
| far terrain summary | yes | no, except broad conservative query |
| collision SDF | maybe | yes |
| conservative collision fallback | maybe | yes if policy allows |
| stale collision | maybe no | no unless explicitly accepted |

Rules:

1. Strict consumers must reject visual-only fallback.
2. Ghost summaries cannot satisfy authoritative collision.
3. Collision products must be generation-compatible with gameplay state.
4. Product fallback must be diagnosable.
5. Contact stability outranks visual LOD.

---

# Character Movement

The first production slice needs character movement over SDF terrain.

Required queries:

- ground distance
- ground normal
- slope angle
- step/ledge classification
- water depth
- surface material
- footstep event position
- collision correction
- movement blocker query

Movement should consume collision/query products, not renderer field state.

---

# SDF Character Collision

Animated SDF characters produce collision/query products.

Possible first representation:

- conservative capsule-like SDF body
- foot contact points
- attack/query volumes
- simple limb bounds

Future:

- per-limb collision fields
- deformable body approximation
- equipment/weapon query fields
- hit location regions

Collision products should update with pose generation.

---

# SDF Prefab Collision

SDF prefab instances may produce:

- static collision field
- conservative bounds
- interaction query product
- navigation blocker
- hit/query volume

Examples:

- rock collision
- ruin wall collision
- tree trunk collision
- riverbank obstacle field

Prefab visual LOD must not remove strict collision near active bodies.

---

# Terrain and Cave Collision

## Terrain

Terrain collision uses chunk/region products.

Near player:

- strict collision products resident
- updated with terrain operation generation
- no visual fallback

Far:

- no strict collision required
- broad summaries allowed for planning only

## Caves later

Caves require sector/portal-aware collision residency.

Rules:

- keep current sector strict collision resident
- keep path-back sector collision safe
- never unload collision under active bodies
- portal-visible geometry must have appropriate collision if reachable

---

# Water Queries

Water may provide query products:

- is point in water
- water depth
- surface height
- flow vector
- wetness
- movement slow modifier
- buoyancy region

Full fluid simulation is not required for first slice, but query products must be structured.

Visual water cannot be used as authoritative water collision/query unless explicitly paired with a query product.

---

# Collision LOD and Transition Safety

Collision LOD must avoid instability.

Rules:

1. Do not switch collision product while an active body is contacting it.
2. Pin strict products near the player/enemies.
3. Use hysteresis for collision residency.
4. Use conservative fallback only if certified.
5. Emit diagnostics when fallback is used.
6. Visual LOD and physics LOD are independent.
7. Contact caches must be invalidated on product generation change.

---

# Broadphase / Narrowphase Split

## Broadphase

Identifies candidates.

Possible structures:

- spatial hash
- chunk map
- region bounds
- AABB tree
- active body set
- prefab bounds index

## Narrowphase

Uses field queries.

Examples:

- sphere/capsule vs SDF
- character support query
- ray/shape cast against field
- point/volume overlap
- swept query later

Broadphase can use coarse summaries. Narrowphase needs strict products.

---

# Multiplayer Authority

Multiplayer must explicitly classify products.

Likely rules:

| Product | Authority |
|---|---|
| terrain operation log | server/world authoritative |
| strict collision product | server-authoritative or server-validated |
| player movement result | server-authoritative/validated |
| visual terrain product | client-derived |
| water visual product | client-derived |
| water gameplay query | server-authoritative if gameplay relevant |
| local contact debug | client-local |

Replicate operations/generations, not render caches.

---

# Runtime Integration

Physics runtime consumes:

- active body set
- strict product resolver
- collision residency table
- movement query products
- broadphase products
- generation/freshness diagnostics

It produces:

- contacts
- movement corrections
- query results
- interaction events
- dirty/invalidation requests where allowed
- diagnostics

---

# Diagnostics

Physics/collision diagnostics should expose:

- missing strict collision
- stale collision product
- visual fallback rejected
- ghost summary rejected
- collision/render generation mismatch
- active body on unloading product
- unsafe LOD transition
- contact instability
- missing water query product
- missing prefab collision product
- multiplayer authority mismatch

---

# Open Questions

1. What is the first strict collision product format: sampled SDF, analytic SDF, or hybrid?
2. What is the minimum character controller query set?
3. Should collision product descriptors live in `world_sdf` or a future physics domain?
4. How are collision products ratified?
5. What is the first broadphase structure?
6. How are active body regions pinned for streaming?
7. What collision fallback is allowed for generated terrain?
8. How are SDF prefab collision products authored?
9. What collision state is authoritative in multiplayer?
10. How do collision products interact with future fluid products?

---

# Design Decisions

1. Collision is SDF-first.
2. Visual products are not strict collision products.
3. Ghost summaries cannot satisfy authoritative collision.
4. Physics LOD and render LOD are independent.
5. Collision products must have product generation metadata.
6. Active contact regions pin strict products.
7. Water query products are separate from visual water.
8. Multiplayer authority is explicit per product.
9. Diagnostics are required.
10. First production slice requires character movement over SDF terrain.

---

# Implementation Phases

## Phase 1: Collision Product Contracts

Deliver:

- collision product descriptor
- movement query descriptor
- interaction query descriptor
- broadphase summary descriptor
- diagnostics vocabulary

## Phase 2: Terrain Collision Query

Deliver:

- strict SDF terrain query
- ground distance/normal/slope
- generation compatibility checks
- diagnostics

## Phase 3: Character Movement Queries

Deliver:

- capsule/body movement query
- footstep surface query
- water depth query hook
- trample event hook

## Phase 4: Prefab Collision

Deliver:

- SDF prefab collision products
- bounds index
- strict/fallback rules
- diagnostics

## Phase 5: LOD and Residency Safety

Deliver:

- active body product pinning
- collision LOD hysteresis
- fallback rejection diagnostics

## Phase 6: Enemy/Gameplay Queries

Deliver:

- enemy collision products
- attack/query volumes
- influence/AI handoff

## Phase 7: Multiplayer Authority

Deliver later:

- server validation/generation checks
- authoritative collision classifications
- replicated invalidation rules

---

# Acceptance Criteria

This design is accepted when:

1. Player movement can query strict SDF terrain collision.
2. Visual products cannot silently satisfy strict physics.
3. SDF prefabs can produce collision products.
4. Animated SDF characters can produce movement/collision products.
5. Collision LOD changes are safe near active bodies.
6. Diagnostics explain missing/stale/fallback collision state.
7. Future multiplayer authority is not blocked by local caches.
