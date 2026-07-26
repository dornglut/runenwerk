---
title: RunenGPU and RunenRender Decomposition Execution Plan
description: Dependency-ordered program from the current combined Runenwerk renderer to clean RunenGPU and RunenRender public boundaries and external repositories.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ./runengpu-architecture-design.md
  - ./runenrender-decomposition-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
  - ../../reports/investigations/runengpu-render-s0-file-disposition.md
  - ../../reports/investigations/runengpu-g2-capabilities-resources-investigation.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../workspace/specs/pt-runengpu-g2-capabilities-resource-descriptors.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU and RunenRender Decomposition Execution Plan

## Purpose

Decompose the current combined Runenwerk renderer into:

```text
RunenGPU
    validated generic GPU execution

RunenRender
    semantic image formation through RunenGPU

Runenwerk
    lifecycle, ECS/domain projection, windows, scheduling, source policy,
    artifacts, recovery, and product integration
```

The program proves intended public boundaries inside Runenwerk before each clean external cutover. This document records the complete sequence; it does not authorize implementation by itself. Only one next phase receives an exact current-main implementation specification and owning issue after its prerequisites are accepted.

## Current state

```text
S0 inventory                         complete
G1A work-resource identity           complete
G2 decision phase                    issue #168 / PR #171
G2 implementation                    issue #172, blocked on PR #171
G3-G8                                pending
GX                                   blocked on G2-G8
R1-R8 and RX                         blocked on GX
```

Target repositories:

```text
dornglut/runen-gpu
dornglut/runen-render
```

## Sequence

```text
S0
-> G1A -> G2 -> G3 -> G4 -> G5 -> G6 -> G7 -> G8
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
R*   internal RunenRender boundary proof on accepted RunenGPU
RX   external RunenRender clean cutover
A1   reusable adapter review
V1+  advanced renderer program
```

`G1A` is the accepted first bounded part of the earlier broad G1 identity/error concept. Later identity/error contracts are introduced only by the phase that owns them.

## Global invariants

Every phase preserves:

- one public package per target repository;
- no Runenwerk or domain types in future framework contracts;
- no direct WGPU ownership in RunenRender;
- no renderer/domain meaning in RunenGPU;
- no RunenGPU/RunenRender dependency in RunenSDF, RunenECS, or RunenUI core;
- no source mirror, compatibility package, forwarding namespace, submodule, source include, or moving-branch dependency;
- no old/new parallel authority after a completed cutover;
- typed identities/references rather than string authority;
- deterministic contract evidence separated from GPU/window/environment evidence;
- proof categories separated into deterministic conformance, boundary integration, visual showcase, and benchmark/stress evidence;
- exact-head `cargo validate` and `git diff --check` before merge;
- exact-head GitHub Actions as merge evidence;
- incremental migration and deletion through G2-G7 rather than deferring cleanup to G8.

## S0 — Complete ownership and consumer inventory

**State: complete.**

Delivered:

- every current GPU/render file, shader, macro, test, example, benchmark, artifact, and downstream consumer;
- responsibility classification as RunenGPU, RunenRender, Runenwerk, adapter, another domain, redesign, or delete;
- identity and allocator classification by semantic owner;
- context/device/queue/resource/frame/surface/window/shutdown control-flow traces;
- persistence, replay, network, wire, cache, trace, and artifact usage classification;
- shader/pipeline/macro ownership map;
- move/stay/redesign/delete matrix;
- focused and canonical validation command inventory;
- first bounded implementation candidate.

S0 is historical evidence. Each later phase re-verifies its current-main subset. S0 does not authorize later implementation or override newer evidence.

# RunenGPU internal proof

## G1A — Owner-scoped logical work-resource identity

**State: complete.**

Delivered:

- future-transferable `GpuWorkResourceId`;
- private owner scope and nonzero local value;
- owner-controlled fallible allocation;
- foreign-owner rejection;
- structured authoring-error propagation;
- deletion of old renderer-owned resource identity authority;
- dependency/source guards for the first future RunenGPU boundary.

G1A did not redesign image formation, graph semantics, shaders, resources, WGPU, surfaces, or producers beyond the accepted identity/error migration.

The current crate-private allocator bridge is seeded from `RenderFlowId`. It is temporary and removed through G3/G4 when GPU-owned work/context authority exists.

## G2 — Capabilities, logical resources, typed handles, and prepared data

**State: decision complete in issue `#168` / PR `#171`; implementation issue `#172` is queued and blocked on the planning merge.**

Goal:

- define normalized capability facts and `Required`, `Preferred` with explicit fallback, and `Disabled` requirement strength;
- define backend-neutral buffer, texture, texture-view, sampler, and timestamp query-set descriptors;
- model resource kind, lifetime, ownership, transfer/observation, reconstruction, and memory intent independently;
- distinguish buffer initialization from texture initialization, including checked texture format, extent, `bytes_per_row`, and `rows_per_image`;
- define kind-typed logical handles whose construction and cross-kind conversion remain private;
- bind texture-view validity to the parent texture lease and checked subresource range;
- define explicit uniform, storage, vertex, indirect, transfer, texture-initialization, and readback-decoding boundaries;
- separate ECS/domain preparation from RunenGPU contracts;
- retain labels and provenance as diagnostics/reconstruction evidence rather than identity/binding authority;
- split current `RenderFlow` declaration authority without moving the facade wholesale;
- migrate every declaration/direct/transitive consumer of the authority G2 replaces;
- delete replaced capability/resource/handle authority without aliases, forwarding modules, or duplicate paths.

G2 does not create a device, queue, shader/pipeline realization, work graph, submission, upload, readback, surface, or external package.

Prerequisites: accepted S0/G1A and the current-main G2 investigation/specification.

Stop conditions include stable-format evidence, an ADR-level owner conflict, need for a later phase to make G2 coherent, typed-layout safety failure, incomplete consumer census, compatibility/duplicate authority pressure, or unrelated current-main canonical validation failure.

## G3 — Access, initialization flow, hazards, immutable work, and internal graph

Goal:

- define buffer ranges and texture subresources used by work;
- define access categories and initialization facts;
- define immutable generic compute/render/copy/clear/resolve/present work fragments;
- infer data dependencies from typed accesses;
- preserve explicit ordering only for real non-data constraints;
- compose and validate one deterministic internal graph;
- reject duplicate/foreign/stale identities, cycles, ambiguous writers, read-before-init, use-after-retire, invalid capability/resource combinations, and inconsistent imports/exports.

The graph is the shared internal correctness/inspection authority. It is not mandatory common-path ceremony.

Prerequisite: accepted G2 resources/capabilities/handles/prepared-data contracts.

## G4 — Context/device admission, shaders, pipelines, binding/layout, and WGPU realization

Goal:

- create GPU-owned context and device admission authority;
- map WGPU features, limits, and format facts into normalized G2 capabilities;
- separate shader source identity/revision/interface intent from filesystem and hot-reload policy;
- define shader and pipeline admission and structured realization failures;
- expose validated binding keys rather than string binding authority;
- bind backend uniform/storage/vertex/indirect layout and derive/macro realization;
- realize logical resources, shader modules, and pipelines through WGPU;
- contain WGPU-specific facts behind normalized contracts;
- remove the temporary `RenderFlowId` resource-owner bridge.

G4 owns context/device and backend realization. It does not execute the G5 proof portfolio.

Prerequisites: accepted G2/G3 contracts and current shader/pipeline/macro consumer evidence.

## G5 — Headless execution, uploads, completion, readback, cancellation, and retirement

Goal:

- create/use a context without window, surface, renderer, ECS, or product types;
- implement ordinary automatic prepare-and-submit and explicit prepare/inspect/submit-prepared through one authority;
- support initial uploads, full and partial updates, staging, and multiple dispatches;
- expose submission completion and cancellation;
- provide asynchronous buffer and texture readback without blocking submission authority;
- normalize texture-to-buffer row padding and format provenance;
- connect last public handle drop to delayed safe backend retirement after relevant submissions complete;
- prove terminal and idempotent shutdown;
- report deterministic planning evidence separately from adapter/device environment evidence.

Required proof ladder:

1. exact inclusive and exclusive 4,097-element `u32` prefix scan;
2. counter reset, scatter/compaction, and indirect-argument focused evidence;
3. headless fixed-seed 160×90 Game of Life for 16 steps with full-grid CPU oracle, live count `2,063`, FNV-1a-64 checksum `0xBD710B88594CD584`, and selected-cell assertions;
4. conditional deterministic integer compute-to-texture and row-padding normalization when admitted by G5 scope.

Prerequisites: G1A-G4.

## G6 — Offscreen graphics and shared consumer proof

Goal:

- execute a known-pattern offscreen clear/draw with texture readback and selected-pixel assertions;
- execute a compute-generated indirect draw whose ordering is inferred from shared resource access;
- prove one context composes at least one render contribution and one independent non-render compute contribution through the same generic contract;
- prove RunenGPU public contracts contain no image-formation or domain meaning;
- run offscreen boids as a representative shared compute/render showcase;
- validate boids with structural graph/resource evidence, agent-count invariants, finite values, bounded positions/ranges, overflow checks, and successful artifact generation rather than exact cross-backend floating-point equality;
- establish environment-bound measurements without unbound performance thresholds.

Boids is not the primary correctness oracle.

Prerequisite: G5 headless execution, completion, and readback.

## G7 — Surfaces, generations, thread affinity, and device outcomes

Goal:

- admit host-provided raw window/display handles without Winit dependency;
- define surface generations, configuration, acquisition, presentation, resize, reconfiguration, retirement, thread affinity, drop order, and multi-surface behavior where supported;
- classify device loss, out-of-memory, timeout, outdated, lost, and reconfiguration outcomes;
- keep window/event policy and product recovery in Runenwerk;
- reuse the accepted G6 known-pattern and boids workloads.

G7 does not create a separate surface-only execution architecture.

Prerequisite: G6 independent device/resource/offscreen execution.

## G8 — Final diagnostics, shutdown, conformance, and residual audit

Goal:

- expose structured GPU provenance, capability, timing, resource, work, submission, completion, readback, surface/device, and terminal facts;
- prove orderly shutdown and no in-flight lifecycle leaks;
- keep severity, storage, user presentation, artifact policy, and recovery in Runenwerk;
- migrate any remaining internal consumers to the same future public boundary;
- remove private reach-through, temporary adapters, and residual duplicate GPU paths;
- prove future RunenGPU source builds/tests without Runenwerk/domain assumptions;
- retain the narrow deterministic and environment-dependent conformance suite;
- complete source, dependency, consumer, stable-format, and external-cutover readiness audits.

G8 is not the phase where G2-G7 postpone ordinary migration/deletion.

Prerequisites: G1A-G7.

## GX — External RunenGPU clean cutover

Goal:

- create and populate `dornglut/runen-gpu` with package `runen-gpu` and crate `runen_gpu`;
- preserve source provenance and license;
- establish independent locked validation, declared Rust edition/MSRV, documentation, and downstream conformance;
- pin Runenwerk to an exact accepted revision;
- migrate every active consumer;
- delete original Runenwerk GPU execution authority and temporary seams.

Completion gate:

- one public package;
- headless compute, uploads, asynchronous readback, offscreen graphics, and surfaces pass;
- one independent non-render consumer proves value;
- one render consumer and one non-render consumer share the same context/generic work API;
- Runenwerk and future RunenRender use public APIs only;
- no Runenwerk/domain types in public contracts;
- no source mirror, forwarding package, compatibility namespace, source include, submodule, moving-branch dependency, duplicate context, duplicate descriptor, or duplicate execution path remains.

# RunenRender internal proof

RunenRender work begins only after GX is accepted and Runenwerk consumes the external RunenGPU package through public APIs.

## R1 — Renderer identities and prepared scene

Goal:

- define renderer semantic identities separately from RunenGPU and source-domain identities;
- define immutable prepared scenes, views, logical targets, and provenance;
- remove planning reach-back into ECS, host windows, UI runtime, simulations, and authoring graphs for the touched spine;
- prove source/domain generation changes invalidate prepared render state explicitly.

Prerequisite: GX accepted RunenGPU dependency and current renderer consumer map.

## R2 — Contributions and deterministic composition

Goal:

- define producer/contribution insert, replace, remove, and retire lifecycle;
- define deterministic composition and conflict handling;
- migrate at least two independent Runenwerk producer families;
- remove product-specific graph variants from the touched path;
- preserve contributor provenance into lowered RunenGPU work.

Prerequisite: R1 prepared scene.

## R3 — Provider and interaction contracts

Goal:

- define provider families/capabilities and common interactions;
- separate provider intersection strategy from path/ray selection;
- prove analytic and field-capable providers without requiring one representation;
- keep source field/SDF semantics in adapters;
- make unsupported provider/interaction combinations structured rather than implicit.

Prerequisite: R1/R2 scene/contribution model.

## R4 — Materials, media, emitters, and environments

Goal:

- define prepared scattering, medium, emitter, and environment contracts;
- separate material authoring/import from rendering semantics;
- preserve source generations and provenance;
- prove multiple provider/material/emitter combinations;
- lower required GPU resources/work only through accepted RunenGPU contracts.

Prerequisite: R3 interactions.

## R5 — Visibility and transport

Goal:

- define query purposes, visibility policy, path state, direct/indirect estimator contracts, and quality tiers;
- lower visibility and transport work through RunenGPU only;
- keep hardware ray tracing optional;
- preserve compute-based field traversal as a valid baseline;
- report unsupported transport and degradation explicitly.

Prerequisites: R3/R4 and accepted RunenGPU capabilities.

## R6 — Radiance cache, history, and reconstruction

Goal:

- define discardable world-space directional radiance cache;
- define source-generation validity, variance/confidence, and update policy;
- define bounded history and reconstruction without mandatory stale final-color dependence;
- prove disocclusion and dynamic-change invalidation;
- keep reconstruction/history semantics in RunenRender rather than RunenGPU.

Prerequisite: R5 transport.

## R7 — Overlay, color, and presentation intent

Goal:

- define neutral overlay primitives and deterministic composition;
- lower overlay work through RunenGPU;
- prove a RunenUI paint-scene adapter without widget/runtime reach-through;
- define color/output and logical presentation intent while keeping windows/surfaces outside RunenRender;
- preserve RunenGPU surface facts as execution facts rather than image semantics.

Prerequisites: R2 contributions and accepted RunenGPU render/surface contracts.

## R8 — Runenwerk adapter migration and anti-cheating proof

Goal:

- migrate scene, world, material-authoring, SDF, UI, editor, procedural, simulation, and product integrations to explicit public seams;
- move shader filesystem/reload, window/lifecycle, product quality, diagnostics presentation, capture/artifact, and recovery policy to Runenwerk;
- prove RunenRender has no direct WGPU, Runenwerk, ECS, SDF, UI, or application dependency;
- remove private reach-through and duplicate renderer paths;
- establish independent conformance and environment-bound performance evidence.

Prerequisites: R1-R7.

## RX — External RunenRender clean cutover

Goal:

- create and populate `dornglut/runen-render` with package `runen-render` and crate `runen_render`;
- depend on an exact accepted RunenGPU revision;
- establish independent locked validation, declared Rust edition/MSRV, documentation, and public downstream conformance;
- pin Runenwerk to exact accepted revisions;
- migrate every active consumer;
- delete original Runenwerk image-formation authority and temporary seams.

Completion gate:

- one public package;
- no direct WGPU ownership;
- no Runenwerk/domain assumptions;
- prepared scene/provider/material/transport/reconstruction/overlay contracts validate independently;
- Runenwerk consumes public adapter seams only;
- no source mirror, forwarding package, compatibility namespace, or duplicate renderer path remains.

# Post-extraction work

## A1 — Reusable adapter review

Review Runenwerk bridges only after both framework APIs stabilize.

Candidates include:

- RunenSDF-to-render provider adaptation;
- RunenUI paint-scene overlay adaptation;
- reusable asset/material preparation;
- conformance/test support.

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

Advanced features do not bypass accepted provider, interaction, material, transport, RunenGPU, validity, or ownership contracts.

# Proof and offline-output policy

## Proof categories

Every phase classifies evidence as:

```text
deterministic conformance
boundary integration
visual showcase
benchmark or stress evidence
```

A visually impressive workload cannot replace exact correctness evidence. Performance measurements are not acceptance thresholds until hardware, driver, OS, backend, power state, build mode, workload, and method are bound.

## First RunenRender proofs

- procedural sky/SDF terrain is the first semantic image-formation proof after standalone RunenGPU acceptance;
- boids follows as simulation-to-render integration;
- the SDF history flow is a later temporal/history ownership proof.

## Offline output

Preferred sequence:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky/SDF/scene sequences after matching RunenRender phases.

Runenwerk owns output clocks, seeds, jobs, bounded in-flight readbacks, filenames, manifests, retries, PNG/EXR encoding, and external video encoding. RunenGPU owns completion/readback facts. RunenRender owns image formation. Neither owns MP4/WebM codecs.

# Phase-spec policy

For each bounded phase:

1. verify actual current `main`;
2. inspect declarations, direct/transitive consumers, backend users, tests, examples, benchmarks, diagnostics, and stable formats;
3. write exactly one decision-complete implementation specification;
4. create exactly one owning implementation issue;
5. implement only the authorized slice;
6. run focused and canonical validation on the exact head;
7. critically review the complete diff;
8. address review/CI failures;
9. merge only with exact-head evidence;
10. publish closeout evidence and update durable state;
11. write the next specification from resulting facts.

Do not prewrite concrete later-phase Rust contracts against unimplemented assumptions.

# Stop conditions

Stop rather than widen scope when:

- an affected value is a stable persisted, replay, network, wire, cache, or external format;
- ownership cannot be separated without an ADR-level decision;
- coherent phase types require implementing a later phase;
- compatibility aliases, forwarding modules, or duplicate authority appear necessary;
- typed-layout safety cannot be established;
- the current consumer census is materially incomplete;
- a proof requires unrelated renderer/domain architecture as a conformance gate;
- current `main` fails canonical validation for an unrelated reason.

# Parallel work

Allowed during bounded phases:

- read-only inventory and control-flow tracing;
- focused benchmark/evidence planning without unbound thresholds;
- independent RunenECS work that does not share manifests, identities, lifecycle owners, or migration seams;
- RunenUI work in its own repository;
- independently owned framework maintenance.

Forbidden:

- parallel implementation of later RunenGPU/RunenRender phases;
- concurrent changes to the same GPU/render identity, descriptor, lifecycle, or canonical planning authority;
- external source movement before the clean-cutover gate;
- broad renderer rewrite;
- speculative package creation;
- compatibility architecture or duplicate temporary runtime paths;
- advanced renderer features that harden accidental current ownership.

# Definition of program completion

The program is complete when RunenGPU and RunenRender each validate independently, Runenwerk consumes exact accepted revisions through public APIs, source provenance and MSRV are recorded, every active consumer is migrated, original implementations and temporary seams are deleted, adapters contain translation rather than duplicate algorithms, proof categories remain distinct, and no dependency cycle, source mirror, compatibility layer, forwarding path, moving-branch dependency, or duplicate execution/render path survives.
