---
title: RunenGPU and RunenRender Decomposition Execution Plan
description: Dependency-ordered program from the current combined Runenwerk renderer to clean RunenGPU and RunenRender public boundaries and external repositories.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-28
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-decomposition-design.md
  - ./runen-family-operational-hardening-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../reports/investigations/runen-family-operational-hardening-investigation.md
  - ../../reports/investigations/runengpu-industry-comparison.md
  - ../../reports/investigations/runengpu-proof-workload-strategy.md
  - ../../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../../reports/closeouts/pt-runen-family-operational-hardening-closeout.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
---

# RunenGPU and RunenRender Decomposition Execution Plan

## Purpose

Decompose the combined Runenwerk renderer into:

```text
RunenGPU
    validated generic GPU execution

RunenRender
    semantic image formation through RunenGPU

Runenwerk
    lifecycle, ECS/domain projection, windows, scheduling, source policy,
    compatibility, recovery, artifacts, and product integration
```

The program proves public boundaries inside Runenwerk before clean external cutovers.
This plan records sequence and phase ownership; only an owning issue and accepted
phase specification authorize implementation.

## Current state

```text
S0 inventory                         complete
G1A resource identity                complete
G2 capabilities/resources            complete at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
G3 decision phase                    complete at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
operational hardening                complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 Rust implementation               candidate corrected after independent review
G4-G8                                pending
GX                                   blocked on G2-G8
R1-R8 and RX                         blocked on GX
```

G3 was implemented from accepted base
`1c645b2bbfcece44dd6ae151cc97559793afa2c2`. The reviewed head
`38abac6bd234d9db3a4544aedbf2dba149538e36` required corrections; corrected code
candidate `905c506e33202405d1bea8c160a05ac92c326c43` remains open, draft, and unmerged
pending fresh exact-head validation and independent review. No G3 merge SHA is
asserted.

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

```text
S0   current-source and consumer inventory
G*   internal RunenGPU public-boundary proof
GX   external RunenGPU clean cutover
R*   internal RunenRender proof on accepted RunenGPU
RX   external RunenRender clean cutover
A1   reusable adapter review
V1+  advanced renderer program
```

Operational hardening adds requirements to existing phases. It does not create G9,
R9, or a shared framework package.

## Global invariants

Every phase preserves:

- one public package per target repository initially;
- no Runenwerk/domain/product types in framework public contracts;
- no direct WGPU ownership in RunenRender;
- no renderer/domain meaning in RunenGPU;
- no dependency from RunenSDF, RunenECS, or RunenUI core to RunenGPU/RunenRender;
- no source mirror, compatibility package, forwarding namespace, submodule,
  source include, or moving-branch dependency;
- no old/new parallel authority after accepted cutover;
- typed identities/references rather than string authority;
- accepted work receives one structured terminal outcome and is not silently dropped;
- bounded queues/caches/history/captures return structured pressure or bounded waits;
- derived caches remain non-authoritative and source-generation-bound;
- Runenwerk owns product recovery, compatibility manifests, persisted captures,
  reproducibility bundles, and artifact encoding;
- deterministic contract evidence remains separate from GPU/window/environment
  evidence;
- proof categories remain separated: correctness, integration, operations, recovery,
  performance, and showcase;
- exact-head `cargo validate`, `git diff --check`, documentation build, and GitHub
  Actions are merge evidence;
- each implementation phase migrates every consumer of replaced authority and deletes
  that authority in the same accepted slice.

## S0 — ownership and consumer inventory

**State: complete.**

S0 classified files, shaders, macros, tests, examples, benchmarks, artifacts,
identities, consumers, lifecycle, persistence, and move/stay/redesign/delete scope.

S0 is historical discovery evidence. Every later slice re-verifies its exact current
subset against `main`.

# RunenGPU internal proof

## G1A — owner-scoped logical work-resource identity

**State: complete.**

Delivered owner-scoped `GpuWorkResourceId`, fallible allocation, foreign-owner
rejection, structured authoring errors, deletion of old renderer identity authority,
and source/dependency guards.

The temporary crate-private `RenderFlowId`-derived owner bridge remains one bounded G3
adapter seam. G4 must delete it when context/work-scope authority exists.

## G2 — capabilities, logical resources, handles, and prepared data

**State: complete at `709aa6aced020ee99405e1e1c3dde7703c77a4d4`.**

Delivered normalized capabilities/requirements, resource descriptors, independent
lifetime/ownership/transfer/reconstruction/memory facts, typed handles, prepared-data
boundaries, explicit render lowering, full consumer migration, and deletion of
replaced authority.

G2 created no device, graph, submission, surface, or external package.

## G3 — access, graph-entry initialization, hazards, work, and preparation

**Planning state: complete at `5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8`.**

**Implementation state: corrected candidate on branch from accepted base
`1c645b2bbfcece44dd6ae151cc97559793afa2c2`; draft PR `#181` awaits new exact-head
independent review at and after corrected code candidate
`905c506e33202405d1bea8c160a05ac92c326c43`.**

Accepted scope:

- checked buffer, texture-subresource, and query ranges;
- exact access categories and graph-entry initialized coverage;
- `QueryResolve` buffer usage;
- render attachment load/clear/store/discard and multisample resolve target;
- standalone checked buffer zero;
- standalone query-set-to-buffer resolve;
- immutable typed compute/render/copy/clear/resolve/present operations;
- operation/access-derived requirements;
- lexical fragment-local hazard orientation;
- typed cross-fragment import/export causality;
- explicit fragment-local non-data order only;
- rejection of redundant data edges, ambiguity, missing causality, cycles,
  read-before-initialization, and inconsistent operations/imports/requirements;
- one deterministic `GpuPreparedWorkGraph` authority;
- one temporary render/GPU-primitive/timing adapter;
- full migration and deletion of replaced renderer-owned generic correctness.

G3 does not create devices, backend leases, runtime generations, submission,
completion, mapping, or retirement.

## G4 — context/device admission, programs, layout, WGPU, and cache compatibility

Goal:

- GPU-owned context/device identity and generation;
- normalized admission and portability classes;
- explicit backend-specific limitations without first-backend leakage;
- WGPU feature/limit/format mapping;
- merged G3 requirement admission;
- shader/program/interface identity separate from filesystem policy;
- validated binding keys and backend layout;
- resource/program/pipeline realization;
- attachment/resolve/query-alignment validation;
- structured realization/degradation failures;
- strict pipeline-cache compatibility facts;
- contained internal WGPU boundary;
- stale-context/generation rejection;
- removal of the temporary `RenderFlowId` owner bridge.

Pipeline cache facts include WGPU/cache schema, backend, adapter, relevant driver,
program/interface generation, pipeline descriptor, enabled features, and limits.
Incompatible cache data is rejected safely and is never correctness authority.

G4 does not execute G5 work.

## G5 — headless execution, progress, pressure, transfer, completion, and retirement

Goal:

- headless context use without windows, renderer, ECS, or product types;
- ordinary prepare-and-submit and explicit inspect/submit-prepared through one
  preparation authority;
- initial uploads, partial updates, staging, multiple dispatches, render attachments,
  and query-resolution encoding;
- asynchronous buffer/texture readback;
- normalized row padding and format provenance;
- explicit native and WebGPU progress model;
- defined progress owner and allowed thread/executor;
- consumer callbacks invoked outside internal locks;
- bounded pending submissions, uploads, maps, readbacks, and backing memory;
- structured pressure outcomes and explicit bounded waits;
- completion delivered exactly once;
- cancellation meaning before/after backend submission and during mapping/shutdown;
- terminal outcomes for all accepted work during shutdown/loss;
- runtime stale-generation/use-after-retirement rejection;
- delayed safe retirement after relevant submissions complete;
- terminal idempotent shutdown.

Required correctness proof:

1. exact inclusive/exclusive `u32` prefix scan over exactly 4,097 elements with full
   output/readback verification;
2. focused counter/compaction/indirect-argument evidence;
3. fixed-seed 160×90 Game of Life for exactly 16 steps with full CPU oracle, exact
   live count `2,063`, FNV-1a-64 checksum `0xBD710B88594CD584`, and selected-cell
   assertions;
4. deterministic compute-to-texture/readback when admitted.

Required operational proof:

- submission saturation;
- readback saturation;
- staging/upload saturation;
- callback/reentrancy;
- lifecycle-point cancellation;
- shutdown with pending work and no lost terminal outcome.

## G6 — offscreen graphics, shared consumers, and narrow overhead comparison

Goal:

- known-pattern offscreen clear/draw with selected-pixel readback;
- compute-generated indirect draw ordered through G3 access;
- one render and one independent non-render consumer on one context;
- offscreen boids as representative integration, not correctness oracle;
- direct-WGPU comparisons for equivalent narrow compute, image-processing, and
  graphics workloads;
- cold/warm program and pipeline characterization;
- CPU preparation/validation, allocations, command recording, staging/readback,
  memory high-water, and GPU timing facts where supported;
- artifact equality/tolerance between framework and direct paths.

Performance evidence is diagnostic. A controlled separately accepted specification is
required before numeric budgets become merge gates.

## G7 — surfaces, generations, device loss, and reconstruction facts

Goal:

- host-provided raw window/display handles without Winit dependency;
- surface identity, generation, configuration, acquisition, presentation, resize,
  reconfiguration, retirement, thread affinity, and multi-surface facts;
- surface-image lease generation and reuse rejection;
- classify outdated, lost, timeout, out-of-memory, device-lost, and backend failures;
- context/device replacement invalidates old backend realizations;
- source-backed resources report reconstructability;
- imported resources report required external reimport;
- non-reconstructable resources report explicit permanent loss;
- Runenwerk retains retry/degrade/recreate/exit policy;
- reuse G6 known-pattern and boids workloads.

G7 does not create a separate surface-only execution architecture.

## G8 — operational conformance, diagnostics, capture facts, and residual audit

Goal:

- structured capability, resource, work, submission, completion, readback,
  pressure, surface/device, cache, generation, reconstruction, and terminal facts;
- no lost accepted work or completion notifications;
- clean shutdown with and without pending work;
- quota saturation and backing-memory release proof;
- cache reuse/rejection proof;
- device-loss invalidation/reconstruction matrix;
- namespaced facts for a Runenwerk-owned versioned reproducibility bundle;
- bounded diagnostics/capture growth and privacy/redaction facts;
- remaining consumers migrated to the future public boundary;
- private WGPU reach-through and temporary adapters removed;
- standalone source/downstream conformance;
- direct-WGPU comparison evidence retained;
- source, dependency, stable-format, and extraction-readiness audits complete.

Runenwerk owns bundle persistence, product severity, UI presentation, recovery, and
artifact codecs.

## GX — external RunenGPU transfer and cutover

Prerequisites:

- G1A-G8 accepted;
- independent one-package conformance;
- exact license/provenance/MSRV/dependency/feature policy;
- at least one render and one independent non-render downstream consumer;
- simple and inspectable paths proven;
- acceptable measured boundary overhead;
- no private WGPU reach-through;
- no duplicate internal path.

Cutover:

1. populate `dornglut/runen-gpu` from accepted authority;
2. validate independently;
3. pin Runenwerk to exact revision/pre-release;
4. migrate active consumers;
5. delete internal authority and temporary seams;
6. prove no mirror, forwarding package, compatibility namespace, submodule, branch
   dependency, or duplicate runtime remains;
7. record closeout.

# RunenRender internal proof

## R1 — renderer semantic identities and prepared-scene foundation

Goal:

- renderer-local identities;
- immutable prepared scene independent of ECS/Runenwerk/WGPU;
- views, logical targets, providers, instances, materials, media, emitters,
  environments, overlays, generations, changed regions, and provenance;
- explicit full-rebuild fallback;
- no raw source-domain IDs as renderer authority.

## R2 — contribution lifecycle and incremental preparation

Goal:

- deterministic producer/contribution insert, replace, remove, and retire-producer;
- conflict/missing-reference diagnostics;
- equivalent full and incremental semantic result;
- affected/unaffected generation and changed-region evidence;
- bounded update-cost characterization;
- at least two independent producer families;
- no ECS mirror world.

## R3 — narrow providers and interaction capabilities

Near-term proof:

```text
Procedural
Analytic
field-backed Solid sufficient for first terrain
Overlay
```

Research candidates:

```text
Volume
Population
RegionalSummary
Liquid
```

Deferred pending accepted consumer evidence:

```text
Fiber
broad hardware-specialized variants
universal provider unification
```

Providers implement only required narrow capabilities such as surface, visibility,
interval, transmittance, raster, material, motion, refinement, or streaming queries.
A universal provider trait is prohibited.

## R4 — materials, media, emitters, and renderer planning

Goal:

- representation-independent scattering/material semantics;
- medium transitions and transmittance;
- unified emitter/environment semantics;
- deterministic image-formation planning;
- no authoring graphs/assets/product registries in RunenRender.

## R5 — visibility, transport, quality, overlays, and current-frame proof

Goal:

- provider-specific query strategies behind shared interactions;
- one semantic transport family and explicit quality ladder;
- current primary visibility/material validity at every tier;
- optional bounded history, not mandatory final-color TAA;
- renderer-neutral overlays and RunenUI bridge proof;
- first procedural sky/SDF terrain semantic proof.

## R6 — render-derived cache and history invalidation

Goal:

- cache/history keys include scene/view/provider/material/quality/algorithm and
  RunenGPU device generations;
- changed-region-aware narrow invalidation;
- full invalidation when facts are incomplete;
- derived caches remain discardable and non-authoritative;
- synthetic volume/provider proof when accepted;
- stale cache use is never quality degradation.

## R7 — target and surface integration through RunenGPU

Goal:

- logical targets lower through RunenGPU resources/surfaces only;
- RunenRender consumes surface/device outcomes without owning windows or product
  recovery;
- offscreen and surface paths share semantic render planning;
- no direct WGPU dependency.

## R8 — renderer operational, performance, capture, and anti-cheating conformance

Goal:

- full versus incremental preparation cost;
- provider-query counts/divergence;
- cache hit/miss/invalidation;
- current-frame and history-dependent paths;
- CPU/GPU memory high-water marks;
- cold/warm pipeline cost inherited through RunenGPU;
- artifact/capture reproducibility;
- namespaced renderer facts for Runenwerk bundles;
- comparison with a simpler renderer/direct path for the same proof;
- no RunenGPU/WGPU private reach-through;
- downstream public-API conformance and extraction readiness.

## RX — external RunenRender transfer and cutover

Prerequisites:

- accepted external RunenGPU;
- R1-R8 accepted;
- independent package validation and public downstream proof;
- exact RunenGPU revision;
- at least two producer families and multiple provider capabilities;
- incremental preparation and cache/history proof;
- no direct WGPU dependency or Runenwerk/source-domain types;
- acceptable measured value versus simpler alternatives.

Cutover mirrors GX: populate `dornglut/runen-render`, validate, pin exact revision,
migrate consumers, delete internal image-formation authority and temporary seams, and
record closeout with no duplicate path.

## A1 — reusable adapter review

Only after both clean cutovers, review whether any Runenwerk bridge has two independent
consumers and stable reusable ownership. Do not pre-create adapter packages.

## V1+ — advanced renderer work

Advanced provider families, hardware-specialized acceleration, sophisticated
transport, many-light reuse, large caches, or application-domain adapters require
separate accepted work after foundational cutovers and proof.

## Phase start checklist

Before every implementation issue:

- verify exact current `main` and accepted base;
- repeat affected declaration/direct/transitive consumer census;
- run canonical baseline validation;
- confirm no accepted stable persisted/replay/wire/cache/external format changes;
- bind exact public/migration/deletion/test scope;
- stop for a new ADR, package, dependency direction, compatibility path, or premature
  later-phase authority.

## Strategic reevaluation gates

Reconsider RunenGPU if no independent non-render consumer exists, ordinary consumers
need raw WGPU, or measured overhead lacks reusable correctness value.

Reconsider RunenRender if a smaller renderer satisfies all accepted proofs, provider
interfaces become universal/runtime-heavy, or incremental prepared scenes require
systematic full rebuilds.

Reevaluation is explicit architecture work, not permission for a hidden bypass.
