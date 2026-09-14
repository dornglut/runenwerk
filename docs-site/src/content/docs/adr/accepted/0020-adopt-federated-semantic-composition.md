---
title: Adopt Federated Semantic Composition
description: Accepted architecture for optional typed cross-owner Plan composition through owner/dialect interfaces without a universal semantic database, owner model, planner runtime, or execution substrate.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-09-14
related_adrs:
  - ./0014-repository-family-extraction-boundaries.md
  - ./0017-cross-authority-consistency-and-graph-semantics.md
  - ./0018-semantic-federation-and-physical-realization.md
  - ./0019-batteries-included-application-composition.md
  - ./0021-ratify-runenrender-semantic-rendering-architecture.md
  - ./0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
  - ./0024-normalize-physical-input-observation-semantics.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../design/active/runen-federated-semantic-composition-design.md
  - ../../design/active/semantic-graph-ir-and-compilation-design.md
  - ../../design/active/gameplay-graph-atr-ir-and-ecs-lowering-design.md
---

# ADR 0020: Adopt Federated Semantic Composition

## Decision

Runenwerk adopts an **optional typed federated composition layer** for work that
materially spans independently owned semantic authorities.

The accepted North Star is:

```text
many owner-normalized semantic models
+ one optional typed cross-owner composition protocol
+ owner-qualified dialects and operation contracts
+ explicit cross-owner relationships, requirements, effects, provenance, and admission facts
+ deterministic partitioning into owner-native planning, lowering, realization, and execution
+ explainable integration and Workbench correlation
```

The governing law is:

```text
One semantic invariant set has one authority.
One composition may span many authorities.
Composition does not normalize away owner semantics.
```

A shared `Plan` may represent one explicit cross-owner composition. It is **not** the
universal semantic representation of every owner query, observation, runtime state,
plan, program, product, or API.

This decision advances ADR 0018 from shared Level-A reasoning vocabulary to a bounded
Level-B composition/interface target. It does not pre-authorize a Level-C shared
planner runtime, database, optimizer, executor, retained dataflow system, or other
universal machinery.

## Why the boundary is narrower than the former North Star

A previous unmerged candidate proposed one universal logical semantic Plan IR, semantic
database/query surface, broad reusable algebras, and federated planning layer for nearly
all Runen semantics.

Current accepted architecture provides stronger evidence against that breadth.

ADR 0023 and its normalized App/runtime semantic model distinguish application
composition, runtime containment, Host, Advancement Policy, lifecycle occurrence, and
owner capability state. Those dimensions are not one generic operation algebra.

ADR 0024 distinguishes device-level observation, grouping, ordering, evidence status,
delivery role, reconciliation/cancellation, identity scopes, state reduction, units,
and consumer meaning. Backend-neutral normalization succeeds by preserving those
semantic distinctions rather than translating them into one universal event or value
ontology.

ADR 0021 gives RunenRender its own `RenderPlan`, binding admission, execution admission,
representation semantics, and owner-native lowering. It explicitly permits future
shared composition while forbidding that layer from absorbing renderer-owned planning
invariants.

The repeated neutral requirement across these structurally different domains is
therefore **typed composition across explicit owner contracts**, not one common semantic
algebra for their internal meaning.

## Relationship to ADR 0014

ADR 0014 remains authoritative for repository-family independence, one-way integration,
clean cutover, exact revision consumption, adapter direction, and the rejection of
shared-core or duplicate semantic authority.

Peer frameworks do not depend on Runenwerk merely to participate in composition.
Runenwerk-owned adapters may expose owner contracts to a composition layer without
moving the owner's reusable semantics into Runenwerk.

This decision does not create `runen-plan`, `runen-semantic`, or another repository or
package. Any later extraction still requires a separate ADR and concrete independent
consumer evidence under ADR 0014.

## Relationship to ADR 0017

ADR 0017 remains the safety foundation:

```text
one semantic invariant set has one authority
foreign authorities are consumed through explicit contracts
versions and consistency remain owner-local
semantic dependency != execution ordering != resource hazard
incremental and clean/full paths preserve owner-declared semantics
capability != requirement != policy != authority
shared extraction requires repeated neutral proof
ordinary owner APIs remain understandable without a substrate
```

This ADR does not supersede ADR 0017's warning against universal `Plan<T>` wrappers,
generic graph interpreters, registries, compiler/evaluator frameworks, or other shared
machinery introduced merely because several systems can be described with similar
words.

Instead, it selects one narrower Level-B target: an optional composition artifact whose
common semantics stop at cross-owner structure and explicitly delegated interfaces.

## Relationship to ADR 0018

ADR 0018 remains authoritative for semantic federation, semantic/physical separation,
owner-local versions, consumer-owned admission, provenance, realization, explicit
effects, and Workbench safety.

ADR 0020 has precedence only over ADR 0018 wording that leaves **all** peer-neutral
composition/interface protocols unaccepted.

The maturity boundary becomes:

```text
Level A
  shared reasoning vocabulary
  ACCEPTED by ADR 0018

Level B
  optional typed federated composition/interface protocol
  ACCEPTED as an architecture target by ADR 0020

Level C
  shared planner runtime / semantic database / optimizer / executor / retained substrate
  NOT PRE-AUTHORIZED
```

Acceptance of Level B does not imply that a concrete shared Rust representation already
exists or that implementation may skip ADR 0017's proof gate.

## Relationship to ADR 0019 and ADR 0023

`App` remains Runenwerk's one live application/runtime composition root.

A future product composition Plan may be useful before runtime construction:

```text
composition Plan
    -> validated owner/plugin/product selections
    -> ordinary App + plugins + resources + adapters
    -> live runtime
```

After lowering, the Plan must not remain a parallel writable application configuration
authority, service locator, meta-runtime, or meta-executor beside `App`.

App/runtime lifecycle, Host, Advancement Policy, containment, and capability ownership
remain governed by ADR 0019, ADR 0023, and their detailed semantic model.

## Relationship to owner-native plans and runtimes

A shared Plan composes owner contracts; it does not replace owner-native planning.

Examples that remain owner-owned include:

```text
RunenRender RenderPlan and renderer admission
RunenGPU work/access graph and GPU execution
RunenECS query/schedule semantics
device-level input observation and reducer semantics
RunenUI mounted/runtime semantics
RunenNet session/delivery/replication semantics
App live composition/runtime state
```

For RunenRender the intended direction is explicitly:

```text
shared composition Plan
    -> RunenRender semantic request / native planning
        -> RunenRender binding + execution admission
            -> RenderWorkSet
                -> RunenGPU
```

Equivalent owner boundaries apply elsewhere.

## Plan role

`Plan` is an optional, typed, inspectable composition artifact for cases where one
composition materially spans owner contracts.

Candidate uses include:

- cross-owner orchestration;
- maintained integration definitions;
- Workbench/tool composition;
- product construction before lowering into `App`;
- multi-owner computations where explicit partition/lowering improves correctness,
  explanation, or reuse.

A direct owner-native API remains valid and often preferable when no cross-owner
composition artifact is useful.

The architecture does not require every owner operation to be converted into Plan form.

## Common composition core

A future shared representation must remain smaller than the semantic models it composes.
Its common contract is directionally limited to facts such as:

```text
owner / dialect-qualified operation identity and version
explicit contract/value references
explicit composition/dependency/correspondence relationships
owner/effect target facts sufficient to prevent hidden mutation
requirements and declared admission inputs
provenance and source maps
composition regions / partition boundaries
explanation and diagnostics
```

This list defines semantic pressure, not final Rust field or trait names.

The common core does **not** pre-authorize:

```text
one universal type lattice
one universal object identity
SELECT / RELATE / TRANSFORM as family-wide operations
relational algebra as the platform ontology
graph algebra as the platform ontology
world entity/component semantics as the platform ontology
one generic effect enum
one generic lifetime/evaluation enum
one universal budget/quality schema
one universal serialization format
```

Concrete dialects may define any of those when their owner semantics justify them.

## Owner/dialect interfaces

An owner or dialect retains authority for:

- operation meaning;
- owner-defined types/contracts;
- semantic verification and diagnostics;
- legal canonicalization and rewrites;
- effect semantics;
- validity and admission requirements;
- owner-native planning/lowering;
- owner-specific explanation.

Generic composition behavior is fail-closed.

An unknown operation may remain representable for routing, provenance, or inspection,
but a generic pass must not rewrite, reorder, fuse, eliminate, substitute, or lower it
without an explicit interface proving the required property.

Cross-dialect transformation requires an explicit integration/owner contract. Structural
similarity is never equivalence proof.

## Federation catalog

Runenwerk may provide a derived, read-oriented **federation catalog** for composition and
tooling.

Directionally it may describe:

```text
participating owner/dialect identities
operation/contract descriptors
schema/type references
explicit relationships/interfaces
provider/lowering availability
capability/requirement metadata
provenance and inspection hooks
```

The catalog is integration metadata, not semantic source state.

It is not:

```text
a writable global registry
a universal object database
a source-of-truth store
a global query engine
a global identity/revision/transaction authority
a required path for direct owner APIs
```

Relational query behavior may exist in a specific dialect or Workbench/provider surface
when useful. It is not implied by the existence of the federation catalog.

## Planning and partitioning

The complete baseline is deterministic orchestration:

```text
verify composition structure
-> collect owner/interface requirements
-> partition explicit owner regions
-> invoke owner-native verification/planning/lowering
-> insert only explicitly legal adaptations/conversions
-> preserve provenance
-> expose explanation
```

Architecture correctness must not depend on a global cost optimizer.

Owner-local optimizers remain owner-local. A cross-owner rewrite or placement decision
is legal only when explicit interfaces establish the semantic and effect conditions
required by that transformation.

Cost, cardinality, locality, residency, transfer, pressure, quality, and budget facts
may be exposed through concrete interfaces. They are not all mandatory universal Plan
semantics.

## Provider and realization boundary

A provider or adapter realizes only owner-qualified operations it explicitly supports.
Provider participation does not transfer semantic ownership.

A provider must preserve the semantic contract it claims to realize or explicitly
report an owner-authorized narrower capability, approximation, adaptation, or rejection.

Physical placement, storage, scheduling, execution, caching, retention, and device state
remain with the applicable owner unless a separately accepted shared contract proves a
neutral repeated invariant.

## Admission

Consumer-owned admission remains final.

A composition layer may carry or explain facts such as revisions, time, scope,
completeness, freshness, capability, residency, provenance, requirements, and fallback.
It does not acquire permission to decide that those facts are sufficient for a concrete
consumer unless that consumer explicitly delegates the admission contract.

```text
composition compatibility
!= owner semantic validity
!= consumer runtime admission
```

## Effects

The composition layer must make cross-owner effects explicit enough to prevent hidden
mutation and unsafe generic transformation.

The common layer may identify an effect target/owner and carry owner-qualified effect
metadata. Concrete commit, retry, idempotency, rollback, ordering, and failure semantics
remain owner-defined.

Do not invent a closed family-wide effect taxonomy merely for planner convenience.

## Lifetime, retention, and pressure

Analogous lifecycle concepts do not automatically share semantics.

Input observation lifetime, renderer retained products, RunenNet delivery/replication
retention and resynchronization, caches, materialization, App advancement, replay
retention, and external jobs remain separate owner concerns.

A composition may state requirements through participating interfaces where needed, but
this ADR does not standardize universal `run`, `watch`, `changes`, `materialize`, or
`compile` evaluation modes.

A shared lifetime/retention interface requires its own repeated proof.

## Workbench

Workbench may consume both shared composition artifacts and owner-native inspection
projections.

It does not require every table, tree, graph, timeline, field, image, resource, or report
to be generated from one Plan/query database.

A composition explanation may show owner regions, relationships, requirements, effects,
provenance, legal adaptations, and lowering decisions. Owner projections continue to
explain owner-specific state.

Inspection remains read-oriented and does not grant foreign mutation or runtime
admission authority.

## Persistence and interchange

No persisted Plan format, cross-process ABI, wire format, stable textual language, or
repository extraction is accepted by this ADR.

A first implementation may be process-local. Persistence requires explicit identity,
versioning, schema/dialect compatibility, migration, validation, and security policy
before it becomes durable authority.

## External architecture lessons

This decision uses external systems as pressure, not templates.

MLIR demonstrates that very different dialect operations can coexist while generic
transforms depend on explicit interfaces and explicit conversion legality. Its small
builtin core is deliberately conservative.

Apache Calcite demonstrates powerful federation and semantics-preserving planning over a
shared **relational algebra**. That is evidence for strong planning when a real common
algebra exists, not evidence that unrelated Runen semantics should be forced into one.

Substrait demonstrates engine-independent relational plans and extension mechanisms, but
its most general custom relations require producer/consumer agreement and are the least
interoperable form. This supports keeping the common Runen layer small and owner-defined.

## Implementation gate

This ADR authorizes no Rust, Cargo, dependency, repository, or runtime change by itself.

Before a concrete shared Plan representation is implemented, a new bounded issue must:

1. select at least two structurally different real cross-owner composition cases;
2. document each participating owner's native normalized semantics and direct API;
3. identify the smallest repeated composition facts/interfaces;
4. prove the proposed shared representation contains no proving-domain semantic branch;
5. preserve a direct owner-native path;
6. prove deterministic owner-native lowering without duplicate semantic authority;
7. prove unknown/new dialect operations fail closed without core rewrites;
8. characterize lifetime, memory, versioning, diagnostics, and cognitive cost;
9. decide whether the first implementation remains Runenwerk-local or has separately
   earned extraction under ADR 0014.

A proving pair should be operationally different. Two relational-like workloads do not
by themselves prove a general cross-owner composition substrate.

## Predecessor-design relationship

The active Semantic Graph IR design retains useful laws around domain-owned graph
semantics, ratification/validation, formed products, source lineage, and avoiding runtime
interpretation of editor graphs.

The active Gameplay Graph ATR design remains a domain-specific speculative
specialization. Its `SELECT / RELATE / TRANSFORM` vocabulary is not promoted into the
shared core by this ADR.

The active Counter proof must not be treated as evidence for a universal App-program
runtime. ADR 0019 and ADR 0023 remain authoritative for application/runtime composition.

These predecessor documents are intentionally unchanged in this adoption slice. Their
final lifecycle disposition remains a later bounded #205 decision after this architecture
is accepted.

## Rejected alternatives

### Restore the former universal logical Plan/database candidate

Rejected because current structurally different owner models prove shared composition
pressure but do not prove one common semantic algebra for their internal meaning.

### Remain permanently at ADR 0018 Level A

Rejected because real cross-owner tools and integrations benefit from an accepted target
for typed, inspectable composition and owner-interface interoperability.

### Universal semantic database/query engine

Rejected because it would either steal source authority or force unrelated semantic
models into one query ontology.

### Universal planner/optimizer

Rejected as a platform requirement. Generic rewrites are safe only where explicit
interfaces prove legality/equivalence/effect safety.

### One mandatory graph/dataflow runtime

Rejected. Composition structure does not imply one execution/progress model.

### ECS, relational algebra, or semantic graphs as the universal ontology

Rejected. Each is useful where its semantics fit and remains local or dialect-owned.

### Independent owners with no shared composition target

Rejected because repeated cross-owner integration, tooling, and explanation deserve a
bounded Level-B architecture target even though owner-native semantics remain separate.

## Consequences

- Runenwerk gains a coherent target for typed cross-owner composition without creating a
  semantic super-domain.
- Owner-normalized semantic models remain the normative source of meaning.
- Direct native APIs remain first-class.
- Generic transformations become explicitly interface-gated and fail closed.
- A federation catalog may improve discovery and tooling without becoming a source-of-
  truth database.
- RunenRender, App/runtime, input, networking, UI, ECS, GPU, and future owners keep their
  native semantic plans/state machines/observation models.
- Workbench can explain shared composition while still consuming owner-native views.
- The architecture gives up the aesthetic simplicity of "everything is Plan" in favor
  of stronger semantic ownership and lower accidental coupling.

## Fitness functions

This decision remains healthy when:

1. every semantic invariant set still has one identifiable owner;
2. a direct owner-native API remains possible without Plan participation;
3. a shared composition never becomes the canonical storage/state model of an owner;
4. unknown dialect operations can remain opaque without unsafe generic transformation;
5. generic rewrites require explicit legality/equivalence/effect interfaces;
6. provider realization never transfers semantic authority;
7. consumer-owned admission remains final unless explicitly delegated;
8. owner-native plans such as `RenderPlan` are composed rather than reimplemented;
9. product composition lowers into the one live `App` runtime root;
10. Workbench inspection does not require one universal query database;
11. no global identity, revision, transaction, store, type ontology, scheduler, executor,
    effect taxonomy, lifetime model, or physical representation appears by implication;
12. concrete shared implementation is accepted only after structurally different real
    consumers prove the same neutral composition invariant.

## Explicit non-scope

This decision does not create or authorize:

```text
Rust implementation or public API spelling
runen-plan / runen-semantic package or repository
new dependency topology
universal semantic database or query engine
universal type/object identity system
global revision / transaction / snapshot
universal planner / optimizer / scheduler / executor
universal graph or dataflow runtime
universal effect or lifetime enum
universal persisted Plan or wire format
RunenRender / RunenGPU / RunenECS / RunenUI / RunenNet semantic changes
App runtime replacement
predecessor-design lifecycle reclassification
roadmap or workflow changes
```

Issue #281 owns this bounded documentation-only architecture adoption. Concrete shared
composition implementation, provider proofs, persistence, extraction, and dependency
changes require later separately accepted work.
