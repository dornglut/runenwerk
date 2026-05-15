---
title: Field Influence AI System Design
description: Deferred detail draft for AI/gameplay influence fields such as threat, scent, sound, visibility, navigation cost, and perception.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-16
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ../active/sdf-procedural-animation-and-animated-models-design.md
  - ./sdf-physics-collision-system-design.md
---

# Field Influence AI System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after AI/influence product ownership, query policy, and authority classes
are accepted.

This document defines influence fields for AI and gameplay.

Influence fields are product families. They are not renderer overlays, although they may be visualized through debug expression products.

---

# Purpose

Runenwerk needs AI/gameplay fields for:

- enemy perception
- threat
- danger
- scent
- sound propagation
- visibility
- faction control
- navigation cost
- desirability
- cover/exposure
- magic/corruption influence
- resource richness
- spawn weighting
- debug heatmaps

The first production slice needs simple enemies and field-aware interaction.

---

# Core Concept

```text
sources + world fields + rules
  -> influence field products
  -> AI/gameplay query products
  -> debug expression products
```

Influence fields are derived products with explicit scope, lineage, freshness, residency, and authority rules.

---

# Design Goals

1. Make AI/gameplay influence field-product driven.
2. Support enemy perception and threat in the first production slice.
3. Support scent, sound, danger, visibility, and navigation cost later.
4. Keep influence products separate from renderer debug overlays.
5. Support streaming/residency and scale bands.
6. Support deterministic generation where appropriate.
7. Support multiplayer authority where gameplay-relevant.
8. Support diagnostics and heatmaps.
9. Avoid AI systems reading private renderer or physics internals.
10. Allow specialized solvers per influence type.

---

# Non-Goals

This design is not:

- a behavior tree design
- a full navigation system
- a complete combat system
- a renderer/debug overlay design
- a multiplayer protocol
- a universal field solver

It defines influence field product contracts and AI/gameplay consumption.

---

# Influence Product Families

## Threat Field

Represents danger or hostile presence.

Sources:

- enemies
- hazards
- cursed zones
- combat events
- nighttime modifiers

Consumers:

- AI
- spawning
- gameplay
- editor diagnostics

## Scent Field

Represents trackable scent or trail.

Sources:

- characters
- creatures
- corpses
- wind/flow
- wetness/material
- decay rules

Consumers:

- enemy AI
- animals/creatures
- tracking gameplay
- debug heatmaps

## Sound Field

Represents sound propagation.

Sources:

- footsteps
- combat
- water
- wind
- objects
- magic events

Consumers:

- enemy perception
- stealth gameplay
- audio debug
- AI planning

## Visibility / Exposure Field

Represents how visible/exposed a location is.

Inputs:

- terrain SDF
- vegetation cover
- lighting/day-night
- fog
- line-of-sight products later

Consumers:

- stealth AI
- enemy targeting
- player feedback
- editor overlays

## Navigation Cost Field

Represents movement cost.

Inputs:

- terrain slope
- water/wetness
- vegetation
- obstacles
- danger
- faction/zone rules

Consumers:

- AI pathing
- gameplay systems
- diagnostics

## Magic / Corruption Field

Represents fantasy-specific influence.

Sources:

- cursed prefabs
- night phase
- enemies
- ruins
- magical flora

Consumers:

- visual material response
- enemy behavior
- gameplay effects
- spawning

---

# Product Model

An influence product should include:

- identity
- influence family/kind
- scope
- scale band
- source lineage
- generation
- freshness
- residency
- authority class
- decay/propagation policy
- value range and units
- query contract
- diagnostics

---

# Sources

Influence sources may be:

- entity/prefab emitters
- character events
- world operations
- material fields
- vegetation fields
- water/wetness fields
- atmosphere/day-night state
- scripted/gameplay events
- procedural generation
- multiplayer authoritative state

Sources must be declared.

---

# Propagation Models

Different fields use different propagation.

Examples:

| Field | Propagation |
|---|---|
| threat | radial falloff / graph propagation |
| scent | decay + wind advection + material absorption |
| sound | attenuation + occlusion + sector propagation |
| visibility | geometric/lighting/cover evaluation |
| magic/corruption | diffusion/region rules |
| navigation cost | derived from surface/material/influence inputs |

Do not force one solver for all influence fields.

---

# Query Contracts

AI/gameplay systems may query:

- sample value at point
- gather local window
- find gradient direction
- find low-threat path
- find high-scent target
- query visibility/exposure
- query source contributors
- query freshness/authority

Queries must declare whether stale/fallback data is allowed.

---

# First Production Slice

Required:

- basic threat field around enemies
- basic player presence/scent or sound field
- enemy perception query
- debug heatmap
- day/night modifier hook
- product diagnostics

Allowed simplification:

- no full pathfinding field yet
- no complex sound occlusion
- no full scent advection
- no faction system

Not allowed:

- hardcoded enemy perception with no product path
- renderer-only debug field as source of AI truth
- no diagnostics
- no future streaming/multiplayer model

---

# Streaming and LOD

Influence products stream by relevance.

Near active gameplay:

- detailed influence products resident
- active AI can query strict/current data

Mid/far:

- summaries
- strategic influence
- dormant AI state
- lower update cadence

Ghost summaries may be allowed for visual/editor continuity, but authoritative AI should follow explicit policy.

---

# Multiplayer Authority

Authority depends on influence type.

Likely rules:

| Product | Authority |
|---|---|
| enemy threat used by AI | server/gameplay authoritative |
| scent used for gameplay tracking | authoritative or validated |
| debug heatmap | client/editor local |
| visual magic aura | client-derived |
| faction control | authoritative |
| local stealth preview | client-local if non-authoritative |

Replicate source events/generations, not derived debug overlays.

---

# Renderer and Editor Integration

Renderer may consume influence products for:

- debug overlays
- material response
- magical glow
- danger haze
- nighttime effects

Renderer must not become source of AI influence truth.

Editor tools should inspect:

- field values
- source contributors
- freshness
- residency
- authority
- diagnostics

---

# Diagnostics

Influence diagnostics should expose:

- missing source
- stale influence product
- missing terrain/material dependency
- fallback active
- ghost summary active
- invalid authority use
- unsupported query
- propagation budget exhausted
- source too old/expired
- consumer used non-authoritative data

---

# Open Questions

1. What is the first enemy perception field: threat, scent, sound, visibility, or a minimal combination?
2. Should influence field descriptors live in a new gameplay/AI domain?
3. What query contract should enemy AI use first?
4. How is field authority represented for multiplayer?
5. How do influence fields interact with navigation?
6. What is the first value representation: scalar grid, sparse field, graph, or hybrid?
7. How does day/night modify threat/perception?
8. How do vegetation and fog affect visibility?
9. What diagnostic overlays are mandatory?
10. How does influence product invalidation integrate with entity movement?

---

# Design Decisions

1. Influence fields are products.
2. AI consumes influence products through query contracts.
3. Debug overlays are not source truth.
4. Different influence types may use different propagation models.
5. First production slice needs simple enemy threat/perception.
6. Day/night can modify influence products.
7. Multiplayer authority is explicit per influence family.
8. Stale/fallback use must be policy-driven.
9. Diagnostics are required.
10. Influence fields remain separate from renderer internals.

---

# Implementation Phases

## Phase 1: Influence Product Contracts

Deliver:

- influence product descriptor
- source descriptor
- query contract
- authority class
- diagnostics vocabulary

## Phase 2: Threat Field

Deliver:

- enemy threat emitter
- scalar threat field
- AI query
- debug heatmap

## Phase 3: Player Presence / Sound / Scent Seed

Deliver:

- player source event
- simple decay field
- enemy perception query
- diagnostics

## Phase 4: Day/Night Modifiers

Deliver:

- phase modifier
- nighttime enemy activity
- visibility/perception adjustment

## Phase 5: Streaming/LOD

Deliver:

- near/far influence products
- dormant summaries
- fallback policy

## Phase 6: Navigation Integration

Deliver later:

- navigation cost influence
- pathing consumer
- cover/exposure products

## Phase 7: Multiplayer Authority

Deliver later:

- authoritative source events
- replicated generations
- client debug separation

---

# Acceptance Criteria

This design is accepted when:

1. Enemies can consume a threat/perception field product.
2. Influence products have lineage, freshness, scope, and authority.
3. Debug overlays visualize influence without becoming truth.
4. Day/night can modify influence behavior.
5. Stale/fallback/ghost influence use is diagnosable.
6. Future scent, sound, visibility, and faction fields fit the same model.
