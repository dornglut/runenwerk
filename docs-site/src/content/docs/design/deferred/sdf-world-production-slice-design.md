---
title: SDF World Production Slice Design
description: Deferred detail draft for an SDF-first production integration scenario.
status: deferred
owner: workspace
layer: cross-domain
canonical: false
last_reviewed: 2026-05-16
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../active/sdf-procedural-animation-and-animated-models-design.md
---

# SDF World Production Slice Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after the relevant owning product contracts and domain boundaries are
accepted.

Animated SDF player and enemy character architecture is now specified by
[`../active/sdf-procedural-animation-and-animated-models-design.md`](../active/sdf-procedural-animation-and-animated-models-design.md).

This document defines the first production integration scenario for Runenwerk.

This is not an MVP, prototype, or throwaway demo. It is a content-limited proof that the real production systems can work together.

---

# Purpose

The first production slice should validate the full SDF-first architecture through a playable, visible world.

The target scene is:

```text
An endless dark-fairytale SDF field with dense grass, a controllable SDF character,
SDF enemies, SDF trees/rocks/ruins, rivers/lakes, field interaction, and a day/night cycle.
```

The slice limits content breadth, not architectural correctness.

---

# Core Principles

1. Build full system contracts first.
2. Use the first scene to prove the systems, not to shortcut them.
3. Keep the world SDF-first.
4. Treat meshes only as optional derived/debug/export/fallback products.
5. Make day/night a core world/render product.
6. Make renderer consumption product-driven.
7. Make diagnostics visible from the start.
8. Avoid MVP language in docs and implementation plans.

---

# Production Slice Goals

The slice should demonstrate:

- endless chunked SDF terrain
- SDF material fields
- SDF player character
- SDF enemy character
- procedural field grass
- grass wind/trample interaction
- SDF trees, rocks, and ruins
- simple structured rivers/lakes
- shoreline wetness
- day/night atmosphere
- dark fantasy/fairytale visual direction
- render product selection and residency
- field-product diagnostics
- stable artifact-free LOD transitions

---

# Non-Goals for This Slice

These are not excluded forever. They are outside the first production integration target.

- full cave networks
- full multiplayer
- full fluid simulation
- advanced GI
- complete animation editor
- complete prefab editor
- large biome set
- advanced combat
- complex quest/gameplay systems
- full particle/VFX stack
- full procedural world authoring UI

The architecture must still leave room for them.

---

# Scene Direction

## Visual tone

The production slice should support a flexible dark fantasy/fairytale mood:

- moonlit grass fields
- twisted trees
- black reeds
- mossy stones
- ruined arches
- mist layers
- glowing fungi or flowers
- cold dawn light
- long dusk shadows
- strange silhouettes
- wet riverbanks
- distant enemy shapes

The vibe should be expressed through products:

- material fields
- biome fields
- vegetation fields
- atmosphere products
- prefab products
- lighting products
- water/wetness products

It should not be hardcoded as renderer behavior.

---

# Product Stack

## World products

Required:

- terrain SDF product
- terrain material field
- biome field
- collision/query product
- render product
- product freshness/lineage diagnostics

## Grass products

Required:

- grass density field
- species/variant rules
- wind response field
- trample/bend field
- recovery/decay state
- near/mid/far LOD products
- vegetation diagnostics

## Character products

Required:

- SDF character body graph
- rig/pose controls
- animation graph
- material masks
- collision/query products
- footstep/interaction emitters
- render products

## Enemy products

Required:

- SDF enemy prefab
- animation products
- perception inputs
- threat/influence emitter
- collision/query products
- render products

## Prefab products

Required:

- SDF tree products
- SDF rock products
- SDF ruin products
- placement products
- material field rules
- collision/query products
- render products
- diagnostics

## Water products

Required:

- river/lake mask
- water surface field
- flow direction field
- shoreline wetness field
- foam/mist field
- simple buoyancy/query product
- render product

## Atmosphere products

Required:

- time-of-day state
- sun direction/color/intensity
- moon direction/color/intensity
- sky/fog/exposure
- material response parameters
- glow/night behavior parameters
- atmosphere diagnostics

---

# Runtime Flow

```text
player/camera moves
  -> product scopes are resolved
  -> streaming/residency targets update
  -> render product resolver selects visible products
  -> SDF GPU residency uploads needed products
  -> render producers prepare contributions
  -> render flows execute
  -> diagnostics report stale/fallback/missing products
```

Interaction flow:

```text
character footstep
  -> emits footstep/trample field event
  -> grass bend field updates locally
  -> wetness/ripple field may update near water
  -> diagnostics expose field update state
```

Enemy flow:

```text
enemy source
  -> emits threat/perception influence
  -> AI queries field products
  -> renderer displays enemy SDF
  -> diagnostic overlay can show influence field
```

Day/night flow:

```text
time advances
  -> atmosphere product updates
  -> sun/moon/fog/exposure update
  -> material and water response update
  -> enemy schedule hooks may observe time state
  -> lighting products refresh where needed
```

---

# SDF-First Character Target

The first character may be simple, but it must use the real SDF character system.

Allowed simplification:

- small number of SDF body parts
- simple idle/walk/run
- simple material masks
- simple collision

Not allowed:

- hardcoded moving capsule as final character model
- mesh skinning as primary path
- renderer-only character shape
- no product lineage/freshness
- no future path to richer rigs

---

# SDF-First Grass Target

The first grass system may support one or two species.

Allowed simplification:

- one grass species
- one reed species near water
- local wind sway
- local trample field

Not allowed:

- hardcoded grass shader without density product
- no product identity
- no LOD transition policy
- no diagnostics
- no future path to species/biome rules

---

# Water Target

The first water system should be structured, not a throwaway plane.

Allowed simplification:

- static river/lake masks
- simple surface field
- simple flow vector
- simple wetness band
- simple mist/foam

Not allowed:

- unscoped renderer-only water plane
- no lineage
- no shoreline/wetness product
- no future bridge to fluid simulation

---

# Day/Night Target

Day/night is required in the first production slice.

Allowed simplification:

- deterministic time cycle
- sun/moon directions
- fog/exposure curves
- material response values

Not allowed:

- hardcoded color lerp only
- no product state
- no diagnostics
- no hooks for gameplay/schedules
- no future lighting invalidation model

---

# Open-World Target

The first slice should support an endless addressable field, with finite resident products.

Required:

- chunk/region scope
- near/mid/far scale bands
- deterministic generation seed
- product residency states
- fallback summaries
- render distance as product selection
- diagnostics for loaded/stale/missing products

Caves can remain future scope, but the streaming model must not block later cave sectors/portals.

---

# Renderer Acceptance

The renderer must prove:

1. It can render SDF terrain products.
2. It can render SDF prefab products.
3. It can render SDF character products.
4. It can render grass/vegetation field products.
5. It can render water/wetness field products.
6. It can consume atmosphere/day-night products.
7. It can show diagnostics overlays.
8. It does not require mesh assets as primary inputs.

---

# Diagnostics Acceptance

The editor/debug layer must show:

- loaded scopes
- selected product bands
- resident/non-resident products
- stale products
- fallback products
- missing products
- product lineage
- rebuild failures
- active diagnostics overlays

---

# Milestones

## Phase 1: Production Contracts

Deliver accepted drafts for:

- Adaptive Field Product System
- SDF Product Renderer Architecture
- Open-World Product Streaming
- Field Product Diagnostics

## Phase 2: SDF World Foundation

Deliver:

- endless chunk/region scope
- SDF terrain product
- material field product
- render product selection
- collision/query product
- diagnostics

## Phase 3: Atmosphere

Deliver:

- time-of-day product
- day/night lighting state
- fog/exposure
- renderer integration

## Phase 4: Character

Deliver:

- SDF player prefab
- basic rig/pose product
- idle/walk/run
- field interaction emitters

## Phase 5: Grass

Deliver:

- density field
- wind response
- trample/bend
- LOD transition
- diagnostics

## Phase 6: Prefabs

Deliver:

- SDF trees
- SDF rocks
- SDF ruins
- placement products
- collision/render products

## Phase 7: Water

Deliver:

- river/lake masks
- water surface field
- flow field
- wetness field
- mist/foam product

## Phase 8: Enemies

Deliver:

- SDF enemy prefab
- animation product
- threat/perception field
- basic AI integration

---

# Open Questions

1. What is the minimum accepted SDF character rig model?
2. How many SDF material channels are required for the first field?
3. What scale bands are mandatory for first production content?
4. What product diagnostics must appear in the first editor overlay?
5. How much water interaction is required before fluid simulation exists?
6. Which field products are authoritative in multiplayer later?
7. How is dark fantasy/fairytale art direction represented as data?
8. What is the minimal accepted combat/enemy interaction?
9. How much generated terrain variation is enough to prove the system?
10. What is the first accepted fallback for missing terrain products?

---

# Design Decisions

1. The first scene is a production slice, not an MVP.
2. SDF is the primary modelling representation.
3. Meshes are not required for terrain, characters, prefabs, or vegetation.
4. Day/night is mandatory.
5. Grass interaction is mandatory.
6. Water/wetness is structured from the start.
7. Renderer must consume products, not own world truth.
8. Diagnostics are required from the start.
9. Caves are deferred but must remain architecturally supported.
10. Multiplayer is deferred but authority boundaries must not be blocked.

---

# Acceptance Criteria

The first production slice is successful when a player can:

1. Move through an endless SDF field.
2. See day/night changes.
3. Walk through grass and affect it.
4. Approach SDF trees, rocks, and ruins.
5. See rivers/lakes and wet shorelines.
6. Encounter SDF enemies.
7. Observe stable LOD without obvious popping.
8. Inspect field products and diagnostics in tooling.
