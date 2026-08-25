---
title: Runenwerk Visual Lab Product Architecture
description: Accepted product architecture for a host-neutral creative workbench that composes native visual studies and domain-owned capabilities without creating parallel semantic authority.
status: accepted
owner: workspace
layer: application / cross-domain product
canonical: true
last_reviewed: 2026-08-12
related_adrs:
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
related_designs:
  - ../active/editor-tool-suite-registry-and-workbench-host-design.md
  - ../active/field-visualizer-product-workflow-design.md
  - ../active/material-lab-and-material-preview-design.md
  - ../active/editor-procedural-content-and-simulation-workflow-plan.md
  - ./field-product-contracts-diagnostics-and-residency-design.md
  - ./execution-fabric-and-product-jobs-design.md
  - ../active/editor-asset-pipeline-and-content-workflow-design.md
  - ../active/runengpu-architecture-design.md
  - ../active/runenrender-decomposition-design.md
---

# Runenwerk Visual Lab Product Architecture

## Status and decision

Visual Lab is an accepted Runenwerk product architecture. It is a host-neutral creative
workbench for **procedural visual systems**: fields, implicit form, generative processes,
temporal or simulated systems, and their visual transformation, comparison, and
inspection.

Visual Lab is exploratory and non-authoritative by default. It may produce derived,
retained, or exported results and may orchestrate explicit proposals to owning domains,
but it does not acquire another domain's authority merely because a result is useful to
that domain.

This design accepts the product direction and durable ownership boundary. It does **not**
authorize implementation, a new production track, a new framework or repository,
RunenGPU roadmap changes, RunenRender activation, RunenUI cutover, a Visual Study file
format, or a new shared substrate. Live implementation requires a separately owned
GitHub issue and the accepted capabilities that implementation consumes.

The governing product rule is:

> Visual Lab owns creative workflow, product-local study meaning, and cross-domain
> composition intent. Existing domains and peer frameworks retain the semantic
> invariants and execution authority they already own.

The governing ergonomics rule is:

> Internal ownership boundaries stay strict; ordinary creative composition should feel
> direct. Safe derived use should not expose architecture ceremony. Durable authority
> transitions remain explicit.

This design applies ADR 0014 and ADR 0017. It does not create a new family-wide law.

## Purpose and product identity

Visual Lab should make Runenwerk's procedural, GPU, field, simulation, and rendering
capabilities directly useful for experimentation. A user should be able to create,
vary, simulate, inspect, compare, retain, capture, export, and deliberately apply visual
results without learning internal adapter, product, graph, ratification, or GPU
boundaries for the common path.

Visual Lab is not the generic home for every artistic feature. Its center is:

```text
procedural visual systems
    fields
    implicit form
    generative processes
    temporal / simulated systems
    visual transformation and inspection
```

A feature belongs naturally when its primary purpose is to explore, generate,
transform, combine, or visualize such a procedural system. Material Lab, Procgen, SDF,
and future simulation domains integrate where useful; Visual Lab does not absorb them.
Deep domain authoring remains with the domain-owned tool or workflow. Visual Lab may
host or compose those capabilities into one experiment-oriented experience.

The product architecture must survive changes in available execution, presentation,
renderer, and UI capabilities without rewriting source semantics.

## Architectural position

```text
                     VISUAL LAB
               Runenwerk creative product

                         |
          +--------------+--------------+
          |                             |
     Native Studies                Domain Capabilities
          |                             |
  product-local meaning             SDF / fields
                                    materials
                                    procgen
                                    future domains
          |                             |
          +--------------+--------------+
                         |
                  composition intent
                         |
                  explicit owner contracts
                         |
      +------------------+------------------+------------------+
      |                  |                  |                  |
   Observe          Derive / Adapt    Retain / Materialize   Apply / Commit
      |                  |                  |                  |
      +------------------+------------------+------------------+
                         |
                  Runenwerk integration
                         |
             +-----------+-----------+
             |                       |
         RunenGPU                  RunenRender
        generic GPU            semantic image formation
        execution                    |
             ^                       v
             +------------------- RunenGPU
```

Visual Lab is an application/product above domain and framework authorities. Runenwerk
remains the integration repository and owns cross-framework translation and product
policy according to ADR 0014.

## Composition model

Visual Lab's composition architecture follows one bounded principle also represented by
the active Tool Suite / Workbench Host design: a host may compose typed capabilities
without taking over their domain semantics. This accepted design fixes only that
product-level invariant. It does not accept by reference that active design's exact
manifests, API names, editor-shell ownership, migration state, or implementation
sequence.

A Visual Lab host may compose product-local native-study capabilities and domain-backed
capabilities. Composition controls product structure, available surfaces, host
capabilities, creative relationships, and workflow. It does not transfer source
semantics into the workbench shell.

### Visual Study composition intent

Visual Lab needs one conceptual answer to "what belongs to this creative experiment?"
without creating a universal semantic or execution graph.

A **Visual Study** is that composition role. Conceptually it may refer to:

```text
product-local study sources
owner-defined domain sources/products
parameter bindings
requested derivations/adaptations
requested outputs
owner-specific evaluation requests
execution-policy references
provenance
```

A Visual Study owns **composition intent and references**. It does not execute SDF,
material, procgen, simulation, rendering, or GPU semantics itself. Owner-defined
contracts, Runenwerk adapters, and existing execution paths realize those requests.

`VisualStudy` is conceptual architecture vocabulary in this design. This document does
not accept a serialized `VisualStudyDocument`, `AssetKind`, schema version, migration
contract, universal trait, or public runtime type. A future persistence contract
requires its own owner and evidence.

The first implementation should keep composition explicit and narrow. Do not build an
extensible compiler, universal operator graph, generalized composition runtime, or
universal adapter registry until multiple concrete consumers prove repeated neutral
structure under ADR 0017.

## Ownership

### Visual Lab owns

- creative workbench/product composition;
- Visual Study composition intent and references;
- product-local native-study semantics where no existing owner applies;
- active study/workspace and product session state;
- parameter-editing, run/reset/step, compare, inspect, retain, capture, export, and
  explicit apply workflows;
- app-local orchestration of owner-specific evaluation requests;
- output requests and product-level execution preferences;
- creative workflow sequencing, naming, capture/export policy, and presentation policy;
- explicit cross-domain Observe, Derive/Adapt, Retain/Materialize, and Apply/Commit
  orchestration;
- product-level current-result selection, comparison, and last-good presentation policy.

### Existing authorities retain

- **RunenSDF:** reusable analytic SDF/implicit-field mathematics and reference-query
  semantics;
- **Runenwerk SDF authoring/source owners:** authored SDF source meaning, project-facing
  mutation, and owner-specific ratification where applicable;
- **world-SDF / field-product owners:** formed world-SDF products, storage/query
  readiness, freshness, lineage, and consumer policy;
- **Field Visualizer:** applicable field-inspection workflow and presentation settings;
- **Material domain / Material Lab:** material source, graph, ratification, and material
  preview semantics;
- **Procgen:** deterministic procedural-generation source, lineage, lowering, and formed
  outputs;
- **future particle, animation, physics, and world-process domains:** their respective
  authored, simulation, and runtime semantics;
- **RunenGPU:** generic GPU capabilities, resources, validated work, execution,
  completion, readback, surfaces, pressure, and backend outcomes at their owning
  boundaries;
- **RunenRender:** semantic image formation, scenes, views, materials/media, methods,
  outputs, and lowering to RunenGPU;
- **RunenUI:** reusable renderer-neutral UI semantics and paint/hit products;
- **Runenwerk:** application lifecycle, host policy, cross-framework adapters, product
  policy, application/runtime sequencing, artifact policy, and integration evidence.

Visual Lab must not mirror or become writable parallel authority for any of these.

## Creative ergonomics invariant

Architecture boundaries are not automatically user-interface boundaries.

Ordinary creative work should support a short loop:

```text
change
    -> run / update
        -> see
            -> compare
                -> keep / export / continue
```

A safe derived connection such as:

```text
Reaction Diffusion
    -> SDF displacement preview
        -> material preview
```

may be presented as one direct creative relationship. The user should not need to
inspect product registries or understand ratification merely to see a derived result.

The product must expose a boundary when the user makes a materially different decision,
for example retaining a result as a durable artifact or requesting mutation of another
semantic owner. Hiding unnecessary ceremony must never mean hiding rejection, stale
results, approximation, unsupported capabilities, or failed authority transitions.

## Workspace classes

### Native visual studies

Some experiments are product-specific enough that a new reusable domain would be
premature. A native study may define its own narrow source meaning, evaluation contract,
state evolution, outputs, and diagnostics while those semantics remain useful only as
part of Visual Lab.

Do not pre-create:

```text
GenericSimulationProgram
UniversalDynamicsGraph
VisualLabField<T>
VisualLabRuntime
```

### Native-study promotion gate

A native study must undergo ownership review when one or more of these becomes true:

- an unrelated product needs the semantics independently of Visual Lab;
- the study gains a substantial reusable public API;
- it develops a durable domain source format with nontrivial migration or validation;
- it becomes authoritative or nonvisual rather than product-local visual computation;
- it owns persistent runtime state whose lifecycle matters outside the study;
- multiple independently useful algorithms or consumers establish a coherent domain;
- an existing domain is now clearly the correct owner.

The review decides whether to keep the study local, move it under an existing owner, or
propose a separately governed domain/extraction design. Product-local studies must not
become an ownership loophole for a hidden general simulation or field framework.

A repeated neutral abstraction may be extracted only after ADR 0017's shared-extraction
gate is satisfied by structurally different proving consumers.

### Domain-backed capabilities

Established semantics stay with existing owners. Examples include:

```text
SDF mathematics              -> RunenSDF
Authored SDF source           -> owning Runenwerk SDF authoring domain
World-SDF products            -> world-SDF / field-product owner
Field inspection              -> Field Visualizer
Material authoring/preview    -> Material domain / Material Lab
Terrain / procedural world    -> Procgen
future particles              -> particle owner
future motion                 -> animation owner
future water / erosion        -> world-process/simulation owner
```

The product may make these experiences visually coherent without pretending their
semantics share one runtime.

## Source, document, and persistence model

The durable architectural concept is **owner-defined source meaning plus explicit
composition intent**, not a universal executable graph or Visual-Lab-owned asset system.

A product-local study may have owner-specific source identity, revision, parameters,
presets, output definitions, and provenance. A Visual Study may refer to those native
sources plus domain-owned sources and products. Domain-backed sources remain documents
or values owned by their domains.

Existing Runenwerk asset/source/artifact/catalog architecture remains authoritative for
durable project assets and generated artifacts where those contracts apply. Visual Lab
does not introduce a second asset identity, catalog, cache, or persistence system.

Persisting Visual Study composition itself is deliberately unaccepted here. A future
schema must define stable identity, references, versioning, validation, migration,
unknown/removed capability behavior, and asset/catalog ownership before it becomes
architecture authority.

## Separate semantic roles

Visual Lab must not collapse source meaning, requested evaluation, execution strategy,
outputs, and interaction state.

Conceptually:

```text
Study Definition
    owner-specific source meaning
    parameters, initial conditions, intrinsic rules

Study Evaluation Request
    owner-specific semantic request
    spatial/domain extent, time horizon, sampling/discretization,
    accuracy/tolerance, or other meaning-bearing evaluation facts where relevant

Composition Intent
    how study/domain sources and products are related creatively

Execution Policy
    semantics-preserving execution choices
    backend preference, latency/budget, batching, parallelism,
    cache/reuse, interactive/offline preference

Output Request
    what should be observed, retained, captured, or exported

Session State
    transient host/UI interaction state
```

There is no universal `StudyEvaluationRequest` schema accepted by this design. Each
native study or domain owner defines only the semantic evaluation facts it actually
needs.

A semantic fact must not be moved into `ExecutionPolicy` merely because it affects cost.
For example, a simulation timestep, domain discretization, or requested time horizon may
change the meaning of a result and therefore belongs to the owning study/domain
contract when that is true. GPU workgroup layout, batching, staging, or cache placement
remain execution concerns.

## Parameter bindings and sampling

A parameter binding is composition intent, not an implicit live reactive runtime.

The default relationship is:

```text
source value/reference
    -> sampled or resolved at run admission
        -> admitted run input
```

Changing a bound source produces new composition/run intent rather than silently
mutating an in-flight evaluation. If a value is intentionally time-varying, streamed,
or interaction-driven, the owning study/domain must expose an explicit temporal/input
contract with appropriate time, freshness, and sampling semantics.

Visual Lab must not interpret "latest available value" from independently changing
owners as a consistency model.

## Temporal state, feedback, and iteration

Visual Lab supports stateful studies without turning feedback into an ordinary graph
back-edge.

Same-evaluation composition is acyclic by default. Feedback crosses an explicit
owner-defined temporal or iterative boundary.

A stateful native study defines the semantics it needs, which may include:

```text
initialization
state meaning and compatibility
step/update rule
time or tick semantics
reset behavior
termination or requested horizon
checkpoint compatibility
failure behavior
```

An iterative rather than temporal study may instead define an initial value, iteration
operator, convergence/tolerance rule, iteration budget, and non-convergence outcome.

Visual Lab does not create a universal simulation graph, temporal graph, feedback node,
or state runtime. Runtime state stored only in GPU resources remains execution state,
not durable source authority.

A retained simulation checkpoint, when a study later needs one, is an owner-defined
derived/materialized state product with provenance and compatibility facts. This design
does not pre-authorize a universal checkpoint format.

## Cross-domain composition relationships

Cross-domain creativity uses explicit owner contracts and four semantic relationship
roles rather than one universal graph/runtime.

### Observe

A consumer reads or inspects another owner's explicit contract without changing source
authority.

### Derive / Adapt

A source value/product is transformed into a non-authoritative value/product suitable
for another consumer. The transformation has explicit semantics and provenance. Preview
is one possible use of a derived value; preview is not the architectural relationship
itself.

### Retain / Materialize

A derived result or candidate is deliberately preserved beyond ordinary transient
preview lifetime. Retention does **not** by itself make the result authoritative in
another domain.

Examples include keeping a comparison result, materializing a generated field product,
or exporting a derived artifact.

### Apply / Commit

A retained or derived result is proposed as a change to an owning domain's durable
semantic state. The target owner validates/ratifies and applies the proposal through its
own command or mutation contract.

For example:

```text
generated field product
    -> derive candidate for authored SDF use
        -> optional retain/materialize
            -> Apply to authored SDF
                -> owning SDF authoring domain validates
                    -> accepted owner state or structured rejection
```

`Bake` is therefore UX vocabulary whose architectural meaning depends on the action. A
bake may only materialize a derived result, while a separate Apply action may request an
authority mutation. Visual Lab must not equate baking with committing.

There is no global Visual Lab transaction that atomically mutates arbitrary independent
owners. A multi-owner apply workflow forms and validates owner-specific proposals and
reports the actual accepted/rejected outcomes. Atomicity exists only where a concrete
owner contract explicitly provides it.

## Adapter ownership and admission

Cross-domain adaptation must not become hidden Visual Lab semantic authority.

The ownership rule is:

```text
source owner
    defines source meaning

target owner
    defines acceptable target input meaning

Runenwerk integration adapter
    defines the explicit translation between those contracts

Visual Lab
    selects and orchestrates the relationship
```

An adapter may be inserted automatically in the ordinary UX only when all of the
following are true:

1. exactly one applicable translation is available for the requested relationship;
2. no unresolved semantic choice would be guessed on behalf of the user;
3. declared loss, approximation, fallback, or degradation is acceptable to the target
   relationship;
4. current source and target facts satisfy the adapter's admission rules.

Relevant admission facts remain adapter/owner-specific and may include:

```text
contract version
channel or component meaning
dimensionality
units and scale
coordinate frame / space
time or interval
scope / coverage
source generation / freshness
consumer or authority class
precision / tolerance
loss / approximation
fallback policy
provenance / correspondence
host capability
```

This list is not a universal adapter record or mandatory common schema.

If several translations are semantically reasonable, Visual Lab must ask for or expose
the meaningful choice rather than pick one arbitrarily. Once chosen, that decision is
part of composition intent and remains inspectable.

## Run admission and result currency

There is no global Runenwerk snapshot or transaction, but one Visual Lab invocation must
still know which inputs it actually consumed.

Conceptually, an app-local **run admission** resolves the current request into one
compatible input set:

```text
study/source revision(s)
+ composition intent revision
+ owner-local product generations
+ owner-specific evaluation request
+ execution policy
+ output request
+ required capability facts
    -> admitted run
        -> execution
            -> run result
```

Run admission is a conceptual app/product role, not an accepted universal public type.
It uses only the consistency facts that its concrete consumers require.

Every asynchronous result is associated with the intent/input set it admitted. A result
may become the current preview only if it is still compatible with current creative
intent. A slower superseded run must not overwrite a newer current result merely because
it completed later.

Superseded results may still be retained, compared, or explicitly revisited. Cancellation
is an optimization and lifecycle request, not the correctness mechanism for result
currency.

## Result lifecycle, history, and last-good behavior

Visual Lab distinguishes result lifetime from authority:

```text
Current Preview
    replaceable current derived result

Preview History
    bounded session-local comparison history

Retained Result
    explicitly kept derived result/candidate

Published / Exported Artifact
    durable artifact under Runenwerk asset/output policy

Applied Domain State
    accepted state owned by the target domain
```

Preview history, retained checkpoints, and other histories require explicit bounds,
pressure behavior, and pruning policy. Merely appearing in history never promotes a
result to source authority.

When a new preview fails, Visual Lab may preserve the prior valid result only under an
explicit product policy. A preserved prior result must remain visibly classified as
last-good, stale, fallback, or otherwise non-current according to the owning contracts.
A failure must never be presented as though the preserved result were produced by the
new request.

## Determinism, equality, and reproducibility

Visual Lab must not promise stronger determinism than the owning study/domain and
execution path can provide.

A concrete study/evaluation contract defines the correctness relation meaningful for
its outputs. Useful conceptual classes include:

```text
exact
reproducible within a declared implementation/environment scope
tolerance-equivalent
visual-only / intentionally non-authoritative
```

These are design examples, not a new mandatory Visual Lab enum. Existing domain and
execution determinism classes remain authoritative where they apply.

Seeds, source revisions, evaluation requests, adapter choices, capability facts, and
execution/provenance facts should be retained when needed to explain or reproduce a
result. GPU floating-point execution or another implementation may be non-bit-exact
across environments unless a concrete owner contract proves otherwise.

A visual-only or tolerance-equivalent result may be perfectly valid for creative work.
If such a result is later proposed to an authoritative domain, the target owner decides
whether that candidate is admissible.

## Incremental evaluation

Interactive performance may reuse state, caches, products, or narrow change evidence,
but optimization must not change semantic meaning.

For the same admitted semantic input set, incremental evaluation must be observationally
equivalent to a clean/full evaluation under the owning study/domain's declared equality
or tolerance semantics.

Consequences:

- trusted narrow change evidence may permit narrow recomputation;
- missing, unknown, or incompatible change evidence broadens invalidation;
- when correctness cannot be established, perform a clean/full rerun;
- cache hits change cost, not result meaning;
- lost incremental history must have a clean fallback where correctness requires it.

Visual Lab does not create a universal reactive runtime or incremental database to
satisfy this rule.

## Graph policy

Visual Lab may provide coherent graph authoring and graph-shaped inspection, but graph
meaning remains domain-owned.

```text
domain/graph     structural connectivity only
SDF graph         owning SDF-authoring meaning
Material graph    material meaning
Procgen graph     procgen meaning
future sim graph  owning simulation-domain meaning
```

Do not create `VisualLabGraphRuntime` or treat semantic dependency, execution order,
GPU resource hazards, invalidation dependencies, UI containment, and temporal feedback
as interchangeable.

A future UI may visually project a Visual Study and its cross-domain relationships as a
single composition diagram. That projection does not make it a universal executable
semantic graph. Temporal/iterative feedback remains explicitly owned even if a diagram
shows it visually.

## Visualization and rendering boundary

Direct RunenGPU-backed output remains appropriate when the **study or source domain
itself defines the image/data result** and no renderer-owned scene semantics are needed.
Examples include:

```text
field heatmaps and channel views
distance contours
histograms
raw simulation-state views
compute-generated procedural images
generative textures
workload-specific visual artifacts
simple point/line/particle study outputs
```

Reuse Field Visualizer where it already owns the applicable field-inspection workflow.

RunenRender is required when forming the requested image depends on renderer-owned
semantics such as:

```text
renderer scene/view meaning
material and surface appearance
lighting and shadows
visibility
media / rendered volumes
transport / estimator policy
scene composition
stylized or nonphysical shading
renderer history / reconstruction
```

The rule is:

> A study-defined image may remain a direct Visual Lab consumer of RunenGPU. Image
> formation that requires renderer semantics belongs to RunenRender.

This prevents Visual Lab from becoming a parallel general renderer without incorrectly
forcing every meaningful procedural image through RunenRender.

Runenwerk retains persisted capture selection, image/video encoding, artifact naming,
retention, and product policy. RunenGPU executes generic work; it does not become image
or artifact persistence authority.

## Execution boundary

Visual Lab introduces no scheduler or execution fabric.

```text
study/domain source
    -> owner validation / ratification where applicable
        -> formed semantic request/product
            -> Runenwerk adapter / existing product-job path where applicable
                -> RunenGPU or another owning execution path
```

Product jobs, publication barriers, and query snapshots use the accepted execution
fabric where those semantics apply. A small product-local study may lower directly
through a bounded RunenGPU adapter when forcing it through ECS or a generic product
graph would add no semantic value.

Visual Lab is a downstream RunenGPU consumer. Attractive output supplements but never
replaces RunenGPU's own conformance, pressure, lifecycle, and recovery proofs.

## Host strategy

Visual Lab is **source- and semantics-neutral across hosts**, not lowest-common-
denominator in experience.

```text
                    Visual Lab Product
                           |
               portable study semantics
                           |
         +-----------------+-----------------+
         |                 |                 |
      CLI Host        Visual Lab GUI      Full Editor
      batch/run        live/interactive   project-aware
      inspect          compare/timeline   owner apply flows
      export           manipulate         asset workflows
```

Hosts may expose different capabilities. A CLI does not need to reproduce direct
manipulation; a GUI does not need to hide batch controls merely because CLI exists.
Portable source meaning, composition intent, owner-specific evaluation meaning, output
intent, and retained owner references are the stable layer.

Do not build durable product architecture around Runenwerk's legacy internal UI. A
bounded early host may be replaced later, provided study/domain semantics are not
embedded in that shell.

Standalone RunenUI remains the reusable future UI authority. Runenwerk adopts it only
after its separate capability-based adoption gate is satisfied; Visual Lab does not
create a compatibility UI framework or force premature partial adoption.

## Capability gates

This accepted design depends on **capabilities**, not RunenGPU/RunenRender milestone
labels. The canonical roadmap and owning issues decide when those capabilities become
available.

When generic GPU execution, completion, and readback are available, Visual Lab may run
native compute studies and produce direct study-defined outputs.

When generic offscreen graphics are available, Visual Lab may add direct workload/data
visualization and study-defined graphics without creating RunenRender semantics.

When reusable surface presentation is available, suitable direct workloads may gain
interactive presentation, resize, and continuous preview.

When the required public RunenRender semantic contracts are available, Visual Lab may
form images that require renderer-owned scenes, views, materials, lighting, volumes,
transport, stylization, or reconstruction.

When RunenUI satisfies its independent Runenwerk adoption gate, Visual Lab may gain a
reusable RunenUI-backed GUI host.

These gates state product capability requirements only. They do not duplicate, rename,
resequence, or activate the canonical RunenGPU, RunenRender, or RunenUI roadmaps, and
they imply no ordering between independent gates that their owning authorities do not
already require.

## Provenance, inspection, and cost visibility

Federated ownership increases causal depth, so Visual Lab must make provenance a
product feature rather than buried diagnostic metadata.

A visible result should be traceable conceptually through:

```text
Output
    -> output request
        -> admitted run / evaluation
            -> visualization or RunenRender contribution
                -> adapter / derived product
                    -> source study or domain contract
                        -> source revision / generation / parameters
```

The exact inspection UI is future work, but the architecture should preserve enough
identity, lineage, generation, admission, and diagnostic evidence to answer:

- what produced this result;
- which source/product generations were used;
- which adapter and semantic choice was used;
- whether a stale/fallback/last-good product contributed;
- whether approximation or degradation occurred;
- where a failed adaptation or authority transition occurred;
- how to reproduce or explain a retained result when the owning contracts support it.

Performance and transfer details may remain progressive-disclosure inspection data.
Where adapters cause materialization, CPU/GPU transfer, readback, duplication, or other
meaningful cost, expert inspection should be able to expose that fact without making it
common-path ceremony.

## Validation strategy

Avoid a Cartesian test matrix across every study, domain, host, and output mode.
Validate by authority seam:

```text
domain/study tests
    source/evaluation semantics

adapter tests
    translation, admission, approximation, and failure behavior

run/result tests
    input admission, stale completion, last-good, retention, and provenance

host-capability tests
    supported presentation/execution behavior

vertical golden journeys
    a small number of end-to-end creative workflows
```

Future implementation should prove at least:

- one product-local native study from source/evaluation intent to retained output;
- one real cross-domain Derive/Adapt path with truthful admission/provenance;
- the short creative loop `change -> run/update -> see -> compare/keep`;
- stale asynchronous completion cannot overwrite newer current intent;
- failed preview cannot masquerade a prior last-good result as current;
- clean/full and incremental evaluation agree under the owner's equality/tolerance
  contract where an incremental path exists.

Visual comparisons supplement structured assertions; they do not replace framework
conformance or semantic tests.

Concrete first-workload choices, implementation slices, and milestone dependencies
belong to the future implementation umbrella and canonical roadmap rather than this
accepted architecture.

## Non-goals

Do not create:

```text
VisualLabCore
VisualLabFields
VisualLabDynamics
VisualLabRuntime
VisualLabGraphRuntime
VisualLabRenderer
VisualLabAssetSystem
VisualLabScheduler
UniversalField<T>
UniversalCreativeGraph
UniversalSimulationProgram
UniversalStudyEvaluationRequest
UniversalAdapterRegistry
UniversalCheckpointFormat
```

Do not:

- duplicate Material Lab, Field Visualizer, Procgen, RunenSDF, RunenGPU, RunenRender,
  RunenUI, or existing asset/product authority;
- expose internal ownership ceremony in the common creative path when one safe derived
  relationship can be resolved automatically and truthfully;
- infer an adapter where more than one meaningful semantic mapping exists;
- treat type compatibility as sufficient cross-domain semantic compatibility;
- equate materialization/baking with target-domain authority mutation;
- bypass target-domain validation/ratification for durable authority transitions;
- promise atomic multi-owner mutation without a concrete owner transaction;
- let product-local native studies grow into undeclared reusable domains;
- make graph-canvas state runtime or source authority;
- hide temporal feedback as an ordinary graph edge;
- make parameter binding an implicit latest-value reactive runtime;
- make runtime GPU state durable source authority;
- force all hosts to implement identical interaction capabilities;
- expand the legacy UI architecture for Visual Lab;
- create a temporary renderer that becomes permanent;
- move Visual Lab semantics into RunenGPU;
- duplicate milestone sequencing from the canonical roadmap;
- use visual output as a substitute for framework conformance evidence.

## Fitness functions

The architecture remains valid when:

- every established semantic invariant remains owned by its existing domain/framework;
- Visual Lab has a clear center in procedural visual experimentation rather than
  becoming the default owner for all creative tooling;
- a product-local study is understandable without learning a universal creative runtime;
- native studies trigger ownership review when they become independently reusable,
  authoritative, or domain-sized;
- Visual Study composition owns references and creative intent, not foreign-domain or
  execution semantics;
- source meaning, owner-specific evaluation meaning, composition intent, execution
  policy, outputs, and UI/session state remain distinct;
- parameter bindings have explicit sampling/time semantics rather than implicit latest
  reads;
- temporal feedback and iteration cross explicit owner-defined boundaries;
- cross-domain use remains classifiable as Observe, Derive/Adapt,
  Retain/Materialize, or Apply/Commit and does not silently transfer authority;
- automatic adaptation occurs only when one semantically unambiguous admitted mapping
  exists;
- retained/materialized results remain distinct from applied owner state;
- asynchronous stale results cannot silently replace newer current intent;
- last-good/fallback presentation remains truthful;
- histories and checkpoints have bounds and pruning behavior;
- a useful study supports a short creative loop without requiring the user to understand
  internal product, adapter, graph, ratification, or GPU boundaries;
- durable domain sources remain references to owner-defined documents/products;
- no Visual Study persistence schema is implied before separately accepted ownership,
  versioning, validation, and migration;
- host-specific experiences may differ without changing portable source semantics;
- output provenance can identify the source/admission/adapter/result path needed to
  explain a result;
- study-defined images may use direct RunenGPU execution while renderer-semantic image
  formation uses RunenRender;
- later RunenUI and RunenRender adoption does not require rewriting study/source
  semantics;
- incremental paths remain clean/full equivalent under owner equality/tolerance;
- determinism claims never exceed the concrete owner/execution contract;
- RunenGPU remains domain-neutral and independently conformant;
- validation focuses on owner contracts plus a few representative vertical journeys;
- no shared substrate is extracted without ADR 0017's proof gate;
- this document expresses capability requirements rather than duplicating roadmap
  sequencing or live implementation status.

## Delivery and continuation

Issue #236 owns acceptance of this documentation-only product architecture.

After acceptance, do not add Visual Lab to the durable implementation roadmap merely
because the design exists. A future implementation umbrella must select concrete proof
workloads, establish actual capability dependencies, define persistence only if needed,
and own live sequencing/status through GitHub issues and the canonical roadmap.
