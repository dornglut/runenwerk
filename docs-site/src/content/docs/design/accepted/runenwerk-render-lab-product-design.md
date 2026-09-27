---
title: Runenwerk Render Lab Product Design
description: Accepted product architecture for Render Lab as a Runenwerk-owned renderer conformance, inspection, visual-proof, comparison, and stress consumer over RunenRender and RunenGPU.
status: accepted
owner: workspace
layer: product / app / renderer-integration
canonical: true
last_reviewed: 2026-09-27
publication: reference
pagefind: false
related_adrs:
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../adr/accepted/0021-ratify-runenrender-semantic-rendering-architecture.md
related_designs:
  - ./runenrender-decomposition-design.md
  - ../active/runenrender-internal-decomposition-execution-plan.md
  - ./renderer-temporal-reconstruction-and-dynamic-resolution-design.md
  - ./ui-designer-workbench-product-design.md
related_roadmaps:
  - ../../workspace/planning/roadmap.md
---

# Runenwerk Render Lab Product Design

## Status

This is the accepted product architecture for **Runenwerk Render Lab**.

It defines durable product ownership, boundaries, proof stages, and acceptance laws. It does not by itself authorize Rust/Cargo implementation work. Live activation, implementation slices, and exact-head evidence remain owned by GitHub issues and pull requests under the canonical roadmap and Runenwerk governance.

RL0 acceptance was censused on accepted `main` after #541 / PR #562. The recorded source state is provenance only; future implementation must re-census exact current `main` rather than reuse an old SHA as authority.

A 2026-09-12 authority reconciliation moved the smallest #552 finite-evaluation normalization before RL1 after #566 demonstrated concrete deterministic result-formation pressure.

A 2026-09-27 authority reconciliation records the accepted RL1/RL2 product reality, the evidence-driven temporal/performance branch that followed RL2, Engineering ADR 0009's RunenShader boundary, and the accepted R8 activation disposition from #898. It preserves durable product and framework boundaries without copying live issue, pull-request, check, branch, or priority state into this document.

## Purpose

RunenRender now has a normalized semantic spine, permanent public-RunenGPU execution proofs, and a maintained Render Lab consumer spanning headless deterministic evidence and native real-time interaction. Runenwerk still needs that product consumer to keep turning framework contracts into visible, inspectable, repeatable workloads and concrete public-surface pressure without moving renderer semantics into product code.

Render Lab exists to provide that consumer.

It is the product-level place for:

- controlled rendering scenarios;
- meaningful human-viewable renderer proof;
- renderer result and diagnostic inspection through legitimate public contracts;
- method/representation comparison;
- deterministic and stochastic conformance evidence appropriate to each method;
- bounded scale/stress characterization;
- headless regression evidence;
- interactive renderer experiments;
- concrete pressure on public RunenRender/RunenGPU ergonomics.

It is not a second renderer architecture.

## Product decision

Render Lab is a **distinct Runenwerk product identity**.

The ownership stack is:

```text
Runenwerk Render Lab
    scenarios / experiment choice / execution policy /
    comparison policy / evidence presentation / artifacts
        |
        v
RunenRender
    renderer scene / request / method / planning /
    admission / result semantics
    explicit shader-artifact -> GPU-program bridge where applicable
        |                         |
        v                         v
RunenShader                  RunenGPU
shader source /              generic physical
canonical artifact           GPU execution
```

RunenShader and RunenGPU are sibling framework authorities. Engineering ADR 0009 assigns reusable shader-source/toolchain and canonical-artifact meaning to RunenShader while RunenGPU retains canonical program admission and physical GPU execution. A RunenRender consumer integration may compose both; Render Lab does not acquire either authority merely because it exercises the composed path.

Source domains and peer frameworks keep their own semantic authority. Runenwerk adapters may translate source meaning into RunenRender meaning, but Render Lab does not become a source-domain ontology. Current consumer adoption of a sibling framework remains separately issue-owned; this design does not claim an integration is complete merely because the durable ownership relationship exists.

Render Lab is not:

```text
RunenRender framework semantics
RunenShader framework/toolchain semantics
RunenGPU framework semantics
an Editor document model
an Editor viewport mode
Material Lab
UI Designer
a generic scene editor
a universal renderer-test DSL
a generic Workbench reflection protocol
a path-tracing framework
a global conformance truth authority
```

## Accepted delivery state and durable findings

The RL0 census remains historical provenance. Accepted delivery since then established the product pressure that RL0 was designed to create.

### Runenwerk ownership remains correct

Runenwerk is the integration/product repository. `ARCHITECTURE.md` and the platform architecture assign application lifecycle, windows/event-loop policy, product policy, diagnostics presentation, applications, and cross-framework adapters to Runenwerk while RunenRender owns semantic rendering. RunenShader owns reusable shader-source/toolchain and canonical-artifact semantics, and RunenGPU owns generic physical GPU execution.

That remains sufficient architectural authority for a Runenwerk-owned Render Lab. No Editor, Workbench, shader-toolchain, or GPU backend becomes Render Lab semantic authority through hosting or use.

### RL1 established the product-accessible deterministic path

Accepted RL1 created the maintained `apps/runenwerk_render_lab` product and the `founding-direct` scenario through legitimate public RunenRender and RunenGPU contracts. The maintained product can form a meaningful deterministic render, observe public semantic result evidence, perform a separate product-owned readback, compare against an independent CPU oracle, and retain labelled product visualization/evidence artifacts without importing proof-only R6 machinery.

The earlier RL0 facts that the founding renderer was proof-local and `RenderResult` was not yet a product-facing surface are therefore historical starting conditions, not current implementation truth.

### RL2 established the native real-time consumer

Accepted RL2 extended the same product into native deterministic presentation with neutral input and continuous orbit, pan, and zoom. The interactive path presents newly rendered frames without intentional synchronous CPU readback.

Subsequent controlled performance work demonstrated meaningful output-resolution-dependent GPU pressure. That evidence does not create a universal RunenRender performance guarantee; it justifies product-owned characterization and the later fixed-internal-resolution / reconstruction-quality branch.

The maintained dedicated Render Lab app is current implementation truth. Its existence does not make one host shape part of Render Lab semantics and does not justify a generic `RunenLab` host framework.

### Render Lab is real R8 dogfood, not standalone conformance

The accepted R8 census in #898 established `R8_IMPLEMENTATION_SLICES_JUSTIFIED`. Render Lab is a real maintained Runenwerk consumer of the candidate renderer surface, including headless/offscreen and native paths. It is therefore valid public-surface pressure and dogfood.

It is not an independent standalone conformance consumer. R8 must still qualify the future public surface, reconcile the shader-artifact handoff, prove materially different method generality, and close standalone validation/provenance/cutover obligations through their owning work before RX.

### RunenShader adoption is a consumer boundary, not product authority

Engineering ADR 0009 establishes the long-term source-to-execution composition:

```text
shader source
    -> RunenShader canonical artifact
        -> RunenRender-owned artifact/program bridge
            -> RunenGPU program admission / execution
```

Render Lab may inspect and compare public outcomes from that composed path. It must not become the shader compiler, artifact store, GPU-interface authority, or a product-owned fallback around the accepted framework boundary.

### #552 deterministic normalization remains the finite-evaluation baseline

#566 demonstrated that a maintained finite floating-point evaluator could not truthfully form `RenderResult` for arbitrary `Exact` numeric requests under the earlier contracts. The accepted #552 normalization therefore separates requested semantic target meaning, admitted semantic/model approximation, finite-evaluation fidelity/error, and numeric realization.

That deterministic correction does not pre-author stochastic estimator, confidence, convergence, progressive-session, or accumulation semantics. Those remain consumer-gated by a materially different maintained method.

## Normalized product model

Render Lab preserves six distinct roles:

```text
SCENARIO
    controlled product-owned situation to exercise

RENDER INTENT
    RunenRender scene/request semantics formed for that scenario

METHOD / EXPERIMENT CHOICE
    compatible method, representation, comparison, or trial choice

EXECUTION POLICY
    host/product constraints such as repetition, mode, capture, and budget

OBSERVED EVIDENCE
    semantic outcomes, diagnostics, values, oracle/comparison facts, and cost observations

PRESENTATION / ARTIFACT
    UI views, images, reports, recordings, and persisted evidence
```

These roles must not collapse into one generic `RenderLabConfig` authority.

The product flow is conceptually:

```text
Scenario
    -> form renderer intent
    -> choose legal experiment
    -> execute through RunenRender -> RunenGPU
    -> observe legitimate evidence
    -> classify according to the scenario contract
    -> present and/or retain product evidence
```

This is a product workflow, not a universal engine runtime pipeline.

## Scenario ownership

A Render Lab scenario is product-owned controlled intent.

It may define only what is needed to establish a repeatable consumer, for example:

```text
scenario identity / revision
fixture or source setup
expected observation/output intent
legal method families
comparison/oracle policy
controlled mutation sequence
bounded stress parameters
expected supported/unsupported behavior
```

A scenario must not become:

- a second `RenderSceneSnapshot`;
- a generic world/scene authoring format;
- persisted RunenRender scene authority;
- a source-asset ontology;
- a universal renderer DSL;
- a store of mirrored private RunenRender state.

Scenario-to-renderer formation happens through explicit product/adaptor code and normal RunenRender contracts.

The initial catalogue should reuse accepted founding semantics where practical: analytic surfaces, a field/SDF-backed surface, minimum diffuse appearance, controlled directional illumination, perspective observation, spectral radiance, forward depth/distance, object identity, and a scalar radiance probe.

## Method and experiment law

Render Lab may deliberately select or compare methods because method choice can be part of a **product experiment** even when it is not ordinary requested renderer meaning.

Preserve:

```text
same requested semantic target
!= same RenderMethod
```

Useful experiment classes include:

```text
method A vs method B on the same semantic target
CPU/reference oracle vs RunenRender result
clean/full vs incremental execution
representation A vs representation B when both are legal
cold vs retained/warm execution when retained state exists
repeated stochastic evaluations under one declared evaluation contract
```

Every comparison declares the equality, tolerance, or statistical criterion that makes sense for that scenario. Render Lab must not standardize one universal pixel-equality or quality metric.

## Evidence model

Render Lab owns **product evidence policy**, not renderer truth.

A scenario run may correlate only legitimate observable facts, such as:

```text
public RunenRender request/result/diagnostic facts
public plan/admission/inspection evidence when exposed
public RunenGPU execution/status/capability evidence
output/readback/presentation values through legitimate paths
CPU/reference-oracle values
scenario-specific expected facts
product-owned timing/cost observations
```

A product run may be classified `pass`, `fail`, or `inconclusive` according to that scenario's declared evidence contract. That classification is product evidence. It is not a framework-wide truth certificate and does not mutate RunenRender or RunenGPU authority.

Output bytes, encoded images, screenshots, videos, manifests, and reports remain product/presentation artifacts rather than `RenderResult` semantics.

## Interactive and headless parity

Render Lab maintains two host modes over the same underlying scenario/run/evidence model:

```text
headless
    execute named scenarios
    emit deterministic-structure machine-readable evidence
    retain review artifacts when requested

interactive
    choose scenario/experiment
    inspect output/evidence/diagnostics
    interact with the renderer workload
```

These modes must not construct semantically different renderer paths for the same scenario merely because one has a window.

The current maintained implementation is the dedicated `apps/runenwerk_render_lab` product established by RL1 and extended by RL2. That placement is implementation truth, not semantic ownership. Future host refactoring may reuse genuinely app-neutral machinery only when it preserves the same product contracts and does not make the Editor shell, Workbench packages, or a generic `RunenLab` framework the authority for Render Lab.

## Public-surface rule

Render Lab is an adversarial **public-contract consumer**.

It must not:

- import test-only R6 modules;
- depend on crate-private RunenRender state;
- reach through RunenRender into private RunenGPU/WGPU state;
- become a shader compiler/toolchain owner or bypass an accepted RunenShader artifact boundary for convenience;
- copy private framework state into a long-lived product mirror;
- add privileged friend APIs only for the Lab.

When a scenario cannot be implemented honestly through current public contracts:

```text
scenario pressure
    -> exact owner census
    -> separate bounded framework issue
    -> smallest stable owner-correct contract correction
    -> scenario proves the corrected public path
```

This is a feature of Render Lab: it reveals whether the candidate standalone RunenRender boundary is actually usable.

## RL1 — accepted meaningful visual proof

RL1 established the first meaningful human-viewable render produced through the new semantic spine. Its accepted product boundary remains a regression law for later Render Lab work.

The retained RL1 contract is one named deterministic `founding-direct` scenario that:

- uses a legitimate product-accessible RunenRender -> public RunenGPU path;
- renders at a genuinely human-inspectable useful resolution rather than the proof-only `2 x 2` lattice;
- retains one product-owned visual artifact;
- retains structured scenario/semantic/oracle/diagnostic evidence;
- proves the field-backed representation participates where the scenario declares it;
- does not use private/test-only framework reach-through.

A review bundle remains conceptually:

```text
render-lab/founding-direct/
    radiance.<product-owned image format>
    evidence.<structured product format>
    [depth visualization]
    [identity visualization]
```

File formats, resolution, and visualization mapping are product decisions rather than RunenRender semantics. The founding radiance contract is spectral, not generic RGB; a grayscale or false-color mapping is valid only when explicitly product-owned and labelled as visualization.

RL1 is headless-capable. Interactive shell breadth was not required to establish the first product proof.

The durable pressure rule remains: when a later scenario cannot be implemented honestly through public framework contracts, fix the smallest owner-correct framework boundary rather than importing proof code or introducing a privileged Lab API.

## RL2 — accepted deterministic real-time product proof

RL2 established the first real-time native Render Lab over the same founding deterministic semantics.

Its retained interaction law is:

```text
continuous mouse input
    -> camera intent changes
    -> appropriate renderer request/state change
    -> RunenRender evaluation
    -> RunenGPU execution
    -> newly presented frame
```

The founding interaction scope remains:

```text
orbit
pan
zoom
```

The reference workload target remains:

```text
60 presented frames per second
<= 16.67 ms nominal frame interval
continuous mouse-driven interaction
image updates while dragging, not only after release
no intentional synchronous CPU readback in the interactive presentation path
```

This is a product/reference-workload target, not a universal RunenRender performance guarantee. Controlled performance claims record the relevant hardware/device, backend, output/internal extents, scenario, RenderMethod, build profile, and measurement procedure required to interpret the observation.

Screenshots and short interaction recordings are optional review artifacts. They are not a durable RL2 acceptance dependency and do not authorize a generic media-capture subsystem.

RL2's accepted performance pressure led to controlled output-extent and internal-resolution characterization. That successor work belongs to the evidence-driven temporal/performance branch below; it does not change RL2's semantic ownership.

## Product surface

The maintained interactive surface remains intentionally bounded to what current scenarios require:

```text
Scenario
    scenario / variant selection

Render
    observation/output summary
    legal method/experiment choice
    execute/re-execute

Output
    presented product visualization

Evidence
    available public renderer provenance/outcome
    oracle/comparison result
    structured diagnostics

Execution
    bounded timing/work/capability observations when legitimately available
```

Do not build a scene editor, node graph, timeline, asset browser, material authoring suite, or general profiler merely to make Render Lab look complete.

## Artifact ownership

Runenwerk may persist Render Lab evidence because capture/encoding/persistence policy belongs above RunenRender.

Potential product artifacts include:

```text
scenario/run manifest
structured diagnostics/evidence
reference/comparison summary
captured visualization or numeric output
performance observation
review screenshot / short interaction capture
```

Artifacts are evidence about a run. They are not a framework persistence schema and are not required for ordinary RunenRender use.

## Stress and scale role

Render Lab is the preferred product for controlled renderer scale characterization once real scenarios exist.

Possible product-owned dimensions include:

```text
scene object count
representation count
incremental change size
observation/output count
emitter/population count
representation refinement/detail
retained-state pressure
output size
repeated invocation count
```

Stress scenarios report measured behavior. They do not declare aspirational universal limits or encode product presets such as `Low/High/Ultra` into renderer semantics.

## Sequencing and proof ladder

The accepted deterministic foundation is complete:

```text
#541 semantic RenderResult/readback proof
    -> RL0 accepted Render Lab product design
    -> #552 deterministic finite-evaluation normalization
    -> #566 maintained deterministic execution seam
    -> RL1 meaningful deterministic visual artifact
    -> RL2 deterministic real-time interactive Render Lab
```

After RL2, the roadmap is **dependency-branched**, not one serialized feature ladder:

```text
accepted RL2
    |
    +--> deterministic temporal / performance pressure
    |       -> controlled output/internal-resolution characterization
    |       -> fixed-resolution execution
    |       -> controlled reconstruction-quality evidence
    |       -> spatial or temporal reconstruction work only when evidence justifies it
    |       -> R8 scale / quality / retained-state evidence
    |
    +--> framework public-boundary qualification
    |       -> consumer-qualified RunenShader artifact handoff
    |       -> ordinary future-public RunenRender surface qualification
    |       -> diagnostics / result ergonomics / exact RX source boundary
    |
    +--> materially different method pressure
            -> stochastic-specific finite-evaluation extension only if P1 requires it
            -> P1 stateless multi-bounce path-tracing RenderMethod
            -> direct-vs-path-tracer proof through the same semantic spine
            -> RL3 interactive path-tracing/material-view scenario
            -> adaptive stochastic evaluation pressure only when demonstrated

all mandatory R8 evidence
    -> R8 closure
        -> RX clean standalone cutover
```

These branches may proceed in parallel only when accepted issues, dependencies, files, and authority do not conflict. One branch does not become accepted dependency authority for another merely because both use Render Lab.

The durable canonical framework order remains:

```text
R7 -> R8 -> RX
```

The accepted #898 R8 census already established that R8 qualification may proceed before every generality gate is complete. P1/direct-vs-path evidence is therefore a remaining R8 **closure** obligation for the two-method requirement, not a prerequisite that retroactively precedes R8 activation.

Temporal reconstruction/history pressure is likewise not serialized behind RL3. It is a distinct renderer concern justified by deterministic temporal/output-quality workloads. Conversely, temporal history does not authorize progressive stochastic evaluation/session semantics.

## #552 and P1 boundary

The deterministic subset of #552 required by #566 is accepted and exercised by RL1/RL2. It establishes:

```text
requested semantic target
!= admitted semantic/model approximation
!= finite-evaluation fidelity/error
!= numeric realization
```

Do not extend this vocabulary with estimator confidence, stochastic convergence, progressive accumulation, or session semantics merely because temporal reconstruction work exists. Those concepts answer different questions.

P1 remains the planned first materially different maintained RunenRender method pressure: a stateless multi-bounce path-tracing method proved headlessly and correctness-first through the same scene/request/planning/admission/result spine. P1 is **RunenRender method work**, never Render Lab algorithm code.

Before P1 implementation, re-census exact current pressure. Add stochastic/statistical finite-evaluation contracts only when the real method requires them. The second maintained method must be materially different enough to prove the shared `RenderMethod` abstraction; do not manufacture a nominal second method solely to satisfy an R8 matrix.

Raw noisy finite stochastic output may be useful correctness/diagnostic evidence for P1. It does not define the normal interactive product experience.

## RL3 and adaptive refinement

RL3 follows a maintained P1/direct-vs-path proof and adds an interactive path-tracing/material-view scenario with controlled geometry/material/lighting, camera interaction, and inspectable evaluation evidence.

The later adaptive-refinement consumer supplies concrete pressure specifically for **progressive stochastic evaluation** state such as accumulation, continuation, convergence, cancellation, and any `RenderSession`-like contract required to preserve that work.

The required UX law is:

```text
first presented frame = coherent / usable
additional useful work = improves detail and/or evaluation fidelity
scene/camera/material change = invalidates only incompatible evidence
```

Temporal reconstruction history, jitter, motion/depth inputs, and fixed/dynamic-resolution reconstruction have their own accepted renderer contracts and may be justified earlier by deterministic temporal pressure. They must not be conflated with path-tracing accumulation or used to pre-author a generic retained-evaluation/session framework.

Do not pre-author progressive stochastic machinery merely because RL3 is planned.

## R8 closure and RX checkpoint

The initial extraction-readiness census has already occurred: accepted #898 concluded that bounded R8 qualification is justified while recording concrete missing/partial gates. Do not repeat that census as a prerequisite to starting R8, and do not interpret it as RX authorization.

Before final R8 closure and RX, current evidence must establish at least:

```text
real maintained Runenwerk consumer uses the candidate public RunenRender surface
ordinary headless/offscreen public API works without proof-private constructors
two materially different maintained RenderMethods share one semantic spine
direct-vs-path or equivalent method-generality evidence is owner-correct
RunenShader artifact handoff and RunenGPU execution authority are reconciled
public RunenGPU only; no WGPU/private reach-through
structured diagnostics and semantic result definedness are usable publicly
independent downstream public-API conformance exists beyond Render Lab dogfood
standalone validation/examples/provenance/release obligations are ready
consumer migration + predecessor deletion can be one clean cutover
remaining work is maturity/scale rather than unresolved ownership/API repair
```

Render Lab can provide maintained dogfood, visual comparison, performance/quality pressure, and bounded scale evidence. It cannot certify standalone conformance by itself.

After P1 and the direct-vs-path-tracer proof, refresh the R8 gate matrix rather than opening a second extraction-readiness authority. RX remains governed by the canonical RunenRender R8/RX plan and requires the clean standalone transfer/cutover; no compatibility facade, mirror, source include, or dual renderer authority is justified.

## Adjacent lab boundary

Render Lab remains renderer-focused.

```text
Render Lab
    RunenRender methods / representations / results / rendering conformance and stress

Reaction-Diffusion Lab
    simulation/field semantics + realtime RunenGPU compute + visualization

Cellular-Automata Lab
    discrete simulation semantics + realtime RunenGPU compute + visualization

World / Terrain Lab
    RunenSDF + RunenSpatial + Runenwerk world policy + RunenRender integration
```

Repeated product-neutral host/input/inspection machinery may be extracted only after structurally different real consumers prove the repeated neutral primitive. RL0 does not authorize a generic `RunenLab` framework.

## Explicit non-goals

This design does not authorize:

- RunenRender semantic changes merely to populate UI;
- RunenGPU contract changes without separate owner pressure;
- publicizing all private RunenRender proof state;
- importing R6 test/proof modules into product code;
- a universal Workbench inspection ABI;
- a generic scenario/scene/fixture DSL;
- an Editor compatibility layer for Render Lab;
- general scene/material/texture authoring;
- gameplay/runtime-preview integration;
- world/chunk streaming;
- broad profiler/benchmark infrastructure;
- shader-editor or hot-reload product features;
- render-farm/distributed orchestration;
- generic image/video/EXR pipeline design;
- hardware-ray-tracing work merely for Lab completeness;
- progressive stochastic sessions/accumulation/convergence without a concrete consumer;
- treating temporal reconstruction history as generic stochastic evaluation state;
- path tracing in the product layer;
- treating Render Lab dogfood as independent standalone conformance;
- claiming R8 complete from one product;
- extracting generic Lab infrastructure from one proof.

## Validation and acceptance laws

Each implementation slice must re-resolve exact current authority and prove only its own boundary.

RL0 originally accepted these laws; the sequence-specific item below is historical provenance and is superseded by the current proof ladder above:

1. post-#541 exact-main RunenRender/product census;
2. canonical Runenwerk product ownership;
3. RunenRender and RunenGPU authority preserved;
4. current editor-host machinery treated as implementation evidence, not authority;
5. current private/test-only RunenRender seams identified rather than silently bypassed;
6. normalized scenario/intent/experiment/execution/evidence/presentation separation;
7. headless/interactive parity law;
8. then-current RL1/RL2 proof ladder and #552 deferral recorded as canonical at RL0 acceptance;
9. no Rust/Cargo implementation in RL0;
10. documentation build, repository validation, and exact-head CI green for the unchanged reviewed documentation head.

Later slices must add focused product evidence in addition to the normal repository baseline.

## Cold-start path and live work

Cold-start order for Render Lab work:

1. this document — product mission, ownership, normalized model, and proof ladder;
2. [Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md) — product/Workbench ownership;
3. [RunenRender Architecture and Decomposition Design](./runenrender-decomposition-design.md) — renderer semantic authority;
4. [RunenRender internal execution plan](../active/runenrender-internal-decomposition-execution-plan.md) and the [canonical roadmap](https://github.com/dornglut/runenwerk/blob/main/docs-site/src/content/docs/workspace/planning/roadmap.md) — durable framework sequence;
5. [Renderer Temporal Reconstruction and Dynamic Resolution Design](./renderer-temporal-reconstruction-and-dynamic-resolution-design.md) when the selected work concerns internal resolution, reconstruction, temporal inputs, or history;
6. Engineering ADR 0009 / RunenShader authority when the selected work concerns shader source, canonical artifacts, or shader-to-GPU handoff;
7. the current owning GitHub issue/PR — live activation, implementation scope, blockers, and validation evidence.

Historical issue comments and prior editor renderer designs are evidence only. They do not override this accepted design or current code truth.

## Final position

Render Lab is the maintained Runenwerk consumer that makes the RunenRender boundary prove itself in product reality:

```text
product owns the experiment
renderer owns rendering meaning
shader toolchain owns canonical artifacts
GPU owns physical execution
evidence crosses boundaries through explicit contracts
deterministic visual and real-time proof precede stochastic generalization
temporal-quality and method-generality pressure remain distinct
R8 may qualify incrementally but closes only on its full evidence set
public-surface pressure is fixed at the owning boundary
shared infrastructure is extracted only after repeated proof
```

That is the accepted Render Lab product architecture.
