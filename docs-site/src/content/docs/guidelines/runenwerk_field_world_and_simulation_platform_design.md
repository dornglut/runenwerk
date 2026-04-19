# Runenwerk Field World and Simulation Platform Design

## Status
Draft for implementation

## Purpose
Define the long-term architecture for Runenwerk as:

- a **field world platform** for editable, multiscale, large-world substrates
- a **simulation platform** composed of specialized simulation domains
- a **contract layer** that allows those domains to interact coherently without collapsing them into one solver or one universal world form

This document is not a rendering-only design, not a physics-only design, and not a temporary feature roadmap.
It defines the long-term architecture that should let Runenwerk support:

- editable SDF-driven and field-driven worlds
- large-world chunking, partitioning, and multiscale field coverage
- rendering, picking, collision, navigation, diagnostics, and tooling products
- rigid physics, cloth, soft bodies, fluids, snow, erosion, and animation interaction
- future collaboration, replication, and remote tooling
- strong doctrine-aligned ownership boundaries

The primary goal is to define **what kind of platform Runenwerk is becoming**, what the major ownership boundaries are, and which architectural mistakes must be rejected from the start.

---

## Strategic Decision

Runenwerk should be built as a **multi-reality field world platform with a modular simulation platform on top of it**.

The correct long-term model is not:

- a traditional renderer with a few SDF features
- a traditional physics engine with extra systems bolted on
- one giant universal field solver
- one universal world structure consumed directly by every subsystem
- one consumer architecture treated as the owner of platform truth

Instead, the platform should be organized as:

1. **Field World Platform**
   - the editable world substrate
   - multiscale field formation and runtime products
   - chunking, clipmaps, caches, invalidation, and consumer-ready products

2. **Simulation Platform**
   - the family of specialized simulation domains
   - rigid bodies, deformables, fluids, snow, erosion, animation interaction, and world processes

3. **Field–Simulation Contract Layer**
   - stable query, mutation, constraint, and material-exchange interfaces between the two

This is the intended architecture.

### Core governing sentence

**The field world is the shared substrate. The simulation platform is the family of specialized domains that act through it. The contract layer is what keeps them coherent without fusing them together.**

---

## Architectural Position Relative to the Governing Doctrine

This design inherits the governing stance that Runenwerk is a **multi-reality world platform**.

That means:

- authored world definitions are not runtime truth merely because they exist
- runtime systems do not own all world forms directly
- consumers observe or receive shaped products, not private owner state
- field products, simulation products, render products, and shared products are all governed realities with explicit formation and exposure rules

This design therefore follows these doctrine-level principles:

- **formation over implicit equivalence**
- **ratification over ambient mutation**
- **observation frames over raw authority access**
- **expression products over renderer-private internals**
- **retained propagation structures over ad hoc dissemination**
- **Rust-enforced boundaries over prose-only separation**

---

## Problem Statement

Runenwerk needs an architecture that can support all of the following at once:

- editable high-fidelity world geometry
- large spatial scope
- efficient rendering and picking
- fast incremental updates to changed regions
- collision and simulation interaction with edited terrain/world matter
- multiple consumer products beyond final rendering
- multiple simulation domains with different solvers and timescales
- future streaming, replication, collaboration, and tooling

A naive architecture fails quickly.

### Failure mode 1 — brute-force scene evaluation
Evaluating the full world distance function directly for every consumer and every query does not scale.

### Failure mode 2 — one representation for every consumer
A single world structure is not the best form for rendering, physics, fluids, snow, tooling, streaming, and replication simultaneously.

### Failure mode 3 — one giant universal solver
Rigid bodies, cloth, soft bodies, fluids, snow, erosion, and animation do not naturally belong in one monolithic solver.

### Failure mode 4 — direct coupling between editor/runtime/render/simulation
If tools, rendering, physics, and world storage all depend on each other’s private internals, the platform becomes fragile and unscalable.

---

## Core Vision

Runenwerk should treat the world as a **field-readable, field-writable, multiscale substrate**.

That substrate is not limited to visible geometry. It may include:

- signed distance
- gradient / surface normal information
- material channels
- density / hardness / support
- moisture / temperature
- snow / sediment / accumulation state
- provenance / edit lineage markers
- future domain-specific channels where justified

These fields do not mean that every subsystem becomes a field solver.
They mean that the platform gains a **shared world interaction language**.

That allows:

- rendering to consume field products for image generation
- picking to consume field or derived picking products
- rigid bodies to query collision against field-derived collision representations
- fluids and snow to react to the same world substrate
- erosion and deposition to modify world matter through explicit mutation paths
- animation and tools to interact with the same world through stable contracts

---

## Architectural Philosophy

### 1. The field world is the shared substrate, not the universal solver
The field world provides a common interaction language for world queries, mutations, and consumer product formation.
It does not replace every specialized simulation model.

### 2. Specialized domains remain specialized
Rigid physics, cloth, soft bodies, fluids, snow, erosion, and animation keep the right internal model and solver for their own semantics.
The architecture should not force them into one numerical method or one state representation.

### 3. Formed products are first-class consumer truth
Consumers should not depend primarily on raw authored edit stacks or arbitrary runtime internals.
They should consume **formed products** built for their purpose.

### 4. Multiscale representation is fundamental
Near, mid, and far world bands may legitimately use different formed products and different runtime budgets.
This is part of the architecture, not a late optimization pass.

### 5. Incremental invalidation is a first-class system
Changed regions, dependent products, rebuild priorities, and freshness states must be explicit.
The platform should not rely only on informal dirty flags.

### 6. Consumers are distinct and replaceable
Rendering, picking, collision, navigation, tooling, and simulation are different consumers.
They may share upstream lineage, but they must not be collapsed into one consumer model.

### 7. Timescale separation is fundamental
Fast contacts, local interactions, background world processes, and far-field maintenance do not belong on one implicit cadence.
The architecture must make rate separation explicit.

### 8. Composition and write-target semantics matter
Runenwerk should preserve where world changes come from and where they are written.
Authored layers, session layers, simulation layers, and tooling overlays must remain distinguishable.

---

## Top-Level Architecture

## 1. Field World Platform
Owns the editable world substrate and its multiscale formed/runtime products.

Responsibilities:

- authored field-world definitions
- normalized and validated authored structures
- formed chunk-local field products
- multiscale clipmap / band products
- runtime chunk and band residency
- field invalidation and rebuild lineage
- world-query interfaces
- expression products for rendering, picking, overlays, diagnostics, and future consumers
- derived collision/query products where appropriate

## 2. Simulation Platform
Owns the family of specialized simulation domains and simulation orchestration.

Responsibilities:

- fixed-step simulation orchestration
- substeps and rate classes
- activation and sleeping
- spatial activation and simulation tiers
- cross-domain scheduling and budget enforcement
- domain ownership for rigid physics, deformables, fluids, snow, erosion, animation interaction, and world processes

## 3. Field–Simulation Contract Layer
Owns the stable cross-domain contracts.

Responsibilities:

- field query interfaces
- field mutation interfaces
- constraint interfaces
- material exchange interfaces
- invalidation and rebuild requests
- cross-timescale handoff rules

## 4. Consumer Product Families
This design assumes multiple downstream consumer families from the start.

Examples:

- render and picking products
- collision and navigation products
- diagnostics and tooling products
- remote preview and shared products
- offline analysis or automation products

### Critical boundary rule
The consumer product families are **not** the owner of the field world platform and **not** the owner of simulation truth.
They are shaped consumers of governed products.

---

## Field World Platform

## Scope
The Field World Platform is the substrate architecture beneath rendering, picking, collision generation, tooling, and simulation interaction.

It is not the viewport architecture, not the physics engine, and not a narrow renderer feature stack.
It is the world-field architecture those systems depend on.

### Identity of the Field World Platform
The Field World Platform should be understood as a governed family of world products with explicit lifecycle, lineage, and consumer boundaries.

It should answer questions such as:

- what world field products exist
- how they are formed
- what spatial scope and scale band they cover
- which changes invalidate them
- which consumers may observe or receive them
- how they are retained, rebuilt, or propagated

## Core Responsibilities

### A. Authored field world
The authored world may include:

- ordered SDF edits
- layered field edits
- procedural terrain definitions
- authored material stamping rules
- field-affecting gameplay content
- authored non-destructive edit layers
- authored world-process parameters where appropriate

The authored world is not executed directly by consumers as the primary long-term runtime model.

### B. Normalized field world
Normalization may include:

- canonicalized edit ordering
- validated layer references
- normalized material channel bindings
- schema migration
- structural flattening where required

### C. Formed field products
The main long-term move is this:

Runenwerk should compile authored field-world definitions into **formed products**.

Examples:

- chunk-local distance products
- chunk-local material channel products
- chunk-local hierarchy summaries
- chunk-local occupancy/support products
- clipmap-band products
- collision-formation products
- navigation/query products
- diagnostics/provenance products

This is the most important distinction in the design.

The authored world may remain edit-friendly and composition-friendly.
The formed products are what make the platform scalable for runtime consumers.

### D. Instantiated runtime field state
At runtime, Runenwerk should manage:

- resident chunk products
- resident clipmap windows
- camera-relative or focus-relative field windows
- product generations and freshness markers
- rebuild queues
- budgets and priorities
- invalidation lineage state

### E. Expression products
The field world platform must support multiple consumer-facing products.

Examples:

- scene color
- picking ids
- overlays
- debug visual products
- clipmap visualization
- brick residency visualization
- invalidation heatmaps
- provenance and freshness views
- remote preview products

---

## Field Product Taxonomy

Runenwerk should treat field-world products as a coherent taxonomy.

### 1. Surface field products
Examples:
- signed distance
- gradient
- occupancy
- support

### 2. Material field products
Examples:
- material id / weights
- density
- hardness
- wetness
- thermal state

### 3. Multiscale field products
Examples:
- near-detail chunk products
- mid-band products
- far clipmap products

### 4. Query and collision products
Examples:
- collision formation inputs
- navigation/support fields
- gameplay query fields

### 5. Expression products
Examples:
- render outputs
- picking outputs
- debug outputs
- remote preview outputs

### 6. Provenance and diagnostics products
Examples:
- product lineage
- invalidation reason data
- rebuild cost summaries
- freshness state

### Product law
A field product should always have, at minimum:

- stable identity
- scope and band information
- source lineage
- version / freshness metadata
- declared consumer class
- retention and rebuild policy

---

## Multiscale World Representation

Runenwerk should explicitly adopt a **multiscale world model**.

### Near field
High-fidelity local products.
Examples:
- sparse detail bricks
- fine chunk-local distance/material products
- high-fidelity collision formation inputs

### Mid field
Coarser regional products.
Examples:
- reduced-detail chunk products
- hierarchical summaries
- coarser support/material representations

### Far field
Very coarse large-scope products.
Examples:
- clipmap-backed distance/material bands
- overview/navigator field products
- far collision/query approximations where needed

### Principle
Near, mid, and far are not merely “quality settings.”
They are legitimate formed products with distinct runtime ownership and budgets.

---

## Spatial Structure

Runenwerk should use explicit spatial structures rather than rely on full-world scans.

The architecture should support, where appropriate:

- chunk partitioning
- region summaries
- hierarchical summaries
- clipmap windows
- spatial activation sets
- edit influence structures
- product coverage windows

### Recommended long-term direction
The strongest candidate architecture is a hybrid of:

- chunk-local formed products
- sparse local detail storage
- hierarchical skip/summary metadata
- clipmap-backed far/mid coverage
- runtime residency and rebuild budgets

This should be treated as the expected long-term direction.

---

## Invalidation and Rebuild Model

Invalidation must be first-class.

### Causes of invalidation
Examples:
- authored edit changed
- procedural input changed
- material layer changed
- terrain generation parameters changed
- simulation mutation affected a region
- world-process update changed matter distribution
- upstream product generation changed

### Invalidation outputs
The platform should track:

- affected region(s)
- affected product(s)
- affected scale band(s)
- affected consumer classes
- freshness state
- rebuild priority and budget class

### Rebuild policy classes
Examples:
- immediate
- budgeted incremental
- lazy on demand
- idle/background
- manual/tool-triggered

### Dependency rule
Invalidation should propagate through explicit product dependency relationships.
A product should know what upstream products or source realities it depends on, and rebuilds should follow those declared dependencies rather than hidden procedural knowledge.

### Principle
Do not model invalidation as an ad hoc “mark some chunks dirty” mechanism only.
Treat it as explicit product lineage and dependency behavior.

---

## Field Query Model

The field world platform should expose explicit query contracts.

Examples:

- sample signed distance
- sample gradient / normal
- sample material channels
- sample density / support / occupancy
- gather local field window
- ray/query against field products
- query product freshness and provenance metadata where allowed

### Rule
Queries should resolve against declared field products or views, not private storage internals.

---

## Field Mutation Model

The platform should expose explicit mutation contracts for world-affecting systems.

Examples:

- add matter
- remove matter
- stamp material
- carve/subtract volume
- smooth/blend/sharpen region
- deform region
- enqueue erosion/deposition effect
- enqueue rebuild for affected products

### Rule
A field mutation is not the same thing as directly mutating every runtime product.
It mutates the owning reality for its scope and triggers governed invalidation/formation paths.

---

## Expression and Consumer Products

The field world platform must support many consumers.

Consumers may include:

- renderer
- picking
- collision generation
- navigation/query systems
- tooling and diagnostics
- remote preview and collaboration
- offline analysis

### Rule
Each consumer receives shaped products through declared contracts.
No consumer should assume entitlement to private owner structures.

---

## Simulation Platform

## Purpose
The Simulation Platform orchestrates and hosts specialized simulation domains that operate over and through the field world substrate.

It is not one giant solver.
It is a modular family of domains.

### Identity of the Simulation Platform
The Simulation Platform should answer questions such as:

- which domains are active
- at what cadence they run
- which scopes they own
- how they query and mutate the field world
- how they exchange constraints and matter
- how they coordinate across timescales

## Core Responsibilities

### A. Simulation orchestration
Owns:

- fixed-step model
- substeps
- rate classes
- sleep/wake behavior
- simulation tiers
- budget enforcement
- scheduling and cross-domain ordering
- broad activation policies

### B. Domain hosting
Owns the existence and lifecycle of distinct simulation domains.

### C. Cross-domain coordination
Owns timing, ordering, and contract-governed interaction between domains.

---

## Simulation Domains

### 1. Rigid and character physics
Owns:
- rigid bodies
- kinematic bodies
- character movement
- contacts
- constraints
- triggers
- solver state

### 2. Deformation domain
Owns:
- cloth
- rope
- soft bodies
- attachments
- compliant constraints

### 3. Material transport domain
Owns:
- water movement
- snow movement
- sediment transport
- deposition and redistribution
- wetness / accumulation behavior

### 4. Animation interaction domain
Owns:
- skeletal pose evaluation
- root motion
- procedural motion hooks
- secondary motion handoff points
- authored/runtime interaction boundaries

### 5. World-process domain
Owns slower environmental processes.
Examples:
- long-term erosion
- melting/freezing
- weathering
- biome-scale accumulation/change

### Solver independence rule
Each domain keeps the right internal model and solver for its semantics.
A domain may internally use bodies, particles, lattices, constraints, graphs, procedural rules, or other representations as justified.
The field world is their shared world interaction substrate, not their forced internal state format.

---

## Simulation Tiers and Timescales

Runenwerk should explicitly support multiple timescales.

### Example classes
- contact-critical
- gameplay-critical
- interaction-local
- background world-process
- remote/approximate
- paused/retained

### Principle
Not every domain and not every region runs at the same cadence.
Cross-timescale interaction must be explicit rather than incidental.

---

## Activation and Spatial Scope

Simulation should scale by relevance and locality.

Mechanisms may include:

- spatial activation
- simulation ownership by chunk/region where appropriate
- sleep/wake behavior
- representation switching
- local high-fidelity windows
- approximate or deferred distant simulation

### Principle
Large total world scale should not imply globally active high-fidelity simulation.

---

## Field–Simulation Contracts

These contracts are the real architectural bridge.

## 1. Query contracts
Allow domains to read the shared world substrate.

Examples:
- query signed distance and gradient
- query support or occupancy
- query material/density/hardness
- query local field windows
- query changed regions since a known generation where allowed

## 2. Mutation contracts
Allow domains to request governed changes to the field world.

Examples:
- remove matter due to digging
- deposit matter due to sediment or snow
- stamp deformation into world substrate
- request blending/smoothing
- request material replacement

## 3. Constraint contracts
Provide a shared language for world-relative and cross-domain constraints.

Examples:
- attachments
- anchors
- contacts
- joints
- soft constraints
- animation handoff constraints

## 4. Material exchange contracts
Provide a shared language for moving and transforming matter.

Examples:
- erosion
- deposition
- compaction
- melt/freeze
- transfer between reservoirs or regions

## 5. Contract safety rules
These contracts should be explicit types and protocols, not informal side-channel mutation.

The contract layer should make the following distinctions explicit:

- query vs mutation
- local/session effect vs ratified world effect
- visual-only rebuild vs gameplay-relevant rebuild
- synchronous request vs deferred work item
- world-authoring write target vs simulation-produced write target
- allowed capability scope for each caller

### Rule
The contract layer exists to preserve coherence without requiring implementation fusion.

---

## Layered Authoring and Composition

Runenwerk should support layered authored world composition.

Long-term examples may include:

- base authored world layer
- authored content-pack layers
- session-local preview layers
- simulation-produced transient layers
- tooling/debug annotation layers where appropriate

### Principle
The platform should make it possible to distinguish:

- where a change is authored
- where a change is temporary
- where a change is simulation-derived
- where a change is ratified vs session-local

This is especially important for non-destructive edits and future collaboration.

### Edit-target rule
A composed world view should not erase where writes go.
The architecture should preserve explicit write-target semantics so that systems know whether they are mutating authored content, session overlays, simulation-owned transient matter, or another governed layer.

---

## Observation and Tooling

The platform must support rich observation and diagnostics.

Examples:

- product availability and freshness
- residency state
- invalidation reason visibility
- rebuild cost visibility
- clipmap/chunk coverage visualization
- material and support inspection
- simulation tier/activation inspection
- cross-domain provenance views

### Rule
Tooling should observe through declared observation and expression products, not through private subsystem internals.

---

## Physics and Collision Position

The architecture should not assume one collision strategy forever.

### Current long-term stance
Collision may be formed separately from render products.
Examples:
- chunk-local collision meshes
- field-query collision support
- hybrid representations

### Principle
Rendering representation and collision representation may legitimately differ.
The important requirement is shared lineage and governed formation, not identical internal format.

---

## Replication and Sharing Fit

The architecture must leave room for sharing and replication.

Possible future shared forms include:

- remote preview products
- collaborative field edit streams
- simulation summaries
- selection/annotation feeds
- review/debug channels

### Rule
Shared forms should be scoped and shaped.
They are not raw simulation-state dumps.

---

## Nine-Layer Alignment

### Layer 1 — Runtime Simulation
Owns live simulation state and active domain execution.

### Layer 2 — Mutation / Ratification
Owns accepted world and simulation mutations for governed scopes.

### Layer 3 — Retention / Recovery
Owns rebuildable or durable retention for field products, snapshots, journals where justified, and recovery lineage.

### Layer 4 — Observation
Owns consumer-facing observation models for diagnostics, tools, and analytics.

### Layer 5 — Authority / Partition
Owns chunk/partition authority, exposure, and which scopes may produce or mutate products.

### Layer 6 — Asset / Content
Owns authored field-world definitions, edit layers, materials, and future authored simulation/world definitions.

### Layer 7 — Expression
Owns render, picking, overlay, debug, and other consumer-facing expression products.

### Layer 8 — Sharing / Replication
Owns remote/shared forms and propagation structures.

### Layer 9 — Editor / Tooling
Owns editor tooling, viewports, diagnostics surfaces, authoring tools, and local session workflows.

---

## Rust Constitution for This Architecture

The design should be enforceable in Rust.

### Required categories of explicit types
Examples:

- field product ids
- chunk ids / region ids / band ids
- product generations / freshness markers
- invalidation classes
- rebuild policy classes
- query capabilities
- mutation capabilities
- simulation domain ids
- timescale / rate-class enums
- constraint kinds
- material exchange operation enums
- write-target identifiers

### Capability rule
Subsystems should act through scoped capability-bearing contracts where possible rather than ambient authority.

### Error rule
Core contracts should use structured error domains.

### Boundary rule
Illegal crossings should be hard to express in code.
A consumer should not be able to bypass field products and reach private owner internals merely because references are available nearby.

---

## What This Design Intentionally Rejects

- a single universal world structure directly consumed by every subsystem
- one giant field solver for every kind of simulation
- brute-force whole-world evaluation as the primary long-term runtime approach
- renderer-private world ownership as the platform center
- physics-private world ownership as the platform center
- treating clipmaps or multiscale products as mere optional optimizations
- ad hoc dirty flags as the only invalidation architecture
- collapsing rendering, collision, simulation, tooling, and sharing into one consumer model

---

## Recommended Long-Term Direction

The strongest current architectural direction for Runenwerk is:

- authored field-world layers
- normalized/validated authored structures
- formed chunk-local field products
- multiscale clipmap or band products
- explicit runtime residency and budgets
- explicit invalidation lineage and rebuild policy
- expression products for rendering, picking, overlays, and diagnostics
- separate but lineage-linked collision/query products
- a simulation platform composed of specialized domains operating through explicit field–simulation contracts

This should be treated as the canonical direction.

### Why this direction is stronger than the obvious alternatives
It is stronger than:

- brute-force whole-world SDF evaluation
- a render-centric world architecture
- a physics-centric world architecture
- a single universal field solver
- a one-representation-for-all-consumers approach

because it preserves:

- doctrine alignment
- multiscale scalability
- consumer replaceability
- domain specialization
- incremental rebuildability
- future collaboration and tooling fit

---

## Success Criteria

This design is successful if Runenwerk can eventually support all of the following without ownership refactors:

- editable high-fidelity world geometry
- large-world streaming and multiscale coverage
- rich render and picking products
- incremental changed-region rebuilds
- separate collision/query formation paths
- rigid physics, deformables, fluids, snow, erosion, and animation interaction through shared contracts
- diagnostics-rich tooling and provenance visibility
- clear ratification, retention, sharing, and migration behavior for major world products

---

## Risks and Watchpoints

The architecture should actively watch for these failure modes during implementation:

- consumer-specific shortcuts becoming de facto platform ownership
- authored edit stacks leaking into hot consumer paths as the primary runtime model
- implicit write targets and unclear mutation ownership
- field products without explicit lineage or freshness state
- simulation domains bypassing contract boundaries for convenience
- clipmaps and scale bands treated as optional tuning rather than real product classes
- too much abstraction before the first strong producer path is proven

---

## Short Version

Runenwerk should become:

- a **Field World Platform** for editable, multiscale, formed world products
- a **Simulation Platform** made of specialized simulation domains
- a **Field–Simulation Contract Layer** that lets those domains interact coherently without collapsing them into one solver

The field world is the shared substrate.
The simulation platform is the domain family that acts through it.
The contract layer is what keeps the whole system coherent, scalable, and extensible.

This is the long-term architecture most consistent with the governing doctrine and the strongest foundation for rich, large-scale, editable, living worlds.

