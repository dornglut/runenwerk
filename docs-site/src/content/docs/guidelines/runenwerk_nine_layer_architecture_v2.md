---
title: Architecture Design
description: Architecture Design
---

# Runenwerk Nine-Layer Engine Platform Architecture
_Last revised: 2026-04-15_

## Status

This document defines the **intended long-term top-level architecture** for Runenwerk.

It replaces the earlier hexa-layer draft with a stricter and more extensible model. The new model is designed to fit the repository’s current shape rather than impose an abstract architecture from outside. In particular, it aligns with:

- the existing split between reusable `domain/`, runtime composition in `engine/`, transport and replay concerns in `net/`, and concrete app wiring in `apps/`
- the editor runtime’s existing distinction between **document state**, **runtime world state**, **ID projection**, and **undo/redo history**
- the net workspace’s existing distinction between **transport-agnostic contracts**, **concrete transport runtime**, **simulation identity vocabulary**, and **history/replay**

This architecture is intended to be stable at the **top level**. Internal contracts, crate boundaries, and sublayers can continue to evolve.

---

## Purpose

Runenwerk is not only a game engine runtime. It is intended to become a platform that can support:

- hot ECS-based simulation
- deterministic scheduling where required
- transactional authoritative mutation
- durable persistence, replay, and recovery
- derived projections and subscriptions
- world partitioning, streaming, and authority ownership
- authored content and asset workflows
- multiple representations and viewport backends
- networking and replication
- editor and tooling workflows
- future support for animation, GI, large worlds, multiplayer, and collaboration

The goal is **not** to make the entire engine a database.

The goal is **not** to make ECS pretend to be the only storage, query, persistence, or editor model.

The goal is to build an engine/editor platform with **nine explicit layers**, where each layer owns a different class of truth, contracts, and lifecycle.

---

## Design principles

### Primary goals

- Keep hot runtime execution data-oriented and scheduler-friendly
- Make mutation boundaries explicit and authoritative
- Make durable state recoverable and replayable
- Make query/read models derived and rebuildable
- Keep authority and scale concerns explicit
- Treat authored content as a first-class platform concern
- Prevent renderer and viewport contracts from coupling directly to ECS storage
- Treat networking/replication as a first-class platform concern, not a transport afterthought
- Treat tools, previews, and editors as first-class capabilities

### Non-goals

- Do not force every runtime structure into durable storage
- Do not force every render or view structure into relational or DB-shaped abstractions
- Do not make editor-facing document state identical to loaded runtime state
- Do not make transport or protocol details leak into gameplay/runtime semantics
- Do not make one abstraction pretend to solve simulation, persistence, query, authority, rendering, and tooling simultaneously

### Architectural rules

1. **Runtime truth is not durable truth.**
2. **Durable truth is not query truth.**
3. **Authoring document truth is not instantiated runtime truth.**
4. **Representation truth is not ECS storage truth.**
5. **Transport-visible truth is not identical to local runtime truth.**
6. **Tool previews are not committed world state.**
7. **Partition ownership must always be explicit.**
8. **Cross-layer mutation must enter through the mutation/commit boundary.**
9. **Derived layers must be rebuildable from authoritative committed inputs.**
10. **Top-level layers own responsibilities; sublayers own implementation details.**

---

# The nine layers

1. Runtime Simulation Layer
2. Mutation / Commit Layer
3. Persistence / Recovery Layer
4. Projection / Query Layer
5. Authority / Partition Layer
6. Asset / Content Layer
7. Representation / Extraction Layer
8. Networking / Replication Layer
9. Editor / Tooling Layer

Each layer owns a distinct kind of truth or contract.

---

# 1. Runtime Simulation Layer

## Responsibility

The Runtime Simulation Layer is the hot engine core.

It owns the currently loaded world state and the data structures used for frame and tick execution.

## Owns

- ECS world state
- scheduler execution
- fixed-step and frame-step orchestration
- frame-local and tick-local state
- transient runtime caches
- animation runtime state
- physics runtime state
- culling and runtime visibility state
- simulation-time messaging primitives
- runtime-side tool preview state when required for interaction

## Does not own

- durable persistence guarantees
- audit history
- query indexes
- content import/cook pipelines
- transport/protocol semantics
- editor shell presentation
- backend-specific render packets

## Design principles

- data-oriented
- cache-aware
- deterministic where required
- explicit schedule phases
- explicit ownership of resources and messaging primitives
- free to use specialized runtime data structures

## Notes for Runenwerk

This layer should remain free to use archetypes, sparse sets, tick buffers, broadcast streams, work queues, animation pose buffers, GI caches, and other hot-path structures without forcing them through durability or projection semantics.

---

# 2. Mutation / Commit Layer

## Responsibility

The Mutation / Commit Layer is the **authoritative write boundary**.

It turns incoming intent into validated, ordered, atomic world mutation and produces a stable commit artifact that downstream layers can consume.

## Core stages

### A. Intent
The requested action.

### B. Plan
The validated and normalized mutation plan.

### C. Apply
The authoritative world mutation against loaded runtime state.

### D. Commit
The stable artifact emitted after successful apply.

## Owns

- mutation ingress
- validation and invariants
- commit/abort boundaries
- transaction semantics
- stable mutation metadata
- causality metadata
- undoable mutation units where applicable
- domain-specific consistency policy

## Does not own

- long-term storage implementation
- projection indexing
- renderer/backend specifics
- editor shell widgets

## Required guarantees

A mutation unit must define:

- who is allowed to submit it
- which state it can affect
- whether it is atomic, partition-local, or distributed
- which invariants must hold before commit
- which version counters advance
- which downstream layers must observe the resulting commit

## Commit record requirements

Every durable-ready commit record should contain at least:

- `commit_id`
- `transaction_id`
- `origin`
- `authority_scope`
- `affected_domains`
- `affected_partitions`
- `base_versions`
- `result_versions`
- `commit_timestamp`
- semantic mutation payload
- optional physical diff payload

---

# 3. Persistence / Recovery Layer

## Responsibility

The Persistence / Recovery Layer makes committed truth durable, recoverable, and replayable.

## Owns

- append-only journals
- snapshots/checkpoints
- replay
- crash recovery
- schema migration boundaries
- retention and archival policy
- recovery-time validation hooks

## Does not own

- hot runtime scheduling
- query indexing
- editor widget state
- transport-specific replication logic

## Design principles

- append-first
- deterministic recovery
- explicit schema versioning
- checkpointed replay
- domain-scoped durability policies

## Durability classes

Runenwerk should classify durable intent by domain:

### Strong durability
- editor scene/document commits
- authored asset metadata
- inventory/economy/progression domains
- authoritative replicated world mutations that must survive restart

### Weak or recoverable-only durability
- diagnostics snapshots
- replay checkpoints
- cache warm-state

### Non-durable
- culling results
- animation scratch
- transient preview state
- GPU staging buffers
- temporary debug overlays

---

# 4. Projection / Query Layer

## Responsibility

The Projection / Query Layer owns derived read models.

It turns committed truth into queryable, subscribable, consumer-optimized models.

## Owns

- indexes
- materialized views
- search models
- editor outliner/inspector projections
- diagnostics projections
- replication-facing projections
- subscriptions and observer feeds

## Does not own

- primary mutation
- source-of-truth runtime scheduling
- transport I/O
- renderer backend implementation

## Design principles

- read models are derived
- denormalization is allowed
- rebuildability is mandatory
- subscriptions observe commits, not arbitrary hot state mutation

## Important Runenwerk implication

The current editor runtime already hints at this distinction: document state, runtime world state, and parity/projection checks are separate concerns. That direction should be generalized rather than collapsed.

---

# 5. Authority / Partition Layer

## Responsibility

The Authority / Partition Layer owns scale, locality, routing, and authority boundaries.

## Owns

- partition topology
- authority ownership
- residency policy
- simulation islands
- routing between authority domains
- authority migration
- partition-local checkpoint coordination
- cross-partition mutation policy

## Does not own

- transport framing
- renderer packets
- editor widget layout
- hot per-system simulation logic

## Partition concepts

Runenwerk should not overload a single “partition” term to mean everything. Distinguish at least:

- **streaming unit**
- **authority unit**
- **simulation unit**
- **replication scope**
- **checkpoint domain**

Sometimes these align. Often they do not.

## Cross-partition mutation classes

A mutation must declare which class it belongs to:

- **local atomic** — must complete within one authority scope
- **coordinated distributed** — requires explicit multi-scope orchestration
- **eventual mirrored** — applied locally, propagated asynchronously
- **disallowed** — invalid unless lifted to a higher authority boundary

---

# 6. Asset / Content Layer

## Responsibility

The Asset / Content Layer owns authored content truth across source, import, cooked, and instantiable forms.

## Owns

- asset identity
- content versioning
- scene/document schemas
- prefab/template definitions
- source asset records
- import/build/cook outputs
- dependency graph
- invalidation
- content compatibility and migration rules

## Does not own

- instantiated runtime world truth
- editor shell widgets
- transport session logic
- backend render execution

## Why this is its own layer

Content truth is not the same as runtime simulation truth and not the same as durable session truth.

A scene file, prefab, material graph, or imported mesh description must remain stable and manageable even when no world instance is running.

## Runenwerk fit

This aligns with the existing split between editor-facing persistence and runtime scene state rather than collapsing both into the world itself.

---

# 7. Representation / Extraction Layer

## Responsibility

The Representation / Extraction Layer transforms runtime and content truth into consumer-facing representations.

This is the boundary that prevents ECS storage from becoming the accidental renderer contract.

## Owns

- extracted scene representations
- render proxies
- picking proxies
- overlay representations
- debug representations
- LOD/proxy selection results
- backend-neutral render packets
- viewport-scene extraction contracts

## Does not own

- gameplay simulation rules
- transport sessions
- asset import pipelines
- editor shell panels

## Design principles

- extraction is explicit
- renderer input is derived, not direct ECS access
- multiple representation families are first-class
- one world may produce different extracted views for different consumers

## Representation families

Runenwerk should support at minimum:

- SDF primitives
- static meshes
- skinned meshes
- terrain/chunk renderables
- lights
- particles
- decals
- volumetrics
- GI/debug structures
- editor gizmos
- tool overlays
- picking-only representations

---

# 8. Networking / Replication Layer

## Responsibility

The Networking / Replication Layer owns how truth crosses process, machine, and user boundaries.

## Owns

- protocol envelopes and version compatibility
- replication semantics
- input ingestion contracts
- snapshot/delta policy
- interest management
- prediction/reconciliation policy
- network-visible identity
- lane/transport policy abstraction
- collaboration transport feeds where applicable

## Does not own

- raw gameplay authority rules
- renderer extraction
- editor panel logic
- asset cook pipelines

## Design principles

- contract-first
- transport-replaceable
- explicit distinction between local-only and replicated state
- explicit distinction between authoritative, predicted, interpolated, and observed state

## Runenwerk fit

This matches the current `net/` subtree direction:

- `engine_net` as the transport-agnostic contract layer
- `engine_net_quic` as the concrete transport/runtime adapter
- `engine_sim` as shared deterministic identity vocabulary
- `engine_history` as replay/history substrate

That direction should be preserved.

---

# 9. Editor / Tooling Layer

## Responsibility

The Editor / Tooling Layer makes the platform authorable.

It owns human-facing workflows, tools, previews, interaction state, and panel/view composition.

## Owns

- editor sessions
- selection and inspection flows
- tool lifecycle
- preview state
- viewport interaction state
- panel-facing presentation state
- undo/redo UX policy on top of mutation history
- diagnostics overlays
- authoring workflow orchestration

## Does not own

- durable journal implementation
- transport framing
- renderer backend execution
- content import backend details
- authority routing internals

## Design principles

- tools are first-class
- previews are explicit
- begin/update/commit/cancel flows are explicit
- editor document state and runtime instance state are related but not identical
- viewports are editor surfaces, not only render surfaces

## Runenwerk fit

This layer aligns strongly with the current editor app/runtime split, including:

- document state
- runtime world state
- history-backed commands/transactions
- selection/inspection
- viewport interaction state
- preview/commit flows

The architecture should preserve and generalize that separation.

---

# Cross-cutting foundations

## Identity model

Runenwerk should define stable identities for at least:

- `EntityId`
- `ComponentTypeId`
- `ResourceTypeId`
- `AssetId`
- `DocumentId`
- `PartitionId`
- `CommitId`
- `TransactionId`
- `ProjectionId`
- `ViewportId`
- `SessionId`

Identity policy must answer:

- stable across restart or not
- local-only or globally routable
- authority-owned or content-owned
- replay-safe or runtime-only

## Version model

Runenwerk should define explicit versions for at least:

- world version
- partition version
- document version
- asset version
- projection version
- schema version
- replication protocol version

Without versioning, replay, undo/redo, cross-partition routing, and conflict detection remain underspecified.

## Consistency classes

Not every mutation deserves the same guarantees.

Define at least:

- **strict atomic**
- **ordered eventual**
- **local ephemeral**

This prevents DB-grade semantics from leaking into hot transient systems unnecessarily.

---

# Canonical data flows

## Mutation flow

1. A tool, game domain, script, network input, or automation source emits intent.
2. Intent enters the Mutation / Commit Layer.
3. Validation, authority checks, and planning occur.
4. Runtime Simulation Layer is mutated atomically where permitted.
5. A stable commit record is produced.
6. Persistence / Recovery records the commit according to durability policy.
7. Projection / Query updates derived read models.
8. Authority / Partition routes or mirrors partition-relevant effects.
9. Networking / Replication emits network-visible deltas or snapshots where applicable.
10. Editor / Tooling refreshes session-facing presentation and subscriptions.

## Recovery flow

1. Persistence / Recovery loads latest valid checkpoint and journal tail.
2. Runtime Simulation Layer is reconstructed.
3. Projection / Query rebuilds or replays derived views.
4. Authority / Partition restores topology and ownership state.
5. Networking / Replication restores protocol/runtime state where necessary.
6. Editor / Tooling reconnects to current derived presentation state.

## Viewport flow

1. Runtime Simulation supplies loaded state.
2. Asset / Content supplies authored metadata or cooked representation inputs as needed.
3. Representation / Extraction produces renderer-neutral scene/view packets.
4. Editor / Tooling adds selection, gizmos, overlays, and preview state.
5. Renderer adapters consume extracted packets.
6. Picking and interaction feed new editor intent back into the mutation boundary.

---

# Dependency direction between layers

At the top level, dependency direction should be disciplined:

- Runtime Simulation may depend on domain/runtime primitives, but should not depend on Editor / Tooling.
- Mutation / Commit may coordinate Runtime, Authority, and policy metadata, but should not depend on renderer backends.
- Persistence / Recovery depends on commit artifacts, not on editor widgets.
- Projection / Query depends on committed truth, not on arbitrary hot runtime internals.
- Authority / Partition coordinates Runtime and Mutation boundaries, but should not own transport encoding.
- Asset / Content should remain usable without a running world.
- Representation / Extraction depends on Runtime and Content inputs, not vice versa.
- Networking / Replication depends on authority, commit, simulation vocabulary, and protocol contracts, not on editor shell code.
- Editor / Tooling composes several lower layers but should not become the hidden owner of them.

---

# Recommended repository mapping

This architecture should map onto the repository like this:

## `domain/`
Own reusable engine-agnostic contracts and subdomain models.

Likely long-term homes:
- simulation vocabulary
- scene/document contracts
- editor-core contracts
- content schema contracts
- projection model contracts
- extraction model contracts where engine-agnostic

## `engine/`
Own runtime composition, schedule integration, render/runtime plugins, and engine-specific execution surfaces.

## `net/`
Own networking and replay/history contracts plus transport/runtime adapters.

## `apps/`
Own concrete app composition, shell wiring, and process-level integration.

## `adapters/`
Own integration to external runtimes such as Godot.

---

# Crate/module direction

A credible long-term target is something close to:

- `domain/runtime/*`
- `domain/scene/*`
- `domain/content/*`
- `domain/editor/*`
- `domain/representation/*`
- `engine/runtime/*`
- `engine/render/*`
- `engine/commit/*`
- `engine/projections/*`
- `engine/authority/*`
- `net/engine_net`
- `net/engine_net_quic`
- `net/engine_sim`
- `net/engine_history`
- `apps/runenwerk_editor`

The exact crate layout can evolve. The top-level layer ownership should remain stable.

---

# Migration priorities

## Phase 1 — lock terminology and ownership
- adopt the nine-layer model in docs
- define identity and version vocabulary
- classify current modules by owning layer
- remove obviously misplaced ownership

## Phase 2 — harden mutation/commit boundaries
- formalize intent/plan/apply/commit stages
- define commit record schema
- define consistency classes
- make undo/redo ride on mutation history rather than ad-hoc editor logic

## Phase 3 — strengthen persistence/recovery
- formalize journal and snapshot contracts
- define durability policy by domain
- align replay and recovery vocabulary with `engine_history`

## Phase 4 — formalize projections
- separate authoritative state from read models more aggressively
- generalize editor projections
- define rebuildable projection policies

## Phase 5 — promote representation/extraction
- make viewport/render extraction contracts explicit
- prevent renderer backends from depending directly on ECS storage shapes
- define picking/debug/tool overlay extraction separately

## Phase 6 — formalize authority/partition boundaries
- define local vs distributed mutation classes
- define residency, simulation island, authority zone, and replication scope contracts
- add explicit cross-partition policy

## Phase 7 — consolidate content layer
- formalize scene/document/content schemas
- align editor persistence and runtime instantiation around content truth
- add dependency and invalidation rules

## Phase 8 — align networking/replication explicitly
- keep transport-agnostic contract-first `net/` design
- define network-visible state boundaries against commit/authority/projection layers
- align recovery and replication semantics where needed

---

# Why this is better than the earlier hexa-layer draft

The hexa-layer draft was directionally good, but it under-modeled several important boundaries:

- it did not give **content** first-class ownership
- it did not make **representation/extraction** explicit enough
- it left **networking/replication** too implicit
- it did not formalize identity, versioning, or commit artifacts strongly enough
- it still allowed document/runtime/projection concerns to blur

The nine-layer model fixes those gaps while staying disciplined.

---

# Why this is not inflated

A layer should exist only when it owns a distinct kind of truth, lifecycle, or contract.

This model passes that test:

- runtime hot state
- authoritative mutation
- durability and replay
- derived read models
- authority and scale
- authored content
- extracted representations
- remote sync
- human authoring workflows

These are not arbitrary categories. They are real architectural fault lines.

---

# Final position

Runenwerk should be:

- runtime-first where hot execution matters
- commit-oriented where authoritative mutation matters
- durable where recovery matters
- projection-rich where queries and subscriptions matter
- authority-aware where scale matters
- content-first where authored truth matters
- representation-explicit where rendering and picking matter
- network-explicit where remote state matters
- tooling-first where authoring matters

It should not be:

- pure ECS only
- pure DB only
- pure editor app only
- pure renderer-first
- pure transport-first

It should be an **engine/editor platform with explicit layered capabilities**.

This is the recommended long-term top-level architecture.
