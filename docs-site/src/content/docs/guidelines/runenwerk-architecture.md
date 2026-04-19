---
title: Runenwerk 9-Layer Architecture Doctrine
description: Canonical governing architecture doctrine for Runenwerk.
---

# Runenwerk Architecture Doctrine
_Last revised: 2026-04-17_

## Status

This document defines the recommended long-term governing architecture for Runenwerk.

The architecture is now defined primarily by:

- world realities
- governing laws
- meaning domains
- transformation contracts
- observation frames
- propagation structures
- retention strategies
- stability classes
- reconciliation policies
- migration paths
- Rust-enforced boundaries

The nine platform layers still exist, but they are no longer the main story. They remain the stable platform map beneath the doctrine.

---

# 1. Purpose

Runenwerk is intended to become a world platform: an engine, editor, simulation runtime, content system, collaboration surface, and multiplayer-capable foundation.

The architecture must support:

- authored world content
- executable runtime simulation
- large-world streaming and partitioning
- rendering and non-render consumer extraction
- authoritative and non-authoritative collaboration
- durable recovery and migration
- undo/redo and authoring history where required
- multiplayer replication and remote presence
- tooling and inspection workflows
- future support for animation, GI, offline processing, automation, and distributed operation

The architecture must do this without collapsing every concern into one universal data model, one storage model, or one runtime owner.

---

# 2. Final architectural stance

Runenwerk is a **multi-reality world platform**.

A world does not exist in one universal form. It exists in several governed realities, each with:

- a purpose
- an owner
- legal operations
- a stability class
- a retention strategy
- an exposure policy
- a reconciliation policy
- migration rules

These realities are related, but not interchangeable.

No consumer, tool, subsystem, or remote peer is entitled to direct access to every reality.

Every movement between realities must occur through explicit transformation contracts.

---

# 3. The governing laws

The architecture is defined by the following laws.

## Law 1 — Multiplicity
A world may exist in multiple realities at once.

These realities are not “copies.” They are governed forms of the same platform world with different guarantees and consumers.

## Law 2 — Formation
A world definition is not executable merely because it exists.

Authored structures become executable only by passing through explicit formation stages.

## Law 3 — Meaning
Each reality belongs to a meaning domain.

A meaning domain owns the semantics, invariants, and legal operations for the realities it governs.

## Law 4 — Ratification
A local change is not automatically governing world change.

Only ratified change may advance authoritative world evolution.

## Law 5 — Observation
No consumer observes authority directly.

Every consumer observes through a declared observation frame.

## Law 6 — Propagation
Shared and expressed realities are not emitted ad hoc.

They move through retained, scope-aware propagation structures.

## Law 7 — Reconciliation
Concurrent divergence is legal only where explicitly allowed.

Every reality must declare its reconciliation policy.

## Law 8 — Stability
Every reality must declare what form of repeatability or invariance it promises.

## Law 9 — Retention
Every reality must declare whether and how it remains reconstructable or durable.

## Law 10 — Migration
Cross-scope movement is explicit migration.

Authority shifts, content publication, import flows, replication handoff, and cross-partition movement may never be incidental side effects.

## Law 11 — Replaceability
Consumer-facing systems must depend on contracts, not hidden owner internals.

## Law 12 — Rust enforcement
Core architectural boundaries must be expressible in Rust types, capabilities, and contracts.

If a major boundary exists only in prose, it is not yet strong enough.

---

# 4. Reality model

Runenwerk should define at least the following realities.

## 4.1 Authored reality
Human-authored world definitions and asset-linked structures.

Examples:
- scene documents
- prefab definitions
- material graphs
- authored gameplay data
- import settings
- authored partition metadata
- future workspace/editor/tool definitions

Properties:
- exists without a running world
- editable by tools and workflows
- not directly executable
- may be collaboration-capable in selected domains

## 4.2 Normalized reality
Authoring structures after validation, canonicalization, and schema alignment.

Examples:
- migrated documents
- canonicalized graph forms
- normalized references
- validated content manifests

Properties:
- closer to formation input than raw authoring
- still not runtime execution state
- ideal boundary for validation and migration tooling

## 4.3 Formed reality
Executable-ready world packages or prepared runtime-oriented content products.

Examples:
- baked component layouts
- prepared streaming chunks
- compiled material products
- navigation-ready spatial products
- prelinked representation inputs

Properties:
- optimized for instantiation and runtime use
- derived from authored/normalized reality
- may exist in multiple target forms for different runtimes or backends

## 4.4 Instantiated reality
Loaded world state created from formed reality and ready for runtime ownership.

Examples:
- loaded entities before simulation advancement
- instantiated partition contents
- loaded runtime resources
- runtime mappings from source content to live instances

Properties:
- executable ownership begins here
- source linkage should remain explicit where needed
- may be partial or streamed

## 4.5 Simulated reality
Hot runtime world evolution under engine rules.

Examples:
- ECS world state
- runtime resources
- transient simulation state
- physics state
- animation runtime state
- authority-local execution state

Properties:
- optimized for execution, not storage
- may contain non-durable state
- may have stronger or weaker stability requirements depending on domain

## 4.6 Ratified reality
Change that has been accepted as governing change for an authority scope.

Examples:
- accepted scene edit transaction
- authoritative gameplay mutation
- accepted authority transfer step
- accepted world state transition metadata

Properties:
- this is the boundary between local activity and governing change
- not all local/session actions reach this reality
- ratification policy varies by domain and scope

## 4.7 Retained reality
Recoverable or reconstructable reality substrate.

Examples:
- durable snapshots
- journals
- retained audit trails
- checkpoints
- recovery packages

Properties:
- durable or rebuildable according to retention strategy
- not every domain requires full history retention
- historical richness is a domain choice, not a universal rule

## 4.8 Observed reality
Consumer-optimized observed forms for inspection, query, diagnostics, and UI.

Examples:
- outliner views
- inspector view models
- search indexes
- diagnostics tables
- analytics summaries
- editor-facing world summaries

Properties:
- derived, not authoritative
- optimized for observation and human/tool consumption
- rebuildable from allowed sources

## 4.9 Expressed reality
Consumer-facing expression products for rendering, picking, overlays, and externalization.

Examples:
- render packets
- picking packets
- overlay packets
- debug visual products
- remote preview payloads

Properties:
- derived, not authoritative
- may be highly consumer-specific
- may share retained intermediates via propagation structures

## 4.10 Shared reality
Remote-visible or multi-participant-visible world forms.

Examples:
- replication snapshots
- deltas
- shared selection feeds
- remote review channels
- presence/state visibility products

Properties:
- not identical to simulated reality
- scoped by trust, interest, capability, and authority
- may include collaboration-specific forms that never become authoritative simulation input directly

## 4.11 Session reality
Local human or tool session state.

Examples:
- selection
- hover
- active gizmos
- panel state
- drafts
- previews
- cursor state
- temporary focus state

Properties:
- not authoritative
- often ephemeral
- may be shareable selectively, but does not become ratified by default

## 4.12 Workflow reality
In-flight cross-step orchestration state.

Examples:
- import progress
- publish flows
- authority migration state
- coordinated multi-scope transactions
- pending compensation state

Properties:
- governs migration paths
- may emit side effects
- must model partial completion and compensation explicitly

---

# 5. Meaning domains

Runenwerk should be organized around meaning domains, not a universal master model.

A meaning domain owns:
- the semantics of its realities
- legal operations
- validation rules
- reconciliation policy
- ratification policy
- migration constraints

Suggested core meaning domains:

- world authoring
- content formation
- runtime simulation
- partition and authority
- observation and tooling
- expression and rendering
- sharing and replication
- workflow orchestration
- retention and recovery
- diagnostics and analysis

## Translation boundary rule
No meaning domain may directly consume another domain’s private structures as if they were its own.

Cross-domain interaction must occur through explicit contracts and translation boundaries.

---

# 6. Formation model

The content/runtime split is generalized into a Runenwerk-native formation model.

## 6.1 Formation path
The default world formation path is:

`Authored -> Normalized -> Formed -> Instantiated -> Simulated`

Not every domain needs every stage materially persisted, but the conceptual stages should remain explicit.

## 6.2 Formation responsibilities
Formation may include:
- schema migration
- canonicalization
- validation
- dependency resolution
- structural flattening
- runtime packaging
- streaming partition product generation
- representation-support product generation
- asset binding and compilation
- future editor/workspace/tool host formation where authored editor-definition workflows exist

## 6.3 Formation target plurality
One authored structure may produce more than one formed product.

Examples:
- one scene may produce a runtime package, a streaming package, a diagnostics package, and a remote-preview package
- one material graph may produce products for several expression backends
- one authored editor/workspace definition may later produce host-ready compositions, packaged editor products, or specialized tool arrangements

## 6.4 Source linkage rule
Where round-tripping or inspection matters, instantiated and formed realities should retain source lineage metadata.

---

# 7. Ratification model

Ratification replaces the more borrowed “commit-centric” framing as the primary doctrine.

## 7.1 Ratification principle
A change governs world evolution only after it passes ratification for its scope.

## 7.2 Ratification classes
Suggested classes:
- immediate local ratification
- authority ratification
- coordinated ratification
- deferred ratification
- non-ratifying session change

## 7.3 Ratified artifact
Instead of a generic commit artifact, Runenwerk should define a ratified change contract such as:

`RatifiedChange`

Suggested contents:
- ratification id
- transaction id
- causality id
- origin
- authority scope
- affected domains
- affected partitions/scopes
- base versions
- result versions
- semantic operations
- ratification class
- reversibility class
- retention hint
- timestamp metadata

## 7.4 Ratification and session state
Session reality does not become ratified automatically.

Selections, previews, hover state, and drafts remain local unless a domain-specific path promotes them intentionally.

---

# 8. Observation model

Observation replaces projection/query as the public doctrine.

## 8.1 Observation frame
An observation frame defines how a consumer may observe a source reality.

An observation frame declares:
- source reality class
- consumer kind
- allowed source data
- staleness tolerance
- freshness markers
- identity exposure rules
- shaping/transformation rules

## 8.2 Examples of observation frames
- outliner frame
- inspector frame
- diagnostics frame
- search frame
- analytics frame
- replication inspection frame

## 8.3 Observation rule
No consumer should be forced to understand authority-internal structure merely to observe the world.

---

# 9. Expression model

Expression replaces the narrower representation/extraction language as the public doctrine.

## 9.1 Expression frame
An expression frame defines how a consumer receives a consumer-facing expressed form.

Examples:
- render expression frame
- picking expression frame
- overlay expression frame
- remote preview expression frame
- offline analysis expression frame

## 9.2 Expression rule
Expression products are derived and replaceable.

Backends and consumers should depend on expression contracts, not on private runtime owner structures.

## 9.3 Expression graph
Runenwerk should support retained expression graphs where useful.

An expression graph may hold:
- source dependencies
- consumer dependencies
- invalidation rules
- cached intermediates
- budget and priority metadata
- version references

---

# 10. Sharing and propagation model

Sharing and expression often need retained dissemination logic.

## 10.1 Propagation structure
A propagation structure is a retained, scope-aware structure that moves shared or expressed forms toward consumers.

Examples:
- interest-managed remote state propagation
- render-view propagation
- tool overlay propagation
- diagnostics feed propagation

## 10.2 Propagation rule
Shared and expressed realities should not rely only on ad hoc full-world scans if a retained scope-aware structure materially improves scalability or correctness.

## 10.3 Sharing rule
Shared reality is governed by:
- authority scope
- trust capability
- audience scope
- relevance/interest
- privacy/exposure rules
- stability and freshness needs

---

# 11. Reconciliation model

Reconciliation generalizes merge/conflict language.

## 11.1 Reconciliation policy
Every reality must declare one of the following classes or an equivalent domain-specific policy:

- forbidden
- reject on ratification
- rule-merged
- structure-merged
- session-local only

## 11.2 Domain examples
- authoritative simulation: usually forbidden or reject on ratification
- collaboration presence: often rule-merged
- scene metadata: may be rule-merged or structure-merged in selected subdomains
- binary asset bodies: often forbidden or replace-only
- session tools: often session-local only

## 11.3 Reconciliation rule
No domain may assume mergeability by default.

Merge must be earned by structure, semantics, and domain guarantees.

---

# 12. Stability model

Stability is broader than strict determinism.

## 12.1 Stability classes
Suggested classes:
- ephemeral
- presentation-stable
- observationally stable
- partition-stable
- replay-stable

## 12.2 Meaning
- **ephemeral**: no replay or repeatability promise
- **presentation-stable**: consumer-facing presentation remains acceptably stable
- **observationally stable**: externally visible observations remain stable under declared conditions
- **partition-stable**: stable within an authority or simulation scope
- **replay-stable**: deterministic replay within declared assumptions

## 12.3 Rule
Every reality and migration path must declare the strongest stability class it requires.

---

# 13. Retention model

Retention replaces the more borrowed storage-mode framing.

## 13.1 Retention strategies
Suggested strategies:
- ephemeral
- rebuildable
- state-retained
- checkpoint-retained
- journal-retained
- audit-retained

## 13.2 Rule
A reality must declare whether it is:
- required for recovery
- optional for recovery
- rebuildable from earlier realities
- session-only
- permanently durable

## 13.3 Important rule
Not every domain requires journal retention.

Historical richness is a domain decision, not a platform-wide moral virtue.

---

# 14. Migration model

Migration replaces “workflow/saga” as the public doctrine for cross-scope movement.

## 14.1 Migration path
A migration path is an explicit governed path by which ownership, state, content, or authority moves across realities, scopes, or domains.

Examples:
- content import migration
- publish migration
- authority migration
- cross-partition transfer migration
- replication reconciliation migration
- editor-to-runtime promotion migration

## 14.2 Migration contract
A migration path should declare:
- source scope
- target scope
- required preconditions
- side effects
- compensation behavior
- retry class
- terminal failure behavior
- visibility policy

## 14.3 Failure classes
Suggested migration failure classes:
- retryable
- compensatable
- requires intervention
- terminal
- external side effect already emitted

---

# 15. Identity, version, and causality

This remains essential.

## 15.1 Stable identity types
Runenwerk should define explicit newtypes for identities such as:
- entity ids
- component ids
- resource ids
- document ids
- asset ids
- partition ids
- viewport ids
- session ids
- ratification ids
- transaction ids
- migration ids

Each identity must declare:
- local or global scope
- restart stability
- replay safety
- ownership domain

## 15.2 Version sets
Runenwerk should define explicit version types for:
- world version
- partition version
- document version
- asset version
- frame/expression version
- protocol version
- schema version

## 15.3 Base and result versions
Every ratifiable change should declare the versions it assumed where required, and every successful ratification should declare the versions it advanced.

## 15.4 Causality
Causality must remain explicit so the platform can reason about:
- ordering
- parentage
- rebases
- divergence
- reconciliation lineage
- migration lineage

---

# 16. Capability and trust

Trust is cross-cutting and not optional.

Every participant should be classifiable, for example:
- local trusted tool
- local untrusted script
- remote authoritative host
- remote non-authoritative peer
- importer
- automation agent
- collaboration participant

Capabilities should govern:
- which realities may be observed
- which transformations may be requested
- which realities may be ratified
- which migration paths may be initiated
- which propagation channels may be consumed

---

# 17. The nine platform layers

The nine platform layers remain the stable structural map:

1. Runtime Simulation
2. Mutation / Ratification
3. Retention / Recovery
4. Observation
5. Authority / Partition
6. Asset / Content
7. Expression
8. Sharing / Replication
9. Editor / Tooling

These layers are still useful top-level ownership boundaries.

But they are no longer the primary doctrine.

The primary doctrine is the world-reality model and governing laws defined above.

---

# 18. Rust constitution

Runenwerk should be implemented in a way that makes illegal architecture hard to express.

## 18.1 Boundary types
Reality classes, scopes, versions, ids, and capabilities should be distinct types, not loosely interchangeable primitives.

## 18.2 Typestate where lifecycle matters
Use typestate where an object moves through strong lifecycle phases.

Examples:
- authored -> normalized -> formed
- requested -> validated -> ratifiable -> ratified
- draft -> active preview -> committed/cancelled
- received -> verified -> accepted share payload

## 18.3 Enums for closed policy spaces
Use enums for:
- retention strategy
- reconciliation policy
- stability class
- ratification class
- migration failure class
- exposure policy

## 18.4 Traits only where extension is truly open
Good candidates:
- observation frames
- expression frames
- import/export adapters
- propagation backends
- storage/retention adapters

Avoid using traits merely to model finite policy spaces that should be closed enums.

## 18.5 Capability tokens over ambient authority
Do not rely on broad ambient mutability where a scoped capability can express intent more safely.

## 18.6 Structured error domains
Long-term core contracts should use structured error types rather than stringly failures.

## 18.7 Stable opaque handles
Use stable handle/id types for architecture-visible identities instead of leaking internal collection choices.

---

# 19. Current codebase alignment

The current codebase already contains early evidence for this doctrine.

It already separates, at minimum:
- document-like state
- executable world state
- session/tool state
- observed UI models
- command/history behavior

This is visible in the current editor runtime, where document state, ECS world state, session state, tool state, and observed panel models are already distinct concerns even though they are still aggregated too broadly in one runtime façade during the current stage of development.

That means the multi-reality doctrine is not speculative; it is a formalization and strengthening of a direction that is already visible in the codebase.

---

# 20. Explicit non-goals

Runenwerk v6 does **not** require:

- a single universal world model
- full journal retention for every domain
- replay-stable behavior for every subsystem
- mergeable collaboration for every reality
- direct runtime equivalence with authored structures
- one universal propagation mechanism for all consumers
- one universal ratification policy across all domains

---

# 21. What this architecture is strongest at

This architecture is strongest where a platform must simultaneously support:
- editor and runtime coexistence
- large-world formation and streaming
- derived consumer forms for many consumers
- selective authority and sharing rules
- reversible authoring where appropriate
- multiplayer and remote review/collaboration
- durable recovery without making every domain event-sourced

Its strength is not that it invents entirely new mechanics.

Its strength is that it places known mechanics under a stronger and more native governing doctrine:

- realities instead of one master truth
- formation instead of implicit executable equivalence
- ratification instead of universal local mutation
- observation frames instead of raw authority exposure
- propagation structures instead of ad hoc dissemination
- reconciliation policy instead of assumed mergeability
- migration paths instead of accidental cross-scope side effects
- Rust-enforced contracts instead of prose-only boundaries

---

# 22. Better inspiration than quantum

Quantum inspiration is useful only in a limited structural sense:
- multiple governed descriptions
- observation depending on frame
- transformations between descriptions

But it is not the best primary inspiration.

A better primary inspiration for Runenwerk is a synthesis of:

## 22.1 Compiler architecture
Because formation, normalization, lowering, target products, and explicit intermediate representations are a closer match for authored -> formed -> instantiated flows.

## 22.2 Distributed systems
Because ratification, migration, propagation, scope, authority, and recovery are essential for multiplayer, collaboration, and partitioned worlds.

## 22.3 Control systems / systems engineering
Because stability classes, feedback, observation, and bounded operational guarantees matter more than metaphorical quantum language.

## 22.4 ECS/editor runtime practice
Because authored/runtime separation, replaceable consumer forms, and executable-world concerns are concrete and proven in engine architecture.

## 22.5 Physics-style phase/state thinking
Only at a restrained conceptual level: one world may legitimately exist in different operational states with different constraints.

### Final recommendation on inspiration
Do **not** use quantum as the primary inspiration.

Use this instead:

**compiler architecture + distributed systems + systems engineering + engine/editor runtime design**

That combination is cleaner, more modern, more credible, and more useful for implementation.

---

# 23. Final position

Runenwerk Architecture v6 is the recommended long-term governing doctrine for the platform.

Runenwerk should be understood as a **multi-reality world platform**.

Its center of gravity is not:
- the ECS alone
- the database alone
- the renderer alone
- the editor shell alone
- the network stack alone
- one imported enterprise pattern

Its center of gravity is:
- the realities a world may occupy
- the laws governing movement between them
- the contracts consumers use to observe or receive them
- the stability, retention, reconciliation, and migration guarantees attached to them
- the Rust boundaries that make those rules enforceable

The nine layers remain the structural map.

The world-reality doctrine is the governing architecture.
