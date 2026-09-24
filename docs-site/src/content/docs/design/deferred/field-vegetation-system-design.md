---
title: Field Vegetation System Design
description: Deferred detail draft for SDF-first grass, reeds, plants, density fields, wind response, trample fields, LOD, and diagnostics.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./day-night-atmosphere-system-design.md
  - ./water-wetness-field-system-design.md
---

# Field Vegetation System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after vegetation product ownership, renderer handoff, and interaction
field contracts are accepted.

This document defines vegetation as field products, not as individually authored mesh instances.

The first production slice focuses on grass and reeds, but the system should support broader vegetation later.

---

# Purpose

Runenwerk needs field-driven vegetation for:

- dense grass fields
- reeds near water
- moss
- low plants
- flowers
- glowing fungi
- magical/cursed plants
- future shrubs, roots, vines, and foliage volumes

Vegetation should respond to:

- terrain SDF
- material fields
- biome fields
- slope
- moisture/wetness
- wind
- day/night
- character movement
- water proximity
- gameplay/influence fields

---

# Core Concept

Vegetation is generated from products:

```text
terrain + material + biome + moisture + seed
  -> vegetation density product
  -> vegetation render product
  -> interaction products
  -> diagnostics
```

Do not persist every blade.

Use deterministic generation:

```text
chunk/region seed + vegetation rule + field inputs = vegetation product
```

---

# Design Goals

1. Represent vegetation as field products.
2. Support dense grass without per-blade authored state.
3. Support SDF/procedural near vegetation.
4. Support artifact-free LOD transitions.
5. Support wind response.
6. Support character trample/bend interaction.
7. Support wetness/water influence.
8. Support day/night visual response.
9. Support deterministic streaming/regeneration.
10. Support diagnostics and overlays.

---

# Non-Goals

This design is not:

- a mesh grass system
- a final plant authoring UI
- a botany simulation
- a full ecosystem simulator
- a renderer implementation
- a fluid/wind solver

It defines vegetation field products and contracts.

---

# Product Families

## Vegetation Rule Product

Defines species and placement rules.

Fields:

- species identity
- allowed biome
- allowed material
- slope range
- moisture range
- water distance
- density function
- height range
- color/material response
- wind response profile
- trample response profile
- day/night response
- LOD policy

## Vegetation Density Product

Spatial product describing vegetation distribution.

Inputs:

- terrain surface
- material/biome fields
- moisture/wetness
- slope
- seed
- authored overrides
- procedural rules

## Vegetation Render Product

Renderer-facing product.

May include:

- near procedural blade/clump recipe
- mid clump fields
- far material contribution
- wind/trample inputs
- LOD fade state

## Vegetation Interaction Product

Tracks temporary or persistent interaction:

- bend
- trample
- recovery
- burn/cut later
- wetness impact
- snow cover later

## Vegetation Diagnostic Product

Exposes:

- density
- species selection
- wind response
- trample state
- LOD bands
- missing inputs
- stale products

---

# Grass Model

The first grass system should support:

- one base grass species
- one dark/fairytale variant
- one water-edge reed species
- optional glowing flower/fungi density later

Near representation:

- procedural SDF blades or clumps
- wind deformation
- local trample deformation
- material variation

Mid representation:

- clump fields
- reduced blade detail
- wind phase preserved

Far representation:

- density/normal/material contribution
- no individual blades
- stable color/roughness/shimmer contribution

---

# Reeds and Water Edge Vegetation

Reeds use:

- water proximity
- wetness field
- shoreline mask
- flow direction
- wind response
- taller clump rules

They should be produced by the same system with water-aware rules.

---

# Inputs

Required inputs:

- terrain SDF/surface product
- surface normal/slope
- material field
- biome field
- moisture/wetness field
- deterministic seed
- wind field or atmosphere wind proxy
- trample/bend field
- scale band

Optional inputs:

- shade/light field
- day/night state
- influence/corruption field
- snow/ash/wetness accumulation
- gameplay override zones

---

# Interaction Model

Characters, enemies, water, wind, and events may emit vegetation interactions.

Examples:

| Event | Vegetation Effect |
|---|---|
| player footstep | local bend/trample |
| enemy movement | bend/trample |
| wind gust | directional sway |
| water overflow | wet flattening |
| fire later | burn/remove |
| magic field | glow/lean/color shift |

Interaction fields should have decay/recovery rules.

---

# Trample/Bend Field

The trample/bend field should be scoped and temporary by default.

Fields:

- position/scope
- direction
- intensity
- timestamp/generation
- decay profile
- recovery profile
- source type
- authority class

Near grass uses trample data directly.

Far grass may ignore or summarize it.

---

# Wind Response

Vegetation wind response should consume:

- global wind/atmosphere product
- local flow field where available
- turbulence/noise seed
- species response profile
- height/stiffness

Wind should not be hardcoded per shader only. It should be represented as a product input.

---

# Day/Night Response

Vegetation may respond to day/night:

- color shift
- wet/dawn sparkle
- glow at night
- opening/closing flowers
- fog interaction
- enemy/hazard vegetation visibility

These are product rules, not hardcoded visual hacks.

---

# LOD and Artifact Control

Vegetation must avoid popping.

Rules:

1. Stable procedural seeds.
2. Cross-fade between bands.
3. Dithered fade where appropriate.
4. Preserve density visually.
5. Match color/material contribution across bands.
6. Hysteresis for LOD changes.
7. Trample state must not visibly snap.
8. Wind phase should remain coherent across bands.

---

# Streaming

Vegetation products stream by chunk/region/biome scope.

Near:

- density field resident
- render field resident
- interaction field resident

Mid:

- density summary resident
- reduced render field resident

Far:

- material contribution only
- no interaction field unless needed

Editor-selected scopes may force diagnostics/preview residency.

---

# Renderer Integration

Renderer consumes:

- vegetation render product
- density product
- species rules
- wind product
- trample/bend product
- material variation
- scale band
- diagnostics flags

Renderer does not own vegetation truth.

---

# Physics and Gameplay

Grass is usually visual/interaction state, not strict collision.

But vegetation can affect gameplay through separate products:

- stealth/cover field
- movement slow field
- harvestable resource field
- fire/fuel field later
- scent/visibility modifier later

These must be explicit gameplay products, not inferred from render grass alone.

---

# Diagnostics

Vegetation diagnostics should show:

- density field
- species selection
- biome/material inputs
- water/wetness inputs
- wind inputs
- trample/bend state
- LOD band
- fallback state
- missing field inputs
- stale products

---

# Open Questions

1. What is the first vegetation rule descriptor shape?
2. How much SDF blade detail is required in near field?
3. Should grass be raymarched, instanced procedural SDF, or hybrid?
4. What is the first trample field storage format?
5. How long should trample recovery last?
6. How does water/wetness modify grass rules?
7. What is the first wind source before a full wind field exists?
8. Which vegetation products are gameplay-relevant?
9. What diagnostic overlays are mandatory?
10. How are glowing night plants represented?

---

# Design Decisions

1. Vegetation is field/product-driven.
2. Individual blades are not authored persistent entities.
3. Grass is generated deterministically.
4. Wind and trample are product inputs.
5. Water/wetness affects vegetation.
6. Day/night may affect vegetation.
7. LOD transitions must be artifact-safe.
8. Render and gameplay vegetation products are separate.
9. Diagnostics are required.
10. First production slice supports grass and reeds.

---

# Implementation Phases

## Phase 1: Vegetation Product Contracts

Deliver:

- vegetation rule descriptor
- density product descriptor
- render product descriptor
- interaction product descriptor
- diagnostics vocabulary

## Phase 2: Grass Density

Deliver:

- deterministic density over terrain/material/biome
- chunk/region scope
- debug overlay

## Phase 3: Grass Rendering

Deliver:

- near procedural grass
- mid clump representation
- far material contribution
- LOD transitions

## Phase 4: Wind Response

Deliver:

- wind input product
- species response profile
- coherent sway

## Phase 5: Trample/Bend

Deliver:

- footstep emitter integration
- trample/bend field
- decay/recovery
- diagnostics

## Phase 6: Water Edge Vegetation

Deliver:

- wetness input
- reed species
- shoreline density behavior

## Phase 7: Day/Night Response

Deliver:

- vegetation color/glow response
- night/dawn material behavior

---

# Acceptance Criteria

This design is accepted when:

1. Grass is generated from field products.
2. Grass responds to wind.
3. Grass responds to character movement.
4. Reeds can be produced near water.
5. Near/mid/far LOD transitions avoid popping.
6. Vegetation diagnostics expose density, inputs, and interaction state.
7. Renderer does not own vegetation truth.
