---
title: Adaptive Field Product System Design
description: Superseded draft for a generalized field-product platform.
status: superseded
owner: workspace
layer: cross-domain
canonical: false
last_reviewed: 2026-05-12
superseded_by:
  - ../accepted/sdf-first-field-world-platform-design.md
  - ../accepted/field-product-contracts-diagnostics-and-residency-design.md
---

# Adaptive Field Product System Design

## Status

Superseded draft.

Replaced by `../accepted/sdf-first-field-world-platform-design.md` and
`../accepted/field-product-contracts-diagnostics-and-residency-design.md`.

This document defines a parent architecture for field-based world representation and derived runtime products in Runenwerk.

The design is intentionally broader than rendering. It describes how world data, simulation data, rendering data, physics data, AI data, editor previews, diagnostics, and multiplayer-relevant products can share common product rules without being forced into one universal solver or one universal data structure.

---

# Purpose

The Adaptive Field Product System exists to make large, dynamic, field-based worlds manageable.

Runenwerk should support worlds where many systems interact through spatial fields:

* signed-distance terrain and caves
* material and substance volumes
* wind and flow
* water and liquids
* smoke, fog, heat, snow, ash, sediment, and wetness
* lighting and radiance
* physics collision and support
* navigation and movement cost
* AI influence, scent, sound, threat, and faction control
* editor previews, overlays, heatmaps, and diagnostics
* open-world streaming and multiplayer synchronization

The central rule is:

**Do not build one graph, one cache, one solver, or one world representation for every consumer. Build explicit, lineage-aware, multiscale products with declared consumers, invalidation, residency, query contracts, and diagnostics.**

---

# Core Concept

A **field product** is a formed, typed, versioned piece of world data.

It is not necessarily authored directly. Most field products are derived from authored world state, operation logs, simulation state, imported assets, or other products.

A field product answers questions such as:

* What area of the world does this product cover?
* What scale or LOD band is it for?
* What kind of data does it contain?
* Which system is allowed to consume it?
* What source data produced it?
* Is it current, stale, fallback, or missing?
* Can it be streamed, rebuilt, cached, or approximated?
* What diagnostics explain its reliability?

Examples:

| Product                               | Example Consumer                         |
| ------------------------------------- | ---------------------------------------- |
| High-resolution signed-distance field | rendering, editing, collision formation  |
| Coarse terrain summary                | far rendering, streaming, AI planning    |
| Collision query field                 | physics and gameplay                     |
| Wind velocity field                   | particles, foliage, smoke, audio         |
| Water level-set field                 | liquid rendering, buoyancy, wetness      |
| Wetness field                         | materials, footprints, drying simulation |
| Threat influence field                | AI decision making                       |
| Radiance field                        | lighting                                 |
| Product freshness report              | editor/debug tools                       |

---

# Design Goals

1. Treat field products as first-class architecture units.
2. Support many field families without forcing them into one solver.
3. Keep authored truth, simulation truth, render products, physics products, and editor products separate.
4. Make identity, lineage, freshness, scope, LOD, residency, and diagnostics explicit.
5. Support open-world scale through chunk, region, clipmap, and summary products.
6. Support large caves, interiors, and irregular spaces through sector, portal, and connected-region products.
7. Support multiplayer by replicating authoritative operations and product generations, not local render caches.
8. Support incremental rebuilds from changed regions instead of whole-world recomputation.
9. Support editor and tooling inspection without exposing private runtime internals.
10. Allow specialized domains to evolve independently while sharing field-product infrastructure.

---

# Non-Goals

The Adaptive Field Product System is not:

* a renderer
* a physics engine
* a fluid solver
* a wind solver
* a material graph
* an ECS storage layer
* a multiplayer replication protocol
* a GPU resource allocator
* a replacement for specialized simulation domains

It provides the shared product model, dependency model, invalidation model, residency model, query model, and diagnostics model that those systems can use.

---

# Architectural Doctrine

## Field products are formed consumer data

Authored data should stay author-friendly.

Runtime consumers should receive products formed for their needs.

For example:

* the renderer should consume render-facing surface, material, volume, and lighting products
* physics should consume collision/query products
* fluid simulation should consume boundary and material products
* AI should consume influence and navigation products
* editor panels should consume expression and diagnostic products

## Product contracts are more stable than storage layout

A product's contract should remain stable even if its storage changes.

The same product family may use different storage depending on scale or consumer:

* dense grid
* sparse bricks
* octree
* clipmap
* graph
* particles
* mesh proxy
* table summary
* GPU-packed cache

Consumers should rely on the product contract, not private storage details.

## Simulation domains own their own invariants

Water, wind, heat, snow, erosion, AI influence, collision, and lighting do not share one correct update method.

They may share product infrastructure, but they should not be collapsed into one universal solver.

## Runtime caches are derived

Runtime caches may exist for speed:

* GPU buffers
* sparse atlases
* temporal histories
* acceleration structures
* page tables
* propagation queues
* per-view products

These are derived from products or product inputs. They must not silently become authoritative world truth.

## Diagnostics are part of the contract

A product must be inspectable.

If a product is stale, missing, fallback-only, low-confidence, non-resident, or failed-preserved, that state should be explicit and diagnosable.

---

# Product Model

Every field product should have a descriptor.

The descriptor should include:

| Field            | Meaning                                                                                       |
| ---------------- | --------------------------------------------------------------------------------------------- |
| Product identity | Stable identity for this product instance.                                                    |
| Product family   | Surface, material, flow, liquid, influence, radiance, collision, expression, diagnostic, etc. |
| Product kind     | Specific meaning inside the family.                                                           |
| Spatial scope    | Chunk, region, sector, clipmap window, basin, viewport, or non-spatial scope.                 |
| Scale band       | Near, mid, far, summary, preview, collision, offline, or domain-specific band.                |
| Consumer class   | Renderer, physics, simulation, AI, editor, network, tooling, etc.                             |
| Source lineage   | Assets, operations, upstream products, simulation generations, producer version.              |
| Freshness state  | Current, stale, potentially stale, fallback, missing, failed-preserved, rebuilding.           |
| Residency state  | Resident, non-resident, pending load, pending unload, ghost summary, fallback resident.       |
| Retention policy | Frame-local, session-local, cacheable, persisted, shared, archival.                           |
| Rebuild policy   | Immediate, budgeted, lazy, idle, manual, offline, never.                                      |
| Query contract   | How consumers may sample, gather, bind, inspect, or resolve this product.                     |
| Diagnostics      | Issues, confidence, unsupported states, coverage gaps, or warnings.                           |

---

# Product Lifecycle

A product can move through these states:

| State             | Meaning                                                             |
| ----------------- | ------------------------------------------------------------------- |
| Declared          | The system knows this product should exist.                         |
| Forming           | Product generation is in progress.                                  |
| Current           | Product matches its source lineage.                                 |
| Potentially Stale | Product may still be usable but an upstream input changed.          |
| Stale             | Product is outdated and should not be selected by strict consumers. |
| Fallback          | A lower-quality or older product is intentionally used.             |
| Failed Preserved  | Rebuild failed, but a previous valid product remains available.     |
| Missing           | No usable product exists.                                           |
| Retired           | Product has been superseded and should not be selected normally.    |

---

# Product Families

## Surface Field Products

Surface products describe geometry-like field information.

Examples:

* signed distance
* gradient
* normal approximation
* occupancy
* support
* surface crossing summaries
* hierarchy summaries
* extracted preview surfaces

Consumers:

* renderer
* editor viewport
* picking
* collision formation
* navigation
* wind obstacle formation
* liquid boundary formation

## Material and Substance Products

Material and substance products describe what the world is made of or what exists in a volume.

Examples:

* material weights
* density
* hardness
* friction
* wetness
* temperature
* moisture
* snow amount
* sediment
* smoke
* fog
* ash
* magical substance
* corruption or blight

Consumers:

* renderer
* fluid simulation
* erosion
* heat/fire
* gameplay
* audio
* editor tools

## Flow Field Products

Flow products describe vector movement through space.

Examples:

* wind velocity
* water current
* turbulence
* vorticity
* pressure tendency
* permeability
* advection support

Consumers:

* particles
* smoke/fog
* foliage
* cloth/lightweight visual simulation
* audio
* weather
* fluid systems
* gameplay modifiers

## Liquid and Fluid Products

Liquid products describe water and other fluids.

Examples:

* liquid level set
* liquid volume occupancy
* velocity field
* pressure field
* divergence diagnostic field
* foam field
* splash support field
* wetness field
* buoyancy query field
* fluid render surface
* sediment transport field

Consumers:

* fluid simulation
* renderer
* physics interaction
* erosion
* material systems
* editor debug views

## Influence Field Products

Influence products describe abstract gameplay, AI, perception, and world-state influence.

Examples:

* threat
* danger
* desirability
* scent
* sound propagation
* visibility
* faction control
* resource richness
* navigation cost
* tactical cover
* heat risk
* magic aura

Consumers:

* AI
* gameplay systems
* navigation
* spawning
* procedural behavior
* editor heatmaps

## Radiance Products

Radiance products describe lighting-related field data.

Examples:

* irradiance
* directional radiance
* emissive contribution
* occlusion-aware lighting summaries
* temporal lighting history
* lighting confidence
* light-leak diagnostics

Consumers:

* renderer
* viewport preview
* lighting debug tools
* material preview tools

## Collision and Query Products

Collision/query products are formed specifically for interaction and movement.

Examples:

* collision field
* collision mesh
* support field
* walkability field
* broadphase summary
* picking acceleration product
* gameplay query volume

Consumers:

* physics
* character movement
* gameplay interaction
* editor picking
* navigation

## Expression Products

Expression products are visible or inspectable outputs.

Examples:

* scene color
* picking IDs
* overlays
* field slices
* volume previews
* vector glyph fields
* heatmaps
* product debug views
* remote preview frames

Consumers:

* editor viewport
* runtime presentation
* debug panels
* automation and review tools

## Provenance and Diagnostic Products

Diagnostic products explain the product system itself.

Examples:

* freshness report
* lineage report
* missing dependency report
* rebuild backlog report
* streaming residency report
* stale product heatmap
* ghost summary visualization
* solver instability report
* invalidation graph view

Consumers:

* editor tools
* validation routines
* debugging workflows
* CI/regression checks

---

# Scope Model

Products are scoped. Scope defines where and when a product applies.

Common scopes:

| Scope          | Use                                              |
| -------------- | ------------------------------------------------ |
| Chunk          | Local field formation and streaming.             |
| Region         | Coarser streaming, summaries, rebuild planning.  |
| Sector         | Caves, interiors, portals, connected spaces.     |
| Basin          | Rivers, lakes, water systems, drainage.          |
| Clipmap window | Camera/focus-relative multiscale products.       |
| View           | Per-viewport or per-camera products.             |
| World summary  | Far-field, strategic, or background products.    |
| Non-spatial    | Global settings, producer metadata, diagnostics. |

Large open worlds should not stream only raw level data. They should stream typed products by scope and priority.

---

# Scale Band and LOD Model

LOD is represented by scale-banded products.

A product should declare what scale band it belongs to.

Common bands:

| Band      | Meaning                                                           |
| --------- | ----------------------------------------------------------------- |
| Near      | High-detail active region around player, camera, or editor focus. |
| Mid       | Medium-detail local context.                                      |
| Far       | Coarse visual, simulation, or planning context.                   |
| Summary   | Very coarse region/world representation.                          |
| Preview   | Editor or fast authoring product.                                 |
| Collision | Physics-safe product, possibly different from render LOD.         |
| Offline   | High-quality non-realtime product.                                |

Different consumers may choose different LODs.

Example:

| System   | Near                          | Far                           |
| -------- | ----------------------------- | ----------------------------- |
| Renderer | high-res surface and material | coarse visual summary         |
| Physics  | precise collision             | no collision or broad summary |
| AI       | detailed local influence      | strategic region summary      |
| Water    | active local solver           | coarse basin state            |
| Lighting | local lighting product        | far radiance summary          |

Rendering LOD, physics LOD, AI LOD, and simulation LOD should not be forced to match.

---

# Open-World Streaming Model

Streaming is product-based.

A product can be:

* resident
* non-resident
* pending load
* pending unload
* stale
* fallback
* ghost summary
* missing

Example near the player:

| Product               | State                  |
| --------------------- | ---------------------- |
| high-res terrain SDF  | resident               |
| collision field       | resident               |
| local material field  | resident               |
| local water field     | resident if relevant   |
| local influence field | resident if relevant   |
| local lighting field  | resident or rebuilding |

Example far away:

| Product                | State                     |
| ---------------------- | ------------------------- |
| high-res terrain SDF   | non-resident              |
| coarse terrain summary | resident                  |
| collision field        | missing or unnecessary    |
| lighting field         | ghost summary or fallback |
| AI field               | strategic summary only    |

A ghost summary is a lightweight continuity product. It is not full authority.

Allowed uses:

* far lighting continuity
* distant terrain approximation
* rough influence continuity
* editor streaming visualization

Disallowed uses unless explicitly designed:

* precise collision
* authoritative liquid simulation
* exact gameplay interaction
* final physics correction

---

# Large Caves and Interiors

Large caves and interiors need more than distance-based streaming.

They often need sector, portal, and connected-region products.

Useful cave products:

| Product                          | Purpose                                       |
| -------------------------------- | --------------------------------------------- |
| cave SDF surface product         | cave shape and sculpting                      |
| cave collision product           | player and object interaction                 |
| portal/sector visibility product | visibility and streaming decisions            |
| airflow product                  | wind through tunnels                          |
| acoustic product                 | sound propagation and reverb zones            |
| humidity/wetness product         | cave material and gameplay state              |
| radiance/darkness product        | lighting behavior                             |
| navigation product               | AI movement and pathing                       |
| influence products               | danger, scent, faction, resource, magic, etc. |

Example flow:

1. Player approaches cave entrance.
2. Entrance sector products become high priority.
3. Connected nearby tunnel sectors stream collision, surface, and lighting products.
4. Far cave branches retain summaries.
5. Editor/debug tools can show which cave sectors are resident, stale, fallback, or missing.

---

# Multiplayer Model

Multiplayer should replicate authoritative changes and product generations, not every derived cache.

Usually replicated:

* world operations
* entity/source events
* authoritative simulation deltas
* product generations
* dirty regions
* gameplay-relevant field state
* committed terrain or material mutations

Usually not replicated:

* GPU buffers
* local render caches
* view-local lighting history
* editor-only overlays
* temporary LOD products
* local debug heatmaps

Authority depends on product family.

| Product               | Multiplayer Treatment                               |
| --------------------- | --------------------------------------------------- |
| terrain operation log | authoritative and replicated                        |
| collision product     | server-authoritative or server-validated            |
| gameplay water state  | authoritative or server-approved                    |
| AI influence field    | authoritative if AI/gameplay uses it                |
| visual wind           | often client-derived                                |
| lighting/radiance     | usually client-derived                              |
| editor diagnostics    | local unless collaborative tooling requires sharing |

Example:

1. A player destroys a cave wall.
2. Server records the authoritative world operation.
3. Affected chunks/regions get new generations.
4. Clients receive the operation or generation invalidation.
5. Clients rebuild, stream, or fallback according to product policy.
6. Gameplay-critical products are validated by the authoritative side.
7. Visual products may be locally regenerated.

---

# Dependency and Lineage Model

A product may depend on:

* source assets
* world operation logs
* authored layers
* material products
* simulation generations
* imported artifacts
* upstream field products
* dynamic runtime observations
* stream/residency state
* viewport/camera state

Dependencies must be declared.

A product should be able to explain why it is current, stale, fallback, missing, or failed-preserved.

Lineage is required for:

* rebuild decisions
* multiplayer generation checks
* editor diagnostics
* cache validity
* product comparison
* regression testing
* safe fallback selection

---

# Invalidation Model

Invalidation describes what changed and what products are affected.

It should be more precise than a generic dirty flag.

An invalidation record should include:

* cause
* affected scope
* affected scale band
* affected product family
* affected consumer class
* upstream generation change
* freshness transition
* rebuild priority
* rebuild budget class
* fallback permission
* diagnostics

Common invalidation causes:

* terrain edit
* material edit
* imported asset revision changed
* world operation committed
* simulation emitted mutation
* dynamic source moved
* water source changed
* wind source changed
* light/emissive source changed
* streaming region loaded/unloaded
* product formation failed
* runtime budget skipped an update

---

# Query Model

Consumers access products through query contracts.

A query contract should define:

* value type
* coordinate space
* scope
* scale band
* freshness requirements
* fallback allowance
* approximation rules
* diagnostics behavior

Example queries:

* sample signed distance at point
* gather field window around player
* resolve best collision product for body
* resolve visual terrain product for view
* sample wind velocity in region
* sample water level and flow
* sample AI threat value
* inspect product freshness
* inspect product lineage
* request fallback product

Consumers should not read private storage internals.

---

# Simulation Contract Model

Simulation domains consume and produce products through explicit contracts.

A simulation may:

* consume boundary products
* consume material/substance products
* consume flow products
* publish new products
* publish mutation requests
* publish invalidation records
* publish diagnostics

Examples:

| Simulation   | Consumes                                          | Produces                                          |
| ------------ | ------------------------------------------------- | ------------------------------------------------- |
| Wind         | obstacle fields, terrain summaries, weather zones | velocity, turbulence, advection fields            |
| Liquid       | terrain SDF, material permeability, gravity       | level set, velocity, pressure, foam, wetness      |
| Heat/fire    | material, fuel, airflow, wetness                  | temperature, smoke, burn state, emissive products |
| Snow         | support, wind, temperature, material              | accumulation, compaction, meltwater               |
| Erosion      | slope, material, water flow                       | sediment, deposition, mutation requests           |
| AI influence | entities, sound, scent, visibility, navigation    | threat, desirability, cost, perception fields     |
| Lighting     | surface, material, emissive, occlusion            | radiance, irradiance, lighting diagnostics        |

A simulation must not silently mutate authoritative field truth unless it owns the write target or goes through a governed mutation path.

---

# Runtime Cache Model

Runtime caches are allowed but derived.

Examples:

* GPU product buffers
* sparse brick atlases
* clipmap page tables
* spatial acceleration structures
* temporal histories
* propagation queues
* per-view selection sets
* simulation tiles

Rules:

1. Runtime caches must track product generation or source generation.
2. Runtime caches must be invalidated when upstream products change.
3. Runtime caches must expose stale/fallback state when relevant.
4. Runtime caches must not become hidden authoritative truth.
5. Persisted caches need lineage, freshness, and recovery policy.

---

# Diagnostics Model

Diagnostics should be first-class.

The system should expose issues such as:

* missing product
* missing upstream dependency
* stale product
* product generation mismatch
* invalid scope
* invalid scale-band request
* unsupported consumer class
* failed formation
* failed-preserved fallback
* ghost summary use
* excessive rebuild backlog
* insufficient runtime budget
* solver instability
* low confidence
* coverage gap
* product lineage ambiguity
* unauthorized mutation path

Editor tools should be able to answer:

* Why is this region stale?
* What products exist here?
* What is loaded?
* What is fallback-only?
* What failed to rebuild?
* What system depends on this product?
* Which source changed?
* Which multiplayer generation is active?

---

# Ownership Boundaries

## Foundation

Foundation may own reusable low-level vocabulary such as typed identity, diagnostics, ratification, schema, and command contracts.

Foundation should not own world-field policy, rendering policy, simulation behavior, or runtime product formation.

## Domain field/world crates

Domain crates should own engine-agnostic product contracts, descriptors, ratifiers, query contracts, and invalidation contracts.

Examples:

| Concern                                          | Owning Area             |
| ------------------------------------------------ | ----------------------- |
| SDF primitives and queries                       | SDF domain              |
| Spatial coordinates and partitioning             | Spatial domain          |
| Chunk and streaming policy contracts             | Chunking domain         |
| World operation logs and invalidation            | World operations domain |
| Field product descriptors and world SDF payloads | World SDF domain        |
| Asset source/artifact descriptors                | Asset domain            |
| Neutral graph substrate where needed             | Graph domain            |

## Simulation domains

Specialized simulation domains own their solver state and invariants.

Examples:

* wind/weather
* liquids/fluids
* heat/fire
* snow/erosion
* AI influence
* physics/collision runtime

## Engine runtime

Engine runtime owns execution:

* scheduling
* runtime resources
* product producer execution
* render execution
* GPU resources
* streaming execution
* runtime cache formation
* plugin composition

## Apps and editor

Apps and editor tooling own presentation, command routing, panels, workflows, and projections.

They consume expression, diagnostic, and product inspection contracts.

They do not own field-world invariants.

---

# Example Workflows

## Terrain edit

1. User sculpts terrain.
2. World operation is validated and committed.
3. Affected chunks/regions are invalidated.
4. Surface products rebuild.
5. Collision products rebuild or fallback.
6. Navigation, lighting, wind, and water products refresh if affected.
7. Editor shows freshness and diagnostics.

## Water source placed

1. User places water source.
2. Source is validated by the owning domain.
3. Basin/region products are invalidated.
4. Liquid products form: level set, velocity, wetness, foam.
5. Rendering consumes visual water products.
6. Physics consumes buoyancy/query products.
7. Materials consume wetness products.
8. Multiplayer replicates authoritative source/state decisions if gameplay-relevant.

## Wind zone changed

1. Wind source changes.
2. Wind products for affected regions become stale.
3. Obstacles and terrain products are sampled as boundary inputs.
4. Flow products rebuild within budget.
5. Particles, foliage, smoke, and audio consume the updated field.
6. Far regions use summaries or fallback.

## AI scent field

1. Creature emits scent source.
2. Scent product consumes source events, wind, terrain occlusion, and decay rules.
3. AI queries scent field.
4. Editor can display scent heatmap.
5. Multiplayer replicates only gameplay-relevant source/state, not local debug overlays.

## Large cave streaming

1. Player enters cave entrance sector.
2. Connected sectors become high-priority.
3. Surface, collision, lighting, acoustic, and navigation products load for nearby sectors.
4. Far branches remain summaries.
5. Ghost summaries provide non-authoritative continuity.
6. Editor diagnostics show which cave products are resident, stale, fallback, or missing.

---

# Invariants

1. Every product has stable identity.
2. Every product declares family and kind.
3. Every product declares scope.
4. Every product declares scale band or explicitly opts out.
5. Every formed product has source lineage.
6. Every product has freshness state.
7. Every product has residency state when runtime loading applies.
8. Every product declares consumer class.
9. Every product has retention policy.
10. Every product has rebuild policy.
11. Consumers use query contracts, not private storage.
12. Runtime caches are derived unless explicitly retained as cache artifacts.
13. Invalidation flows through declared dependencies.
14. Ghost summaries are not full authority.
15. Fallback usage is explicit and diagnosable.
16. Simulation domains do not bypass governed mutation paths.
17. Multiplayer-relevant authority is explicit per product family.
18. Product formation failures preserve diagnostics.

---

# Failure Modes and Mitigations

## One universal product structure

Failure:
A single structure becomes mandatory for all systems.

Mitigation:
Keep metadata common, but allow storage, query, update, and solver models to differ by family.

## Renderer-owned world truth

Failure:
Render caches become the only useful world representation.

Mitigation:
Keep render products downstream from field products and mark render caches as derived.

## Simulation bypasses product contracts

Failure:
A simulation mutates field-world truth directly.

Mitigation:
Require explicit mutation requests, ownership, validation, and invalidation.

## Dirty flags replace lineage

Failure:
A product is dirty but nobody knows why.

Mitigation:
Use typed invalidation records and product dependency tracking.

## Ghost summaries hide missing data

Failure:
Approximate summaries hide streaming or formation errors.

Mitigation:
Make ghost use bounded, consumer-specific, and visible in diagnostics.

## Over-abstraction before proof

Failure:
The product system becomes abstract before one strong path works.

Mitigation:
Implement one narrow complete slice first.

Recommended first slice:

1. SDF world edit.
2. Dirty chunks/regions.
3. Formed SDF field product.
4. Preview or collision product.
5. Freshness/provenance diagnostics.

---

# Implementation Phases

## Phase 1: Product Contract Foundation

Deliver:

* product descriptor vocabulary
* product family and kind taxonomy
* scope descriptors
* scale-band descriptors
* source lineage model
* freshness states
* residency states
* retention and rebuild policies
* diagnostic issue vocabulary
* ratification rules

## Phase 2: Dependency and Invalidation Contracts

Deliver:

* product dependency records
* invalidation records
* affected scope and scale-band tracking
* rebuild policy classification
* fallback permission model
* validation tests

## Phase 3: First Concrete Product Path

Deliver a narrow end-to-end path:

* SDF/world operation change
* affected chunk/region invalidation
* formed field product descriptor
* field preview or collision product
* freshness and provenance diagnostics

## Phase 4: Runtime Residency

Deliver:

* resident/non-resident states
* pending load/unload states
* ghost summary metadata
* fallback resolution
* stale product diagnostics
* editor inspection

## Phase 5: Query Contracts

Deliver:

* query contract descriptors
* product selection by consumer/scope/band
* stale/fallback query rules
* diagnostics for unsupported queries
* tests for consumer isolation

## Phase 6: First Simulation Product Slice

Deliver one specialized simulation path.

Good candidates:

* wind over SDF obstacle products
* simple influence field over regions
* liquid boundary preview product

## Phase 7: Advanced Product Families

Add broader families as needed:

* liquid products
* flow/wind products
* heat/fire products
* influence products
* navigation products
* lighting products
* collaborative/remote preview products

Each family must enter through product contracts, not private runtime shortcuts.

---

# Acceptance Criteria

The design is successful when Runenwerk can support these without ownership refactors:

* SDF-derived render products
* collision products sharing lineage with render products but using different representation
* editor-visible field product freshness and provenance
* incremental changed-region rebuilds
* open-world product streaming
* large cave sector/portal product streaming
* wind products over terrain and obstacle fields
* liquid products over field boundaries
* influence fields for AI and gameplay
* lighting products as one field family among others
* ghost/fallback summaries with diagnostics
* multiplayer replication of authoritative operations and generations
* local regeneration of non-authoritative visual products

---

# Short Note on an Earlier GI-Specific Concept

An earlier idea focused on an adaptive graph for SDF-based global illumination. The useful parts of that idea are retained only as possible implementation details for the lighting product family: adaptive placement, directional lighting summaries, occlusion-aware propagation, temporal confidence, and bounded updates.

Those ideas are not the parent architecture. The parent architecture is the Adaptive Field Product System.

---

# Summary

The Adaptive Field Product System is Runenwerk's long-term architecture for field-based world data.

It provides shared rules for product identity, scope, LOD, lineage, freshness, residency, invalidation, query contracts, diagnostics, and consumer ownership.

It does not force rendering, physics, fluids, wind, AI, lighting, caves, multiplayer, and editor tooling into one universal structure.

Instead, each system gets the product shape it needs while staying connected through explicit dependencies and diagnostics.
