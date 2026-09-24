---
title: Day Night Atmosphere System Design
description: Deferred detail draft for time-of-day, sun/moon, atmosphere, fog, exposure, material response, and product invalidation.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ../accepted/sdf-product-renderer-and-gpu-residency-design.md
  - ./field-vegetation-system-design.md
  - ./water-wetness-field-system-design.md
---

# Day Night Atmosphere System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after time/celestial product ownership and renderer/runtime handoff are
accepted.

This document defines day/night and atmosphere as first-class product-driven systems.

Day/night is required for the first production slice.

---

# Purpose

Runenwerk needs a day/night system that affects:

- sky
- sun/moon lighting
- fog
- exposure
- material response
- water appearance
- vegetation response
- glowing flora/fungi
- enemy schedules
- AI/perception products
- future radiance/lighting products
- diagnostics and editor controls

It must not be reduced to a renderer-only color lerp.

---

# Core Concept

```text
time state
  -> celestial state
  -> atmosphere products
  -> lighting/material/fog/water/vegetation responses
  -> renderer and gameplay consumers
```

Day/night state is a product source that downstream systems may consume.

---

# Design Goals

1. Make time-of-day explicit and inspectable.
2. Represent sun and moon as structured state.
3. Produce renderer-facing atmosphere products.
4. Support fog, exposure, sky, and ambient response.
5. Support material and vegetation reactions.
6. Support water appearance changes.
7. Support enemy/gameplay schedule hooks.
8. Support later lighting/radiance product invalidation.
9. Support editor controls and diagnostics.
10. Preserve dark fantasy/fairytale art direction through data.

---

# Non-Goals

This design is not:

- a full astronomical simulation
- a complete weather system
- a full global illumination system
- a gameplay schedule system
- a renderer implementation
- an editor UI specification

It defines product contracts and runtime behavior for day/night and atmosphere.

---

# Product Families

## Time Product

Represents world time.

Fields:

- normalized time of day
- day count
- time scale
- paused/controlled state
- calendar/season hooks later
- generation

## Celestial Product

Represents sun/moon/celestial bodies.

Fields:

- sun direction
- sun color
- sun intensity
- moon direction
- moon color
- moon intensity
- phase parameter
- shadow softness
- horizon state

## Atmosphere Product

Renderer-facing state.

Fields:

- sky gradient
- fog color
- fog density
- ambient color
- exposure
- horizon color
- night glow parameters
- mist intensity
- weather hooks later

## Material Response Product

Material-facing state.

Examples:

- wet sparkle at dawn
- night glow activation
- moss/stone color shifts
- water tint
- frost/dew hooks later

## Gameplay Schedule Product

Optional downstream product.

Examples:

- enemy active at night
- safer daylight
- flowers open/close
- magical activity phases

---

# Day Phases

Baseline phases:

| Phase | Visual/World Behavior |
|---|---|
| Dawn | mist, wet grass, soft warm edge light |
| Day | muted field, cold sun, lower enemy activity |
| Dusk | long shadows, fog thickens, threat increases |
| Night | moonlight, glow flora/fungi, stronger enemies |
| Deep Night | darkest fog, high magic/threat tone |

These are data-driven response profiles.

---

# Renderer Integration

The renderer consumes prepared atmosphere products:

- sun/moon direction
- sky/fog/exposure
- ambient terms
- material response parameters
- water tint/reflection parameters
- vegetation glow parameters

The renderer should not own time truth.

---

# Product Invalidation

Time changes can invalidate downstream products.

Examples:

| Downstream Product | Invalidation Behavior |
|---|---|
| atmosphere render product | updates every frame/tick |
| material response product | updates by phase or curve |
| vegetation render product | updates by phase/glow response |
| water render product | updates by tint/reflection/fog |
| radiance product later | updates on budget or visible scopes |
| enemy schedule product | updates on phase transitions |

Avoid full-world recomputation every frame.

Use update cadence and product policy.

---

# Update Cadence

Different products update at different rates.

| Product | Cadence |
|---|---|
| time state | tick/frame |
| sky/exposure | frame or render tick |
| material response | phase/curve-driven |
| vegetation glow | phase/curve-driven |
| enemy schedules | phase transition or gameplay tick |
| radiance products | budgeted/visible scopes later |

---

# Dark Fantasy / Fairytale Controls

The atmosphere system should expose data controls for:

- cold sun
- strong moonlight
- fog density
- dusk shadow strength
- night glow color
- magical haze
- mist near water
- dawn wetness sparkle
- sinister eclipse-like variants later

The art direction should be configured through product data, not hardcoded renderer behavior.

---

# Interaction with Vegetation

Vegetation may consume atmosphere products for:

- nighttime glow
- dawn wetness/sparkle
- color shifts
- wind/fog response
- opening/closing flowers
- magical phase behavior

---

# Interaction with Water

Water may consume atmosphere products for:

- moon reflection color
- fog/mist density
- dawn/dusk tint
- night darkness
- shoreline glow/fairytale effects

---

# Interaction with Enemies and AI

Enemies may consume schedule products:

- spawn/activity phase
- aggression phase
- visibility/perception modifiers
- threat field intensity
- sleep/dormant periods

This should remain a gameplay/AI product consumer, not renderer policy.

---

# Editor Controls

Editor/debug controls should support:

- scrub time of day
- pause time
- force phase
- inspect sun/moon values
- inspect fog/exposure
- inspect downstream invalidations
- view product diagnostics
- compare phase profiles

---

# Diagnostics

Diagnostics should expose:

- invalid time state
- missing atmosphere product
- stale atmosphere product
- invalid sun/moon configuration
- material response missing
- downstream invalidation backlog
- radiance update skipped due to budget later
- enemy schedule product missing later

---

# Multiplayer

Time authority must be explicit.

Likely model:

- server/world authority owns canonical time for multiplayer
- clients render interpolated/smoothed atmosphere
- local render caches are not authoritative
- schedule/gameplay products use authoritative time

---

# Open Questions

1. Does time-of-day start in a domain crate or engine runtime?
2. What is the first time scale?
3. Should moon phase be included immediately?
4. How are phase profiles authored?
5. Which material response channels are required first?
6. Which enemy schedule hooks are required first?
7. Does atmosphere produce a formal field product or render contribution only at first?
8. How are radiance products invalidated later?
9. How should editor time scrubbing affect runtime simulation?
10. What is authoritative in multiplayer?

---

# Design Decisions

1. Day/night is mandatory.
2. Day/night is product-driven.
3. The renderer consumes atmosphere products but does not own time truth.
4. Sun and moon are explicit.
5. Fog and exposure are first-class.
6. Material, vegetation, and water responses consume atmosphere state.
7. Enemy schedule hooks are supported but not renderer-owned.
8. Full GI is not required for first production slice.
9. Diagnostics are required.
10. Dark fantasy/fairytale mood is data-driven.

---

# Implementation Phases

## Phase 1: Time and Celestial Contracts

Deliver:

- time product descriptor
- sun/moon product descriptor
- phase model
- diagnostics

## Phase 2: Atmosphere Render Product

Deliver:

- sky/fog/exposure product
- renderer buffer handoff
- editor inspection

## Phase 3: Material and Water Response

Deliver:

- material response parameters
- water tint/reflection/fog response
- diagnostics

## Phase 4: Vegetation Response

Deliver:

- night glow profile
- dawn/dusk vegetation response
- grass/reed integration

## Phase 5: Schedule Hooks

Deliver:

- enemy activity phase product
- gameplay consumer handoff
- multiplayer authority notes

## Phase 6: Lighting/Radiance Bridge

Deliver later:

- visible-scope lighting invalidation
- budgeted radiance refresh hooks
- diagnostics

---

# Acceptance Criteria

This design is accepted when:

1. Time-of-day is explicit and inspectable.
2. Sun and moon state are structured products.
3. Renderer consumes atmosphere state through prepared products.
4. Fog/exposure are product-driven.
5. Material, vegetation, and water can react to day/night.
6. Enemy/gameplay schedule hooks are architecturally supported.
7. Diagnostics can explain atmosphere and downstream product state.
