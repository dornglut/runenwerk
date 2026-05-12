---
title: Procgen Field Product System Design
description: Deferred detail draft for deterministic procedural generation as a producer of scoped SDF and field products.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./sdf-prefab-composition-system-design.md
  - ./field-vegetation-system-design.md
---

# Procgen Field Product System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after generator descriptor ownership, cache-key lineage, and authored
overlay policy are accepted.

This document defines procedural generation as a producer of field products.

Procgen must not directly bypass authored world operations, field product lineage, or diagnostics.

---

# Purpose

Runenwerk needs procedural generation for:

- endless SDF terrain
- material/biome fields
- grass/vegetation density
- SDF prefab placement
- rivers/lakes
- caves later
- resource/magic/fairytale features
- enemy/spawn influence
- world summaries
- diagnostic previews

The procgen system should produce deterministic, scoped products.

---

# Core Concept

```text
seed + generation rules + scope + upstream products
  -> procgen candidate products
  -> ratification/formation
  -> field products
  -> consumers
```

Generation should be deterministic by scope and lineage.

---

# Design Goals

1. Make procedural generation product-driven.
2. Support deterministic infinite world generation.
3. Support authored edits as overlays/operations, not lost regeneration.
4. Produce SDF, material, vegetation, prefab, water, and influence products.
5. Preserve lineage and cache keys.
6. Support chunk, region, biome, basin, and later cave scopes.
7. Support regeneration and invalidation.
8. Support diagnostics and previews.
9. Avoid runtime-only hidden generation truth.
10. Prepare for later multiplayer and distributed generation.

---

# Non-Goals

This design is not:

- a final terrain algorithm
- a biome authoring UI
- a complete cave generator
- a gameplay spawn system
- a renderer
- a physics system

It defines procgen contracts and product responsibilities.

---

# Product Families Produced

Procgen may produce:

| Product | Purpose |
|---|---|
| Terrain SDF product | base land shape |
| Material field product | soil, rock, moss, mud, etc. |
| Biome field product | ecological/art-direction regions |
| Vegetation density product | grass/reeds/plants |
| Prefab placement product | trees, rocks, ruins |
| Water mask/product | rivers/lakes/basins |
| Influence seed product | danger/magic/resource distributions |
| Summary product | far/world streaming summary |
| Diagnostic product | generation preview and errors |

---

# Deterministic Generation

Generation keys should include:

- world seed
- generator identity
- generator version
- scope identity
- upstream product generations
- parameter set
- authored override generation

A generated product must be reproducible from its lineage.

---

# Authored Edits

The current world is:

```text
generated base + authored operations + simulation state = formed product
```

Rules:

1. Generated base is not overwritten by local edits.
2. Edits are stored as operations or authored layers.
3. Regeneration respects authored overrides.
4. Product lineage records both generated and authored sources.
5. Diagnostics expose conflicts.

---

# Generator Types

## Terrain Generators

Produce:

- terrain SDF
- height/shape fields
- slope/support summaries
- region summaries

## Material/Biome Generators

Produce:

- material channels
- biome masks
- moisture/fertility
- dark-fairytale region modifiers

## Placement Generators

Produce:

- SDF prefab instance placements
- spacing/density rules
- seed-driven variation
- placement diagnostics

## Vegetation Generators

Produce:

- grass/reed density
- species selection
- seasonal/night behavior hooks

## Water Generators

Produce:

- river paths
- lake masks
- basin scopes
- flow direction seeds
- wetness bands

## Cave Generators Later

Produce:

- cave sector graph
- tunnel/chamber SDF products
- portal/connectivity products
- cave material fields
- cave summaries

---

# Product Formation Flow

```text
generation request
  -> resolve scope
  -> resolve generator rules
  -> resolve upstream products
  -> generate candidate products
  -> ratify candidate products
  -> publish formed products
  -> record diagnostics
```

Generated candidates should not silently become accepted truth.

---

# Streaming Integration

Procgen runs on demand by product scope.

Examples:

- player approaches region
- product missing
- product stale due to generator version
- editor requests preview
- background prefetch
- multiplayer generation sync

Generation must obey budgets.

---

# Multiplayer

For deterministic generation:

- replicate seed/rules/generation versions
- replicate authored operations
- replicate authoritative simulation state
- do not replicate derived render caches

When deterministic generation is not guaranteed, server authority must publish product generations or formed products.

---

# Diagnostics

Procgen diagnostics should expose:

- missing generator rule
- invalid parameters
- non-deterministic output
- missing upstream product
- ratification failure
- placement conflict
- water/terrain conflict
- biome/material mismatch
- cache key mismatch
- stale generator version
- authored override conflict

---

# First Production Slice

Required:

- endless terrain SDF generation
- material/biome fields
- grass density products
- tree/rock/ruin placement products
- simple water masks
- diagnostics

Deferred:

- full caves
- full biome editor
- complex ecosystem simulation
- procedural quests
- advanced enemy spawning

---

# Open Questions

1. Should procgen be a new domain crate or implemented first inside existing world/product domains?
2. What is the first generator descriptor format?
3. How are generator versions tracked?
4. What is the first deterministic terrain algorithm?
5. How are authored edits layered over generated terrain?
6. How are river/lake basins generated?
7. How are prefab placements ratified?
8. What procgen products are authoritative in multiplayer?
9. How much generation can happen at runtime versus offline/cache?
10. How should generated products be debugged in editor?

---

# Design Decisions

1. Procgen produces products, not hidden runtime world truth.
2. Generation is deterministic by seed/scope/version where possible.
3. Authored edits are operation/layer overlays.
4. Generated candidates are ratified before acceptance.
5. Product lineage includes generator identity and version.
6. Generation is budgeted and diagnosable.
7. Procgen can produce SDF, material, vegetation, prefab, water, and influence products.
8. Caves are future scope but must fit the same model.
9. Multiplayer uses seeds/rules/operations/generations, not local render caches.
10. First production slice needs terrain, material, grass, placement, and water masks.

---

# Implementation Phases

## Phase 1: Generator Descriptor Contract

Deliver:

- generator identity
- version
- parameters
- input/output product declarations
- diagnostics vocabulary

## Phase 2: Terrain/Base Field Generation

Deliver:

- deterministic chunk/region SDF generation
- material/biome seed products
- lineage/cache keys

## Phase 3: Placement and Vegetation

Deliver:

- prefab placement products
- grass density products
- placement diagnostics

## Phase 4: Water Masks

Deliver:

- simple river/lake mask generation
- basin/segment descriptors
- wetness seed products

## Phase 5: Authored Overlay Integration

Deliver:

- operation overlay handling
- regeneration invalidation
- conflict diagnostics

## Phase 6: Streaming/Budgeting

Deliver:

- on-demand generation
- prefetch
- generation budget diagnostics

## Phase 7: Caves Later

Deliver:

- cave sector generation
- cave SDF products
- portal/connectivity products

---

# Acceptance Criteria

This design is accepted when:

1. Procgen outputs are field products.
2. Generated products have lineage and cache keys.
3. Authored edits overlay generated products safely.
4. Grass/prefab/water products can be generated from scoped rules.
5. Generation failures produce diagnostics.
6. Infinite terrain can be regenerated deterministically.
7. The model supports future cave generation.
