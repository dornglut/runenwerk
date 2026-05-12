---
title: Field VFX Particles System Design
description: Deferred detail draft for field-driven VFX, particles, fog, mist, embers, fireflies, smoke, magic effects, and renderer/runtime handoff.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./day-night-atmosphere-system-design.md
  - ./water-wetness-field-system-design.md
  - ./field-vegetation-system-design.md
---

# Field VFX Particles System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after VFX product ownership, visual-only/gameplay-relevant classification,
and renderer handoff are accepted.

This document defines field-driven VFX and particle products.

VFX are consumers and producers of field products. They should not be renderer-only hacks when they affect gameplay, simulation, or persistent world state.

---

# Purpose

Runenwerk needs VFX for:

- mist
- fog
- smoke
- embers
- fireflies
- magic particles
- water spray
- splash particles
- dust/pollen
- glowing spores
- cursed haze
- fairytale atmosphere effects

The first production slice likely needs mist, fireflies/glow particles, water mist/foam, and possibly grass/field disturbance.

---

# Core Concept

```text
field sources + atmosphere/water/wind/material products
  -> VFX product
  -> renderer/runtime particles
  -> optional field feedback
  -> diagnostics
```

Some VFX are purely visual. Others produce or consume gameplay/simulation fields.

---

# Design Goals

1. Make VFX product-driven.
2. Support field-advection and atmosphere/water inputs.
3. Support dark fantasy/fairytale visual effects.
4. Separate visual-only effects from gameplay-relevant effects.
5. Support GPU/runtime particle execution without making it truth.
6. Support streaming and LOD.
7. Support diagnostics.
8. Prepare for future wind, smoke, fire, and fluid coupling.
9. Avoid one-off renderer-only effects that block product integration.
10. Support editor preview and product inspection.

---

# Non-Goals

This design is not:

- a full particle editor
- a complete fluid solver
- a complete fire/smoke simulation
- a renderer shader specification
- a gameplay effect system
- a networking protocol

It defines VFX product contracts and runtime boundaries.

---

# VFX Product Families

## Particle Emitter Product

Defines source behavior.

Examples:

- fireflies
- embers
- magic motes
- splash spray
- pollen
- cursed ash

## Volume VFX Product

Defines volumetric effects.

Examples:

- fog bank
- mist over water
- smoke cloud
- magical haze
- cave dust later

## Advection Field Product

Defines movement influence.

Inputs:

- wind field
- water flow
- character disturbance
- thermal updraft later

## Render Particle Product

Renderer-facing particle state.

May include:

- spawn parameters
- particle buffers
- material/visual profile
- LOD state
- sort/transparency policy
- lifespan data

## VFX Diagnostic Product

Exposes:

- missing inputs
- stale field
- budget exhaustion
- fallback effects
- emitter scope
- runtime/GPU state

---

# VFX Source Types

Sources may be:

- prefab emitters
- water/wetness events
- atmosphere/day-night state
- vegetation interaction
- character footsteps
- enemy/magic sources
- material/substance fields
- procgen features
- gameplay events

Sources must declare whether they are visual-only or gameplay-relevant.

---

# Visual-Only vs Gameplay-Relevant

## Visual-only examples

- decorative fireflies
- sparkle at dawn
- distant mist shimmer
- visual water spray

## Gameplay-relevant examples

- smoke that blocks visibility
- poison cloud
- magic field that affects enemies
- fire/heat source
- sound-producing particle event

Gameplay-relevant VFX must publish explicit field/influence products. Renderer particle state is not authority.

---

# First Production Slice VFX

Required candidates:

- low mist over water
- nighttime fireflies or glowing spores
- atmospheric fog/mist
- footstep dust/wet ripple visual
- optional magic haze near ruins

Allowed simplification:

- simple procedural emitters
- local particle buffers
- visual-only fireflies
- no full smoke simulation

Not allowed:

- hidden gameplay effects in renderer-only particles
- no diagnostics
- no product scope
- no future field coupling

---

# Renderer Integration

Renderer consumes prepared VFX products:

- emitter state
- particle buffers
- volume parameters
- atmosphere inputs
- water/fog inputs
- LOD state
- diagnostics flags

Renderer does not own VFX source truth when effects are product-relevant.

---

# Streaming and LOD

VFX products stream by scope and relevance.

Near:

- active emitters
- higher particle density
- field advection

Mid:

- reduced density
- simplified simulation

Far:

- volume/fog summary
- material/atmosphere contribution
- no per-particle detail

LOD transitions should fade, not pop.

---

# Day/Night Integration

Day/night can drive:

- fireflies at night
- glowing fungi/spores
- fog thickening at dusk
- dawn sparkle
- cursed haze at deep night
- enemy/magic VFX phase changes

These should be product rules.

---

# Water/Wetness Integration

Water can drive:

- mist
- foam particles
- splash
- wet-footstep effects
- shoreline haze

Water products provide source fields. VFX products render and optionally feed interaction products.

---

# Wind/Flow Integration

When wind/flow products exist, VFX can consume:

- velocity
- turbulence
- flow direction
- eddies
- cave airflow later

Until then, a simple atmosphere wind proxy may be used as a product input.

---

# Diagnostics

VFX diagnostics should expose:

- missing emitter source
- missing advection field
- stale atmosphere/water input
- particle budget exhausted
- fallback active
- visual-only effect used as gameplay source
- emitter outside resident scope
- invalid lifetime/spawn rate
- GPU/runtime buffer state

---

# Multiplayer

Most visual VFX are client-derived.

Replicate:

- authoritative source events
- gameplay-relevant effect state
- product generations
- persistent hazards

Do not replicate:

- local particle buffers
- visual-only random seeds unless needed for sync
- local renderer caches
- editor-only overlays

---

# Open Questions

1. What is the first runtime particle representation?
2. Do VFX descriptors belong in asset, material, or a future VFX domain?
3. What VFX are mandatory for the first production slice?
4. What effects are gameplay-relevant versus visual-only?
5. How are particle buffers represented as products?
6. How does transparency sorting work with SDF rendering?
7. What is the first advection input before full wind exists?
8. How are VFX authored for prefabs?
9. What VFX state is multiplayer-authoritative?
10. What diagnostics overlays are mandatory?

---

# Design Decisions

1. VFX are product-driven.
2. Renderer particle state is derived.
3. Visual-only and gameplay-relevant VFX are separate.
4. Day/night and water are primary early VFX inputs.
5. Future wind/flow advection must fit the same model.
6. LOD transitions must fade.
7. Diagnostics are required.
8. First production slice supports mist/fireflies/spores/water effects.
9. Gameplay-relevant VFX publish influence/substance products.
10. Multiplayer replicates source events, not local render caches.

---

# Implementation Phases

## Phase 1: VFX Product Contracts

Deliver:

- emitter descriptor
- volume VFX descriptor
- render particle product descriptor
- visual/gameplay authority classification
- diagnostics

## Phase 2: Atmosphere VFX

Deliver:

- fog/mist product
- day/night-driven fireflies/spores
- renderer handoff

## Phase 3: Water VFX

Deliver:

- water mist
- foam/splash visual products
- wet-footstep effect hook

## Phase 4: Field Advection

Deliver:

- atmosphere wind proxy input
- later wind/flow product input
- local turbulence

## Phase 5: Gameplay-Relevant VFX

Deliver later:

- smoke visibility field
- poison/magic influence field
- authority rules

## Phase 6: Streaming/LOD/Diagnostics

Deliver:

- VFX residency
- LOD fade
- budget diagnostics
- overlay inspection

---

# Acceptance Criteria

This design is accepted when:

1. VFX are represented as scoped products.
2. Visual-only and gameplay-relevant effects are separated.
3. Day/night can drive VFX.
4. Water can drive mist/foam/splash products.
5. Renderer consumes VFX products without owning source truth.
6. VFX diagnostics expose missing/stale/budget/fallback state.
7. Future wind, smoke, fire, and magic effects fit the architecture.
