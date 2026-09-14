---
title: Runen Federated Semantic Composition and Plan Interface Design
description: Active detailed design for optional typed cross-owner composition, owner-qualified dialect interfaces, deterministic partition/lowering, federation catalog metadata, and explainable Workbench integration.
status: active
owner: workspace
layer: cross-domain
canonical: true
last_reviewed: 2026-09-14
related_adrs:
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../adr/accepted/0020-adopt-federated-semantic-composition.md
  - ../../adr/accepted/0021-ratify-runenrender-semantic-rendering-architecture.md
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ../../adr/accepted/0024-normalize-physical-input-observation-semantics.md
related_designs:
  - ./semantic-graph-ir-and-compilation-design.md
  - ./gameplay-graph-atr-ir-and-ecs-lowering-design.md
  - ./typed-app-program-counter-proof-design.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../guidelines/domain-program-architecture-pattern.md
---

# Runen Federated Semantic Composition and Plan Interface Design

## Status and role

Active detailed design under ADR 0020.

ADR 0020 accepts the **architecture boundary** for optional typed federated composition.
This document elaborates the target model, safety invariants, interface pressure, proving
strategy, and future implementation constraints.

It does not claim a shared Plan IR, catalog, dialect runtime, planner, provider registry,
or persistence format exists in current Rust.

Current owner APIs, code, tests, accepted owner ADRs, and owner-normalized semantic models
remain authoritative for current behavior.

## Purpose

Runenwerk repeatedly composes independently owned semantic systems:

```text
application/product composition
world / ECS integration
renderer inputs and output requests
UI / render integration
input / product / UI adaptation
network / ECS / simulation integration
field / world / render integration
Workbench inspection and correlation
```

The platform needs a coherent way to describe and inspect some of these **cross-owner
compositions** without creating a universal semantic super-domain.

The target is:

```text
owner-native meaning and APIs
        |
        | explicit participation
        v
optional typed composition artifact
        |
        | owner/dialect interfaces
        v
deterministic owner-native partition / planning / lowering
        |
        v
specialized execution and owner state
```

The target is **not**:

```text
everything is Plan
everything is a query
everything is a graph
everything is a database row
everything is a shared runtime operation
```

## Core doctrine

The architecture is governed by four laws:

```text
owner normalization precedes federation
composition does not transfer semantic authority
shared transformation requires explicit semantic interfaces
direct native APIs remain first-class
```

A composition representation is useful only where it makes a real multi-owner boundary
more verifiable, reusable, inspectable, or explainable than direct wiring.

## Semantic layers

Keep four layers distinct.

### Layer 1 — owner semantic model

Each owner defines its own invariants and normalized semantic dimensions.

Examples:

```text
RunenRender
  scene lineage, request semantics, representations, RenderPlan,
  semantic binding admission, execution admission, RenderResult

Runenwerk App/runtime
  composition, containment, Host, Advancement Policy,
  lifecycle occurrence, owner capability state

Device-level input
  observation, grouping, ordering, evidence, delivery,
  reconciliation, identities, state reduction, quantities

RunenGPU
  resource/program/work/access/execution semantics

RunenNet
  sessions, delivery, replication consistency/recovery,
  participant-input prediction/reconciliation
```

These semantic models are not projections of a shared Plan schema. They are the primary
owner contracts.

### Layer 2 — explicit cross-owner composition

When one operation or product genuinely combines several owner contracts, Runenwerk may
represent that integration as an optional typed composition artifact.

This layer owns only the **composition relationship** and Runenwerk integration semantics
that actually belong between owners.

### Layer 3 — owner-native planning and lowering

The composition layer delegates owner-specific semantic planning, verification,
admission, and lowering to the owner or to an explicitly accepted Runenwerk adapter.

It does not copy those algorithms into a generic planner.

### Layer 4 — physical realization and execution

CPU work, ECS scheduling, GPU execution, network sessions, host event loops, external
services, caches, retained products, and other physical/runtime mechanisms keep their
specialized owners.

## When a shared Plan is justified

A shared `Plan` is justified only when the composition itself is a durable useful
artifact.

Good pressure includes:

- one composition spans multiple owner contracts;
- validation needs to reason about the cross-owner boundary before execution;
- source lineage or explanation across the boundary is materially useful;
- deterministic partition/lowering reduces repeated integration glue;
- tooling must inspect the composition without reaching through owner internals;
- several products would otherwise reproduce the same Runenwerk-owned integration.

A shared Plan is **not** justified merely because:

- an owner already has an internal plan or graph;
- a value can be drawn as a node;
- an API call has inputs and outputs;
- a subsystem has a lifecycle;
- a query-like analogy exists;
- Workbench wants to display the value.

## Plan is not owner state

A shared Plan is an inert semantic composition description until an owning integration
boundary verifies/adopts/lowers it.

It does not become:

- live owner state;
- a mutable service registry;
- a global transaction;
- a universal snapshot;
- a runtime scheduler;
- a retained graph interpreter;
- a second application runtime.

If a product composition Plan lowers into `App`, the resulting `App` and participating
owners become live runtime authority according to ADR 0019/0023. The Plan may remain as
read-only provenance/explanation if useful, but not as parallel writable truth.

## Conceptual Plan structure

Do not freeze Rust spelling before implementation proof. The semantic pressure is
approximately:

```text
CompositionPlan
  identity/version only if the owner/use case requires it
  operations/regions
  explicit relationships/dependencies
  contract references
  requirements
  effect summaries
  provenance/source maps
  diagnostics/explanation facts
```

### Owner-qualified operation reference

Every operation participating in the shared layer identifies the semantic owner/dialect
that defines its meaning.

Directionally:

```text
OwnerOperationRef
  owner / dialect identity
  operation identity
  semantic version or compatibility identity where required
```

The shared core does not interpret owner-qualified operation payloads beyond explicit
interfaces.

### Contract/value reference

Operations may refer to typed owner contracts or produced values.

The shared layer needs enough information to avoid accidental type/contract mismatch,
but it does not require a universal structural type system.

Compatibility may be established by:

- exact owner-qualified contract identity;
- an owner-defined compatibility relation;
- an explicit adapter/conversion contract.

No generic structural duck-typing is implied.

### Relationships and dependencies

The shared representation may carry explicit relationships needed by the composition.
Their meaning must remain declared.

Do not collapse:

```text
semantic dependency
execution ordering
resource hazard
provenance correspondence
containment
lifecycle dependency
invalidation dependency
```

into one untyped edge.

A relation may be generic only when its invariant is actually shared; otherwise it is
owner/dialect-qualified.

### Regions and partitions

A region identifies a composition boundary that may be verified/planned/lowered by one
owner or provider contract.

Region structure exists to preserve delegation, not to create a universal execution
region model.

Nested owner-native plans may remain opaque from the shared layer except for interfaces
the owner intentionally exposes.

## Core validation

The shared layer may perform validation that is genuinely structural or explicitly
interface-backed.

Directionally valid checks include:

- referenced owner/dialect/operation identity exists in the selected composition
  context;
- required contract references resolve;
- cross-owner dependencies are explicit;
- an effect target/owner is identified where generic transformation safety needs it;
- required adapter/conversion relationships are declared;
- provenance/source-map references are structurally valid;
- partition/region ownership is explicit;
- an owner/interface verifier accepted owner-specific payload/semantics.

The shared core must not invent owner-specific validity rules.

## Dialect and owner interfaces

The implementation should prefer small opt-in interfaces over one broad mandatory
`Dialect` god-trait.

Conceptual interface capabilities include:

```text
verify operation / region
resolve contract compatibility
summarize requirements
summarize effects relevant to cross-owner transformation
canonicalize owner-local form
prove equivalence / reorder / duplication / elimination safety
partition or select owner/provider handling
lower to owner-native request/plan/product/work
explain owner-specific semantics
```

These are semantic capabilities, not required trait names.

An owner implements only the interfaces it can truthfully support.

## Unknown-operation law

Extensibility is safe only if unknown semantics fail closed.

A shared tool may preserve, route, display provenance for, or structurally contain an
unknown owner operation. It must not:

- rewrite it;
- reorder it across effects;
- fuse it;
- duplicate it;
- eliminate it;
- infer purity;
- infer idempotency;
- infer commutativity;
- lower it through another provider;

unless an explicit recognized interface proves the corresponding property.

This is the central anti-overreach rule for an extensible Plan.

## Rewrite and optimization law

There is no generic optimizer authority merely because a Plan exists.

### Owner-local rewrite

An owner/dialect may define semantics-preserving canonicalization or optimization inside
its own region.

### Interface-generic rewrite

A generic pass may transform operations from multiple dialects only through an
interface whose contract is sufficient to prove semantic equivalence and effect safety.

### Cross-owner rewrite

A transformation crossing owner boundaries requires an accepted integration contract
from the relevant owners or Runenwerk integration authority.

### Cost is secondary to legality

Cost information can choose among already legal alternatives. It does not make an
otherwise illegal transformation legal.

The correctness baseline is deterministic straightforward lowering with no cost-based
search.

## Federation catalog

A future `FederationCatalog`-like concept is optional read-oriented integration
metadata.

It may answer questions such as:

```text
which owner/dialect operation descriptors participate?
which contract identities and versions are known?
which explicit conversions/adapters exist?
which verifier/planner/lowering interfaces are available?
which providers can realize an accepted operation/region?
which provenance/explanation hooks are available?
```

It must derive from actual owner/integration registration and must not become a second
writable semantic model.

It does not own:

- owner runtime state;
- domain objects;
- ECS entities;
- renderer scene state;
- GPU resources;
- input held state;
- network sessions;
- application configuration;
- global revision or transaction semantics.

## No universal database surface

The platform may expose relational or query projections through concrete tooling or
dialects. Do not make `runen.db()` or equivalent a required root abstraction.

A relational dialect may be excellent for:

- tabular inspection;
- catalog querying;
- materialized analysis;
- data-processing products;
- source adapters whose semantics are genuinely relational.

It must not redefine:

- App lifecycle;
- device input observations;
- renderer request/planning semantics;
- GPU resource hazards;
- network session authority;
- UI interaction state;

as relational data merely for uniformity.

## Provider model

A provider is a realization/lowering participant, not semantic authority by default.

A provider advertises only what an explicit owner/integration contract authorizes it to
handle.

Directionally a provider may report:

```text
supported owner operation/interface families
required physical capabilities/traits
produced physical traits
supported representations/localities
known legal conversions
operational availability
cost/pressure facts where meaningful
```

The shared layer must keep separate:

```text
semantic support
semantic admission
operational availability
physical realization
current residency
```

Provider selection cannot silently weaken semantics.

## Deterministic federation baseline

The first complete shared planning behavior should be deliberately simple:

```text
1. validate structural composition
2. invoke owner/interface verification
3. derive explicit owner regions
4. choose the only or deterministic accepted realization for each region
5. insert only declared legal adapters/conversions
6. invoke owner-native planning/lowering
7. preserve source/provenance mappings
8. emit explanation/diagnostics
```

If multiple physical choices later create real value, a cost model may choose among
**semantically legal** alternatives. This is an optimization feature, not the semantic
foundation.

## Admission boundary

A composition layer may collect and route admission facts, but final semantic admission
belongs to the consuming owner unless explicitly delegated.

Applicable facts may include:

```text
owner revision/generation
time/tick/interval
scope/coverage
completeness
freshness
capability/accuracy
availability/residency
provenance/correspondence
policy/fallback
```

The shared layer must preserve which owner defines each fact and which consumer decides
sufficiency.

No global compatibility cut is introduced.

## Effect boundary

The shared layer needs enough effect information to avoid unsafe composition and
transformation.

Minimum generic pressure is:

```text
is an effect relevant to cross-owner planning present?
which authority or external environment can change?
what owner-qualified interface governs ordering/commit/retry semantics?
```

Do not force every owner into one closed effect enum.

Owner-defined effect interfaces may expose additional facts such as:

- read/write sets;
- ordering constraints;
- idempotency;
- retry legality;
- commit boundaries;
- compensation/recovery;

only where those concepts are valid.

## Lifetime and retained state

The former universal lifetime family:

```text
run / watch / changes / materialize / compile
```

is not part of the shared semantic core.

Those words hide distinct owner contracts:

```text
input stream admission and state reduction
RunenNet retention/recovery and replication resynchronization
renderer products/history/cache
query publication
App bounded advancement
GPU completion/progress
external jobs
```

A future shared lifetime interface may emerge after repeated proof. Until then, a
composition references the owner-specific lifetime contract it actually uses.

## Budgets and pressure

Do not make every composition carry a universal budget envelope.

Concrete owners/providers may expose:

- memory pressure;
- transfer size/cost;
- latency or throughput targets;
- bounded history;
- quality/accuracy/tolerance;
- device limits;
- locality/residency;

through explicit interfaces where those facts affect legal realization.

Generic tooling may display comparable facts. It must not infer common semantics from
shared numeric units alone.

## Failure and diagnostics

The shared layer should distinguish failure ownership before inventing one global error
enum.

At minimum diagnostics should preserve:

```text
composition/structural rejection
owner semantic rejection
consumer admission rejection
adapter/conversion rejection
provider unsupported/unavailable
physical/runtime failure
resource/pressure failure
external effect failure
```

Owner diagnostics should remain intact and source-mapped. Integration diagnostics add
cross-owner context rather than flattening owner codes into generic strings.

## Provenance and source maps

Composition loses value quickly if lowering becomes opaque.

Maintain explicit mappings where applicable:

```text
authored/product source
-> composition operation/relationship
-> owner-native request/plan/product
-> physical/executable work
-> result/diagnostic
```

Provenance is correspondence, not shared identity.

Runtime IDs must not silently become persisted Plan identity.

## Workbench integration

Workbench is a major consumer but not the semantic owner of federation.

Workbench may show:

- composition graph/regions;
- owner/dialect operation descriptors;
- cross-owner relationships;
- admission requirements and responsible owner;
- effects and effect owner;
- provenance/source maps;
- provider/lowering choices;
- owner diagnostics;
- physical/runtime facts exposed by providers;
- owner-native projections alongside Plan explanation.

Workbench must remain able to inspect an owner that has no Plan representation at all.

This prevents a debugging convenience from becoming mandatory runtime architecture.

## App/product composition example

A future product composition may use an inert Plan-like description:

```text
product intent
  -> select Scene capability
  -> select Render capability
  -> select host
  -> select product behavior
  -> validate explicit compatibility
  -> lower to ordinary App composition
```

After lowering:

```text
App
  owns Runenwerk live application/integration lifecycle
owner plugins/resources
  own their respective capability state
```

The Plan is not a second plugin registry or runtime configuration database.

## RunenRender example

Suppose an integration composes world/source data with a renderer invocation.

The shared layer may describe:

```text
source-owner published contract
-> explicit adapter/correspondence
-> RunenRender request-scoped semantic input
-> RunenRender request
```

Then delegation is:

```text
RunenRender
  verifies renderer meaning
  forms native RenderPlan
  performs semantic binding admission
  performs execution admission
  lowers admitted work to RunenGPU
```

The shared layer does not interpret representation validity, rendering accuracy,
method semantics, or renderer-specific admission.

## Device-input example

Normal device input does **not** need Plan participation.

The accepted path remains:

```text
backend report
-> normalized device observation
-> deterministic admission/reduction
-> product/UI/drawing/camera adapters
```

A higher-level composition tool might describe which adapter consumes which published
input contract. That does not turn observations, ordering, reconciliation, or held state
into generic Plan semantics.

This is an explicit negative proof that Plan participation is optional.

## Gameplay/semantic-graph example

A gameplay or another semantic-graph domain may retain an authored graph and domain IR
with its own semantics.

If its formed product spans ECS, networking, rendering, fields, or another owner, the
domain compiler may emit or invoke a shared composition artifact **only for the
cross-owner boundary**.

`SELECT / RELATE / TRANSFORM` therefore remain candidate semantic-graph/gameplay
vocabulary rather than universal core operations.

## Direct API and progressive disclosure

Ordinary owner APIs must not require users to understand federation internals.

Progressive disclosure is:

```text
ordinary path
  owner/product API

cross-owner composition path
  optional typed Plan/composition builder where it materially helps

inspection path
  Plan explanation + owner projections + provenance

expert path
  owner-native lower-level planning/realization APIs
```

The composition layer earns its existence by reducing cross-owner complexity, not by
moving complexity into every owner call.

## Persistence policy

No persisted Plan contract is accepted yet.

A later persistence decision must define:

- stable owner/dialect/operation identity;
- schema and semantic version compatibility;
- external/untrusted input validation;
- migration;
- unknown extension preservation/rejection;
- capability requirements;
- effect replay safety;
- source/provenance identity;
- security/trust boundaries.

Process-local composition should be considered the conservative first implementation
unless a real persisted consumer requires more.

## External-system interpretation

### MLIR

Carry forward:

- small shared IR mechanics;
- owner/dialect-defined semantics;
- explicit interfaces for generic analyses/transforms;
- explicit conversion legality;
- extensible dialect participation.

Do not carry forward by analogy:

- the assumption that every Runen owner is naturally one compiler IR;
- SSA/value semantics as the mandatory family ontology;
- compiler pass infrastructure as the universal runtime model.

### Apache Calcite

Carry forward:

- deterministic logical/physical separation;
- provider/adaptor boundaries;
- explainable pushdown/partitioning;
- semantics-preserving rewrite discipline;
- cost choosing among legal alternatives.

Do not generalize relational algebra into unrelated owner semantics. Calcite's power
comes from a real common relational substrate.

### Substrait

Carry forward:

- engine-independent logical contracts where a common algebra exists;
- typed extension/version discipline;
- explicit custom-operation interoperability limits.

The least-powerful-extension principle applies directly: keep the shared Runen core
smaller than owner dialects.

## Maturity model

### Level A — semantic grammar

Accepted and implemented in architecture documentation through ADR 0017/0018.

No shared runtime representation required.

### Level B — federated composition/interface protocol

Accepted as the architectural target by ADR 0020.

Still needs concrete implementation proof before Rust/public API stabilization.

### Level C — reusable shared machinery

Examples:

```text
shared IR package
shared planner framework
persistent federation catalog
cross-owner optimizer
shared executor/dataflow runtime
```

None are pre-authorized. Each requires repeated proof and a separate accepted design.

## First implementation proof requirements

Before creating a shared Plan implementation, the owning issue must provide a corpus with
at least two structurally different cross-owner compositions.

For each case record:

- semantic owners;
- direct/native API path;
- repeated Runenwerk integration currently required;
- exact composition facts proposed for sharing;
- owner-specific facts intentionally excluded;
- admission owner;
- effects and effect owner;
- required provenance;
- lowering destination;
- lifetime and pressure implications;
- why a direct adapter alone is insufficient.

The proposed shared representation passes only if removing either proving case still
leaves a coherent neutral contract with no proving-domain branches.

## Candidate first-proof classes

Do not preselect implementation solely from this design, but stronger proving diversity
would look like:

```text
one application/product construction composition
+
one semantically different runtime/data owner integration such as rendering
```

or another pair with clearly different owner semantics.

Two similar query/data pipelines are insufficient evidence for a family-wide substrate.

## Required implementation tests

A future first implementation should prove at minimum:

1. direct owner-native use still works without federation;
2. two structurally different compositions use the same neutral core;
3. owner-specific payload/operations remain owner-qualified;
4. unknown operations can be preserved/diagnosed but are not unsafely transformed;
5. generic rewrite fails without the required semantic interface;
6. declared legal adaptation preserves explicit provenance;
7. consumer admission remains with the consumer;
8. owner-native planning remains owner-native;
9. product composition lowers to ordinary `App` rather than creating parallel runtime
   authority;
10. deterministic lowering is sufficient without a cost optimizer;
11. diagnostics preserve owner-specific codes/context;
12. no global identity/revision/transaction/store/executor appears as an implementation
   convenience.

## Stop conditions

Stop and redesign if implementation requires any of the following:

```text
copying owner semantic rules into generic core
making every owner API return Plan
universal relation/graph/world type hierarchy
one writable semantic database
one generic lifecycle/evaluation enum for unrelated owners
one generic effect enum that erases owner rules
generic rewrites based on structural similarity rather than interfaces
planner decisions that silently weaken semantics
Plan as persistent parallel App truth
Workbench reach-through into owner private mutable state
mandatory owner dependency on Runenwerk federation
```

## Predecessor disposition after ADR acceptance

This design deliberately does not mutate predecessor files in the ADR adoption slice.
After ADR 0020 is accepted, #205 may perform a separate bounded lifecycle disposition.

Expected evaluation basis:

### Semantic Graph IR design

Preserve durable domain-owned graph semantics, validation/ratification, source mapping,
formed products, deterministic lowering, and no runtime editor-graph interpretation.

Do not promote its generic `SELECT / RELATE / TRANSFORM` conceptual family to the shared
federation core merely because several graph domains may use it.

### Gameplay Graph ATR design

Treat as domain-specific future design dependent on still-missing gameplay contracts.
Its compiler/lowering ideas may remain useful, but it is not cross-domain proof for a
universal semantic algebra.

### Typed App Program Counter proof

Treat as historical/specialized proof intent until a current App-program owner is
accepted. It must not supersede ADR 0019/0023 application/runtime ownership.

## Decision summary

Runenwerk should gain **shared composition without shared semantic ownership**.

The durable target is:

```text
owner semantic models stay native
        |
explicit opt-in interfaces
        |
small typed cross-owner composition artifact
        |
deterministic delegation / partition / adaptation
        |
owner-native planning and execution
        |
explainable provenance and Workbench correlation
```

The architecture intentionally rejects the simpler-looking but weaker model where all
Runen semantics are first translated into one universal Plan/database ontology.
