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
workbench for procedural visuals, generative systems, fields, SDF studies, simulations,
materials, and related visual experimentation.

This design accepts the product direction and ownership boundary. It does **not**
authorize implementation, a new production track, a new framework/repository, RunenGPU
roadmap changes, RunenRender activation, RunenUI cutover, or a new shared substrate.
Live implementation requires a separately owned GitHub issue and the prerequisites of
the capability being consumed.

The governing product rule is:

> Visual Lab owns the creative workflow and product-local study meaning. Existing
> domains and peer frameworks retain the semantic invariants they already own.

This design applies ADR 0014 and ADR 0017. It does not create a new family-wide law.

## Purpose

Visual Lab should make Runenwerk's procedural, GPU, field, simulation, and rendering
capabilities directly useful for creative exploration. A user should be able to create,
simulate, inspect, compare, animate, capture, bake, and export visual results without
turning the product into a universal field system, graph runtime, renderer, UI
framework, scheduler, or asset database.

The same product architecture must survive several host and framework stages:

```text
G5 headless/CLI creative compute
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

Visual Lab's composition architecture follows the same bounded principle currently
represented by the active Tool Suite / Workbench Host design: a host composes typed
tool/study capabilities without taking over their domain semantics. This accepted design
fixes only that product-level invariant. It does not accept by reference the active
Workbench design's exact manifests, API names, current editor-shell ownership, or
implementation sequence.

A Visual Lab host may install product-local native-study suites and existing or future
domain-backed suites. Suite composition controls product structure, surfaces, provider
routing, host capabilities, and product workflow. It does not transfer source semantics
into the workbench shell.

The durable Visual Lab product model must remain host-neutral. Current editor-shell or
legacy-UI implementation types are adapters/implementation facts, not permanent Visual
Lab source ontology. The same study and domain contracts may later be hosted by a CLI,
a dedicated Visual Lab application, the full editor, or a RunenUI-backed workbench.

## Ownership

### Visual Lab owns

- creative workbench/product composition;
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
references to owner-defined source/products
output definitions
provenance
```

Domain-backed sources remain domain documents such as material or procgen documents.
Visual Lab stores references and product/session state rather than copying their source
truth.

Existing Runenwerk asset/source/artifact/catalog architecture remains authoritative for
durable project assets and generated artifacts where those contracts apply. Visual Lab
does not introduce a second asset identity, catalog, cache, or persistence system.

## Separate meaning, execution, and output

Visual Lab explicitly separates:

```text
Study Definition
    what should be computed or evolved

Run Profile
    how this invocation should be performed

Output Request
    what should be observed or retained
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

Cross-domain creativity uses explicit products and three product-level relationship
classes rather than one universal semantic graph.

### Observe

A consumer inspects another owner's explicit product without changing source authority.
For example, a simulation result may be inspected by the Field Visualizer.

### Preview Adapt

A foreign product becomes temporary derived input for another workflow. The adapted
result remains preview/local derived state and does not mutate target-domain truth.
For example, a reaction-diffusion field may be previewed as an SDF displacement.

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

This preserves ADR 0017's cross-authority read and semantic-authority rules.

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

Cross-domain links are explicit product/adaptation relationships, not hidden graph
back-edges.

## Visualization boundary

Before RunenRender exists, Visual Lab may own narrow artifact/diagnostic visualization
mappings such as:

```text
scalar -> color ramp
vector -> direction/magnitude image
distance -> contour image
simulation state -> palette
trails -> accumulated image
volume -> selected slice
```

Reuse Field Visualizer where it already owns the applicable inspection workflow.

These mappings must not grow into a parallel general renderer. General materials,
lighting, visibility, shadows, media, transport, reconstruction, scene rendering, and
stylized/nonphysical image formation remain RunenRender semantics.

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

## Host and UI strategy

Visual Lab is host-neutral:

```text
                    Visual Lab Product
                           |
              host-neutral studies/workflows
                           |
         +-----------------+-----------------+
         |                 |                 |
      CLI Host        Visual Lab App     Full Editor
        early             later            workspace
```

Do not build durable product architecture around Runenwerk's legacy internal UI.
Initial interaction may use CLI/study files, presets, artifact output, and a bounded
non-authoritative preview harness where justified.

Standalone RunenUI remains the reusable future UI authority. A Runenwerk cutover occurs
only after its separate capability-based adoption gate is satisfied; Visual Lab does
not create a compatibility UI framework or force premature partial adoption.

## RunenGPU and RunenRender progression

### G5 — creative compute

The first useful Visual Lab slice may consume G5's accepted public execution,
completion, and readback boundary for headless creative work. Candidate first studies:

1. Reaction Diffusion;
2. Procedural Image;
3. Flow Painting;
4. SDF Study.

These exercise persistent state, repeated submissions, texture/buffer work, fixed-step
sequences, readback, and visual artifacts while remaining application consumers.

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

A headless run may emit image/data sequences and a Runenwerk-owned manifest recording
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
- bypass domain validation or ratification;
- make graph-canvas state runtime or source authority;
- make study document order implicit execution authority;
- expand the legacy UI architecture for Visual Lab;
- create a temporary renderer that becomes permanent;
- move Visual Lab semantics into RunenGPU;
- change the RunenGPU/RunenRender durable sequence;
- use visual output as a substitute for framework conformance evidence.

## Fitness functions

The architecture remains valid when:

- every established semantic invariant remains owned by its existing domain/framework;
- a product-local study is understandable without learning a universal creative runtime;
- study meaning, run policy, output artifacts, and UI/session state remain distinct;
- cross-domain use is classifiable as Observe, Preview Adapt, or Bake/Commit and does
  not silently transfer authority;
- durable domain sources remain references to owner-defined documents/products;
- Visual Lab can run headlessly before RunenUI/RunenRender without creating replacement
  UI or renderer authority;
- later RunenUI and RunenRender adoption does not require rewriting study/source
  semantics;
- RunenGPU remains domain-neutral and independently conformant;
- no shared substrate is extracted without ADR 0017's proof gate.

## Delivery and continuation

Issue #236 owns acceptance of this documentation-only product architecture.

After acceptance, do not add Visual Lab to the durable implementation roadmap merely
because the design exists. A future implementation umbrella must establish the actual
activation dependency and scope. The likely first implementation threshold is accepted
RunenGPU G5 public execution/readback capability, but live status and exact sequencing
belong to that future issue and the canonical roadmap at activation time.
