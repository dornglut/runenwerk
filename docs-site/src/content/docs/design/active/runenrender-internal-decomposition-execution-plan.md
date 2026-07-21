---
title: RunenGPU and RunenRender Decomposition Execution Plan
description: Dependency-ordered program from the current combined Runenwerk renderer to clean RunenGPU and RunenRender public boundaries and external repositories.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./runengpu-architecture-design.md
  - ./runenrender-decomposition-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU and RunenRender Decomposition Execution Plan

## Purpose

Decompose the current combined Runenwerk renderer into:

```text
RunenGPU
    validated general GPU execution

RunenRender
    image formation through RunenGPU

Runenwerk
    lifecycle, windows, domain extraction, adapters, product policy, recovery
```

The program proves intended public boundaries inside Runenwerk before each clean
external cutover.

This document records the complete sequence. It does not authorize implementation.
Only the next phase receives an exact implementation specification after its
prerequisites and current-source evidence are complete.

## Sequence

```text
S0
-> G1 -> G2 -> G3 -> G4 -> G5 -> G6 -> G7 -> G8
-> GX
-> R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8
-> RX
-> A1
-> V1+
```

Where:

```text
S0   current-source and consumer inventory
G*   internal RunenGPU boundary proof
GX   external RunenGPU clean cutover
R*   internal RunenRender boundary proof on RunenGPU
RX   external RunenRender clean cutover
A1   reusable adapter review
V1+  advanced renderer program
```

## Global invariants

Every phase preserves:

- one public package per target repository;
- no Runenwerk types in future framework contracts;
- no direct WGPU ownership in RunenRender;
- no renderer/domain meaning in RunenGPU;
- no RunenGPU/RunenRender dependency in RunenSDF, RunenECS, or RunenUI core;
- no source mirror, compatibility package, forwarding namespace, submodule, or
  moving-branch dependency;
- no old/new parallel runtime path after a completed cutover;
- deterministic contract evidence separated from GPU/window/environment evidence;
- exact-head `cargo validate` before merge.

## S0 — Complete ownership and consumer inventory

Goal:

- enumerate every current GPU/render file, shader, macro, test, example, benchmark,
  artifact, and downstream consumer;
- classify every responsibility as RunenGPU, RunenRender, Runenwerk, adapter,
  another domain, redesign, or delete;
- classify every current identity and allocator by semantic owner;
- trace context/device/queue/resource/frame/surface/window/shutdown control flow;
- identify persistence, replay, network, cache, trace, and artifact use of runtime
  IDs;
- inventory validation commands and environment-dependent GPU evidence.

Required output:

```text
complete file and consumer inventory
identity and stable-format classification
shader/pipeline/macro ownership map
surface/window/device/drop-order trace
move/stay/redesign/delete matrix
focused and baseline command inventory
first bounded implementation candidate
```

S0 changes no Rust behavior and creates no implementation spec until the output is
reviewed.

Stop when any current owner or consumer remains unknown.

# RunenGPU internal proof

## G1 — Identity, error, and ownership guard

Goal:

- create future-transferable GPU-execution identities and structured errors for the
  smallest current execution spine;
- separate GPU identities from renderer semantic and Runenwerk producer/product
  identities;
- reject invalid, forged, foreign, stale, wrapping, or exhausted identity use;
- establish dependency/source guards for the future RunenGPU boundary.

G1 must not redesign image formation, graph semantics, shaders, resources, WGPU,
surfaces, or producers beyond mechanical identity/error migration.

Prerequisite: accepted S0 identity/consumer classification.

## G2 — Capabilities and resource descriptors

Goal:

- define normalized capabilities and requirement strength;
- define backend-neutral buffer/texture/view/sampler/query descriptors;
- define initialization, lifetime, memory intent, imports, exports, and provenance;
- separate authoritative domain source state from GPU realizations.

Prerequisite: G1 identities/errors.

## G3 — Access, hazard, and workload graph

Goal:

- define ranges/subresources and access categories;
- define immutable work fragments and compute/render/copy/clear/resolve/present
  nodes;
- compose and validate bounded execution graphs;
- reject cycles, hazards, read-before-init, use-after-retire, ambiguous writers,
  and invalid capability/resource combinations.

Prerequisite: G2 resources/capabilities.

## G4 — Shader and pipeline admission

Goal:

- separate source identity/revision/interface intent from filesystem policy;
- define shader/pipeline admission and structured realization failures;
- move WGPU module, binding, and pipeline realization behind the future RunenGPU
  boundary;
- decide macro retention only from actual ABI consumer evidence.

Prerequisites: G2/G3 contracts and S0 macro/shader inventory.

## G5 — Headless compute, upload, and readback

Goal:

- create a context without a window or surface;
- execute compute workloads;
- support staging upload and asynchronous readback;
- prove submission/completion/retirement and terminal shutdown;
- report adapter/device evidence separately from deterministic validation.

Prerequisites: G1–G4.

## G6 — Offscreen graphics and shared consumer proof

Goal:

- execute render workloads into offscreen targets;
- prove one context composes at least one render fragment and one non-render compute
  fragment;
- prove RunenGPU public contracts contain no image-formation or domain meaning;
- establish practical performance and allocation baselines.

Prerequisite: G5 headless execution.

## G7 — Surfaces and device outcomes

Goal:

- admit host-provided raw handles without Winit dependency;
- define surface generations, configuration, acquire/present, resize, retirement,
  thread affinity, drop order, and multi-surface behavior;
- classify device-loss and out-of-memory outcomes;
- keep product recovery in Runenwerk.

Prerequisite: G6 independent device/resource execution.

## G8 — Diagnostics and internal anti-cheating proof

Goal:

- expose structured GPU provenance, timing, surface/device, submission, and terminal
  facts;
- move product presentation/artifact policy to Runenwerk;
- migrate current internal consumers to the same future public boundary;
- remove private reach-through and duplicate GPU paths;
- prove future RunenGPU modules build/test without Runenwerk/domain assumptions.

Prerequisites: G1–G7.

## GX — External RunenGPU clean cutover

Goal:

- create/populate `Crystonix/runen-gpu` with package `runen-gpu`/crate `runen_gpu`;
- preserve source provenance and license;
- establish independent validation, MSRV, docs, and downstream conformance;
- pin Runenwerk to an exact accepted revision;
- migrate every active consumer;
- delete original Runenwerk GPU execution authority and temporary seams.

Completion gate:

- one public package;
- headless compute and offscreen graphics pass;
- one non-render consumer proves independent value;
- Runenwerk and future RunenRender use public APIs only;
- no source mirror or duplicate context/resource/workload path remains.

# RunenRender internal proof

## R1 — Renderer identities and prepared scene

Goal:

- define renderer semantic identities separately from RunenGPU and source-domain
  identities;
- define immutable prepared scene, views, logical targets, and provenance;
- remove planning reach-back into ECS, host windows, UI runtime, simulations, and
  authoring graphs for the touched spine.

Prerequisite: GX accepted RunenGPU dependency and current renderer consumer map.

## R2 — Contributions and deterministic composition

Goal:

- define producer/contribution insert, replace, remove, and retire lifecycle;
- define deterministic composition and conflict handling;
- migrate at least two independent Runenwerk producer families;
- remove product-specific graph variants from the touched path.

Prerequisite: R1 prepared scene.

## R3 — Provider and interaction contracts

Goal:

- define provider families/capabilities and common interactions;
- separate provider intersection strategy from path/ray selection;
- prove analytic and field-capable providers without requiring one representation;
- keep source field/SDF semantics in adapters.

Prerequisite: R1/R2 scene/contribution model.

## R4 — Materials, media, emitters, and environments

Goal:

- define prepared scattering, medium, emitter, and environment contracts;
- separate material authoring/import from rendering semantics;
- preserve source generations and provenance;
- prove multiple provider/material/emitter combinations.

Prerequisite: R3 interactions.

## R5 — Visibility and transport

Goal:

- define query purposes, visibility policy, path state, direct/indirect estimator
  contracts, and quality tiers;
- lower visibility and transport work through RunenGPU only;
- keep hardware ray tracing optional;
- report unsupported transport and degradation explicitly.

Prerequisites: R3/R4 and accepted RunenGPU capabilities.

## R6 — Radiance cache, history, and reconstruction

Goal:

- define discardable world-space directional radiance cache;
- define source-generation validity, variance/confidence, and update policy;
- define bounded history and reconstruction without mandatory stale final-color
  dependence;
- prove disocclusion and dynamic-change invalidation.

Prerequisite: R5 transport.

## R7 — Overlay, color, and presentation intent

Goal:

- define neutral overlay primitives and deterministic composition;
- lower overlay work through RunenGPU;
- prove a RunenUI paint-scene adapter without widget/runtime reach-through;
- define color/output and logical presentation intent while keeping windows and
  surfaces outside RunenRender.

Prerequisite: R2 contributions and accepted RunenGPU render/surface contracts.

## R8 — Runenwerk adapter migration and anti-cheating proof

Goal:

- migrate scene, world, material-authoring, SDF, UI, editor, procedural, simulation,
  and product integrations to explicit public seams;
- move shader filesystem/reload, window/lifecycle, product quality, diagnostics
  presentation, and recovery policy to Runenwerk;
- prove RunenRender has no direct WGPU/Runenwerk/ECS/SDF/UI dependency;
- remove private reach-through and duplicate renderer paths;
- establish conformance and performance baselines.

Prerequisites: R1–R7.

## RX — External RunenRender clean cutover

Goal:

- create/populate `Crystonix/runen-render` with package `runen-render`/crate
  `runen_render`;
- depend on an exact accepted RunenGPU revision;
- establish independent validation and public downstream conformance;
- pin Runenwerk to exact revisions;
- migrate every active consumer;
- delete original Runenwerk image-formation authority and temporary seams.

Completion gate:

- one public package;
- no direct WGPU ownership;
- no Runenwerk/domain assumptions;
- prepared scene/provider/material/transport/overlay contracts validate
  independently;
- Runenwerk consumes public adapter seams only;
- no duplicate renderer path remains.

# Post-extraction work

## A1 — Reusable adapter review

Review Runenwerk bridges only after both framework APIs stabilize.

Candidates include:

- RunenSDF-to-render provider adaptation;
- RunenUI paint-scene overlay adaptation;
- reusable asset/material preparation;
- test/conformance support.

Keep a bridge in Runenwerk unless an independent consumer proves stable ownership.

## V1+ — Advanced renderer program

After RX, advanced work may include:

- field-ray wavefront transport;
- provider-specific acceleration;
- many-light/reservoir sampling;
- directional radiance caches and path guiding;
- liquids, fibers, volumes, translucency, and subsurface transport;
- bounded temporal/spatial reuse;
- stylization and high-quality display pipelines;
- reference rendering and progressive accumulation.

Advanced features do not bypass the accepted provider, interaction, material,
transport, RunenGPU, and validity contracts.

## Phase-spec policy

No implementation spec is active now.

After S0:

1. review current files, consumers, IDs, lifecycles, validation, and disposition;
2. write exactly one G1 specification against current `main`;
3. implement, validate, review, merge, and close G1;
4. write the next spec from resulting facts;
5. repeat through GX, then R1–RX.

Do not prewrite later phase contracts against unimplemented assumptions.

## Parallel work

Allowed during S0 and bounded phases:

- read-only inventory and control-flow tracing;
- focused benchmarks and evidence planning;
- independent RunenECS work that does not share manifests/owners;
- RunenUI work in its own repository;
- separately owned RunenSDF clean-cutover work.

Forbidden:

- concurrent changes to the same GPU/render identity or lifecycle boundary;
- external source movement before internal proof;
- broad renderer rewrite;
- speculative package creation;
- duplicate temporary runtime paths used outside an unmerged branch;
- advanced renderer features that harden accidental current ownership.

## Definition of program completion

The program is complete when RunenGPU and RunenRender each validate independently,
Runenwerk consumes exact revisions through public APIs, source provenance is
recorded, every active consumer is migrated, original implementations are deleted,
adapters contain translation rather than duplicate algorithms, and no dependency
cycle, source mirror, compatibility layer, or duplicate path survives.
