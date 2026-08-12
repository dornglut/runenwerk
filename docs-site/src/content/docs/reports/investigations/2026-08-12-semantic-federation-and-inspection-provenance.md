---
title: Semantic Federation and Inspection Provenance Investigation
description: Primary-source provenance, cross-domain pressure tests, rejected universal models, and synthesis behind Runenwerk's semantic-federation and physical-realization architecture.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-08-12
related_docs:
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../guidelines/authority-centered-boundary-architecture.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
---

# Semantic Federation and Inspection Provenance Investigation

## Purpose

This report records the research and pressure testing behind ADR 0018.

The investigation asks a deliberately difficult question:

> Can Runenwerk have one strong meta-abstraction that humans and tools can reason
> about across UI, fields, spatial systems, rendering, GPU execution, ECS, assets,
> networking, and future domains without sacrificing owner autonomy, specialized
> performance, or clear semantics?

The answer is **yes at the semantic-reasoning and inspection level, but not as one
universal runtime representation**.

The resulting direction is:

```text
one semantic grammar
many semantic authorities
explicit typed contracts and relationships
owner-local validity/version facts
consumer-specific compatibility/admission
many physical realizations
specialized execution
optional federated inspection
```

This report is evidence, not implementation authority. ADR 0018 owns the accepted
decision. Any future reflection/interchange API or shared runtime requires separate
authority.

## Research method

### Evidence hierarchy

The external comparison prefers:

1. current official technical specifications and API documentation;
2. current project architecture/reference documentation;
3. current canonical Runenwerk and standalone-framework architecture;
4. accepted Runenwerk ADRs;
5. current source/PR evidence where it demonstrates a concrete owner contract;
6. inference, explicitly labeled as synthesis rather than established external fact.

### Attribution discipline

For every external system, this report distinguishes four questions:

```text
Established mechanism
  What does the external system actually do?

Runen adaptation
  Which architectural lesson is useful here?

Explicit rejection
  Which assumption or mechanism must not be imported universally?

Runen-specific synthesis
  What combination or boundary selection is specific to this architecture?
```

The report does not claim that MVCC, materialized views, dialect lowering, scene
indexes, incremental dependency tracking, dataflow, columnar memory, render graphs,
controllers, or backpressure are Runen inventions.

## Current Runen authority baseline

The research begins from accepted Runen authority rather than from an external pattern.

ADR 0014 requires independently useful peer frameworks and rejects a universal
`RunenCore` or convenience meta-framework. Framework-owned identities remain
framework-owned and Runenwerk adapters perform cross-framework mapping.

ADR 0017 adds the family law:

```text
One semantic invariant set has one authority.
```

and establishes:

- explicit owner contracts for foreign reads;
- no universal world transaction/revision/snapshot;
- consumer-owned admission of compatible multi-authority input sets;
- distinct Command/Query/Event/Projection/Snapshot/Product/Cache/etc. roles;
- clean/full equivalence for incremental systems;
- graph-semantic separation;
- capability/support/requirement/policy/authority distinctions;
- strong shared-extraction gates;
- progressive disclosure.

The task here is not to weaken those constraints. It is to determine whether they can
be complemented by a **positive common reasoning model**.

# Part I — Runen owner pressure tests

## RunenSDF — field and capability-sensitive query

Standalone RunenSDF is a strong negative test for universal relational or graph
models.

Its durable semantics center on:

```text
signed field values
validated bounds and transforms
exact/conservative numerical capability
sampling
ray/projection/sweep queries
hit / terminal miss / error outcomes
```

A field can be represented as a relation of sample points, but that would not capture
its primary semantic contract. An analytic field may never be fully enumerated. A
sampled grid is one physical approximation/realization. A GPU implementation may use a
program and resources rather than host-visible values.

### Fit to the candidate grammar

```text
Authority
  RunenSDF numerical and query semantics

Contract
  field/sample/query/capability contracts

Operation
  Query / Derive / future Adapt or Realize

Validity
  bounds, finite values, capability, numerical guarantees

Realization
  analytic CPU, sampled structure, compiled/GPU form, etc.

Effect
  normally absent from pure query semantics
```

### Pressure result

A universal `Table`, `Relation<T>`, or `DataflowNode` would distort this owner.

The field instead proves the semantic-vs-physical law:

> An implementation representing the same conceptual field source cannot claim an
> exact-distance semantic contract if its realization only provides a weaker
> approximation outside the declared tolerance.

The correct outcome is weaker capability, explicit approximation, adaptation, or
rejection.

## RunenUI — retained hierarchy, products, interaction, and commit

Standalone RunenUI has a fundamentally different shape:

```text
application state
  -> transient View/Element hierarchy
  -> keyed reconciliation
  -> persistent mounted runtime tree
  -> computed style
  -> layout
  -> semantic tree + hit-test scene + paint scene
  -> host/backend integration
```

The mounted tree owns runtime identity, lifecycle, focus, interaction, invalidation,
and other retained mechanics. Semantic, layout, hit-test, paint, and diagnostics are
deliberately distinct products rather than one universal node graph.

### Fit

```text
Authority
  application state owner, mounted runtime, semantic publication owner, host

Contract
  View/Element input, Actions, semantic contributions, layout/paint products,
  host requests

Operation
  Derive, Reconcile as owner-local retained-identity algorithm, Commit, Execute/Effect

Validity
  mounted generations, surface identity, publication revisions, target lineage

Realization
  retained tree, layout structures, semantic arena, paint representation

Effect
  application updates, host work, external/native integration
```

### Pressure result

UI disproves the idea that one semantic contract must have one universal inspection
shape.

One UI publication can usefully expose:

```text
Tree      mounted/semantic hierarchy
Table     node properties and diagnostics
Graph     semantic relationships/focus/causal links
Timeline  revisions, actions, input, traces
```

Therefore table/tree/graph/timeline are better modeled as **inspection projections**
than as mutually exclusive platform semantic types.

## RunenSpatial — relation/set derivation plus real reconciliation

RunenSpatial owns spatial identity/addressing, hierarchy, deterministic demand, and one
host-defined availability class per streaming controller.

The design intentionally separates:

```text
desired
availability
operation
last transition failure
```

A demand source publishes complete desired coverage. Multiple sources form effective
demand deterministically. Streaming then compares desired demand with independently
reported backend availability.

### Fit

```text
Authority
  RunenSpatial address/demand/availability invariants

Contract
  demand snapshots/transactions, requests, backend events, status

Operation
  Derive effective demand; genuine desired/observed Reconcile loop

Validity
  namespace, checked coordinate/radius facts, request identity, controller state

Realization
  maps/queues/index structures chosen privately

Effect
  actual IO remains host/backend owned
```

### Pressure result

Relational/set reasoning is highly natural for demand:

```text
source -> desired cells
combine/filter/rank -> effective demand
```

But the same domain still needs specialized hierarchy/address arithmetic and a state
machine for availability transitions.

It also proves why shared nouns are dangerous: `Resident` here means only that one
controller's opaque availability transition completed. It is not world-product
certification, renderer residency, or GPU realization.

## RunenRender — strongest database/view/change-set proving domain

RunenRender's canonical target is particularly compatible with database-derived ideas:

```text
RenderSceneStore
  -> commit(RenderSceneUpdate)
      -> RenderSceneCommit
          - RenderSceneSnapshot
          - RenderChangeSet

RenderSceneSnapshot
+ RenderRequest
+ RenderInputSet
  -> RenderMethod
      -> RenderPlan
          -> AdmittedRenderPlan
              -> RenderWorkSet
                  -> RunenGPU
```

The scene store may internally use:

```text
persistent tables
generation arenas
structural sharing
copy-on-write pages
paged scene databases
lazy query views
compiled dense renderer tables
```

without exposing any one storage representation as renderer semantic authority.

### Fit

```text
Snapshot
  immutable owner-local semantic read cut

ChangeSet
  correctness-relevant incremental evidence

Projection/View
  natural for querying scene subsets and derived state

Product
  owner-defined semantic output where applicable

Plan
  request-local semantic image-formation decision

Admission
  execution/environment/representation compatibility

Realization
  representation providers, acceleration structures, GPU resources/programs
```

### Pressure result

RunenRender is the strongest reason to adopt database vocabulary while also being a
strong reason **not** to make the platform a database. Its architecture explicitly
requires freedom to switch between physical store organizations and representation
methods while preserving scene semantics.

## RunenGPU — strongest logical/physical stress test

RunenGPU deliberately separates backend-neutral logical resource, program, interface,
work, and access contracts from private backend realizations.

WGPU realization is bound to exact admitted context/device generation. G3's work graph
has specialized semantics around resource access, initialization, hazards, causality,
and execution preparation.

### Fit

```text
Authority
  context/device-generation realization and GPU execution invariants

Contract
  logical resources, program/interface descriptors, work, results/status

Operation
  Admit, Realize, Execute

Validity
  context identity, device generation, capabilities, format/alignment,
  resource/program compatibility

Realization
  private WGPU resources/programs/layouts/pipelines/commands

Effect
  submission, mapping/readback, surface/native/backend interaction
```

### Pressure result

RunenGPU proves that:

```text
logical semantic contract != backend object
```

and that generic inspection cannot require host-visible payloads.

A Workbench should be able to inspect resource identity, descriptor, lineage,
realization affinity, pressure, work graph, submission status, and diagnostics without
forcing a texture or buffer to be copied to the CPU. Payload readback is a separate
explicit capability/effect.

## RunenECS and RunenScheduler — collection/query semantics with storage/execution split

Current RunenECS extraction design owns entity/component/resource/world lifecycle,
queries/filters, deferred structural mutation, and ECS access declarations. It
explicitly refuses to freeze archetypes, dense columns, sparse sets, or another
storage organization into the durable semantic contract.

Scheduler ownership is separately constrained:

```text
runen_schedule
  neutral dependency/access planning

RunenECS
  systems, world/resource access, command semantics

Runenwerk
  application/frame/tick/product lifecycle policy
```

### Fit

ECS is naturally inspectable as collections/relations, but storage remains private.
System access declarations also provide useful metadata for planning and inspection.

### Pressure result

The correct lesson is not `everything is ECS`. It is:

> Canonical typed APIs can often carry enough information for safety, planning, and
> inspection without asking authors to write an independent metadata graph.

This becomes important for the future Workbench metadata-truth doctrine.

## Asset/product pipeline — source DAG, effectful execution, ratification

The asset architecture separates semantic planning from host execution:

```text
Source
  -> AssetSourceDescriptor
  -> ImportSettings
  -> deterministic ImportPlan
  -> ImportJob / FieldProductJob execution
  -> candidate
  -> owner/domain ratification
  -> catalog publication / formed product
```

### Pressure result

Assets provide a strong counterexample to the slogan `everything is a pure transform`.

The following are materially different:

```text
Derive / Plan
  semantic description of what should be formed

Execute / Effect
  host IO, external importer process, file writes

Candidate
  result not yet accepted as owner truth

Ratify / Commit
  semantic owner acceptance

Product
  published owner-defined result

Persist / Materialize
  retention/physical-lifetime decision
```

Materializing an import output does not automatically ratify it. A persisted artifact
is not automatically source authority.

## Networking — projection, stream, delta, compatible publication cut

Current networking interest/streaming design uses per-connection interest and retains
explicit full-resynchronization behavior when required history is missing or pruned.

### Fit

```text
Projection
  connection-specific relevant state

Stream / delta
  transfer/change mechanism

Validity
  connection/tick/history context

Admission
  publication-specific compatible cut

Pressure
  budgeted/bounded replication and transport

Effect
  network delivery
```

### Pressure result

A renderer frame cut and a network publication cut may both combine values from
multiple owners but use legitimately different revision/time/freshness requirements.

This reinforces:

```text
versions are owner-local
compatibility is contextual
admission is consumer-owned
```

# Part II — external inspiration and provenance

## MLIR — dialects, interfaces, and explicit conversion

### Primary sources

- <https://mlir.llvm.org/docs/DefiningDialects/>
- <https://mlir.llvm.org/docs/Interfaces/>
- <https://mlir.llvm.org/docs/DialectConversion/>
- <https://mlir.llvm.org/docs/Dialects/Builtin/>

### Established mechanism

MLIR uses dialects to define domain-specific operations, types, and attributes. Dialect
interfaces and operation/type/attribute interfaces let generic infrastructure ask
common questions without hard-coding every dialect-specific object. Dialect conversion
makes transformation explicit through legality, rewrite patterns, type conversion, and
materialization.

MLIR's declarative definitions can also generate boilerplate and documentation from a
canonical domain description. Its builtin dialect is intentionally small relative to
its reach: universally available constructs carry a high compatibility and design cost.

### Runen adaptation

Runen borrows the architectural pattern:

```text
shared structural/reasoning grammar
  !=
domain-owned semantics
```

and the idea that cross-domain translation should be explicit rather than hidden in a
universal object model.

The interface lesson is also valuable for tooling: a generic inspector should ask an
owner-supported question rather than reaching into private representation.

### Explicit rejection

Runen does not adopt one executable MLIR-like IR for UI state, fields, spatial
controllers, rendering, GPU execution, networking, and assets.

MLIR earns a shared IR because its participating objects are compiler representations.
Runen includes persistent mutable authorities, state machines, external effects,
streams, runtime resources, and controllers that are not naturally one compiler IR.

### Runen-specific synthesis

Use the MLIR lesson **above** runtime representation:

```text
common architecture questions
+ owner-specific vocabulary
+ explicit adapters/realizations
+ optional generic inspection interfaces later if proven
```

rather than one universal executable operation graph.

## PostgreSQL — MVCC, snapshots, views, and materialized views

### Primary sources

- <https://www.postgresql.org/docs/current/mvcc.html>
- <https://www.postgresql.org/docs/current/rules-views.html>
- <https://www.postgresql.org/docs/current/rules-materializedviews.html>

### Established mechanism

PostgreSQL uses multi-version concurrency control so readers can operate against a
coherent database-visible state without observing arbitrary in-place mutation. Views
represent derived relational definitions. Materialized views persist query results and
retain the defining query so the result can later be refreshed.

### Runen adaptation

The useful lessons are:

```text
coherent immutable/versioned read cuts
logical derived view != retained materialized result
read stability can be separated from writer progress
```

RunenRender's immutable scene snapshot and UI publication models already demonstrate
similar owner-local advantages without being databases.

### Explicit rejection

PostgreSQL owns one database transaction/version universe. Runenwerk intentionally has
multiple semantic authorities. Importing one global transaction or world revision would
collapse those owner boundaries.

### Runen-specific synthesis

The federated law is stronger for Runen's topology:

```text
Versions are owner-local.
Compatibility is contextual.
Admission is consumer-owned.
```

A renderer, network publisher, editor inspector, and offline reproducibility tool may
each define a different legal cross-authority cut.

## Materialize — views, maintained results, and indexes

### Primary sources

- <https://materialize.com/docs/concepts/views/>
- <https://materialize.com/docs/concepts/indexes/>

### Established mechanism

Materialize distinguishes ordinary logical views from maintained results and indexes.
Indexes can keep query results in memory for efficient serving. Materialized views keep
incrementally updated results in durable storage. These choices affect retention and
serving cost without being the same conceptual operation as the view definition.

### Runen adaptation

Runen adopts the distinction among these questions:

```text
View
  What is the derived read model?

Materialization
  Is a derived result retained?

Index
  What retained structure accelerates access?
```

### Explicit rejection

Runen does not define `Product` as a synonym for materialized view and does not require
one incremental-view engine for all derived state.

### Runen-specific synthesis

Runen adds semantic ownership as a separate axis:

```text
Product
  owner-defined semantic publication

Materialization
  retention choice

Index
  access-cost choice

Cache
  discardable reuse choice
```

A Product may be lazy, materialized, indexed, cached, persisted, or GPU-resident.
Those physical/retention choices do not by themselves determine semantic authority.

## OpenUSD / Hydra Scene Index — queryable views and change notices

### Primary sources

- <https://openusd.org/release/api/class_hd_scene_index_base.html>
- <https://openusd.org/release/api/class_hd_single_input_filtering_scene_index_base.html>

### Established mechanism

`HdSceneIndexBase` is an abstract queryable scene-data interface. It can also notify
observers when prims are added, removed, dirtied, or renamed. Filtering scene indexes
observe input indexes and provide transformed views. Scene-index display names and tags
support user interfaces that inspect scene-index chains or graphs.

An important performance detail is also explicit: a scene index can tell whether it is
observed and avoid preparing notices when no observer exists.

### Runen adaptation

Hydra is the strongest graphics-domain evidence for:

```text
queryable owner views
change notices alongside queries
transformed/filtering view chains
inspection of intermediate representations
optional observation work
```

### Explicit rejection

Hydra operates in a broad but still graphics-scene-oriented semantic universe. Runen
does not force UI runtime state, arbitrary mathematical fields, GPU execution work,
networking, schedulers, or assets into scene prims.

### Runen-specific synthesis

The Workbench should federate **multiple owner-selected inspection projections** rather
than constructing one global Scene Index:

```text
RunenUI        tree / table / graph / timeline
RunenRender    scene tables / relationships / changes
RunenGPU       resource table / work graph / timeline
RunenSDF       field sampler / capability record
RunenSpatial   demand / availability projections
...
```

The Workbench composes the inspection experience while owners retain semantics.

## Salsa — tracked queries and memoized recomputation

### Primary sources

- <https://salsa-rs.github.io/salsa/how_salsa_works.html>
- <https://salsa-rs.github.io/salsa/reference/algorithm.html>

### Established mechanism

Salsa represents application inputs and tracked functions in a database. Tracked
functions memoize results, record dependencies, and use revision/change information to
avoid recomputation when inputs have not semantically changed. The computational model
is strongest when tracked functions behave deterministically from declared inputs.

### Runen adaptation

Salsa provides evidence that owner-local derived systems can gain substantial value
from:

```text
explicit dependency tracking
memoization
revision-aware reuse
semantic change pruning
```

### Explicit rejection

Runen does not create one Salsa database or revision universe for all authorities and
does not force effectful state machines, host IO, or GPU execution into pure tracked
queries.

### Runen-specific synthesis

The family standardizes the correctness law instead of the mechanism:

```text
incremental(admitted input X)
  ==
clean/full(admitted input X)

under owner-defined equality/tolerance
```

Salsa can remain a candidate implementation for domains whose semantics fit it.

## Bazel Skyframe — dependency completeness and clean rebuild safety

### Primary sources

- <https://bazel.build/reference/skyframe>
- <https://bazel.build/versions/6.0.0/reference/skyframe>

### Established mechanism

Skyframe models immutable `SkyValue`s, `SkyKey`s, `SkyFunction`s, and an explicit node
dependency graph. Functions are expected to obtain inputs through declared dependency
lookups; undeclared input reads can lead to incorrect incremental builds. With complete
dependency information, Bazel can invalidate only affected reverse dependencies and
run independent functions in parallel.

Bazel's documentation also describes a deliberate reluctance to rely on arbitrary
in-place incremental mutation when proving equivalence to a clean rebuild is difficult.
Where possible, expensive steps are decomposed into smaller cleanly rebuildable units.

### Runen adaptation

The useful family lessons are:

```text
dependency evidence must be complete enough for the claimed incremental correctness
hidden input dependencies are dangerous
clean reconstruction is a powerful safety/reference model
```

### Explicit rejection

Runen does not encode all runtime interaction as `SkyKey`/`SkyValue` nodes or make one
build-style DAG the owner of UI, simulation, GPU, rendering, and networking behavior.

### Runen-specific synthesis

Asset/product compilation and selected derived systems may use Skyframe-like ideas.
The shared family rule remains about evidence and clean/full correctness rather than one
incremental evaluator.

## Differential Dataflow and Timely — diffs, logical time, and arrangements

### Primary sources

- <https://timelydataflow.github.io/differential-dataflow/chapter_5/chapter_5.html>
- <https://timelydataflow.github.io/differential-dataflow/>

### Established mechanism

Differential Dataflow models changing collections through data associated with logical
time and differences. Its arrangements maintain indexed forms of changing collections
that operators can share instead of independently rebuilding equivalent indexes.
Timely/Differential's model demonstrates that serious iterative/incremental dataflow
requires explicit progress and logical-time semantics rather than treating feedback as
an ordinary timeless graph edge.

### Runen adaptation

Borrow the lessons:

```text
change evidence needs context
shared maintained indices can remove repeated work where semantics genuinely match
real iteration/feedback needs explicit temporal/progress semantics
```

### Explicit rejection

Runen does not introduce universal logical time, progress frontiers, arrangements, or a
family-wide Differential runtime.

Pure queries, descriptor normalization, UI commit, GPU resource admission, and many
other operations do not benefit from paying that cost.

### Runen-specific synthesis

ADR 0017's explicit feedback classification remains the family model:

```text
temporal feedback
bounded/fixed-point iteration
reconciliation
interaction loop
distributed prediction/correction
```

A domain may choose Differential-like execution only where its semantic/time model
actually fits.

## Apache Arrow — physical-layout tradeoffs and narrow interchange

### Primary sources

- <https://arrow.apache.org/docs/format/Columnar.html>
- <https://arrow.apache.org/docs/format/CDataInterface.html>
- <https://arrow.apache.org/docs/format/CDeviceDataInterface.html>

### Established mechanism

Arrow specifies a language-neutral columnar memory format designed around goals such as
sequential locality, constant-time random access, SIMD/vectorization friendliness, and
zero-copy sharing. These goals make deliberate tradeoffs: mutation is comparatively
expensive.

The Arrow C Data Interface is deliberately smaller than the complete Arrow implementation
and lets independent libraries exchange Arrow-formatted data without requiring a shared
implementation dependency. The C Device Data Interface extends this concept to
device-resident memory so data can be exchanged without unnecessarily moving buffers
through CPU memory.

### Runen adaptation

Arrow strongly supports the architectural principle:

> Physical representation is a workload decision rather than semantic identity.

It also demonstrates that useful interoperability can be narrower than a complete
shared runtime/library.

### Explicit rejection

Runen does not standardize all data as Arrow, all data as columnar, or all Workbench
projections as tables.

It also rejects the assumption that generic inspection requires CPU-visible bytes.

### Runen-specific synthesis

A future table-oriented Workbench projection might optionally support Arrow-like bulk
interchange if two real owners prove the need. A GPU-oriented inspector might later use
an explicit device-aware transfer capability.

Those would be Level-B implementation decisions. The family-wide Level-A rule is only:

```text
semantic contract != physical realization
```

## Substrait — compute semantics separate from physical representation

### Primary sources

- <https://substrait.io/about/>
- <https://substrait.io/spec/specification/>
- <https://substrait.io/extensions/>
- <https://substrait.io/relations/basics/>

### Established mechanism

Substrait defines a cross-language specification for relational data-compute operations
and their semantics. Its project description explicitly contrasts its role—what should
be done to data—with Arrow's standardized memory representation.

Substrait also defines several extension levels and recommends choosing the least
powerful extension mechanism that solves a problem because more general custom
relations reduce interoperability.

Its project rationale discusses a high bar for adding common features across major
independent data technologies, with extension points preventing the common core from
becoming a bottleneck.

### Runen adaptation

Useful lessons are:

```text
semantic interoperability can be distinct from physical representation
common vocabulary should avoid one implementation's jargon
extensions should be explicit
prefer the least powerful common mechanism sufficient for the problem
common-core additions need multiple independent proving systems
```

### Explicit rejection

Substrait is intentionally relational. Runen does not adopt relational algebra as the
universal semantics of fields, UI hierarchy, GPU work, assets, controllers, or effects.

Runen also does not accept one serialized cross-domain executable plan format.

### Runen-specific synthesis

Runen's semantic federation is intentionally broader and less executable. It provides a
small language for ownership, contracts, relationships, validity, realization, and
effects across heterogeneous domain shapes.

If a future relational Workbench subsystem needs portable query plans, Substrait may be
reevaluated locally rather than becoming the platform meta-IR.

## Bevy ECS — typed access declarations without a second authoring graph

### Primary sources

- <https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParam.html>
- <https://docs.rs/bevy/latest/bevy/ecs/system/struct.Query.html>

### Established mechanism

Bevy systems are commonly ordinary typed Rust functions. `SystemParam` values declare
how systems access ECS world data. `SystemParam::init_access` registers accesses used by
a parameter and must reject conflicting access. `Query` provides typed selective
component access without requiring arbitrary direct `World` access.

### Runen adaptation

The important lesson is ergonomic and architectural:

> If canonical typed APIs already encode access or semantic facts, derive planning or
> inspection metadata from those contracts rather than forcing users to author a
> second graph that can drift.

### Explicit rejection

Runen does not adopt ECS as the universal application or Workbench ontology and does not
couple semantic federation to Bevy's `World` or storage model.

### Runen-specific synthesis

Future owner adapters should prefer:

```text
existing typed contracts/descriptors
  -> generated or derived inspection facts
```

and only duplicate metadata when no canonical owner contract can express the needed
fact.

## Unreal Render Dependency Graph — specialized semantics enable specialized optimization

### Primary source

- <https://dev.epicgames.com/documentation/en-us/unreal-engine/render-dependency-graph-in-unreal-engine>

### Established mechanism

RDG records render commands and declared resources into a graph, compiles that graph,
and uses the resulting whole-frame knowledge for validation, resource lifetime
management, transient memory aliasing, barriers/transitions, pass culling, asynchronous
compute scheduling, parallel command recording, and diagnostics/visualization.

The setup/execute split and pass restrictions give RDG sufficiently precise semantics
to perform those optimizations safely.

### Runen adaptation

RDG supports ADR 0017's specialized-graph law:

> Strong optimization comes from precise edge/resource meaning, not from making all
> relationships one generic graph.

### Explicit rejection

Runen does not promote render-resource dependencies into a universal family graph edge.
Semantic dependency, scheduler readiness, invalidation, containment, and GPU resource
hazard remain distinct.

### Runen-specific synthesis

The Workbench may display many graph-shaped projections while preserving the owner and
edge semantics of each graph species.

## Kubernetes controllers — desired/observed reconciliation

### Primary source

- <https://kubernetes.io/docs/concepts/architecture/controller/>

### Established mechanism

Kubernetes controllers are control loops that observe current cluster state and act to
move it toward desired state.

### Runen adaptation

Use `reconciliation` only where:

```text
desired state
and
observed/actual state
```

can change independently.

RunenSpatial streaming/backend availability is a natural example.

### Explicit rejection

A pure deterministic transformation such as source normalization, shader parsing, or
product formation is not reconciliation simply because it has an input and output.

Runen does not create one universal controller runtime.

### Runen-specific synthesis

`Reconcile` is treated as a temporal pattern composed from observations, decisions, and
effects rather than as one generic semantic edge.

## Reactive Streams — narrow interoperability and backpressure

### Primary source

- <https://github.com/reactive-streams/reactive-streams-jvm>

### Established mechanism

Reactive Streams standardizes asynchronous stream interaction across implementations
with mandatory nonblocking backpressure so receivers do not require unbounded buffers.
The specification deliberately focuses on the boundary protocol rather than defining a
complete universal stream-transformation library.

### Runen adaptation

The useful lesson is the **level of standardization**:

```text
standardize the narrow pressure/safety contract
leave internal execution/transformation implementation-specific
```

This supports Runen's family-wide pressure doctrine.

### Explicit rejection

Not every Runen change path becomes a `Publisher`/`Subscriber`, and no universal stream
runtime is authorized.

### Runen-specific synthesis

Each owner may use queues, demand, budgets, polling, GPU submission capacity, streaming
requests, or another mechanism while still exposing bounded pressure and terminal
outcomes where the boundary requires them.

## seL4 — capability as actual authority

### Primary source

- <https://docs.sel4.systems/Tutorials/capabilities.html>

### Established mechanism

In seL4, a capability is an unforgeable token carrying rights to an object; possession
of a suitable capability is actual authority to invoke permitted operations.

### Runen adaptation

The important lesson is terminological precision:

```text
support/capability fact
  !=
consumer requirement
  !=
host/security policy permission
  !=
authority to mutate/control
```

### Explicit rejection

Runen does not import a kernel capability-object model into ordinary feature-support
APIs or create one family-wide capability registry.

### Runen-specific synthesis

ADR 0017's capability terminology keeps owner-specific capability facts while refusing
to infer permission or authority from support alone.

# Part III — cross-source synthesis

## Recurring external separation pattern

The strongest common lesson is not one specific mechanism. It is **separation of
semantic level from physical/execution level**.

| System | Semantic/logical concern | Separate physical/execution concern | Runen pressure |
| --- | --- | --- | --- |
| MLIR | dialect operations/types and legality | lowering/target representation | keep owner semantics distinct from realization |
| PostgreSQL | view/query and transaction-visible state | storage/materialized result | view/snapshot does not dictate physical form |
| Materialize | logical view | materialization/index placement | retention and access acceleration are orthogonal |
| Hydra | queryable scene view | concrete scene-index implementations/filter chains | inspect transformed views without stealing source authority |
| Salsa | tracked query semantics | memoized dependency database execution | incremental mechanism can stay owner-local |
| Skyframe | dependency-complete immutable nodes | incremental evaluator | clean rebuild remains a safety reference |
| Differential | changing logical collections | arrangements/progress runtime | advanced incremental execution has real runtime cost |
| Arrow | logical schema + columnar interchange contract | exact buffer/device layout | physical representation is workload-specific |
| Substrait | relational compute semantics | Arrow/engine execution and physical plan | semantic interoperability need not dictate memory format |
| Bevy | typed system/query access | ECS storage/scheduler implementation | derive metadata from canonical typed contracts |
| RDG | render pass/resource semantics | compiled barriers/lifetimes/scheduling | specialized graph meaning earns specialized optimization |
| Kubernetes | desired/observed control semantics | controller execution | reconciliation is a specific temporal pattern |
| Reactive Streams | boundary demand/backpressure contract | stream implementation/operators | narrow interoperability beats universal runtime |
| seL4 | authority-bearing capability | kernel object implementation | support fact and authority are not interchangeable |

## What is established versus what is synthesis

The following are established ideas, not Runen inventions:

```text
versioned/snapshot reads
views and materialized views
indexes
memoization and incremental dependency tracking
logical time and dataflow diffs
columnar data and zero-copy interchange
domain-specific IR/dialects
queryable scene indexes and change notifications
typed ECS access declarations
render graphs
controller reconciliation
backpressure
capability-based authority
```

The potential Runen contribution is the **boundary selection across a heterogeneous
engine/workbench family**:

```text
independent semantic authorities
  +
owner-local identities/revisions
  +
consumer-specific cross-authority admission
  +
common semantic reasoning questions
  +
semantic/physical realization separation
  +
owner-local incremental mechanisms under one correctness law
  +
specialized graph/time/execution semantics
  +
optional multi-projection Workbench inspection
  +
no mandatory common runtime/store/IR
```

This report calls that synthesis **semantic federation without runtime centralization**.

# Part IV — rejected universal models

## Universal relational/database model

### Strengths

- excellent queryability;
- mature filtering/join/aggregation vocabulary;
- snapshots and transactions;
- materialization and indexing;
- incremental-view maintenance;
- strong generic tooling.

### Failure against Runen

- analytic fields are not naturally finite relations;
- UI persistent identity and hierarchy have owner-specific reconciliation semantics;
- GPU work graphs need hazard/access semantics rather than generic joins;
- controller state machines and effects are not merely relational transforms;
- one database revision/transaction would violate independent authority versions;
- physical layout requirements vary dramatically across CPU/GPU/spatial/render paths.

### Disposition

Reject as universal ontology. Use relational techniques where the domain or inspection
projection is genuinely relational.

## Universal graph model

### Strengths

- can represent almost any connectivity;
- visually inspectable;
- enables generic traversal algorithms.

### Failure

Representability is not semantic equivalence.

```text
semantic dependency
!= scheduler readiness
!= GPU hazard
!= invalidation dependency
!= hierarchy
!= correspondence
```

A universal edge would either become semantically empty or accumulate a large tagged
union of domain-specific meanings.

### Disposition

Reject. Preserve ADR 0017 graph taxonomy and let the Workbench render graph projections
without owning edge semantics.

## Universal dataflow runtime

### Strengths

- explicit input/output dependencies;
- incremental propagation;
- parallel scheduling;
- potential optimization/fusion;
- inspection/provenance.

### Failure

A serious runtime needs answers about:

```text
time/progress
feedback
state retention
indexing
backpressure
scheduling
side effects
completion
```

Those answers differ substantially between UI, GPU work, networking, assets, fields,
and pure compilation.

### Disposition

Reject as family runtime. Domain-local dataflow remains valid when semantics justify it.

## Domain Program as universal ontology

### Strengths

- excellent for authored, durable, versioned intent;
- natural compiler/evaluator structure;
- source maps and diagnostics;
- host-independent artifacts.

### Failure

The following are not naturally durable authored programs:

```text
GPU resource handle
input event
mounted UI runtime state
spatial availability
renderer snapshot
network stream
query result
scheduler readiness
```

### Disposition

Keep Domain Program as a strong specialized pattern, not the platform root type.

## ECS as universal ontology

### Strengths

- data-oriented processing;
- explicit component queries;
- parallel access reasoning;
- efficient simulation storage.

### Failure

Assets, renderer semantic scene formation, GPU resource lifetime, compiler/source
programs, UI hierarchy, and networking do not become clearer merely by encoding them as
entities/components.

### Disposition

Use ECS where ECS semantics fit. Do not make it the Workbench ontology.

## Universal scene graph / Scene Index

### Strengths

- excellent hierarchy/scene inspection;
- proven graphics tooling;
- change observation/filtering.

### Failure

Too graphics-centric for generic fields, scheduler work, networking, build products,
GPU execution, and arbitrary application authority.

### Disposition

Borrow Hydra's inspection architecture, not its universal data shape.

## Universal compiler IR

### Strengths

- typed operations;
- legality/verification;
- lowering;
- optimization;
- inspection/provenance.

### Failure

Persistent mutable authorities, state machines, external resources, streams, effects,
and desired/observed controllers are not merely compiler representations.

### Disposition

Borrow MLIR's dialect/interface/lowering discipline. Reject one universal executable IR.

## Universal event sourcing

### Strengths

- auditability;
- replay;
- accepted-fact history;
- provenance.

### Failure

Expensive/awkward for transient high-frequency state and unnecessary for pure derived
computation. It also does not replace physical realization, scheduling, or query
semantics.

### Disposition

Use in domains where event history is itself required authority; reject as universal.

# Part V — candidate semantic federation model

## Six recurring architecture questions

The cross-domain tests converge on:

```text
AUTHORITY
CONTRACT
OPERATION / RELATIONSHIP
VALIDITY / PROVENANCE
REALIZATION
EFFECT
```

The power of this model is that it does **not** require every domain to have one object
for every question.

A pure local transform may only need:

```text
Contract + Operation
```

A cross-authority GPU-backed render invocation may need all six.

A Workbench can ask these questions consistently even when the owner answers with very
different domain values.

## Operation/relationship vocabulary

The useful common verbs are:

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

These are architecture vocabulary, not API names.

They complement rather than replace contract roles such as `Command`, `Query`,
`Snapshot`, `Product`, `Plan`, and `Work`.

For example:

```text
Query
  contract role

Observe
  relationship in which a foreign consumer uses that query/read contract
```

or:

```text
RenderPlan
  owner-specific Plan contract

Admit
  operation that checks the plan against execution/environment facts
```

This avoids a type hierarchy that would collapse different questions.

## Roles and facets are orthogonal

A recurring architecture failure mode is treating every useful noun as another linear
stage.

The investigation instead distinguishes:

### Semantic/boundary roles

```text
Snapshot
Product
Projection
Plan
Work
Status
...
```

### Derivation/retention/physical facets

```text
authoritative vs derived
lazy vs retained/materialized
indexed vs unindexed
cached vs uncached
persisted vs transient
CPU vs GPU vs remote vs opaque
```

This permits accurate combinations without inventing another universal `Reality` for
every cross-product.

## Versions and compatibility

The combined database and Runen pressure tests support:

```text
Versions are owner-local.
Compatibility is contextual.
Admission is consumer-owned.
```

This avoids the false simplicity of a global frame/snapshot while still requiring
consumers to state what coherence means.

## Inspection joins are deliberately weaker than runtime joins

A future Workbench may offer database-like correlation and navigation:

```text
show render objects derived from world entity X
show GPU resources realizing render representation Y
show stale representations grouped by source revision
```

Those operations are **inspection correlations**.

They do not prove that the displayed values form a valid current render or simulation
input. Runtime admission remains owner/consumer semantics.

This distinction is essential if the Workbench gains powerful generic table/query
features later.

# Part VI — Workbench inspection model

## Why projections rather than one storage schema

The owner tests show that a contract may naturally support several useful views.

Directional inspection projection classes are:

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

These are presentation/query capabilities, not domain ontology.

## Examples

### RunenUI

```text
Tree
  mounted and semantic topology

Table
  node properties, bounds, support, diagnostics

Graph
  semantic relationships, focus or causal links

Timeline
  input/action/publication/trace revisions
```

### RunenSpatial

```text
Table
  demand sources, effective demanded cells, availability status

Tree/custom spatial view
  hierarchy and coverage

Timeline
  requests/backend events/transitions
```

### RunenRender

```text
Table
  objects, materials, emitters, representations

Graph
  typed relationships or plan dependencies

Timeline
  revisions/change sets/sessions

Image/custom view
  semantic render outputs and diagnostic visualization
```

### RunenGPU

```text
Table
  resources/programs/pipelines/submissions

Graph
  work/access/hazard structure

Timeline
  submission/progress/completion

Resource/custom
  memory, pressure, context/generation affinity
```

### RunenSDF

```text
Field
  sampled requested region

Record
  capability, bounds, numerical configuration

Report/Table
  query results and diagnostics
```

## Identity

The Workbench must not recreate a universal object ID.

Use:

```text
inspection-session-local handle
```

for navigation/UI bookkeeping.

Owner identity remains owner-local. Cross-owner lineage is a relationship:

```text
(owner A, id A)
  -> provenance/correspondence
(owner B, id B)
```

No persistence or wire stability is implied.

## Lazy and bounded access

A generic inspector must be designed around scale and representation locality.

Target when disabled:

```text
no mandatory copy
no mandatory graph construction
no mandatory registration walk per frame
no CPU readback
no eager field/table materialization
no unbounded history
```

Target when enabled:

```text
lazy
paged/ranged
bounded
pressure-aware
copy-aware
device/location-aware
explicit completeness/drop evidence
```

This is more important than selecting a generic data format early.

## CPU/GPU transfer boundary

Arrow's C Device Interface provides useful evidence that device-aware exchange can be
standardized without pretending device and CPU memory are identical.

For Runen, the more conservative rule is:

```text
metadata/lineage inspection
  does not imply payload readback

payload inspection
  requires explicit owner capability and cost
```

A texture inspector may show dimensions, format, generation, provenance, memory
pressure, last use, and producer/consumer work without ever reading pixels.

## Metadata truth

A Workbench reflection layer can become a serious maintenance defect if authors write
the real typed API and then manually duplicate the same facts in a separate metadata
schema.

Preferred order:

```text
canonical owner contract
  -> reuse directly
  -> derive/generate if needed
  -> Runenwerk sidecar adaptation
```

Bevy's typed access declarations and MLIR's declarative dialect definitions both show
that machine-readable tooling facts can come from canonical definitions instead of an
independent shadow model.

The exact Rust mechanism remains unaccepted.

# Part VII — extraction levels

## Level A — conceptual vocabulary/law

Examples:

```text
semantic vs physical realization
versions local / compatibility contextual
inspection join != admission
```

No runtime dependency is created.

Acceptance can come from several structurally different domain pressure tests.

## Level B — interoperability/reflection contract

A real API would incur:

```text
versioning
identity/lifetime design
allocation/copy cost
locking/concurrency behavior
serialization/interchange questions
compatibility burden
```

Therefore it needs at least two structurally different concrete proving adapters and a
separate accepted extraction design.

Possible future examples:

```text
peer-neutral inspection projection interface
bulk table projection transfer protocol
common read-only provenance interface
```

None are authorized by this report or ADR 0018.

## Level C — shared runtime mechanism

Examples:

```text
query planner/executor
incremental database
dataflow runtime
shared relation store
optimizer
shared scheduler/executor
```

These require multiple real domains already implementing materially identical
machinery, measured benefit, genuinely common time/storage/pressure semantics, and
benchmarked runtime cost.

Current evidence does not satisfy that bar.

# Part VIII — inspiration matrix

| Source | Established mechanism | Runen adaptation | Explicit rejection | Runen-specific synthesis |
| --- | --- | --- | --- | --- |
| MLIR | dialect-owned ops/types, interfaces, explicit conversion/legality | shared grammar with owner semantics and explicit lowering/adaptation | one universal executable compiler IR | meta-questions above heterogeneous runtimes |
| PostgreSQL | MVCC, snapshots, views, materialized views | coherent owner-local reads; view != retained result | global Runen transaction/revision | contextual cross-owner compatible cuts |
| Materialize | logical views, maintained results, indexes | View/Materialization/Index as separate questions | Product = materialized view; one incremental engine | semantic publication orthogonal to retention/acceleration |
| Hydra | queryable Scene Index, observers, filtering chains | lazy owner-selected projections and change notices | universal scene ontology | federation of heterogeneous projections |
| Salsa | tracked memoized queries and revisions | owner-local dependency/incremental implementation candidate | one Salsa DB/revision for all owners | family correctness law, local mechanism |
| Skyframe | immutable dependency graph, exact invalidation, clean rebuild emphasis | dependency evidence and clean safety reference | every runtime value as build node | incremental proof laws across different mechanisms |
| Differential/Timely | `(data,time,diff)`, arrangements, progress/iteration | explicit temporal semantics where required; shared maintained indices where proven | universal logical time/frontier runtime | feedback taxonomy with owner-local execution |
| Arrow | columnar physical format, C/C Device interchange | workload-specific physical layout; narrow zero-copy exchange | all Runen data columnar/host-visible | semantic contract independent of realization/locality |
| Substrait | implementation-neutral relational compute semantics + extensions | semantic interoperability separated from physical representation; cautious extensions | relational algebra as universal meta model | heterogeneous semantic federation rather than executable relation plan |
| Bevy ECS | typed SystemParam/Query access declarations | derive metadata from canonical typed APIs | ECS as platform ontology | inspectability without parallel authoring graph |
| Unreal RDG | specialized resource/pass graph compiled for validation/optimization | precise graph meaning enables strong specialized optimization | generic family edge/resource semantics | multiple owner graph species under one inspection vocabulary |
| Kubernetes | desired/observed controller loop | reconciliation only for independently evolving state | deterministic transformation = reconciliation | explicit temporal-pattern classification |
| Reactive Streams | narrow asynchronous protocol with mandatory backpressure | boundary-level pressure law | universal Publisher/Subscriber/runtime | common safety law with owner-specific mechanisms |
| seL4 | capability as unforgeable authority-bearing token | capability/support terminology warning | kernel capability-object system for general Runen APIs | support != requirement != policy != authority |

# Part IX — critical risk review

## Risk: architecture theology

A meta-model can become counterproductive if developers spend more time classifying
terms than solving domain problems.

Mitigation:

```text
local domain vocabulary wins
meta terms belong primarily at boundaries and tooling
remove distinctions that do not improve correctness, composition, inspection,
evolution, or performance reasoning
```

## Risk: hidden RunenCore through the Workbench

A central inspector will naturally attract registration, IDs, shared descriptors, and
runtime services.

Mitigation:

- Runenwerk-local sidecar adapters first;
- no peer dependency on inspection infrastructure without Level-B proof;
- no universal semantic ID/revision;
- no Workbench mutation authority over foreign internals.

## Risk: metadata drift

Mitigation:

- canonical typed owner contracts are primary;
- derive/generate where safe;
- manually duplicated facts need explicit consistency tests/ownership.

## Risk: hot-path type erasure or indirection

Mitigation:

- semantic federation is not a mandatory execution layer;
- ordinary owner APIs remain direct;
- inspector projections are optional/lazy;
- no `MetaOperation::execute` common path.

## Risk: database-like UI invents fake global consistency

Mitigation:

```text
inspection correlation/join != runtime admission
```

The Workbench can correlate lineage but cannot certify a render/network/simulation input
cut unless the owning consumer exposes that result.

## Risk: reflection ossifies private implementation

Mitigation:

- inspect published owner contracts or intentional derived projections;
- do not reflect arbitrary private structs as stable API;
- runtime inspection metadata implies no persisted/wire stability.

## Risk: universal projection taxonomy becomes another ontology

Mitigation:

- projection classes are optional tooling capabilities;
- multiple projections per contract;
- `Opaque/Custom` and no generic projection are valid outcomes.

## Risk: provenance becomes too expensive

Mitigation:

- owners expose only the lineage required for their correctness/tooling contracts;
- detailed histories may be bounded;
- provenance descriptors should avoid pinning unbounded source revisions or GPU
  resources.

# Part X — final neutral assessment

## Option A — stop at ADR 0017

This remains the lowest-risk architecture.

It preserves strict boundaries but leaves a real deficiency: the Workbench lacks one
positive cross-domain reasoning model, and humans/tools must understand every owner
before they can ask common questions about flow, validity, realization, and effects.

**Assessment:** safe, but incomplete for Runenwerk's long-term Workbench goal.

## Option B — resurrect broad Meta Kernel

It gives maximum uniformity quickly but creates strong central dependency and premature
implementation commitments.

**Assessment:** reject.

## Option C — universal database/dataflow system

It offers powerful views, queries, incremental maintenance, and tooling, but distorts
several current owners and imports one revision/time/storage/execution model.

**Assessment:** reject as universal; adopt local techniques.

## Option D — universal executable compiler/meta IR

It offers strong verification/lowering/tooling but fits compiler representations better
than mutable runtime authorities and external effects.

**Assessment:** reject as universal; borrow dialect/interface/lowering discipline.

## Option E — semantic federation + optional inspection projections

It provides one human reasoning language, preserves independent owners, leaves physical
layout/execution specialized, and gives a clear path to powerful Workbench inspection.
Its main risk is that conceptual Level-A vocabulary could creep into premature Level-B
or Level-C infrastructure.

**Assessment:** best-supported long-term direction if extraction levels remain explicit.

# Conclusion

The deep investigation supports the following architecture:

```text
One semantic grammar.
Many independent semantic authorities.
Explicit typed contracts and relationships.
Versions remain owner-local.
Compatibility is consumer-specific.
Semantic meaning is independent of physical realization.
Derived retention/index/cache choices remain orthogonal to semantic publication.
Incremental mechanisms remain owner-local under one clean/full correctness law.
Execution and effects stay specialized and explicit.
The Workbench federates optional owner-selected inspection projections.
No universal runtime/store/IR is implied.
```

The strongest inspiration is therefore **not one external system**.

The design combines:

```text
PostgreSQL / Materialize
  snapshot, view, materialization, index distinctions

MLIR / Substrait
  semantic specification separated from target/physical representation

Hydra
  queryable transformed views + change notices + inspection chains

Salsa / Skyframe / Differential
  incremental dependency/change lessons at different levels of power

Arrow
  explicit physical-layout tradeoffs and narrow zero-copy interchange

Bevy
  typed ergonomic access metadata

RDG
  specialized executable graph semantics

Kubernetes
  real desired/observed reconciliation

Reactive Streams
  narrow pressure interoperability

seL4
  precise authority/capability terminology
```

Runen's specific synthesis is to put those lessons at different architectural levels so
that none of them becomes the universal ontology of the engine/workbench family.

That synthesis is the evidence basis for ADR 0018.
