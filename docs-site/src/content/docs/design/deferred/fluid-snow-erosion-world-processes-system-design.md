---
title: Fluid Snow Erosion World Processes System Design
description: Deferred detail draft for larger field-driven world processes beyond initial water products.
status: deferred
owner: workspace
layer: domain / engine-runtime
canonical: false
last_reviewed: 2026-05-12
related_designs:
  - ../accepted/sdf-first-production-capability-map.md
  - ./water-wetness-field-system-design.md
  - ./procgen-field-product-system-design.md
  - ./sdf-physics-collision-system-design.md
---

# Fluid Snow Erosion World Processes System Design

## Status

Deferred detail draft.

The accepted long-term capability map is
`../accepted/sdf-first-production-capability-map.md`. Reactivate this document
only after process product ownership, solver-state descriptors, and mutation
candidate policy are accepted.

This document defines larger world-process simulations beyond the first production water/wetness system.

It covers fluids, snow, erosion, sediment, material transport, and persistent world mutation.

---

# Purpose

Runenwerk's field world should eventually support dynamic world processes:

- liquids and fluids
- smoke/gases later
- snow accumulation
- wetness and drying
- sediment transport
- erosion
- deposition
- mud
- ash
- thermal/material process hooks
- world mutation requests

These systems must be product-driven and must not silently mutate field truth without governed authority.

---

# Core Concept

```text
field products + simulation state + process rules
  -> process products
  -> optional mutation requests
  -> ratified world operations
  -> updated field products
```

Simulation products and world mutation are separate.

A simulation may propose changes. Owning domains validate and accept/reject them.

---

# Design Goals

1. Support fluids, snow, erosion, sediment, and material transport through product families.
2. Keep solver state separate from authored world truth.
3. Use explicit mutation requests for persistent changes.
4. Support streaming/residency and active simulation windows.
5. Support multiple timescales.
6. Support diagnostics for instability, stale inputs, and authority violations.
7. Support future multiplayer authority.
8. Reuse water/wetness product contracts where possible.
9. Keep rendering products separate from solver products.
10. Avoid one universal solver.

---

# Non-Goals

This design is not:

- a full fluid solver specification
- a complete snow simulation algorithm
- a final erosion model
- a renderer design
- a physics solver
- a multiplayer protocol
- a weather system design

It defines world-process product contracts and boundaries.

---

# Process Families

## Fluid Products

Examples:

- liquid level set
- velocity field
- pressure field
- divergence diagnostic
- foam/spray
- buoyancy query
- wetness
- sediment transport

## Snow Products

Examples:

- snow accumulation
- compaction
- meltwater
- wind redistribution
- coverage mask
- footstep deformation
- material response

## Erosion Products

Examples:

- erosion potential
- sediment amount
- deposition field
- slope/material interaction
- water-flow-driven transport
- mutation proposal

## Material Transport Products

Examples:

- ash
- mud
- sand
- dust
- corruption/blight substance
- magical residue

## Thermal/Process Hooks

Examples:

- temperature tendency
- freeze/thaw state
- drying rate
- burn/fuel later

---

# Simulation State vs Product State

Simulation state may include solver-specific internal data.

Product state is the shaped output for consumers.

Rules:

1. Solver internals are not consumer contracts.
2. Product descriptors expose consumer-safe outputs.
3. Persistent mutation requires governed world operations.
4. Runtime caches are derived.
5. Diagnostics explain solver/product validity.

---

# Mutation Authority

World processes may propose mutations.

Examples:

- erosion removes terrain material
- deposition adds sediment
- water saturates soil
- snow accumulates
- fire chars material later

Mutation flow:

```text
simulation process
  -> mutation candidate
  -> owning domain validation/ratification
  -> world operation
  -> invalidation
  -> formed products rebuild
```

No simulation should directly rewrite authoritative field products without a governed mutation path.

---

# Timescale Model

Different processes update at different rates.

| Process | Timescale |
|---|---|
| near active liquid | fixed step / gameplay tick |
| far liquid summary | budgeted/background |
| wetness/drying | slow tick |
| snow accumulation | weather/time tick |
| erosion | slow/background/offline |
| sediment | active near water / background far |
| ash/corruption | event-driven/slow tick |

Timescale must be explicit.

---

# Streaming and Active Windows

Simulation should run in active windows.

Near:

- full solver products
- strict query products
- visual products
- mutation proposals if allowed

Mid:

- simplified process products
- lower update rate

Far:

- summaries
- dormant or background updates
- no high-frequency solver state

Unloaded:

- persisted summaries
- no active solver unless server/background job owns it

---

# Fluid System Bridge

The earlier water/wetness products become inputs or outputs.

First-slice water:

- water mask
- surface field
- flow field
- wetness field

Future fluid solver:

- dynamic level set
- velocity
- pressure
- divergence
- sediment
- foam/spray
- mutation proposals

The product family remains stable while producers evolve.

---

# Snow System

Snow consumes:

- terrain SDF
- material field
- temperature/day-night/weather
- wind/flow field
- vegetation/prefab surfaces

Snow produces:

- accumulation field
- coverage material response
- footstep deformation product
- meltwater/wetness products
- collision/query modifiers where relevant

---

# Erosion System

Erosion consumes:

- terrain slope
- material hardness
- water flow
- sediment state
- vegetation/root support
- weather/time

Erosion produces:

- erosion potential
- sediment transport
- deposition products
- mutation candidates

Erosion should usually be slow/background or offline unless explicitly gameplay-relevant.

---

# Multiplayer Authority

Authority depends on process.

Likely rules:

| Process | Authority |
|---|---|
| visual water ripple | client-derived |
| gameplay water depth | server/world authoritative |
| terrain erosion mutation | authoritative operation |
| snow visual cover | may be generated/client if non-gameplay |
| snow movement effect | authoritative if gameplay-relevant |
| sediment/erosion | authoritative if persistent |
| local particles/foam | client-derived |

Replicate operations and authoritative process state, not solver caches.

---

# Renderer Integration

Renderer consumes products:

- water surface/foam/mist
- snow coverage
- wetness
- sediment/mud
- ash/dust
- material response
- debug overlays

Renderer does not own solver truth.

---

# Diagnostics

World-process diagnostics should expose:

- missing input products
- stale simulation state
- solver instability
- divergence too high
- mutation candidate rejected
- budget exhaustion
- fallback summary active
- ghost summary active
- authority mismatch
- streaming window inactive
- product generation mismatch

---

# First Production Slice Relationship

The first production slice only needs structured water/wetness.

This document exists so first-slice water does not become incompatible with later fluids, snow, and erosion.

---

# Open Questions

1. When should a separate fluid domain crate be introduced?
2. What process products belong in `world_sdf` versus future simulation domains?
3. What is the first solver-state descriptor format?
4. Which world processes can mutate terrain?
5. What mutation candidate format should processes use?
6. How are background/offline process jobs scheduled?
7. How much simulation state is persisted?
8. What is authoritative in multiplayer?
9. How are fluid/snow/erosion diagnostics visualized?
10. How do process products affect material_graph products?

---

# Design Decisions

1. World processes are product-driven.
2. Solver internals are not consumer contracts.
3. Persistent mutation requires governed world operations.
4. First water/wetness products must bridge to future fluids.
5. Timescale is explicit per process.
6. Active simulation windows are required.
7. Renderer consumes products, not solver truth.
8. Multiplayer authority is explicit per process/product.
9. Diagnostics are required.
10. Avoid one universal solver.

---

# Implementation Phases

## Phase 1: Process Product Contracts

Deliver:

- process product descriptor
- solver-state descriptor
- mutation candidate descriptor
- diagnostics vocabulary

## Phase 2: Water Bridge

Deliver:

- compatibility with water/wetness products
- dynamic input/output mapping
- query product bridge

## Phase 3: Active Simulation Windows

Deliver:

- near/mid/far process policies
- residency rules
- background/dormant summaries

## Phase 4: Mutation Candidate Path

Deliver:

- erosion/deposition candidate shape
- validation/ratification path
- operation/invalidation handoff

## Phase 5: Snow Prototype Product Family

Deliver:

- accumulation product
- melt/wetness product
- footstep deformation product

## Phase 6: Fluid Solver Product Family

Deliver later:

- level set
- velocity
- pressure
- divergence diagnostics
- foam/spray

## Phase 7: Multiplayer Authority

Deliver later:

- authoritative process state
- replicated mutation/generation rules
- client-derived visual separation

---

# Acceptance Criteria

This design is accepted when:

1. Fluids, snow, erosion, and material transport fit product contracts.
2. Solver state is separate from product state.
3. Persistent mutations require governed operations.
4. Streaming/active-window policy is explicit.
5. First-slice water can evolve into fluid products.
6. Renderer consumes products without owning simulation truth.
7. Multiplayer authority is not blocked by local caches.
8. Diagnostics explain process state, failures, and authority issues.
