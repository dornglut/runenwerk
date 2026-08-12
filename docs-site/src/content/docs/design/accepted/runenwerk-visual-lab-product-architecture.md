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
workbench for procedural visual systems: fields, implicit form, generative processes,
simulation studies, material/procgen integrations, and their visualization.

This design accepts the product direction and ownership boundary. It does **not**
authorize implementation, a new production track, a new framework/repository, RunenGPU
roadmap changes, RunenRender activation, RunenUI cutover, or a new shared substrate.
Live implementation requires a separately owned GitHub issue and the prerequisites of
the capability being consumed.

The governing product rule is:

> Visual Lab owns creative workflow, product-local study meaning, and cross-domain
> composition intent. Existing domains and peer frameworks retain the semantic
> invariants and execution authority they already own.

The governing ergonomics rule is:

> Internal ownership boundaries stay strict; ordinary creative composition should feel
> direct. Safe derived use should not expose architecture ceremony. Durable authority
> transitions must be explicit.

This design applies ADR 0014 and ADR 0017. It does not create a new family-wide law.

## Purpose and product identity

Visual Lab should make Runenwerk's procedural, GPU, field, simulation, and later
rendering capabilities directly useful for creative exploration. A user should be able
to create, vary, simulate, inspect, compare, animate, capture, bake, and export visual
results without understanding internal adapter, product, ratification, graph, or GPU
boundaries for the common path.

Visual Lab is not a generic home for every artistic feature. Its center is:

```text
procedural visual systems
    fields
    implicit form
    generative processes
    temporal/simulated systems
    visual transformation and inspection
```

A feature belongs naturally when its primary purpose is to explore, generate,
transform, combine, or visualize such a procedural system. Material Lab, Procgen, SDF,
and future simulation domains integrate where useful; Visual Lab does not absorb them.

The same product architecture must survive several host and framework stages:

```text
G5 headless / bounded preview creative compute
    -> later offscreen and interactive RunenGPU presentation
        -> later RunenUI product frontend
            -> later RunenRender semantic image formation
```

Source semantics must not be rewritten merely because presentation or execution
capabilities mature.

## Architectural position

```text
                     VISUAL LAB
               Runenwerk creative product

                         |
          +--------------+--------------+
          |                             |
     Native Studies                Domain Suites
          |                             |
  reaction diffusion                 SDF
  flow painting                      fields
  cellular art                       materials
  generative images                  procgen
                                      later:
                                      particles
                                      animation
                                      world processes
          |                             |
          +--------------+--------------+
                         |
                  composition intent
                         |
                  explicit products
                         |
                 cross-domain links
                         |
            +------------+------------+
            |            |            |
         Observe       Preview      Bake/Commit
            |            |            |
            v            v            v
       inspection    derived use   target owner
                                    validates
                         |
                         v
                   Runenwerk adapters
                         |
             +-----------+-----------+
             |                       |
         RunenGPU              later RunenRender
                                     |
                                     v
                                  RunenGPU
```

Visual Lab is an application/product above domain and framework authorities. Runenwerk
remains the integration repository and owns cross-framework translation and product
policy according to ADR 0014.

## Composition model

Visual Lab's composition architecture follows the bounded principle currently
represented by the active Tool Suite / Workbench Host design: a host composes typed
capabilities without taking over their domain semantics. This accepted design fixes
only that product-level invariant. It does not accept by reference the active Workbench
design's exact manifests, API names, current editor-shell ownership, or implementation
sequence.

A Visual Lab host may install product-local native-study capabilities and existing or
future domain-backed capabilities. Composition controls product structure, available
surfaces, host capabilities, and product workflow. It does not transfer source
semantics into the workbench shell.

### Visual Study composition intent

Visual Lab needs one durable answer to "what belongs to this creative experiment?"
without creating a universal semantic or execution graph.

Conceptually, a Visual Study may record:

```text
VisualStudy
    product-local study sources
    references to domain-owned sources/products
    parameter bindings
    requested preview adaptations
    output requests
    run-profile references
    provenance
```

A Visual Study owns **composition intent and references**. It does not execute SDF,
material, procgen, simulation, render, or GPU semantics itself. Owner-defined adapters
and execution paths realize those requests.

The initial implementation should keep this structure explicit and narrow. Do not build
an extensible compiler, universal operator graph, or generalized composition runtime
until multiple concrete studies prove repeated neutral structure under ADR 0017.

## Ownership

### Visual Lab owns

- creative workbench/product composition;
- Visual Study composition intent and references;
- product-local native study semantics where no existing owner applies;
- active study/workspace and product session state;
- parameter-editing, run/pause/reset/step, compare, inspect, timeline, capture, bake,
  and export workflows;
- run profiles and output requests;
- preview selection and product-level execution preferences;
- app-level sequencing, naming, capture/export requests, and presentation policy;
- explicit cross-domain Observe, Preview Adapt, and Bake/Commit orchestration.

### Existing authorities retain

- **RunenSDF and SDF owners:** reusable SDF/implicit-field mathematics and owned SDF
  products;
- **Field/product owners and Field Visualizer:** field products, query/freshness policy,
  diagnostics, and field-inspection workflows;
- **Material domain / Material Lab:** material source, graph, ratification, and material
  preview semantics;
- **Procgen:** deterministic procedural-generation source, lineage, lowering, and
  formed outputs;
- **future particle, animation, physics, and world-process domains:** their respective
  simulation and runtime semantics;
- **RunenGPU:** generic GPU capabilities, resources, validated work, execution,
  progress, completion, readback, surfaces, and backend outcomes at their owning
  phases;
- **RunenRender:** semantic image formation, scenes, views, materials/media, methods,
  outputs, and lowering to RunenGPU;
- **RunenUI:** reusable renderer-neutral UI semantics and paint/hit products;
- **Runenwerk:** application lifecycle, host policy, cross-framework adapters, product
  policy, sequencing, artifact policy, and integration evidence.

Visual Lab must not mirror or become writable parallel authority for any of these.

## Creative ergonomics invariant

Architecture boundaries are not automatically user-interface boundaries.

Ordinary creative work should support a short loop:

```text
change
    -> run / update
        -> see
            -> compare
                -> save / keep exploring
```

A safe derived connection such as:

```text
Reaction Diffusion
    -> SDF displacement preview
        -> material preview
```

may be presented as a direct creative connection. Visual Lab may infer and carry the
required preview-adaptation contract internally. The user should not need to select
"Preview Adapt", inspect a product registry, or understand ratification merely to see
the result.

The boundary becomes explicit when the operation changes durable authority, for example:

```text
Bake as SDF
    -> SDF source candidate
        -> SDF owner validation / ratification
            -> accepted SDF source
```

Failures must still remain truthful and diagnosable. Hiding unnecessary ceremony must
never mean hiding rejection, stale data, unsupported capabilities, or failed authority
transitions.

## Workspace classes

### Native visual studies

Some experiments are product-specific enough that a new reusable domain would be
premature. Initial candidates include:

```text
Reaction Diffusion
Flow Painting
Cellular Study
Generative Image
Feedback Pattern
```

These may begin as explicit product-owned study definitions such as
`ReactionDiffusionStudy`. Do not pre-create `GenericSimulationProgram`,
`UniversalDynamicsGraph`, `VisualLabField<T>`, or `VisualLabRuntime`.

### Native-study promotion gate

A native study must undergo an ownership review when one or more of these becomes true:

- an unrelated product needs the semantics independently of Visual Lab;
- the study gains a substantial reusable public API;
- it develops a durable domain source format with nontrivial migration/validation;
- it becomes authoritative or nonvisual rather than product-local visual computation;
- it owns persistent runtime state whose lifecycle matters outside the study;
- multiple independently useful algorithms or consumers establish a coherent domain;
- an existing domain is now clearly the correct owner.

The review decides whether to keep the study local, move it under an existing owner, or
propose a separately governed domain/extraction design. Product-local studies must not
become an ownership loophole for a hidden general simulation or field framework.

A repeated neutral abstraction may be extracted only after ADR 0017's shared-extraction
gate is satisfied by structurally different proving consumers.

### Domain-backed suites

Established semantics stay with existing owners:

```text
SDF Study                  -> RunenSDF / SDF product contracts
Field Inspection           -> Field Visualizer
Material                    -> Material Lab
Terrain / procedural world  -> Procgen
future Particle Study       -> particle owner
future Motion Study         -> animation owner
future Water / Erosion      -> world-process/simulation owner
```

The product may make these experiences visually coherent without pretending their
semantics share one runtime.

## Document and resource model

The durable user concept is a study/source document plus explicit references and
outputs, not a universal executable graph.

A Visual-Lab-native study may contain:

```text
study/source identity
source revision
study-specific parameters
presets
references to owner-defined sources/products
output definitions
provenance
```

A Visual Study may reference those native definitions plus domain-owned sources and
products. Domain-backed sources remain domain documents such as material or procgen
documents. Visual Lab stores references and composition/session state rather than
copying their source truth.

Existing Runenwerk asset/source/artifact/catalog architecture remains authoritative for
durable project assets and generated artifacts where those contracts apply. Visual Lab
does not introduce a second asset identity, catalog, cache, or persistence system.

## Separate meaning, execution, output, and session state

Visual Lab explicitly separates:

```text
Study Definition
    what should be computed or evolved

Composition Intent
    how studies/domain products are related creatively

Run Profile
    how this invocation should be performed

Output Request
    what should be observed or retained

Session State
    transient host/UI interaction state
```

Example:

```text
ReactionDiffusionStudy
    feed = 0.037
    kill = 0.061
    ...

PreviewRun
    512 x 512
    1,000 steps
    local visual

FinalSequenceRun
    4096 x 4096
    fixed timestep
    24,000 steps
    capture every 40 steps
```

A study does not own GPU resource placement, dispatch topology, staging, backend
realization, or scheduler optimizations. This separation permits RunenGPU and later
execution improvements without changing creative source meaning.

## Cross-domain composition

Cross-domain creativity uses explicit products and three authority relationship classes
rather than one universal semantic graph.

### Observe

A consumer inspects another owner's explicit product without changing source authority.
For example, a simulation result may be inspected by the Field Visualizer.

### Preview Adapt

A foreign product becomes temporary derived input for another workflow. The adapted
result remains preview/local derived state and does not mutate target-domain truth.
For example, a reaction-diffusion field may be previewed as an SDF displacement.

Safe preview adaptation should normally be lightweight in the user experience. The
relationship class remains explicit in architecture and diagnostics even when the UI
can infer it from the compatible source and target contracts.

### Bake / Commit

A foreign result is converted into a proposal for durable target-domain meaning. The
target owner validates/ratifies it before it becomes accepted source or product truth.
For example:

```text
ReactionDiffusionStudy
    -> generated field product
        -> Bake to SDF source proposal
            -> SDF owner validation
                -> accepted SDF source
```

Bake/Commit is an explicit user-visible authority transition when durable target-domain
truth changes.

This preserves ADR 0017's cross-authority read and semantic-authority rules while
keeping ordinary creative preview composition fluid.

## Graph policy

Visual Lab may provide coherent graph authoring and graph-shaped inspection, but graph
meaning remains domain-owned.

```text
domain/graph     structural connectivity only
SDF graph         SDF meaning
Material graph    material meaning
Procgen graph     procgen meaning
future sim graph  owning simulation-domain meaning
```

Do not create `VisualLabGraphRuntime` or treat semantic dependency, execution order,
GPU resource hazards, invalidation dependencies, and UI containment as interchangeable.

A future UI may visually project a Visual Study and its cross-domain relationships as a
single composition diagram. That projection does not make it a universal executable
semantic graph.

## Visualization boundary

Visual Lab distinguishes **data/artifact visualization** from **semantic image
formation**.

Direct RunenGPU-backed data/artifact visualization may remain appropriate for:

```text
field heatmaps
scalar/vector channel views
distance contours
histograms
raw simulation state
simple particle/line diagnostics
compute-generated images
workload-specific visual artifacts
```

Reuse Field Visualizer where it already owns the applicable inspection workflow.

General semantic image formation belongs to RunenRender once that framework is
available, including:

```text
materials and surface appearance
lighting and shadows
visibility
media / volumes as rendered scene meaning
transport / estimator policy
scene composition
stylized or nonphysical shading
history / reconstruction
```

The rule is:

> "Show me this data or workload result" may remain a direct Visual Lab / RunenGPU
> visualization. "Form an image of this visual world" belongs to RunenRender.

This prevents an early Visual Lab visualization path from becoming a competing renderer.

## Execution boundary

Visual Lab introduces no scheduler or execution fabric.

```text
study/domain source
    -> owner validation / ratification where applicable
        -> formed product or workload request
            -> Runenwerk adapter / existing product-job path
                -> RunenGPU
```

Product jobs, publication barriers, and query snapshots use the accepted execution
fabric where those semantics apply. A small product-local G5 study may lower directly
through a bounded RunenGPU adapter when forcing it through ECS or a generic product
graph would add no semantic value.

Visual Lab is a downstream RunenGPU consumer. Attractive output supplements but never
replaces RunenGPU's own deterministic conformance, pressure, lifecycle, and recovery
proofs.

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
      CLI Host        Visual Lab App     Full Editor
      batch/run        live/interactive   project-aware
      inspect          compare/timeline   bake/commit
      export           manipulate         asset workflows
```

Hosts may expose different capabilities. A CLI does not need to reproduce direct
manipulation; a GUI does not need to hide batch controls merely because CLI exists.
Portable source meaning, references, run-profile meaning, and output intent are the
stable layer.

Do not build durable product architecture around Runenwerk's legacy internal UI.
Initial interaction may use CLI/study files and a bounded preview shell, provided no
study/domain semantics are embedded in that shell.

Standalone RunenUI remains the reusable future UI authority. A Runenwerk cutover occurs
only after its separate capability-based adoption gate is satisfied; Visual Lab does
not create a compatibility UI framework or force premature partial adoption.

## Provenance and inspection

Federated ownership increases causal depth, so Visual Lab must make provenance a
product feature rather than buried diagnostic metadata.

A visible result should be traceable conceptually through:

```text
Output
    -> output request / run profile
        -> visualization or RunenRender contribution
            -> preview adaptation / domain product
                -> source study or domain document
                    -> source revision / seed / parameters
```

The exact inspection UI is future work, but the product architecture should preserve
enough identity, lineage, generation, and diagnostics to answer:

- what produced this result;
- which source/product generation was used;
- whether a fallback or stale product contributed;
- where a failed adaptation or authority transition occurred;
- how to reproduce a retained result when the owning contracts support it.

This turns a weakness of federation into useful creative/debugging visibility.

## Validation strategy

Avoid a Cartesian test matrix across every study, domain, host, and output mode.
Validate by authority seam:

```text
domain/study tests
    semantic correctness

adapter tests
    cross-owner translation and failure behavior

host-capability tests
    supported presentation/execution behavior

vertical golden journeys
    a small number of end-to-end creative workflows
```

Directional future vertical journeys:

```text
Reaction Diffusion -> retained image
Reaction Diffusion -> SDF preview adaptation -> contour/image
SDF Study -> later RunenRender image
same study/source -> CLI and GUI hosts with equivalent output intent
```

Visual comparisons supplement structured assertions; they do not replace framework
conformance or semantic tests.

## RunenGPU and RunenRender progression

### G5 — creative compute

The first useful Visual Lab slice may consume G5's accepted public execution,
completion, and readback boundary for creative compute. Candidate first studies:

1. Reaction Diffusion;
2. Procedural Image;
3. Flow Painting;
4. SDF Study.

These exercise persistent state, repeated submissions, texture/buffer work, fixed-step
sequences, readback, and visual artifacts while remaining application consumers.

A G5 implementation should not be considered a useful product merely because it emits
files. It should provide a bounded short feedback loop appropriate to the available
host, such as parameter change, run/update, visible result, compare/randomize, and save.
The first shell may be disposable; the study/source model must not be.

### G6 — offscreen graphics

After G6 is accepted, Visual Lab may add generic offscreen GPU visualization such as
line/point/particle drawing, image passes, and offscreen composition. This does not
activate or substitute for RunenRender.

### G7 — surfaces

After G7 is accepted, suitable direct RunenGPU workloads may use interactive surface
presentation, resize, camera/view interaction, and continuous visual preview.

### G8 and GX

Operational hardening and standalone RunenGPU transfer remain owned by the RunenGPU
roadmap. Visual Lab does not resequence them.

### RunenRender R phases

The current canonical roadmap blocks RunenRender implementation until accepted
standalone RunenGPU cutover and separate R-phase authorization. When RunenRender later
matures, Visual Lab may consume semantic scenes, views, materials, lighting, SDF/field
rendering, volumes, stylized image formation, and reconstruction through RunenRender
without replacing its existing study/source semantics.

## Initial future implementation concept: VL1 Creative Compute

This design does not activate VL1. A future implementation umbrella may do so after the
required G5 public boundary is accepted.

Directional workloads:

```text
Reaction Diffusion
  persistent state, ping-pong resources, fixed-step simulation,
  repeated submission, visualization, sequence generation

Procedural Image
  texture generation, multi-pass processing, patterns, filtering, readback

Flow Painting
  vector-field evolution, moving state, accumulation, temporal feedback,
  stylized output; not the future general particle authority

SDF Study
  RunenSDF source -> Runenwerk adapter -> RunenGPU ->
  distance/slice/contour visualization
```

VL1 should prefer direct app-local composition over premature generic suite/compiler
machinery. Generic composition infrastructure may be adopted or extracted only when
multiple real studies/domain-backed capabilities demonstrate repeated need.

A run may emit image/data sequences and a Runenwerk-owned manifest recording
study/source revision, parameters, seed, run profile, dimensions, time/step facts,
output requests, useful environment/capability facts, diagnostics, and artifact
checksums. Runtime GPU handles and backend-private identities are never persistence
authority.

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
```

Do not:

- duplicate Material Lab, Field Visualizer, Procgen, RunenSDF, RunenGPU, RunenRender,
  RunenUI, or existing asset/product authority;
- expose internal ownership ceremony in the common creative path when a safe derived
  relationship can be resolved automatically and truthfully;
- bypass domain validation or ratification for durable authority transitions;
- let product-local native studies grow into undeclared reusable domains;
- make graph-canvas state runtime or source authority;
- make study document order implicit execution authority;
- force all hosts to implement identical interaction capabilities;
- expand the legacy UI architecture for Visual Lab;
- create a temporary renderer that becomes permanent;
- move Visual Lab semantics into RunenGPU;
- change the RunenGPU/RunenRender durable sequence;
- use visual output as a substitute for framework conformance evidence.

## Fitness functions

The architecture remains valid when:

- every established semantic invariant remains owned by its existing domain/framework;
- Visual Lab has a clear center in procedural visual systems rather than becoming the
  default owner for all creative tooling;
- a product-local study is understandable without learning a universal creative runtime;
- native studies trigger ownership review when they become independently reusable,
  authoritative, or domain-sized;
- Visual Study composition owns references and creative intent, not foreign-domain or
  execution semantics;
- study meaning, composition intent, run policy, output artifacts, and UI/session state
  remain distinct;
- cross-domain use remains classifiable as Observe, Preview Adapt, or Bake/Commit and
  does not silently transfer durable authority;
- safe derived composition can be presented with minimal ceremony while failures and
  durable authority transitions remain explicit;
- a useful study supports a short creative loop -- change -> run/update -> see ->
  compare/save -- without requiring the user to understand internal product, adapter,
  graph, ratification, or GPU boundaries;
- durable domain sources remain references to owner-defined documents/products;
- host-specific experiences may differ without changing portable source semantics;
- output provenance can identify the source/run/product path needed to explain a result;
- direct RunenGPU visualization remains data/artifact-oriented while semantic image
  formation moves through RunenRender when available;
- Visual Lab can run before RunenUI/RunenRender without creating replacement UI or
  renderer authority;
- later RunenUI and RunenRender adoption does not require rewriting study/source
  semantics;
- RunenGPU remains domain-neutral and independently conformant;
- validation focuses on owner contracts plus a few representative vertical journeys;
- no shared substrate is extracted without ADR 0017's proof gate.

## Delivery and continuation

Issue #236 owns acceptance of this documentation-only product architecture.

After acceptance, do not add Visual Lab to the durable implementation roadmap merely
because the design exists. A future implementation umbrella must establish the actual
activation dependency and scope. The likely first implementation threshold is accepted
RunenGPU G5 public execution/readback capability, but live status and exact sequencing
belong to that future issue and the canonical roadmap at activation time.
