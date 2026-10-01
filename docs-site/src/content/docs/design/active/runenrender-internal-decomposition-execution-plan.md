---
title: RunenRender Decomposition Execution Plan
description: Durable dependency-ordered RunenRender implementation, proof, and external-cutover program downstream of standalone RunenGPU.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-10-01
publication: reference
pagefind: false
related_docs:
  - ./shader-authoring-and-canonical-artifact-policy.md
  - ../accepted/runenrender-decomposition-design.md
  - ../../reports/design/runengpu-phase-requirements-proof-matrix.md
  - ../../reports/investigations/2026-08-04-runenrender-long-term-capability-and-scalability-review.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../adr/accepted/0021-ratify-runenrender-semantic-rendering-architecture.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Decomposition Execution Plan

## Purpose

Implement and prove the accepted RunenRender semantic architecture, then perform a clean
standalone RunenRender transfer, while consuming standalone RunenGPU only through its
public contracts.

The current ownership shape is:

```text
standalone RunenGPU
    reusable generic GPU execution

RunenRender
    semantic rendering through public RunenGPU contracts

Runenwerk
    lifecycle, source/domain projection, windows, scheduling,
    product/recovery/authoring/artifact policy, and integration
```

This document owns durable RunenRender phase responsibility, proof boundaries, and
cutover gates. GitHub issues own activation and live status. Pull requests own delivery
and exact-head evidence. The roadmap owns the high-level sequence. Standalone RunenGPU
owns current reusable GPU semantics and conformance.

Implementation requires an owning issue, accepted current architecture, an exact-current
census, and repository validation. No phase is activated merely because it appears here.

## Standalone RunenGPU prerequisite

The RunenGPU G-phase predecessor program and GX transfer are complete. Current reusable
RunenGPU semantics, implementation, validation, and future framework evolution belong to
`dornglut/runen-gpu`.

Runenwerk currently consumes exact accepted RunenGPU revision:

```text
c292b77ff1a3b3e984dc2263bbbf8d8c06e93777
```

That exact pin is a Runenwerk integration-compatibility fact, not a local RunenGPU
semantic owner. A future repin requires explicit integration review and validation.
Historical G-phase requirement/proof identifiers remain available in the noncanonical
[RunenGPU proof report](https://github.com/dornglut/runenwerk/blob/main/docs-site/src/content/docs/reports/design/runengpu-phase-requirements-proof-matrix.md)
and Git history.

RunenRender must not recreate the retired Runenwerk-local RunenGPU G-phase authority in
order to advance an R phase. If a reusable GPU capability is missing, change the
standalone owner under its own accepted work and then deliberately repin/integrate it.

## Durable sequence

```text
accepted standalone RunenGPU public boundary
    -> R0
    -> R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8
    -> RX
    -> A1
    -> V1+
```

```text
R0   RunenRender normative architecture gate
R1-R7 internal RunenRender semantic/integration proof
R8   predecessor-side public/source-boundary qualification
RX   standalone acceptance + external clean cutover
A1   reusable adapter review
V1+  successor-owned advanced/generalization pressure
```

R0 is mandatory. Historical RunenGPU G-phase sequencing is no longer an active prefix of
this Runenwerk execution plan.

## Global invariants

Every RunenRender phase preserves:

- one public package for the target repository initially;
- no Runenwerk, product, ECS, SDF, UI, editor, or application types in future framework
  public contracts;
- no direct/private WGPU ownership in RunenRender;
- public standalone RunenGPU contracts are the only GPU execution boundary;
- no dependency cycle;
- no source mirror, forwarding namespace, compatibility package, source include,
  submodule, or moving-branch dependency;
- no old/new parallel semantic authority after accepted cutover;
- owner-local typed identities rather than universal cross-framework identity;
- bounded caches, histories, sessions, diagnostics, variants, and retained state expose
  pressure or bounded waits;
- derived state remains non-authoritative and dependency/generation-bound;
- renderer semantic meaning remains distinct from physical GPU realization;
- Runenwerk owns product recovery, integration compatibility, persisted
  capture/reproducibility artifacts, authoring policy, and artifact encoding;
- proof categories remain separated: correctness, integration, operations, recovery,
  performance, showcase, and public-boundary/usability qualification;
- each implementation phase migrates consumers of replaced RunenRender authority and
  deletes that authority in the same accepted slice;
- exact-head validation and repository CI remain merge evidence.

# RunenRender internal proof

## R0 — normative semantic-rendering architecture

R0 is documentation/architecture only.

It establishes:

- semantic rendering as the RunenRender mission;
- explicit non-ownership against RunenGPU, Runenwerk, ECS, SDF, Spatial, UI, assets,
  simulation, persistence, and codecs;
- one renderer-local scene lineage;
- scene/request/request-scoped-input separation;
- renderer-specific observation semantics;
- output meaning/result topology/physical binding separation;
- representation contract/evidence/applicability/availability/realization separation;
- coherent render-method semantics;
- conditional device-independent planning;
- semantic binding admission distinct from execution admission;
- RunenGPU physical lowering boundary;
- incremental/full equivalence;
- the R1-R8/RX dependency order.

R0 does not modify Rust/Cargo, change RunenGPU semantics, populate `dornglut/runen-render`,
or authorize R1.

## Permanent semantic spine

All implementation phases preserve:

```text
RenderSceneStore
    -> commit(RenderSceneUpdate)
        -> RenderSceneCommit
            ├── RenderSceneSnapshot
            └── RenderSceneChangeSet

RenderSceneSnapshot
+ RenderRequest
+ representation/method contracts
    -> conditional RenderPlan

RenderPlan
+ current request-scoped semantic bindings for prerequisites declared by the plan, if any
    -> semantic binding admission

semantically admitted candidates
+ representation availability/realization
+ physical output bindings
+ current RunenGPU environment facts
+ execution requirements
    -> execution admission
        -> AdmittedRenderPlan
            -> RenderWorkSet
                -> RunenGPU
                    -> RenderResult
```

When a plan declares no binding-dependent semantic prerequisite, the semantic-binding
substage is vacuously satisfied. Do not create placeholder binding identity, values,
provider state, or a generic binding store merely to instantiate that empty case.

The ordinary public API may collapse stages ergonomically. Their responsibilities remain
distinct and testable.

## R1 — scene lineage and minimal renderer identity

Goal:

- `RenderSceneStore`;
- `RenderSceneRevision`;
- `RenderObjectId`;
- atomic `RenderSceneUpdate` insert/remove presence mutation;
- immutable `RenderSceneSnapshot`;
- explicit R1-owned `RenderSceneChangeSet`;
- structurally shared or equivalently bounded small-change publication;
- explicit full scene resynchronization;
- no views, representations, acquired output images, execution-environment facts, or
  live source/host state inside the scene snapshot.

R1 does not define same-identity replacement because it owns no replaceable object state
beyond identity/presence. Replacement begins only in the first later phase that owns a
concrete replaceable renderer-semantic record and its equality/revision/change law.

R1 deliberately does **not** introduce generic producer identity/lifecycle,
`RenderContributionId`, generic relationships, representation protocols, space/time,
observations, materials, dynamic-input schemas, availability, or GPU placeholders.

Source producer retirement is an adapter-boundary operation:

```text
source disappears
    -> source/Runenwerk adapter resolves affected RenderObjectIds
    -> one atomic renderer removal update
```

Required proof:

- deterministic insert/remove presence mutation;
- deterministic duplicate-insert, missing-remove, same-object conflict, and accepted
  no-op behavior;
- atomic multi-operation commit and rejected-commit no-publication behavior;
- retained old snapshot remains immutable;
- equivalent full and incremental scene construction;
- precise R1 structural change evidence and explicit full resync;
- bounded small-change publication cost against total scene size;
- at least two independent source/adaptor families using the same renderer mutation
  contract;
- source identities/lifecycle remain outside RunenRender;
- no ECS mirror and no RunenGPU identity.

## R2 — semantic space/time, observations, outputs, and minimal requests

Goal:

- coordinate frames, units, semantic transforms, orientation/handedness where required,
  and spatial support/coverage;
- render time/interval, exposure/shutter support, temporal validity, and motion interval;
- `RenderObservationSpec` and coordinated observations;
- semantic sampling support distinct from sampling algorithm;
- `RenderOutputSpec`;
- output-value meaning, result topology, and applicable radiometric/transport domains;
- semantic tolerance/accuracy distinct from numeric realization;
- minimal `RenderRequest`.

No current residency, physical output binding, sampling strategy, or method planning is
owned here.

Required proof includes:

- perspective observation;
- non-image-grid renderer probe;
- coordinated observation set;
- radiance, depth/distance, and identity semantics;
- scene spatial/temporal change evidence only after R2 semantics exist;
- semantic output meaning independent of physical storage/destination.

## R3 — representations, protocols, relationships, and minimum appearance

Goal:

- `RenderRepresentation` as an open family;
- representation identity only where required;
- intrinsic semantic contract/evidence;
- narrow versioned query protocols and narrow results;
- refinement/error semantics;
- concrete typed scene relationships only when a real consumer requires them;
- instance/occurrence semantics only if independently proved;
- minimum material/medium/emitter/environment contracts required by the founding method.

Representation intrinsic evidence may include protocol revisions, coordinate/unit
conventions, spatial/temporal coverage, accuracy/error vocabulary,
exact/conservative/refinement facts, and provenance.

Do not put current availability/residency into authoritative representation meaning.
`RepresentationOffer` may later exist only as a derived planning/admission view.

Initial proof families include at least:

```text
analytic surface
field/SDF surface
```

Required proof:

- exact analytic query;
- conservative field query with bounded termination/error evidence;
- transform validity classification for field/SDF semantics;
- declared coverage and refinement evidence;
- stable `RenderObjectId` across representation replacement;
- at least one real typed relationship;
- protocol version mismatch and structured unsupported outcomes;
- source SDF mathematics remain outside RunenRender.

## R4 — RenderMethod and conditional semantic planning

Goal:

- `RenderMethod` semantic concept and compatibility;
- request-relative representation applicability;
- request-static applicability derived from request/contracts/evidence;
- typed request-scoped semantic-input requirements only for actual R2/R3/R4 consumers;
- binding-dependent applicability represented as explicit predicates/prerequisites;
- `RenderPlan`;
- semantics-preserving alternatives and explicit bounded approximation envelopes;
- normalized abstract RunenGPU capability/work requirements;
- no current physical RunenGPU handles, residency, surfaces, allocations, pipelines, or
  concrete output bindings.

A `RenderPlan` means:

> these solution families can satisfy the request provided their declared semantic
> prerequisites are admitted.

Required proof:

- CPU-only deterministic planning;
- multiple legal solution families for one request;
- protocol, coverage, output, method, and accuracy incompatibility rejection;
- unresolved binding-dependent applicability remains explicit rather than guessed;
- planning requires no current GPU environment;
- no all-object x all-method requirement;
- illegal semantic substitution is not represented as ordinary fallback.

R4 answers:

> What semantic solution families could satisfy this request, and what prerequisites do
> they require?

## R5 — semantic bindings, availability, output bindings, and execution admission

Goal:

- current source-owner semantic input values/bindings/generations/provenance when a
  concrete R4 contract declares request-scoped semantic-input prerequisites;
- semantic binding admission for those declared prerequisites;
- binding-dependent representation applicability evaluation when such predicates exist;
- current representation availability/realization facts;
- `RenderOutputBinding` physical destinations;
- execution requirements and pressure;
- current RunenGPU capability/device-generation facts;
- execution admission;
- `AdmittedRenderPlan`.

R5 permanently owns request-scoped semantic binding values, generations, provenance, and
admission, but concrete binding API/proof is consumer-gated. If the exact activation
census contains no R4 binding-dependent prerequisite, that substage is semantically
vacuous for the slice. Do not introduce a generic binding ID/store/schema/provider or a
placeholder semantic-input family to make the stage non-empty.

Binding admission may evaluate predicates, eliminate candidates, and specialize choices
already declared by R4. It may **not invent a new semantic alternative outside the
`RenderPlan` solution space**. A zero-prerequisite plan may proceed directly to the R5
operational/execution-admission facts without fabricated binding data. Once a real
request-scoped semantic-input consumer exists, its declared prerequisites must be
admitted before any dependent candidate can execute.

Required proof:

- when concrete binding prerequisites exist, a valid current semantic binding is
  accepted and missing/stale/foreign/temporally incompatible/coverage-incompatible
  bindings are rejected;
- when concrete binding identity/generation exists, binding-generation change does not
  automatically change `RenderSceneRevision`, and semantic input identity remains
  distinct from RunenGPU resource identity;
- representation availability/residency change does not automatically change
  `RenderSceneRevision`;
- RunenGPU device-generation change does not automatically change
  `RenderSceneRevision`;
- semantic output meaning differs from physical destination;
- `unsupported != unavailable`;
- a semantically valid `RenderPlan` can be temporarily inexecutable;
- a zero-binding-prerequisite `RenderPlan` reaches execution admission without synthetic
  semantic binding state;
- legal semantics-preserving realization alternatives and bounded permitted
  approximation work;
- semantic substitution is rejected unless explicitly authorized.

R5 answers:

> Which semantically admitted solution can execute now?

## R6 — first complete semantic renderer and RunenGPU lowering

The first end-to-end renderer proof is deliberately bounded.

Scene:

```text
one analytic sphere
one analytic plane
one field/SDF surface
minimum diffuse material
one directional emitter
```

Observation/output proof A:

```text
perspective observation
HDR radiance
depth
object identity
```

Observation/output proof B:

```text
one non-image-grid scalar radiance probe
```

Method/execution:

```text
one coherent direct-lighting method
minimal physical output bindings
public RunenGPU only
CPU reference probes
```

The proof must use permanent R1-R5 scene, request, representation, protocol, method,
planning, admission, and output contracts. It must also use binding contracts/admission
when the accepted R4 plan for the founding renderer actually declares binding-dependent
prerequisites. R6 must not fabricate a semantic-input consumer merely to make that
substage non-vacuous.

R6 proves both conventional image rendering and the broader semantic-rendering API. It
does not authorize public types named after SDF, direct lighting, preview, or the first
implementation.

## R7 — derived continuity and advanced integration

Introduce only demonstrated advanced requirements:

- explicit derived-state dependency/invalidation structures;
- compiled representations and acceleration;
- renderer-derived residency/realization;
- history, reconstruction, accumulation, and renderer-semantic denoising;
- optional sessions, progress, convergence, continuation, and cancellation;
- compatible multi-observation preparation sharing;
- multi-output sharing and semantic merge;
- readback integration;
- physical presentation/surface binding integration through public RunenGPU;
- semantic partition/merge for multi-device or distributed orchestration.

Runenwerk retains windows, XR/platform runtime lifecycle, presentation/product recovery,
artifact persistence/encoding, remote transport/process lifecycle, retries, and cluster
policy. RunenGPU retains physical surface/device execution.

Required proof includes dependency-driven invalidation, bounded history/session state,
device-generation invalidation for realized derived state, compatible continuation,
incompatible-continuation rejection, reconstruction under pressure, and cache-hit
semantic neutrality.

Real R7 result, readback, presentation, and continuity consumers may supply pressure for
the eventual ordinary public API and meaningful visual integration evidence. R7 does not
freeze that public surface merely because proof machinery exists. Any visual proof must
remain separate from semantic/GPU correctness evidence, and persisted image or media
encoding policy remains Runenwerk-owned.

## R8 — public/source-boundary qualification and extraction readiness

R8 is the final predecessor-side qualification before source authority moves. It
qualifies the current reusable renderer as a standalone-shaped public/source boundary;
it does not require unrelated future renderer capabilities merely to satisfy a maturity
matrix.

Required evidence includes:

- exact-current declaration, source, export, direct/transitive consumer, and writer
  census;
- accepted RunenShader handoff for the maintained renderer source under #719-equivalent
  authority: exact maintained WGSL -> RunenShader canonical artifact -> public RunenGPU
  program-source admission, with shader and GPU diagnostics kept separately owned;
- a curated future standalone public export surface rather than a copy of the mixed
  `engine::plugins::render` module tree;
- proof-private constructors, witness types, and implementation vocabulary narrowed
  unless a real public consumer justifies them;
- an ordinary headless/offscreen public path with progressive disclosure that uses the
  same scene/request/planning/admission/lowering/result authority as any retained
  advanced inspection path;
- at least one maintained Runenwerk consumer using that same future-transferable
  semantic surface where applicable, without a predecessor-only semantic bypass;
- structured human-readable public diagnostics for representative request,
  compatibility, semantic-binding/admission, availability, shader-compilation,
  execution, and result-definedness failures;
- public RunenGPU-only physical lowering and no private WGPU/backend reach-through;
- explicit compatibility selection for the exact accepted RunenGPU and RunenShader
  revisions the successor will consume; do not mechanically repin moving upstream
  state;
- result/readback ergonomics that keep semantic result authority distinct from physical
  byte transport and product artifact encoding;
- one exact transfer/stay manifest that separates framework semantic/method source from
  Runenwerk App/ECS/Winit/UI/World/Editor/Render-Lab/product policy;
- a complete Runenwerk consumer migration/deletion map;
- successor repository/package/licensing/provenance/validation requirements and the
  ADR-0008 cross-repository coordination path ready before successor acceptance;
- no essential current consumer requiring a compatibility facade, source mirror,
  moving predecessor dependency, or other duplicate authority.

R8 does **not** require a second maintained render method, GPU-produced semantic input,
elimination of every founding planner/scale limitation, broad multi-observation/output
sharing, or advanced temporal/transport capability before source-authority transfer.
Those remain important standalone framework pressure. One real maintained method proves
independent usefulness for transfer but does not stabilize the generalized method
extension surface.

Exact ordinary API names and signatures remain consumer-gated. Pre-1.0 clean cutovers
remain allowed; no compatibility aliases are justified merely by extraction.

## RX — external RunenRender transfer and clean cutover

Prerequisites:

- R0-R8 accepted;
- exact current-source/consumer/writer census repeated;
- the accepted R8 candidate public/source boundary is independently useful and ready to
  transfer without essential consumers depending on Runenwerk-private renderer
  semantics;
- exact accepted RunenShader and RunenGPU revisions selected;
- no private RunenGPU/WGPU reach-through;
- successor bootstrap and repository-local validation authority ready;
- the Engineering ADR-0008 coordination initiative active before successor acceptance;
- every active Runenwerk consumer migration and predecessor deletion step ready.

Cutover:

1. populate an unmerged `dornglut/runen-render` successor candidate from accepted
   Runenwerk semantic authority by ownership, not by copying the mixed render directory;
2. establish repository-owned standalone validation, documentation, examples, and
   package/source-boundary guards for the transferred current capability;
3. pin exact accepted RunenShader and RunenGPU revisions;
4. prove package-level downstream public-API conformance against the actual
   `runen-render` package;
5. accept the standalone successor through its repository-owned guarded workflow;
6. under Engineering ADR 0008, make that accepted successor revision the sole semantic
   source authority and freeze the still-present predecessor copy;
7. migrate maintained Runenwerk consumers to the exact accepted successor revision and
   delete predecessor semantic-rendering source and temporary seams in the bounded
   downstream cutover;
8. prove no source mirror, forwarding namespace, compatibility package, source include,
   submodule, moving dependency, duplicate renderer, or private backend reach-through
   remains;
9. record provenance, release, accepted revisions, and cross-repository closeout.

RX is transfer/cutover and actual standalone-package proof, not architecture invention
or public-surface repair. Missing reusable contracts discovered after the ADR-0008
authority switch are corrected in standalone RunenRender, not in the frozen predecessor.

## A1 — reusable adapter review

Only after both RunenGPU and RunenRender clean cutovers, review whether any Runenwerk
bridge has at least two independent consumers, stable host-neutral semantics, and enough
maintenance duplication to justify extraction. Do not pre-create adapter packages or
change dependency direction merely because one bridge exists.

## V1+ — successor-owned advanced renderer pressure

After RX, Runenwerk no longer owns an active RunenRender semantic roadmap. The
standalone repository owns further generality, scale, method, conformance, and advanced
capability work through its own accepted roadmap.

Carry-forward pressure includes the former broad R8 maturity requirements: multiple
materially distinct methods before broad method-extension stabilization, GPU-produced
semantic inputs, planner/acceleration scaling, multi-observation/output sharing, bounded
large-scene behavior, and performance/algorithmic characterization.

Advanced renderer capability may then continue through separately accepted protocols,
representations, methods, outputs, semantic inputs, relationships, appearance
extensions, or derived-state kinds. Examples include:

```text
multi-bounce and bidirectional transport
regional / cellular transport
volumes and sparse scientific fields
populations, fibers, hair, liquids, and deformation
spectral and polarized rendering
differentiable and inverse rendering
learned / neural representations and reconstruction
deep output
XR and foveated rendering
multi-device and distributed rendering
hardware-specialized realizations
```

V1+ does not replace or widen the R0 semantic spine by implication. Any new owner,
dependency, stable format, or shared framework still requires its own accepted evidence.

# Advanced compatibility requirements

## Shader/program ownership

RunenRender owns renderer shader/kernel meaning, semantic variants, method-specific
program families, and the explicit consumer bridge from an accepted RunenShader artifact
into RunenGPU program admission. RunenShader owns reusable shader-source
identity/composition, frontend compilation, canonical artifact formation, source
provenance/mapping, and source-facing diagnostics. RunenGPU owns canonical program
admission, interfaces/layouts/binding compatibility, backend realization, and physical
caches. Runenwerk owns product source-root/package policy, filesystem watching/reload
scheduling, integration/recovery, and product last-known-good policy.

## Determinism and reproducibility

Owner-declared determinism may be structural, numerical-within-tolerance, statistical,
or bitwise only under a constrained environment. Applicable method/protocol/input/seed
and environment facts remain inspectable. Runenwerk owns persisted bundles.

## Trust and extension

Versioned renderer extensions remain bounded, typed, and structured. Do not grant
untrusted authoring arbitrary host callbacks, backend access, or unrelated global
resource access. Product trust policy remains Runenwerk-owned.

## Scale invariants

```text
scene publication
    no mandatory deep copy per commit

planning
    no mandatory all-object x all-method scan

GPU work
    stage-scaled, not logical-object-scaled

submission
    no per-object CPU submission requirement

state
    bounded or pressure-reporting

detail
    semantic-support/error driven; never globally materialize unbounded detail

sharing
    compatible observations/outputs may share preparation and derived state

diagnostics
    bounded and aggregatable
```

# Current-source revalidation gate

Before every R implementation slice:

- resolve exact accepted `main`;
- repeat the affected declaration and direct/transitive consumer census;
- inspect all current source paths relevant to the owning phase;
- verify identities and persisted/wire uses;
- identify host/source reach-back and temporary authority;
- bind exact public, migration, deletion, proof, and guard scope;
- run canonical baseline validation;
- stop for a new ADR/package/dependency/stable format/compatibility path/backend escape
  or premature later-phase authority.

Historical reports and prior phases are evidence, not permission to skip current-source
review.

# Shared logical Plan compatibility

RunenRender remains correct whether a future shared logical Plan architecture is
accepted or rejected.

A future shared Plan layer may express, compose, inspect, partition, or orchestrate
renderer operations. It may not absorb RunenRender-owned scene semantics, observation
semantics, representation validity, method semantics, conditional render planning,
semantic binding admission, execution admission, or renderer-specific RunenGPU lowering.

```text
shared logical Plan
    -> RunenRender semantic request / native planning
        -> RunenRender admission / lowering
            -> RunenGPU
```

Provider realization does not transfer semantic ownership.
