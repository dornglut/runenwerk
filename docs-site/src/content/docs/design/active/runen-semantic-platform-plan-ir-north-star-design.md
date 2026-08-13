---
title: Runen Semantic Platform and Plan IR North Star
description: General-purpose North Star for typed semantic programming, federated logical data and computation, extensible dialects, heterogeneous planning, and specialized execution across the Runen family.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-08-13
related_adrs:
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
related_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ./semantic-graph-ir-and-compilation-design.md
  - ./gameplay-graph-atr-ir-and-ecs-lowering-design.md
  - ./runenecs-extraction-boundary-design.md
---

# Runen Semantic Platform and Plan IR North Star

## Purpose

This document defines the aspirational programming and federation model for long-term Runenwerk.

> **Runenwerk is a general-purpose typed semantic systems platform. Applications, simulations, tools, queries, maintained integrations, and inspectable computations are expressed through one extensible semantic Plan model. Independent Runen systems own domain semantics and specialized physical execution beneath that model.**

The Plan model combines relational, graph, temporal, expression, field, spatial, effect, and domain-specific semantics in one typed logical representation that can be partitioned and lowered across multiple execution providers.

The compact target is:

```text
One typed semantic programming model.
One extensible Plan IR.
Relational + graph + temporal + domain algebras.
Independent semantic authorities.
Owner-native physical realizations.
Heterogeneous execution.
Explicit provenance, capability, and effects.
Declarative maintained integration.
One explainable Workbench model.
```

## One picture

```text
                       APPLICATION / TOOL / SIMULATION
                                   |
                                   v
                         RUNEN SEMANTIC LANGUAGE
                                   |
                              typed Plan IR
                                   |
            +----------------------+----------------------+
            |            reusable semantic algebras       |
            | relational  graph  temporal  expr  world    |
            | spatial     field  domain dialects   ...     |
            +----------------------+----------------------+
                                   |
                              Runen planner
                                   |
                        logical -> physical lowering
                                   |
       +------------+--------------+--------------+-------------+
       |            |              |              |             |
       v            v              v              v             v
    RunenECS   RunenSpatial     RunenSDF     RunenRender    RunenGPU
                     specialized physical execution
```

Semantic composition is shared. Storage, algorithms, scheduling, locality, and realization remain specialized.

# 1. Programming model

The user-facing model converges around three concepts.

### Plan

A `Plan<T>` is a typed semantic description of a computation, query, rule, relationship, maintained integration, product, or application composition. `Expr<T>` represents typed semantic computation within Plans.

Plan construction describes meaning. Evaluation chooses realization and execution.

### Dialects

Dialects contribute semantic vocabulary to the same Plan representation:

```text
runen.core
runen.expr
runen.rel
runen.graph
runen.time
runen.world
runen.spatial
runen.field
runen.sdf
runen.render
runen.gpu
runen.network
runen.ui
runen.asset
runen.app
...
```

A dialect may contribute types, operations, verification, canonicalization, interfaces, lowering, provider capabilities, diagnostics, and inspection formatting.

### Evaluation

The same semantic Plan may be used through explicit evaluation/lifetime modes:

```text
run          evaluate a requested result or effect
watch        maintain an observable current result
changes      expose change output
materialize  retain an explicit reusable realization
compile      form reusable executable work
explain      inspect logical and physical interpretation
```

# 2. Semantic database surface

Runenwerk exposes participating authorities as one logical semantic catalog and query surface:

```rust
let db = runen.db();
```

The semantic database knows explicitly supplied sources, schemas, relationships, dialects, capabilities, provenance, providers, and physical alternatives. Sources retain owner-native storage and execution.

The database is a **logical information and computation fabric**.

# 3. Logical value shapes

The Plan model supports reusable semantic shapes:

```text
Scalar<T>       one typed value
Record<T>       structured value
Relation<R>     typed records or bindings
Graph<N, E>     nodes, typed relationships, topology
Path<N, E>      graph path
Field<D, V>     value over a domain
Stream<E>       temporal occurrences or changes
Snapshot<T>     immutable owner-local observation
Resource<T>     typed resource or capability reference
Product<T>      owner-published semantic result
```

These are logical shapes rather than storage classes. Relations may be archetype columns, arrays, tables, views, GPU buffers, or remote results. Graphs may be explicit or implicit. Fields may be analytic, sampled, procedural, cached, GPU-resident, or remote.

# 4. Plan IR and typed expressions

`runen.core` remains small. Directionally it supplies composition concepts such as:

```text
Source
Value
Parameter
Project
Filter
Derive
Correlate
Scope
Compose
Region
Call
Effect
```

The owned Plan IR is a compact typed AST/DAG with operations, typed values, regions, source descriptors, parameters, semantic effects, dialect operation identities, provenance requirements, and source maps.

Public Rust types stay compact (`Plan<Relation<(Entity, Health)>>`, `Plan<WorldRule>`, `Plan<Application>`); the entire operator tree does not become the Rust generic type.

The expression dialect supports arithmetic, comparison, boolean logic, vectors/matrices, construction, field access, selection, interpolation, conversion, and registered functions.

Three implementation levels coexist:

1. semantic expressions with full planner visibility;
2. registered operations with known contracts and specialized implementations;
3. native Rust stages with explicit input/output/effect contracts.

# 5. Relational and graph algebras

Relational operations are first-class:

```text
Project / Select
Filter
Join
Group
Aggregate
Distinct
Sort
Union
Difference
Window
```

Graph semantics are equally first-class:

```text
Node
Edge
Pattern
Match
Traverse
Reachable
Path
ShortestPath
Neighbors
Ancestor / Descendant
```

Graph matching may produce relational bindings so the algebras compose naturally. Recursive/fixed-point semantics are available where a domain needs transitive closure or iterative computation.

# 6. World dialect

`runen.world` is the general semantic state-and-rule dialect used by gameplay and other world-like simulations. It remains domain-neutral.

Its vocabulary includes:

```text
Entity
Tag<T>
Value<T>
Relation<R>
Global<T>
Event<E>
Command<C>
Rule
```

These describe semantic state above physical storage. A conventional RunenECS provider may lower them to entities, components, resources, relationship indices, ECS queries, and scheduled execution.

Directionally:

```rust
let movement = plan()
    .on(simulation.tick())
    .from(world.entities())
    .where_(has::<Movable>())
    .set::<Position>(
        value::<Position>() + value::<Velocity>() * time.delta()
    );
```

World update forms include assignment, reduction, event emission, relationship change, and entity/state insertion/removal. Their conflict and commit semantics are explicit so deterministic behavior does not depend on incidental thread order.

Randomness and other nondeterministic inputs are explicit semantic sources where reproducibility matters.

# 7. Temporal, spatial, and field dialects

Time is explicit through operations such as `At`, `Between`, `Latest`, `Since`, `Changes`, `Window`, and `Interval`; each authority retains the clock/revision semantics meaningful to it.

Spatial operations may include `Within`, `Contains`, `Intersects`, `Nearest`, `Distance`, `Visible`, `Raycast`, and `Coverage`.

Field operations may include `Sample`, `Gradient`, `Region`, `Bounds`, `Compose`, `Transform`, `Capability`, `Accuracy`, and `IsoSurface`.

A Plan can therefore combine world, spatial, and field semantics while lowering each region to the owner that can realize it best.

# 8. Effects, provenance, and admission

The Plan language distinguishes formation from effects.

Formation operations include selection, filtering, derivation, correlation, matching, sampling, and aggregation. Effectful terminals identify the changing authority or external environment through semantic operations such as `propose`, `commit`, `publish`, `submit`, `send`, `write`, or `execute`.

Cross-owner correspondence is explicit provenance. Runtime admission remains consumer-owned: the consuming operation decides whether revisions, capabilities, scope, time, residency, quality, and fallback form a legal input set.

# 9. Maintained integration

Standard cross-domain integration is represented as a maintained semantic relationship.

Directionally:

```rust
let renderables = db
    .from(world.entities())
    .where_(has::<Model>())
    .select((entity(), value::<Transform>(), value::<Model>()));

bind(render.scene(), renderables);
```

A binding owns lifecycle semantics such as incremental update strategy, resynchronization, buffering, admission, fallback, and lineage. Standard products install canonical bindings automatically; custom products author them when intentionally changing the default relationship.

# 10. Evaluation lifetime, budgets, and pressure

Plans distinguish one-shot evaluation, maintained observation, change streams, retained materialization, indexes, caches, and compiled executable plans.

Plans may carry generic execution requirements such as memory, latency, throughput, result, retention, transfer, quality, and locality budgets. Providers and planners surface expensive transfers, large materializations, pressure, and unsupported combinations through diagnostics and `explain()`.

Budget vocabulary stays general; domains define the quality/capability dimensions meaningful to them.

# 11. Failure, cancellation, and recovery

Execution can represent successful completion, semantic rejection, provider unavailability, capability mismatch, resource pressure, cancellation, partial/dropped observation, retryable external failure, and resynchronization or clean rebuild.

Cancellation follows explicit lifetime ownership. Already committed effects remain distinguishable from derived work that can be discarded. Recovery strategy remains provider/domain-owned but visible at the execution boundary.

# 12. Partitioning, locality, and distribution

Physical planning treats CPU/GPU/device, process/machine/remote service, partition/shard, memory location, residency, representation, ordering, cardinality, index availability, transfer cost, and migration cost as execution traits.

A logical Plan therefore remains stable as a realization evolves from local execution to GPU, multi-process, remote, or distributed execution. Distribution becomes semantic only where a domain contract intentionally gives placement semantic meaning.

# 13. Planner and providers

The long-term planning pipeline is:

```text
typed source program
    -> semantic Plan IR
    -> verification
    -> normalization / canonicalization
    -> logical optimization
    -> provider partitioning
    -> physical planning
    -> lowering / compilation
    -> reusable executable plan
    -> execution / specialization
```

Logical rewrites may include predicate pushdown, projection pruning, constant folding, common-subexpression reuse, join/correlation reordering, graph rewrite, operator fusion, domain rewrites, and dead-rule elimination.

Physical planning chooses providers, indexes, CPU/GPU placement, partitions, conversions, transfers, materialization, incrementality, parallelism, and specialized native implementations.

Correctness depends on semantic contracts and provider lowering rather than optimizer sophistication. Deterministic straightforward lowering is a complete baseline.

A provider conceptually answers what sources and operations it supports, what semantic capabilities it preserves, what physical traits it requires/produces, where data resides, expected costs, incremental support, and resulting provenance.

Independent Runen frameworks retain native APIs and independent usefulness. Plan participation is an integration capability.

# 14. Schema, dialect, and Plan evolution

Stable semantic identity may exist for dialects, types, operations, functions, relations, schemas, and capabilities.

Persisted or compiled Plans may record plan identity/hash, dialect versions, schema versions, extension versions, parameters, source maps, and required capabilities.

Dialect evolution owns compatibility and migration of semantic operations independently of backend storage layout. Plans containing process-local native functions may remain process-local; fully stable semantic Plans may support persistence, caching, remote execution, saved queries, or reproducible diagnostics where useful.

# 15. Explainability and Workbench

Every significant Plan is inspectable semantically and physically.

`plan.explain()` may expose sources, operations, effects, provenance, provider partitioning, lowering choices, placement, indexes/materializations, conversions/transfers, estimated cardinality/cost, retained state, and capability/quality choices.

The Workbench is a Plan consumer. Table, graph, timeline, field, image, resource, and custom views are projections of the same semantic fabric rather than a parallel inspection database.

# 16. Product composition and ergonomics

Application composition may itself use Plan fragments, so named product recipes remain inspectable compositions rather than opaque runtime presets.

The exact Rust spelling remains open, but the language family should remain small and coherent:

```text
formation      from / where / select / derive / correlate / match
world effects  set / reduce / emit / relate / insert / remove
dialects       within / sample / traverse / between / ...
lifetime       run / watch / changes / materialize / compile
understanding  explain / inspect / show
```

The Rust builder is the primary systems-language frontend. Future textual, graphical, generated, or Workbench frontends may target the same Plan IR.

# 17. General-purpose scope

Runenwerk standardizes composition mechanisms rather than application ontologies.

Relations, graphs, fields, time, provenance, effects, capability, partitioning, retention, and planning are common because they recur across structurally different domains. Product- or framework-specific semantics remain in schemas, dialects, and registered operations.

The same platform can therefore host games, simulations, procedural systems, scientific workloads, rendering tools, data-processing tools, editors, and other applications without embedding one product ontology into the common database design.

# 18. Repository-family fit

A possible long-term shape is:

```text
runen-plan
    Plan IR, type system, dialect infrastructure, interfaces

runen-ecs
runen-spatial
runen-sdf
runen-render
runen-gpu
runen-ui
runen-scheduler
...

runenwerk
    semantic catalog/database
    planner
    provider federation
    standard integrations
    application composition
    Workbench foundation

products / tools / games
```

Exact repository placement is a separate decomposition decision. The durable rule is independent semantic ownership plus one coherent composed platform.

# 19. Relationship to predecessor designs

`semantic-graph-ir-and-compilation-design.md` established constrained semantic authoring, source lineage, validation, and lowering rather than runtime interpretation of editor graphs.

`gameplay-graph-atr-ir-and-ecs-lowering-design.md` established declarative gameplay selection/relationship/transformation concepts and compiler-style lowering into ECS, scheduling, networking, and diagnostics.

This North Star generalizes them: the semantic representation becomes a general typed Plan IR rather than a graph-authoring specialization, while ECS/scheduler products become possible physical lowerings rather than the defining gameplay execution ontology.

The predecessor documents remain useful implementation and provenance material until documentation-authority cleanup assigns their final disposition.

# 20. Inspiration and provenance

This design is a synthesis. The sources below inspire specific aspects of the target.

| Inspiration | Idea carried into Runenwerk | Runenwerk synthesis |
| --- | --- | --- |
| Relational algebra / SQL | Declarative projection, filtering, joins, aggregation, set-oriented mutation | One common algebra inside a broader semantic Plan system. |
| [Apache Calcite](https://calcite.apache.org/docs/algebra.html) | Logical operator trees, semantics-preserving rewrites, cost planning, adapters, pushdown, execution conventions, and conversion across engines | Federation planning spans heterogeneous engine authorities and physical providers, not only data stores. |
| [MLIR](https://mlir.llvm.org/docs/Interfaces/) | Extensible dialects, typed operations, interfaces, multiple abstraction levels, progressive lowering | A small Plan core hosts common and owner-specific Runen dialects without closing the IR around one domain. |
| [Substrait](https://substrait.io/extensions/) | Engine-independent logical plans, typed extensions, custom relation/function/source semantics | Semantic/physical separation with extensibility beyond relational computation. |
| LINQ / expression-tree systems | Typed host-language query construction separated from provider execution | Rust builders construct owned typed Plans that may be partitioned across several providers. |
| [Cypher / property-graph querying](https://neo4j.com/docs/cypher-manual/current/patterns/reference/path-patterns-and-graph-patterns/) | Declarative graph patterns, relationships, paths, quantified traversal, shortest-path semantics | Graph algebra is first-class beside relational algebra and produces composable typed bindings. |
| [Datalog / Soufflé](https://souffle-lang.github.io/tutorial) | Declarative rules, recursion, transitive closure | Recursive/fixed-point semantics are available where useful without making all Plans logic programs. |
| [Differential Dataflow](https://timelydataflow.github.io/differential-dataflow/chapter_5/chapter_5.html) | Incremental collections and maintained shared indexed arrangements | Incrementality and reusable indexing are physical strategies beneath stable semantic Plans. |
| [OpenUSD Hydra 2.0](https://openusd.org/dev/api/_page__hydra__getting__started__guide.html) | Queryable views, lazy filtering chains, merging, change propagation, inspectable intermediate representations | Lazy owner-native projections and Workbench inspection apply across the Runen semantic fabric. |
| [Bevy ECS](https://bevy.org/learn/quick-start/getting-started/ecs/) | Small composable world-state pieces, data-oriented execution, parallelism, straightforward Rust authoring | Those execution strengths remain available while world semantics sit above any one ECS realization. |

The synthesis is:

```text
SQL-like declarative programming
    + relational algebra
    + graph query languages
    + Datalog-style recursion where useful
    + Calcite-style federation and planning
    + MLIR-style dialects and lowering
    + Substrait-style semantic/physical separation
    + optional differential-style incrementality
    + Hydra-style lazy inspectable projections
    + ECS-quality data-oriented execution
    + Runen authority, provenance, capability, and admission semantics
```

# 21. Design invariants

1. Runenwerk offers one typed semantic Plan programming model across composed products.
2. Plan IR describes semantic meaning independently of physical realization.
3. Relational and graph semantics are first-class reusable algebras.
4. World state and world rules are semantic concepts above a particular ECS layout.
5. Domain-specific meaning enters through dialects, schemas, and registered operations.
6. The core IR stays small while dialects remain independently extensible.
7. Public Plans are strongly typed without encoding the complete operator tree in Rust generic types.
8. Independent Runen owners remain independently meaningful and retain native APIs.
9. One Plan may be partitioned across several specialized providers.
10. Logical rewrites preserve semantic meaning; physical planning chooses realization and placement.
11. Evaluation lifetime, retention, and materialization are explicit.
12. Resource budgets and boundedness are visible to planning and diagnostics.
13. Failure, cancellation, and recovery have explicit execution semantics.
14. Partitioning, locality, GPU placement, and distribution are physical planning traits.
15. Cross-owner provenance and correspondence remain explicit.
16. Consumer-specific admission governs effect legality.
17. Effects identify the authority or external environment that changes.
18. Standard maintained integrations remove routine application glue.
19. Schema, dialect, and Plan evolution are explicit and versionable.
20. Workbench inspection uses the same semantic fabric and can explain physical execution.
21. Native Rust remains a first-class extension and escape hatch.
22. Common abstractions remain general-purpose rather than embedding product-specific ontologies.

## Decision summary

Runenwerk's long-term target is a **general-purpose typed semantic programming, database, and planning fabric**.

Plan IR is the common semantic representation. Relational, graph, temporal, expression, world, spatial, field, and owner-specific dialects provide domain-appropriate vocabulary. Providers lower Plans into specialized execution such as ECS, spatial indices, analytic or sampled fields, CPU/GPU compute, rendering, networking, persistence, or external services.

This model gives Runenwerk one coherent programming-language family while preserving independent semantic ownership and specialized execution across the Runen framework family.
