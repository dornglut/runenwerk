---
title: Runenwerk Platform Architecture
description: Canonical top-down architecture spine for the Runenwerk integration platform, semantic federation, specialized execution, Workbench inspection, and batteries-included application composition.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-08-12
related_adrs:
  - ../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../adr/accepted/0019-batteries-included-application-composition.md
related_docs:
  - ./repository-family-architecture.md
  - ../guidelines/authority-centered-boundary-architecture.md
  - ../guidelines/domain-program-architecture-pattern.md
  - ../reports/investigations/2026-08-12-semantic-federation-and-inspection-provenance.md
  - ../reports/investigations/2026-08-12-application-composition-and-networking-ergonomics.md
---

# Runenwerk Platform Architecture

## Purpose

This is the canonical top-down architecture spine for Runenwerk.

Use it to answer, in order:

```text
What is Runenwerk?
Who owns meaning?
How do independent systems interact?
How can implementations stay fast and specialized?
How does the Workbench inspect the whole system coherently?
How does an ordinary application remain simple to build?
Which patterns are optional rather than universal?
```

This page summarizes accepted architecture. It does not replace the owning ADRs or
subsystem designs.

## North star

Runenwerk is an **integration, application, and Workbench platform for independently
owned Runen systems**.

Its long-term strength is the combination of:

- independently useful typed frameworks and domain owners;
- explicit semantic ownership and cross-owner contracts;
- freedom to use specialized CPU, GPU, spatial, field, network, persisted, or other
  physical representations;
- specialized execution rather than one universal runtime substrate;
- source lineage, diagnostics, intermediate inspection, and domain-specific tools;
- a federated Workbench that can make heterogeneous systems understandable without
  becoming their authority;
- a batteries-included application experience that composes the internal architecture
  without exposing routine integration ceremony to every product author.

The compact form is:

```text
Independent owners.
One semantic grammar.
Explicit typed integration.
Owner-local versions and consumer-owned admission.
Many physical realizations.
Specialized execution.
One inspectable Workbench.
One batteries-included App experience.
```

The architecture is deliberately **ambitious about product capability** and
**conservative about universal machinery**.

## One picture

```text
                         PRODUCT / APPLICATION

                     one App composition root
                              |
                transparent product/plugin groups
                              |
                 product- and domain-specific intent
                              |
                              v
                     RUNENWERK INTEGRATION

             adapters / lifecycle / product policy / hosts
                 /          |          |          \
                /           |          |           \
               v            v          v            v
          Runen owner   Runen owner  Runen owner  Runen owner
          / framework   / framework  / framework  / framework
               |            |          |            |
               +------------+----------+------------+
                            |
                     typed owner contracts
                            |
              explicit compatibility / admission
                            |
                            v
                    SPECIALIZED REALIZATION

          CPU / ECS / fields / spatial indices / GPU /
          persistence / networking / external services / ...
                            |
                            v
                  specialized work and effects


                       OPTIONAL WORKBENCH

          owner-selected read-oriented inspection projections
               tables / trees / graphs / timelines /
               fields / images / resources / reports
                            |
                   lineage and correlation views

          inspection never becomes foreign semantic authority
```

The arrows do not define one universal pipeline. They show the durable ownership and
integration direction.

# 1. Repository family and dependency direction

Runenwerk is not a universal core that peer frameworks must depend on.

ADR 0014 establishes the repository-family rule:

> Independently useful frameworks own their reusable semantics. Runenwerk owns
> application lifecycle, cross-framework integration, product policy, and adapters.

The family direction is intentionally one-way:

```text
peer frameworks / domain owners
        |
        v
Runenwerk adapters and integration
        |
        v
applications / tools / product hosts
```

A framework may depend on another framework only when a separate accepted decision
proves that dependency independently useful. ADR 0015, for example, accepts:

```text
RunenRender -> RunenGPU
```

because semantic image formation requires GPU execution while RunenGPU remains useful
for non-render workloads.

### Framework independence

A peer framework must not depend on Runenwerk merely to:

- obtain a global identity;
- participate in Workbench inspection;
- use a universal meta runtime;
- access a shared event bus;
- register into a universal database;
- satisfy application integration convenience.

Runenwerk adapters translate identities, prepared inputs, outputs, lifecycle facts,
diagnostics, and policy where cross-framework composition is required.

### No hidden RunenCore

Do not recreate a central dependency through another name such as:

```text
runen-core
runen-meta
runen-db
runen-flow
foundation/meta
universal registry package
universal graph package
```

Shared implementation is extracted only after the accepted repeated-proof gate is
satisfied.

# 2. Semantic ownership

The family-wide ownership law from ADR 0017 is:

```text
One semantic invariant set has one authority.
```

This is more precise than saying that every concept has one owner.

Legitimate layered values can coexist when they own different invariants:

```text
world-authoritative pose
renderer-local transform
GPU-packed transform
```

These values may correspond without sharing identity or authority.

### Authority is semantic, not physical

A value does not become authority merely because it is:

- stored in a database;
- retained for a long time;
- expensive to rebuild;
- resident on the GPU;
- persisted to disk;
- cached;
- indexed;
- consumed by a critical runtime path.

Authority means that the owner defines and enforces the semantic invariant.

# 3. Shared semantic grammar

ADR 0018 gives Runenwerk one common way to **reason** across heterogeneous owners
without forcing them onto one implementation substrate.

Use six recurring architecture questions:

| Question | Meaning |
| --- | --- |
| **Authority** | Who owns the semantic invariant and decides validity? |
| **Contract** | What explicit typed semantic boundary is exposed or accepted? |
| **Operation / Relationship** | How do contracts, values, or authorities relate or transition? |
| **Validity / Provenance** | Under what owner-local context is this meaningful, compatible, and traceable? |
| **Realization** | How is the semantic meaning represented, retained, located, or accelerated physically? |
| **Effect** | Where does authoritative or external state actually change? |

These are architecture questions, **not mandatory wrapper types**.

Do not create generic runtime objects merely to mirror the vocabulary:

```text
MetaAuthority
MetaContract<T>
MetaOperation
MetaNode
RunenObject
UniversalData
```

Domain terminology remains primary inside each owner.

## Useful relationship vocabulary

At cross-domain architecture level, terms such as these can clarify what a boundary
does:

```text
Observe
Propose
Derive
Adapt
Admit
Realize
Execute
Commit
```

The vocabulary is open and explanatory. It is not a closed enum or required pipeline.

Existing owner-specific roles remain meaningful, including:

```text
Command
Query
Event
Projection
Snapshot
Product
Contribution
Descriptor
Catalog
Plan
Work
Status
Diagnostic / Report
```

# 4. Versions and cross-authority consistency

There is no global Runenwerk transaction, universal world revision, universal frame
snapshot, or universal clock that makes every framework value mutually current.

The compact law is:

```text
Versions are owner-local.
Compatibility is contextual.
Admission is consumer-owned.
```

A consumer that combines several authorities admits the exact compatibility facts its
operation needs, which may include:

```text
revision or generation
time / tick / interval
scope and coverage
completeness
freshness or bounded staleness
availability or residency
capability / accuracy
source lineage
identity correspondence
legal fallback
```

Different consumers may legally require different compatible cuts.

For example:

```text
renderer invocation
  world/source revision
  renderer scene revision
  simulation interval
  representation generation
  GPU device generation

network publication
  world/ECS revision context
  interest scope
  simulation tick/history availability
  connection/session state
```

These need not be one transaction.

### Inspection is weaker than admission

A Workbench may correlate:

```text
WorldEntity
  -> RenderObject
      -> GpuResource
```

for debugging and lineage navigation.

That does **not** prove the values currently displayed form a legal runtime input set.

```text
inspection correlation / join
!=
semantic runtime admission
```

The actual consumer or integration boundary remains responsible for compatibility.

# 5. Logical meaning and physical realization

The family-level physical-realization law is:

> **A semantic contract is not defined by its physical realization. A realization may
> constrain capability, precision, validity, locality, residency, lifetime, or cost,
> but it must not silently redefine the semantic contract it claims to realize.**

This is what preserves both correctness **and** performance freedom.

The same semantic role may be realized through structures such as:

```text
ECS archetype columns
sparse sets
persistent tables
copy-on-write pages
BVHs and spatial indices
analytic fields
sampled grids
CPU vectors / SoA / SIMD batches
GPU buffers and textures
compiled programs
persisted artifacts
network payloads
remote services
```

No one physical layout becomes the platform ontology.

### Realization mismatch is explicit

If an implementation cannot preserve a claimed semantic property, it must use an
owner-defined outcome such as:

```text
weaker or different capability
declared approximation / tolerance
explicit semantics-preserving adaptation
rejection
```

It must not silently continue under a stronger label.

This matters especially for numerical fields, representation error, GPU capabilities,
partial residency, lossy transport, and precision modes.

# 6. Views, products, retention, and acceleration

Database-derived vocabulary is useful when the concepts remain distinct.

| Concept | Architectural question |
| --- | --- |
| **Projection / View** | How can derived information be read for this purpose? |
| **Snapshot** | Which immutable owner-local state/revision is being observed? |
| **Product** | What semantic result has an owner intentionally published? |
| **Materialization** | Is derived information retained rather than recomputed? |
| **Index** | What retained structure accelerates access? |
| **Cache** | What discardable/reconstructable state avoids repeated work? |
| **Realization** | What physical representation satisfies the semantic contract here? |

These are orthogonal.

One result may legitimately be:

```text
renderer-owned Product
+ immutable Snapshot
+ materialized
+ indexed
+ GPU-resident realization
```

Retention or acceleration never grants source authority by itself.

### Change evidence

Incremental systems remain owner-specific, but ADR 0017 requires the correctness law:

> For the same admitted semantic inputs, incremental evaluation is observationally
> equivalent to clean/full evaluation under the owner's declared equality/tolerance.

Therefore missing or untrusted narrow change evidence broadens invalidation, causes a
clean rebuild, or triggers full resynchronization as required.

A cache hit changes cost, not semantic meaning.

This law does not imply one incremental database or dependency runtime.

# 7. Specialized execution

Runenwerk deliberately does **not** use one execution ontology.

Different owners may legitimately need:

```text
direct functions
ECS systems
parallel CPU scheduling
async IO
stream processing
GPU compute / graphics work
incremental evaluation
fixed-point iteration
network sessions
reconciliation controllers
external processes
```

The family shares architecture laws where they are truly common, not one executor.

## Graphs remain semantically distinct

Graph shape alone does not justify a universal graph runtime.

The governing law is:

```text
semantic dependency
!= execution ordering
!= resource hazard
!= invalidation dependency
!= containment / hierarchy
```

Examples of graph-shaped structures with different owners include:

```text
source / structural graphs
semantic program graphs
product dependency graphs
incremental dependency graphs
scheduler readiness graphs
render planning graphs
RunenGPU work/access graphs
retained UI/spatial topology
```

A Workbench may display all of them as graphs while preserving each graph's edge
semantics.

## Feedback is explicit

Feedback is not hidden as a generic back-edge.

Owners distinguish patterns such as:

```text
temporal feedback
bounded / fixed-point iteration
desired-observed reconciliation
interaction loops
distributed prediction / correction
```

Use reconciliation only when desired and observed state can actually evolve
independently.

# 8. Shared extraction

Shared implementation is earned by repeated neutral proof.

The accepted sequence is:

```text
design locally
-> prove one real domain
-> prove a structurally different second domain
-> identify concrete repeated implementation/maintenance burden
-> characterize dependency, runtime, memory, serialization and cognitive cost
-> accept a separate extraction decision
-> extract only the repeated neutral primitive
```

A proposed shared contract must remain meaningful if either proving domain disappears
and must contain no proving-domain semantic branches.

This gate distinguishes three useful levels:

```text
Level A
  shared conceptual law / vocabulary
  no runtime dependency

Level B
  shared interoperability / reflection contract
  real API/version/lifetime/pressure cost

Level C
  shared runtime mechanism
  query engine / dataflow / shared store / optimizer / executor / ...
```

ADR 0018 authorizes the semantic-federation vocabulary at Level A. It does not
pre-authorize a Level-B Workbench protocol or a Level-C meta runtime.

# 9. Workbench architecture

Runenwerk's long-term product is not only a game runtime.

It should support a coherent environment for:

- games and runtime products;
- focused authoring tools;
- editor/workbench applications;
- procedural and field workflows;
- rendering and GPU diagnostics;
- simulation inspection;
- asset/content formation;
- networking inspection;
- source lineage and diagnostics;
- headless validation and automation;
- future domain-specific authoring experiences.

The Workbench achieves coherence through **federation**, not by taking ownership away
from its domains.

## Federated inspection

The conceptual direction is:

```text
peer frameworks / domain owners
        |
        v
explicit Runenwerk integration / inspection adapters
        |
        v
optional Workbench inspection session
```

Owners may expose zero or more read-oriented inspection projections, directionally:

```text
Record
Table
Tree
Graph
Timeline
Field
Image
Resource
Text / Report
Opaque / Custom
```

These are tooling views, not semantic storage classes.

One value may support multiple projections. A render scene can be inspected as object
tables, relationship graphs, change timelines, and image/spatial views without becoming
one universal table or graph internally.

## Workbench authority limits

The Workbench must not become:

```text
global semantic ID authority
global revision / clock
domain mutation authority
runtime admission authority
universal payload store
universal serialization format
generic query executor over private owner state
```

Inspection-session handles remain inspection-local. Owner identities stay owner-local.
Cross-owner correspondence is explicit lineage/provenance.

## Inspection cost

The target when inspection is disabled is approximately:

```text
no mandatory payload copy
no mandatory global graph construction
no mandatory per-frame registration walk
no mandatory CPU readback of GPU-resident payloads
no eager field / huge-table materialization
no unbounded history retention
```

When enabled, inspection should be able to remain lazy, paged/ranged, bounded,
pressure-aware, copy-aware, and location/device-aware.

Any concrete peer-neutral inspection protocol requires a separate extraction decision.

# 10. Product and application experience

The internal architecture is intentionally decomposed.

The ordinary application experience should not be.

ADR 0019 establishes:

> **Internal decomposition must not determine application complexity.**

Runenwerk already uses `App` as its runtime composition root. That remains the one
application runtime.

The product-facing target is:

```text
ordinary path
  supported batteries-included product/plugin group(s)
  + product/domain declarations

configuration path
  inspect/configure/replace selected group members
  + owner-specific configuration

expert path
  direct owner/framework plugins
  custom render flows
  custom networking drivers/transports
  specialized adapters and lower-level contracts
```

The three paths use the same underlying owners. Convenience must not create a second
runtime or mirrored configuration authority.

## Product/plugin groups

A future product/plugin group is an ordered, inspectable composition recipe over
existing plugins/configuration.

It is **not** semantic authority.

The final API and memberships are not yet accepted or implemented. A later concrete
design must prove the smallest useful group mechanism and supported product shapes.

The target qualities include:

```text
named/debuggable membership
deterministic order
composable groups where meaningful
member configuration / replacement
removal / disable where legal
explicit duplicate / incompatibility diagnostics
inspection of the effective selected stack
```

Do not create a preset for every capability cross-product.

## Product intent stays visible

The convenience layer removes repeated generic Runen integration wiring. It does not
hide actual game/tool semantics.

Product/domain-owned choices remain visible, such as:

```text
game rules and systems
product-specific render methods/flows
input actions and bindings
world/procedural policy
editor/workbench behavior
specialist device adapters
network protocol/components/input/ownership intent
custom compression or streaming policy
```

Runenwerk owns common integration wiring only when that wiring is genuinely reusable
across supported products.

## Current implementation honesty

The target is not fully implemented.

Current `App` construction still installs broader builtin state than the ideal
product-capability ownership rule, and the `engine` package currently has a broader
compile-time dependency surface than runtime product selection implies.

Therefore:

```text
runtime composition simplicity
!=
compile-time / binary-size modularity
```

Do not claim product groups, App builtin cleanup, or feature-level dependency
subtraction until separately implemented and proven.

# 11. Networking position

Runenwerk's networking architecture is custom engine technology, not merely a transport
wrapper.

Directionally:

```text
engine_net
  protocol / session / replication semantics

engine_net_quic
  Quinn-based QUIC realization

engine integration
  scheduling / authority / simulation / history / diagnostics

application/gameplay
  protocol declarations and game-specific policy
```

Runen-owned networking semantics include authoritative replication, snapshots/deltas,
ACK/baseline/resynchronization, prediction/correction, interest/streaming,
simulation/history/replay integration, diagnostics, and typed/declarative game-network
authoring.

Contained lower layers may implement QUIC, TLS/crypto primitives, sockets, or OS network
mechanics without owning Runen replication semantics.

## Ordinary networking path

The target ordinary gameplay path is registration-driven:

```text
register replicated entities/components
register input streams and ownership routing
write ordinary authoritative ECS/game systems
Runen handles standard extraction/snapshot/delta/apply/ACK/replay plumbing
```

At the current accepted state this target is incomplete; ordinary gameplay may still
need a custom replication driver. The existing networking design/roadmap owns that
implementation work.

Custom drivers remain an expert path for aggregate snapshots, custom compression,
external non-ECS state, large world-streaming representations, rollback-specific
packing, and other genuinely specialized formats.

## Managed backend services

Authentication, cloud persistence, object storage, lobbies/presence, hosted functions,
payments, and similar services are a separate product concern from authoritative game
replication.

Runenwerk does not currently define a universal managed-service abstraction. Provider
integrations or a later neutral abstraction require concrete product/security/
persistence/operational proof.

# 12. Domain Program is a specialized pattern

A Domain Program is useful when a domain genuinely needs durable authored semantic
intent with several concerns such as:

```text
persistence
versioning
inspection
migration
fixtures / conformance
multiple hosts
compiled hot-path artifacts
source maps
reproducibility
```

The useful lifecycle is often:

```text
authoring source / model
-> normalized domain model
-> Domain Program
-> domain-owned graph(s) if useful
-> compiler and/or evaluator
-> runtime artifact / product / output facts
-> host integration
```

But this is **not the universal root of Runenwerk**.

These need not become Domain Programs merely to participate:

```text
GPU resources
mounted UI runtime state
input events
spatial availability
render scene snapshots
network streams
scheduler readiness state
query results
```

Use the canonical [Domain Program Architecture Pattern](../guidelines/domain-program-architecture-pattern.md)
for the full specialized guidance.

# 13. Other specialized patterns remain local

The platform intentionally allows strong local techniques without making them universal.

Examples:

### Relational / database techniques

Useful for scene stores, ECS-like relationships, inspection tables, views,
materialization, indexing, and selected incremental systems.

Not every field, hierarchy, runtime controller, or GPU workload is a relation.

### Dataflow / incremental systems

Useful where changing collections, dependency tracking, logical time, or maintained
results justify the runtime machinery.

Not every operation pays for one dataflow runtime.

### Compiler / lowering architectures

Useful for material, shader, procedural, asset, or other domains with authored semantic
programs and target-specific realizations.

Not every runtime authority is a compiler IR.

### Desired / observed reconciliation

Useful where actual state changes independently from desired state, such as selected
streaming/controllers.

Pure deterministic derivation is not renamed reconciliation.

### Event/history models

Useful where accepted history is itself required for replay, audit, recovery, or
network correctness.

Transient high-frequency runtime state does not become event-sourced by default.

# 14. Rust and API doctrine

Runenwerk should use Rust to make meaningful owner invariants difficult to violate,
without encoding speculative universal ontology into the type system.

Prefer:

```text
explicit owner-local newtypes where identity semantics matter
typed boundary contracts
structured diagnostics/errors
deterministic normalization and planning
immutable prepared/snapshot products where appropriate
explicit capability and requirement types in their owning domains
typestate where a real lifecycle benefits from it
ordinary high-level APIs for common paths
```

Avoid:

```text
one family-wide ObjectId
one family-wide Revision
universal Reality enum
universal Capability enum
MetaNode / MetaOperation execution hierarchy
generic graph edge with unspecified meaning
ambient untyped service lookup
hand-authored metadata that duplicates canonical typed contracts
```

# 15. What Runenwerk deliberately does not universalize

The current architecture does not authorize one shared implementation for:

```text
object identity
revision / transaction / global snapshot
storage engine
ECS
query language / query engine
graph runtime
dataflow runtime
incremental database
logical clock
scheduler / executor
event bus
serialization / wire format
memory layout
GPU representation
compiler / evaluator framework
reconciliation runtime
managed backend services
Workbench reflection ABI
```

Any of these may exist locally where an owner needs it. Shared extraction remains
possible after real repeated proof.

# 16. Current implementation versus long-term target

This spine contains both accepted durable laws and explicitly identified future target
behavior.

### Accepted and current architectural authority

- repository-family ownership and one-way integration direction;
- one semantic invariant set per authority;
- explicit foreign-owner contracts;
- owner-local versions and consumer-owned compatibility/admission;
- graph-semantic separation;
- incremental/full correctness law;
- semantic/physical realization separation;
- semantic-federation reasoning grammar;
- no pre-authorized shared meta runtime;
- `App` as the one runtime composition root;
- product groups as the accepted future composition concept, not yet an implemented
  final API;
- custom Runen networking semantic ownership with contained transport realization.

### Future implementation still separately gated

- concrete product/plugin-group API and memberships;
- moving current `App` builtin resources to owner plugins/groups;
- Cargo feature/dependency topology for smaller builds;
- peer-neutral Workbench inspection/reflection protocol;
- generic Workbench query capabilities;
- low-boilerplate standard ECS networking completion;
- managed backend/provider integrations;
- any new shared runtime mechanism.

Architecture documentation must not describe these future items as already implemented.

# 17. Fitness functions

The architecture remains healthy when:

1. every semantic invariant set has an identifiable owner;
2. peer frameworks remain independently useful;
3. cross-owner reads use explicit contracts rather than private mutable reach-through;
4. multi-owner consumers define sufficient compatibility/admission facts;
5. semantic contracts can change physical realization without silently changing meaning;
6. graph species retain their real edge semantics;
7. incremental systems retain a clean/full or full-resynchronization safety reference;
8. Workbench inspection does not become domain authority or mandatory hot-path
   representation;
9. shared infrastructure is extracted only after structurally different real proofs;
10. routine application paths expose product/domain concepts instead of internal
    integration ceremony;
11. defaults remain inspectable and lower to the same owner plugins used by expert
    paths;
12. custom Runen systems remain meaningfully custom even when they use contained
    lower-level libraries or services;
13. documentation distinguishes accepted architecture, current implementation, future
    target, and historical evidence.

# 18. Cold-start reading path

For architecture work, read in this order:

1. **This page** — current Runenwerk-wide architecture spine.
2. [ADR 0014: Repository Family Extraction Boundaries](../adr/accepted/0014-repository-family-extraction-boundaries.md)
   — repository ownership and dependency direction.
3. [ADR 0017: Cross-Authority Consistency and Graph Semantics](../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md)
   — authority, consistency, graph, incremental, capability, extraction, and
   progressive-disclosure safety laws.
4. [ADR 0018: Semantic Federation and Physical Realization](../adr/accepted/0018-semantic-federation-and-physical-realization.md)
   — positive cross-domain reasoning model, realization separation, and Workbench
   federation direction.
5. [ADR 0019: Batteries-Included Application Composition](../adr/accepted/0019-batteries-included-application-composition.md)
   — product-facing App and usability doctrine.
6. [Repository Family Architecture](./repository-family-architecture.md) and the owning
   subsystem/framework design for the work at hand.
7. [Authority-Centered Boundary Architecture](../guidelines/authority-centered-boundary-architecture.md)
   and [Domain Program Architecture Pattern](../guidelines/domain-program-architecture-pattern.md)
   only when those specialized guidelines apply.

Investigation reports explain the research ancestry behind ADR 0018/0019. They support
decisions; they do not override accepted ADRs or current owner architecture.

## Final position

Runenwerk is not one database, ECS, graph, compiler, meta-framework, editor shell, or
renderer.

It is a custom integration and Workbench platform whose owners can remain specialized
while participating in one coherent product:

```text
owners keep meaning
contracts make boundaries explicit
admission makes multi-owner use deliberate
realizations stay free to optimize
execution stays specialized
inspection makes the whole system understandable
App composition makes the ordinary product path simple
```

That combination—not one universal substrate—is the current Runenwerk North Star.
