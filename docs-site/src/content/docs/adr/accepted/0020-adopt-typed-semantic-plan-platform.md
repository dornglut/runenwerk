---
title: Adopt the Typed Semantic Plan Platform
description: Accepted architecture for one typed extensible logical Plan IR and federated planning layer over independently owned semantic authorities and specialized physical execution.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-08-13
related_adrs:
  - ./0014-repository-family-extraction-boundaries.md
  - ./0017-cross-authority-consistency-and-graph-semantics.md
  - ./0018-semantic-federation-and-physical-realization.md
  - ./0019-batteries-included-application-composition.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../design/active/runen-semantic-platform-plan-ir-north-star-design.md
  - ../../design/active/semantic-graph-ir-and-compilation-design.md
  - ../../design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md
---

# ADR 0020: Adopt the Typed Semantic Plan Platform

## Decision

Runenwerk adopts one typed semantic programming model and one extensible logical
Plan IR for composing meaning across independently owned Runen systems.

The accepted target is:

```text
one typed semantic programming model
+ one extensible logical Plan IR
+ relational / graph / temporal / expression / domain dialects
+ a logical semantic catalog/database surface
+ federated planning and provider lowering
+ specialized owner-native physical realization and execution
+ explicit provenance, capabilities, effects, lifetime, budgets, and explainability
```

The decisive boundary is:

```text
UNIVERSAL LOGICAL SEMANTIC COMPOSITION / PLAN IR     YES
UNIVERSAL FEDERATION / PLANNING LAYER                YES
EXTENSIBLE DOMAIN DIALECTS                           YES
LOGICAL SEMANTIC CATALOG / DATABASE SURFACE          YES

UNIVERSAL PHYSICAL STORAGE ENGINE                    NO
UNIVERSAL PHYSICAL EXECUTION RUNTIME                 NO
UNIVERSAL OBJECT IDENTITY                            NO
UNIVERSAL REVISION / TRANSACTION / SNAPSHOT          NO
ONE MANDATORY SCHEDULER / EXECUTOR                   NO
ONE MANDATORY GRAPH RUNTIME                          NO
ONE PHYSICAL MEMORY / REPRESENTATION MODEL           NO
```

Plan describes semantic meaning, composition, effects, and requirements. Providers
lower admitted Plan regions into owner-native products and execution. Participating
owners retain their identities, versions, invariants, algorithms, storage, scheduling
semantics, physical representations, and native APIs.

This is an accepted North Star. It does not claim that the Plan IR, semantic catalog,
planner, dialects, providers, or Workbench integration exist in current code.

## Why this decision exists

ADR 0018 established a useful semantic-federation grammar while deliberately stopping
at conceptual Level A. That decision preserved owner autonomy and physical
specialization, but its Level-A-only boundary prevents Runenwerk from adopting one
logical programming and planning substrate for cross-domain composition.

Without a common logical Plan layer, applications and tools must repeatedly encode
cross-owner selection, correlation, temporal reasoning, maintained integration,
effects, provenance, provider choice, and explanation in disconnected integration
mechanisms. The result would remain inspectable owner by owner but would lack one typed
semantic program that can be verified, partitioned, lowered, and explained as a whole.

The platform therefore accepts the logical mechanisms while retaining ADR 0018's
physical and authority safeguards. One logical semantic programming/planning substrate
does not imply one physical storage/execution substrate.

## Relationship to ADR 0014

ADR 0014 remains authoritative and is not edited by this decision.

Peer frameworks remain independently usable. They do not depend on Runenwerk merely to
participate in Plan composition, planning, execution, or inspection. Runenwerk-owned
integration/providers may initially adapt peer-framework contracts into the Plan
platform without creating reverse dependencies.

This decision:

- does not create `runen-plan`, `runen-semantic`, or another crate or repository;
- does not alter current dependency topology;
- does not authorize direct new peer-framework dependencies;
- does not waive ADR 0014's clean-cutover or shared-extraction rules.

A future Plan crate or repository requires a separate accepted ADR under ADR 0014,
backed by concrete implementation evidence, a structurally different proving path, and
at least one independent consumer that demonstrates the extracted boundary remains
useful without Runenwerk.

## Relationship to ADR 0017

ADR 0017's core laws remain authoritative:

```text
one semantic invariant set has one authority
foreign owners are consumed through explicit contracts
owner-local consistency remains local
semantic dependency differs from execution ordering and resource hazard
incremental and clean/full paths are observationally equivalent
capability, requirement, policy, and authority remain distinct
ordinary APIs use progressive disclosure
shared extraction requires concrete repeated proof
```

ADR 0017 warned against introducing a universal `Plan<T>` or generic
compiler/evaluator wrapper merely to mirror an architecture taxonomy. This decision
clarifies that warning: it continues to prohibit empty generic wrappers and speculative
shared machinery.

The Plan accepted here is different. It is an explicitly selected semantic programming
IR with typed operations and expressions, dialect semantics, effect contracts,
verification, normalization, planner/provider interfaces, lowering, execution
contracts, provenance, and explainability. Its value comes from executable semantic
composition rather than from renaming architectural roles.

This clarification does not automatically authorize arbitrary shared runtime
machinery. Every concrete implementation and extraction still needs the applicable
evidence, ownership, dependency, lifetime, and performance review.

## Precise supersession of ADR 0018

ADR 0020 has precedence over ADR 0018 only where ADR 0018's Level-A-only conclusion or
non-goals prohibit the following logical platform mechanisms:

- a shared typed logical Plan IR;
- a logical semantic catalog and query/composition surface;
- extensible dialect infrastructure;
- planner/provider interfaces;
- semantics-preserving logical optimization;
- provider partitioning;
- physical planning;
- lowering and compilation;
- Plan explainability.

ADR 0018 remains authoritative for:

- semantic meaning being distinct from physical realization;
- owner-local identities, versions, revisions, and clocks;
- contextual compatibility and consumer-owned admission;
- explicit provenance and source correspondence;
- explicit capability, approximation, quality, and tolerance;
- inspection correlation not implying runtime admission;
- no global store, revision, transaction, or snapshot;
- specialized physical realization and execution.

This supersession does not authorize a universal physical database, incremental
database, dataflow runtime, graph runtime, storage engine, scheduler, executor, object
identity, transaction system, or memory model.

## Relationship to ADR 0019 and `App`

ADR 0019 remains authoritative: `App` is Runenwerk's one live runtime composition root.

Plan becomes the semantic programming and composition model, not a second application
runtime. Directionally, a product/application Plan may lower as follows:

```text
semantic application Plan
    -> lowering / product composition
    -> App + plugins + resources + adapters
    -> runtime
```

Product Plans may make composition explainable before construction, but must not remain
as a persistent parallel application-configuration authority, service locator, or
meta-executor beside `App`. This decision does not claim that Plan-based application
composition exists today.

## Logical semantic database meaning

The semantic catalog/database is a logical information and computation fabric over
explicitly participating sources, schemas, relationships, operations, dialects,
capabilities, provenance, providers, and physical alternatives.

It may support typed selection, correlation, graph traversal, temporal queries,
expressions, aggregation, maintained relationships, effects, and explanation across
heterogeneous owners. Its catalog facts are supplied or derived through explicit owner
contracts and Runenwerk integration.

The term `database` does not grant ownership of source data. It does not require:

```text
one physical store
one global object identity
one global revision or transaction
one mandatory materialization
one storage schema
one consistency cut
private owner-state reach-through
```

An owner may use archetype columns, tables, sparse sets, graphs, spatial indices,
analytic fields, GPU resources, streams, remote services, or another native
representation. The logical catalog describes available semantics and contracts; it
does not replace those authorities.

## Plan and dialect model

Plan is a typed semantic AST/DAG whose nodes describe operations, values, regions,
parameters, sources, effects, dialect identities, requirements, and provenance/source
maps. Public host-language types may preserve result typing without encoding the full
operator tree in Rust generic types.

A small common layer may provide general composition operations. Extensible dialects
provide relational, graph, temporal, expression, world, spatial, field, SDF, render,
GPU, network, UI, asset, application, and future owner-specific semantics.

A dialect may contribute:

- types and typed operations;
- verification and diagnostics;
- canonicalization and semantics-preserving rewrites;
- interfaces and capability requirements;
- lowering/provider contracts;
- explanation and inspection formatting.

The dialect system is open. It must not encode one application ontology or require
core changes for every new domain. Runenwerk standardizes composition mechanisms rather
than application ontologies.

This ADR does not freeze Rust names, builder methods, trait shapes, serialization,
operator inventories, or repository placement.

## Planner/provider boundary

The accepted conceptual pipeline is:

```text
typed semantic program
    -> Plan IR
    -> verification
    -> normalization / canonicalization
    -> optional logical optimization
    -> provider partitioning
    -> physical planning
    -> lowering / compilation
    -> owner-native products / executable work
    -> specialized execution
```

A provider reports which sources and operations it supports, which semantic
capabilities it preserves, its required and produced physical traits, locality and
residency, incremental support, relevant cost/pressure facts, and resulting
provenance. A provider does not gain semantic ownership merely by realizing an
operation.

Deterministic straightforward lowering is a valid complete baseline. Architecture
correctness must not depend on a sophisticated cost model, global optimizer, or
adaptive runtime. Logical optimization is optional and must preserve declared
semantics. Physical planning may remain direct and rule-based until evidence supports
greater sophistication.

## Semantic versus physical authority

Semantic operations are owned by the dialect or domain that defines their meaning.
Physical realization is owned by the provider/execution authority that implements an
admitted form.

The following may correspond without sharing identity or authority:

```text
world semantic entity/relationship
ECS-local entity/component realization
renderer-local scene value
GPU-local resource representation
```

Provider lowering must expose unsupported capability, approximation, tolerance,
quality loss, or representation conversion. It must not silently redefine the
semantic contract to fit a physical implementation.

Independent native owner APIs remain supported ordinary or expert paths. Plan
participation is an integration capability, not the only way a framework may be used.

## Evaluation lifetime

Plan evaluation distinguishes semantic intent from lifetime and retention policy.
Useful modes include one-shot evaluation, maintained observation, change output,
explicit materialization, reusable compilation, and explanation.

These modes are not required to share one runtime. Each concrete evaluation defines:

- the owner of the evaluation lifetime;
- which state is retained and why;
- cancellation and cleanup behavior;
- history, cache, index, and materialization pressure;
- what happens when retained evidence is pruned;
- the clean rebuild or full-resynchronization path where correctness requires it.

Incremental reuse changes cost, not semantic meaning, unless an owning contract
explicitly admits bounded staleness or approximation.

## Effects, provenance, and admission

Plan distinguishes pure/derived formation from operations that change an authority or
external environment. Effectful operations identify the changing authority and expose
their ordering, retry, idempotency, and commit requirements where applicable.

Cross-owner relationships carry explicit source lineage or correspondence. They are
not inferred through a universal object identity.

Consumer-owned admission remains the safety boundary. The consuming operation decides
whether owner-local revisions, time, scope, completeness, freshness, residency,
capability, approximation, quality, provenance, effect policy, and fallback form a
legal input set. A planner may assemble and explain these facts; it may not invent
admission authority.

## Budgets, pressure, and quality

Plans and evaluation requests may express general requirements for memory, latency,
throughput, transfer, retention, locality, result bounds, and quality. Domains define
their own meaningful capability, accuracy, approximation, and tolerance dimensions.

Planners/providers expose unsupported requirements, expensive transfers, large
materializations, bounded-history loss, device pressure, and quality trade-offs through
structured diagnostics and explanation. Budgets must not silently weaken semantics.
When a requested combination cannot be met, the responsible boundary rejects, chooses
an explicitly legal fallback, or reports an admitted approximation.

## Failure, cancellation, and recovery

Execution boundaries distinguish at least:

```text
semantic rejection
provider or source unavailability
capability / admission mismatch
resource or budget pressure
cancellation
partial, incomplete, or dropped observation
retryable external failure
committed effects
clean rebuild / full resynchronization
```

Cancellation follows explicit lifetime ownership. Work that has not committed may be
discarded according to provider policy; committed effects remain observable and must
not be represented as though they were rolled back. Recovery remains owner/provider
specific but is visible through the Plan execution boundary.

## Partitioning and locality

Physical planning may account for CPU/GPU/device, process/machine/service,
partition/shard, memory location, residency, representation, ordering, cardinality,
index availability, transfer cost, and migration cost.

Provider partitioning is explicit and explainable. It does not make placement or
distribution semantic unless an owning dialect intentionally defines it that way.
Cross-provider conversion preserves or explicitly narrows the semantic contract and
records provenance.

## Explainability and Workbench

Significant Plans must be inspectable at logical and physical levels. Explanation may
show sources, operations, effects, provenance, provider partitions, lowering choices,
placement, conversions, transfers, indexes/materializations, retained state,
cardinality/cost estimates, capability choices, quality decisions, and failures.

Workbench is a consumer of the Plan/catalog model. Table, graph, timeline, field,
image, resource, report, and owner-specific projections derive from the same semantic
fabric rather than from a competing universal inspection database.

Workbench does not gain foreign mutation authority or runtime admission authority.
Inspection correlation and a plausible Plan explanation still do not certify that a
particular set of owner-local values is legal for runtime consumption.

## Implementation staging

Implementation is separately authorized and should proceed in this evidence order:

1. assemble a semantic API and representative workload corpus;
2. design the minimal Plan IR and dialect mechanism required by that corpus;
3. prove planner/provider interfaces with deterministic straightforward lowering;
4. prove a world-plus-ECS provider path while preserving both owners;
5. prove a structurally different second provider and cross-provider partition;
6. only then consider extracting reusable Plan infrastructure under ADR 0014.

Each slice must preserve direct native APIs, source maps, effect visibility, owner-local
versions, and consumer-owned admission. No stage may infer that this ADR pre-authorizes
an optimizer, retained database, scheduler, executor, or shared repository.

## Predecessor-design relationship

The active Semantic Graph IR design established constrained authored intent,
validation/ratification, source maps, compiler-style lowering, and the rule that
runtime must not interpret editor graphs every frame.

The active Gameplay Graph ATR design applied those lessons to gameplay selection,
relationships, transformations, ECS/scheduler lowering, networking/authority metadata,
and diagnostics.

They remain useful predecessor and specialized designs and are not modified or retired
by this decision. Future semantic-graph and gameplay authoring may compile into Plan
rather than defining a competing universal IR. Their final lifecycle disposition is a
later bounded documentation-classification decision under issue #205.

## Rejected alternatives

### Retain ADR 0018 Level A only

Rejected as the final North Star because conceptual federation alone does not provide
one typed program that can be verified, partitioned, lowered, executed, and explained
across heterogeneous owners.

### Universal physical database

Rejected. One store, identity, revision, transaction, snapshot, or schema would steal
authority and physical freedom from independent domains.

### Universal execution runtime

Rejected. ECS systems, CPU work, async IO, streams, field evaluation, GPU work,
rendering, networking, external services, and application effects retain specialized
execution and scheduling semantics.

### One mandatory graph/dataflow runtime

Rejected. Logical Plan may contain graph, temporal, relational, or recursive semantics
without imposing one retained graph interpreter, progress model, incremental database,
or dataflow scheduler on every provider.

### ECS as the universal semantic ontology

Rejected. ECS is an important possible world realization, not the owner of fields,
assets, rendering, GPU resources, UI, networking, applications, or all semantic state.

### Independent domain plans with no common IR

Rejected as the platform target because it preserves duplicated composition,
partitioning, provenance, and explanation mechanisms and cannot provide one coherent
semantic programming surface.

### Require advanced cost optimization before adoption

Rejected. Straightforward deterministic lowering can prove the contracts completely;
optimizer sophistication is an optional later improvement.

## Consequences

- Runenwerk gains one logical typed semantic composition and planning substrate.
- Domain owners can add vocabulary through dialects without expanding a closed core
  ontology.
- Logical queries and maintained integrations can span heterogeneous providers while
  physical storage and execution stay owner-native.
- Planner/provider boundaries and explanation become accepted platform concepts.
- `App` remains the one live runtime composition root.
- Peer frameworks keep independent native APIs and repository independence.
- Implementers must make effects, provenance, admission, lifetime, budgets, quality,
  cancellation, and recovery explicit.
- The architecture incurs real IR, verification, evolution, planning, diagnostics, and
  conformance complexity; bounded proofs must earn each implementation step.
- ADR 0018 remains the physical/authority safety foundation except for the precisely
  superseded logical prohibitions.

## Fitness functions

The decision remains healthy only when:

1. ordinary semantic/programming APIs remain typed and understandable;
2. Plan semantics remain independent of physical realization;
3. new dialects participate without modifying a closed application ontology;
4. peer frameworks remain independently useful and do not depend on Runenwerk;
5. planner/provider participation does not transfer semantic authority;
6. deterministic straightforward lowering is sufficient for correctness;
7. owner-local versions and consumer-owned admission remain explicit;
8. effects identify the authority or external environment that changes;
9. approximation, quality, and budgets never silently change meaning;
10. incremental evaluation retains clean/full or full-resynchronization safety;
11. Plan partitioning and physical conversion are explainable;
12. Workbench views derive from the same Plan/catalog fabric without becoming runtime
    admission or foreign mutation authority;
13. direct owner-native APIs remain valid independent and expert paths;
14. product/application Plan lowers into `App` rather than becoming a second runtime;
15. no global identity, revision, transaction, store, scheduler, executor, or physical
    representation appears by implication.

## Explicit non-scope

This decision does not create or authorize:

```text
Rust implementation or public API spelling
runen-plan / runen-semantic crate or repository
dependency-topology changes
new direct peer-framework dependencies
universal physical database or storage engine
global identity, revision, clock, transaction, or snapshot
mandatory retained incremental database or dataflow runtime
universal graph runtime
universal scheduler or executor
universal event bus
universal wire format or serialization
one physical memory, ECS, GPU, or representation model
parallel application runtime or service locator beside App
managed backend service model
RunenGPU, RunenRender, RunenECS, RunenScheduler, networking, or Workbench implementation
predecessor-design deletion or lifecycle reclassification
roadmap, workflow, licensing, or issue-lifecycle changes
```

Issue #281 owns this bounded documentation-only adoption. Concrete Plan implementation,
provider proofs, repository extraction, and dependency changes require later separately
reviewed authority.
