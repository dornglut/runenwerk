---
title: Water Wetness Field System Design
description: Deferred detail draft for SDF/field-first rivers, lakes, wetness, flow, foam, mist, buoyancy, and future fluid handoff.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./field-vegetation-system-design.md
  - ./day-night-atmosphere-system-design.md
---

# Water Wetness Field System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after water/wetness product ownership, basin/river scope policy, and query
contracts are accepted.

This document defines the first production water/wetness field system.

It is not a full fluid simulation design. It defines structured water products that can later be enriched or replaced by fluid solver products.

---

# Purpose

Runenwerk needs rivers and lakes in the first production slice.

Water should affect:

- rendering
- shoreline materials
- wetness
- grass/reeds
- mist/foam
- simple buoyancy/query behavior
- movement/gameplay later
- diagnostics
- future fluid simulation

Water must not start as a renderer-only plane.

---

# Core Concept

```text
water source / mask / basin / flow
  -> water field products
  -> wetness products
  -> render products
  -> query products
  -> vegetation/material interactions
```

Water products are field products with lineage, scope, scale band, freshness, residency, and diagnostics.

---

# Design Goals

1. Represent water as scoped field products.
2. Support rivers and lakes in the first production slice.
3. Support shoreline wetness.
4. Support simple flow direction.
5. Support grass/reed interaction.
6. Support foam and mist products.
7. Support simple buoyancy/query products.
8. Leave a clean bridge to full fluid simulation.
9. Support streaming/residency.
10. Support multiplayer authority later.

---

# Non-Goals

This design is not:

- a full Navier-Stokes fluid solver
- a full ocean renderer
- a full weather/hydrology simulator
- a final water editor UI
- a physics buoyancy implementation
- a renderer shader spec

It defines product contracts and integration rules.

---

# Product Families

## Water Mask Product

Defines where water exists.

Sources:

- authored river/lake paths
- procedural basin generation
- terrain depressions
- future simulation state

## Water Surface Product

Defines visible surface.

Fields:

- surface height or level-set
- normal/detail parameters
- flow direction
- wave/ripple parameters
- scale band

## Flow Field Product

Defines water movement.

Fields:

- direction
- speed
- turbulence/noise
- source/sink influence
- basin/river segment scope

## Wetness Field Product

Defines material wetness near or from water.

Affects:

- ground material
- grass/reeds
- footprints
- mud
- shine/darkening
- drying later

## Foam/Mist Product

Defines visual secondary water effects.

Examples:

- shoreline foam
- river turbulence foam
- low mist over lake
- magic/fairytale mist

## Buoyancy/Query Product

Defines simple gameplay queries:

- is point in water?
- water depth
- surface height
- flow vector
- buoyancy region
- movement slow region

This is not necessarily full physics.

---

# Scopes

Water products may use:

- chunk scope
- region scope
- basin scope
- river segment scope
- lake scope
- shoreline band scope
- view scope for render-only effects

Water basin/river scopes may cut across terrain chunks.

---

# First Production Slice Water

Required:

- river/lake mask
- water surface product
- flow direction field
- shoreline wetness field
- simple mist/foam product
- simple query product
- renderer integration
- diagnostics

Not required yet:

- full liquid simulation
- dynamic flooding
- erosion
- networked water simulation
- complex buoyancy
- underwater rendering

But the design must not block them.

---

# Interaction with Terrain and Materials

Water consumes:

- terrain SDF/surface
- material fields
- slope
- basin/river definitions
- atmosphere/day-night
- vegetation density
- optional flow sources

Water produces:

- wetness field
- material response
- shoreline mask
- visual water product
- query products

---

# Interaction with Vegetation

Vegetation consumes water/wetness:

- reeds near shoreline
- grass density changes near wet soil
- darkened/wet grass material
- flattened wet vegetation
- mist/fairy glow later

Water should not directly own vegetation. It produces wetness/flow products that vegetation consumes.

---

# Interaction with Day/Night

Water consumes atmosphere state:

- moon reflection color
- sky tint
- fog/mist density
- dawn wetness sparkle
- night darkness
- magical glow response

---

# Interaction with Characters

Characters may interact through query/emitter products:

- footstep ripple
- splash
- wet footprint
- movement slow
- sound event
- grass/reed movement near shoreline

Gameplay effects must use explicit query products, not renderer-only water.

---

# Streaming

Water product streaming differs from terrain.

Rules:

1. Basin/river scope may cross chunks.
2. Near water needs surface/wetness/query products.
3. Far water can use visual summaries.
4. Strict query products are required near gameplay interaction.
5. Wetness products may persist longer than visual water products.
6. Ghost summaries may be visual only.

---

# LOD and Transitions

Water must avoid visible artifacts:

- stable shoreline
- smooth normal/detail transitions
- foam fade
- mist fade
- wetness persistence
- no sudden water-level jumps
- no hard reflection/color pop
- no strict query fallback from visual-only products

---

# Future Fluid Simulation Bridge

A later fluid solver may produce:

- dynamic level sets
- velocity fields
- pressure/divergence fields
- sediment
- erosion requests
- dynamic wetness
- foam/spray

The first water products should remain compatible with that future producer.

---

# Multiplayer

Authority depends on water type.

Likely rules:

| Water State | Authority |
|---|---|
| static river/lake mask | replicated/generated world data |
| visual ripple | client-local |
| gameplay water depth | authoritative/server validated |
| dynamic flood later | authoritative simulation |
| wetness visual | often client-derived |
| wetness gameplay | authoritative if gameplay-relevant |

---

# Diagnostics

Water diagnostics should expose:

- missing water mask
- stale water surface
- missing flow field
- missing wetness product
- visual/query mismatch
- ghost summary active
- invalid basin/segment
- missing terrain dependency
- failed water product formation
- stale atmosphere response

---

# Open Questions

1. What is the first water authoring model: path, mask, basin, or generated depression?
2. Should river/lake scopes be part of `world_sdf`, a water domain, or product descriptors only?
3. What is the first water surface representation?
4. How is wetness stored: scalar field, material channel, or separate product?
5. What is required for first buoyancy/query behavior?
6. How do water products cross chunk boundaries?
7. What water states are multiplayer-authoritative?
8. How does water affect grass density and trample fields?
9. What is the first foam/mist representation?
10. How does full fluid simulation replace or enrich the first water products?

---

# Design Decisions

1. Water is a field product family.
2. First water is structured but not full fluid simulation.
3. Water is not a renderer-only plane.
4. Wetness is a product.
5. Flow direction is a product.
6. Foam/mist are products.
7. Query products are separate from visual products.
8. Water can affect vegetation and materials through products.
9. Future fluid simulation must plug into the same product family.
10. Diagnostics are required.

---

# Implementation Phases

## Phase 1: Water Product Contracts

Deliver:

- water mask descriptor
- water surface descriptor
- flow field descriptor
- wetness descriptor
- query product descriptor
- diagnostics

## Phase 2: Static Rivers/Lakes

Deliver:

- authored or generated masks
- surface field
- shoreline wetness
- renderer handoff

## Phase 3: Flow and Foam/Mist

Deliver:

- flow direction field
- simple foam
- simple mist
- atmosphere response

## Phase 4: Character and Vegetation Interaction

Deliver:

- wet grass/reed behavior
- footstep ripple/wetness event
- simple query product

## Phase 5: Streaming and LOD

Deliver:

- basin/segment scope
- near/far water product selection
- fallback/ghost rules
- diagnostics overlays

## Phase 6: Fluid Solver Bridge

Deliver later:

- dynamic level-set input
- velocity/pressure products
- sediment/erosion hooks
- authority rules

---

# Acceptance Criteria

This design is accepted when:

1. Rivers/lakes are represented as field products.
2. Wetness is represented as a product.
3. Water rendering uses product inputs.
4. Grass/reeds can consume water/wetness.
5. Characters can query basic water state.
6. Visual products and query products are separate.
7. Future fluid simulation has a clear product handoff.
8. Diagnostics explain water product state.
