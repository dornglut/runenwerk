---
title: RunenGPU and RunenRender Decomposition Execution Plan
description: Dependency-ordered program from the combined Runenwerk renderer to clean RunenGPU and RunenRender public boundaries and external repositories.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ./runengpu-architecture-design.md
  - ./runenrender-decomposition-design.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-render-s0-inventory.md
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

Every future public boundary is proved inside Runenwerk before one clean external cutover. No phase authorizes later phases implicitly.

## Current state

```text
S0 inventory                         complete
G1A work-resource identity           complete
G2 planning                          active through issue #168
G2 implementation                    next bounded slice after accepted planning
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

`G1A` is the accepted first bounded part of the earlier broad G1 identity/error concept. Further identity/error work is introduced only by the phase that owns it.

## Global invariants

Every phase preserves:

- one public package per target repository;
- no Runenwerk/domain types in future framework contracts;
- no direct WGPU ownership in RunenRender;
- no renderer/domain meaning in RunenGPU;
- no RunenGPU/RunenRender dependency in RunenSDF, RunenECS, or RunenUI core;
- no source mirror, compatibility package, forwarding namespace, submodule, or moving-branch dependency;
- no old/new parallel authority after a completed cutover;
- typed identities and references instead of string authority;
- deterministic contract evidence separated from GPU/window/environment evidence;
- exact-head `cargo validate` and `git diff --check` before merge;
- incremental migration and deletion through G2-G7 rather than deferring cleanup to G8.

## S0 — Source and consumer inventory

**State: complete.**

Delivered:

- complete file and consumer inventory;
- identity and stable-format classification;
- shader/pipeline/macro ownership map;
- context/device/surface/window/drop-order traces;
- move/stay/redesign/delete matrix;
- validation command inventory;
- first bounded implementation candidate.

S0 remains historical evidence. It does not override current-main investigation for later phases.

# RunenGPU internal proof

## G1A — Owner-scoped logical work-resource identity

**State: complete.**

Delivered:

- future-transferable `GpuWorkResourceId`;
- owner-scoped identity and fallible allocation;
- foreign-handle rejection;
- structured authoring-error propagation;
- deletion of the old renderer-owned identity authority;
- dependency guards for the first GPU boundary.

The current crate-private allocator bridge is seeded from `RenderFlowId`. It is temporary and removed through G3/G4 when GPU-owned work/context authority exists.

## G2 — Capabilities, logical resources, typed handles, and prepared data

**State: active planning/next implementation slice.**

Goal:

- define normalized capabilities and requirement strength;
- define backend-neutral buffer, texture, texture-view, sampler, and query-set descriptors;
- model lifetime, ownership, transfer/observation, reconstruction, and memory intent independently;
- define kind-typed logical handles;
- define explicit uniform, storage, vertex, indirect, transfer, and readback-decoding boundaries;
- separate ECS/domain preparation from GPU contracts;
- split current `RenderFlow` declaration authority without moving the facade wholesale;
- migrate and delete replaced capability/resource authority.

G2 does not create a device, pipeline, command, submission, upload, readback, surface, or external package.

Prerequisite: accepted G1A and a current-main G2 census/specification.

## G3 — Access, hazards, immutable work, and internal graph

Goal:

- define buffer ranges and texture subresources used by work;
- define access categories and initialization facts;
- define immutable generic compute/render/copy/clear/resolve/present work;
- infer data dependencies from typed accesses;
- retain explicit ordering only for non-data dependencies;
- compose and validate one deterministic internal graph;
- reject cycles, ambiguous writers, read-before-init, use-after-retire, and invalid combinations.

The graph is an internal correctness and inspection model. It is not mandatory common-path ceremony.

Prerequisite: accepted G2 resources and capabilities.

## G4 — Context/device admission, shaders, pipelines, and WGPU realization

Goal:

- create GPU-owned context and device admission authority;
- map WGPU features, limits, and format facts into normalized G2 capabilities;
- separate shader source identity/interface intent from filesystem and hot-reload policy;
- define shader and pipeline admission;
- expose validated binding keys rather than string binding authority;
- bind backend uniform/storage/vertex/indirect layout and derive/macro realization;
- realize logical resources, shaders, and pipelines through WGPU;
- remove the temporary `RenderFlowId` resource-owner bridge.

G4 owns context/device and backend realization. It does not execute the G5 proof portfolio.

Prerequisites: accepted G2/G3 contracts and current shader/macro consumer evidence.

## G5 — Headless execution, uploads, completion, readback, cancellation, and retirement

Goal:

- execute headless compute without window, surface, renderer, ECS, or product types;
- implement automatic prepare-and-submit and explicit prepare/inspect/submit-prepared through one authority;
- support initial uploads, full and partial updates, staging, and multiple dispatches;
- expose completion and cancellation;
- provide asynchronous buffer and texture readback;
- connect last-handle drop to delayed safe backend retirement;
- prove terminal shutdown.

Required proof ladder:

1. exact inclusive/exclusive 4,097-element `u32` prefix scan;
2. counter reset, scatter/compaction, and indirect-argument focused evidence;
3. headless fixed-seed Game of Life with full-grid oracle, exact checksum, and selected-cell assertions;
4. conditional integer compute-to-texture and row-padding normalization when admitted by G5 scope.

Prerequisites: G1A-G4.

## G6 — Offscreen graphics and shared consumer proof

Goal:

- execute a known-pattern offscreen draw with texture readback and selected-pixel assertions;
- execute a compute-generated indirect draw whose order is inferred from shared resource access;
- compose render and non-render work on one context through the same generic contract;
- run offscreen boids as a representative showcase;
- record structural and bounded invariants separately from tolerant visual evidence;
- establish environment-bound measurements without unbound pass/fail thresholds.

Boids is not the primary correctness oracle.

Prerequisite: G5 headless execution.

## G7 — Surfaces and device outcomes

Goal:

- admit host-provided raw window/display handles without Winit dependency;
- define surface generations, configuration, acquire/present, resize, retirement, thread affinity, drop order, and multi-surface behavior;
- classify device loss, out-of-memory, timeout, outdated, lost, and reconfiguration outcomes;
- keep product recovery and window/event policy in Runenwerk;
- reuse the accepted G6 known-pattern and boids workloads.

G7 does not create a separate surface-only execution architecture.

Prerequisite: G6 independent device/resource execution.

## G8 — Final diagnostics, shutdown, conformance, and residual audit

Goal:

- expose structured GPU provenance, capability, timing, resource, submission, completion, device, surface, and terminal facts;
- prove orderly shutdown and no in-flight lifecycle leaks;
- migrate remaining current internal consumers to the same future public boundary;
- remove private reach-through, temporary adapters, and residual duplicate GPU paths;
- prove future RunenGPU source builds/tests without Runenwerk/domain assumptions;
- complete final source, dependency, consumer, and external-cutover readiness audits.

G8 is not the phase where G2-G7 postpone normal migration or deletion.

Prerequisites: G1A-G7.

## GX — External RunenGPU clean cutover

Goal:

- create and populate `dornglut/runen-gpu` with package `runen-gpu` and crate `runen_gpu`;
- preserve source provenance and license;
- establish independent validation, MSRV, docs, and downstream conformance;
- pin Runenwerk to an exact accepted revision;
- migrate every active consumer;
- delete original Runenwerk GPU execution authority and temporary seams.

Completion gate:

- one public package;
- headless compute and offscreen graphics pass;
- one independent non-render consumer;
- Runenwerk and future RunenRender use public APIs only;
- no source mirror, forwarding package, duplicate context, duplicate descriptor, or duplicate execution path remains.

# RunenRender internal proof

RunenRender begins only after GX is accepted.

## R1 — Renderer identities and prepared scene

Define renderer-owned identities, immutable prepared scenes, views, logical targets, and provenance. Remove planning reach-back into ECS, windows, UI runtime, simulations, and authoring graphs for the touched spine.

## R2 — Contributions and deterministic composition

Define contribution insert/replace/remove/retire lifecycle, deterministic composition, conflict handling, and at least two independent Runenwerk producer families.

## R3 — Providers and interactions

Define provider families, capabilities, interactions, and intersection strategy. Keep source field/SDF semantics in adapters.

## R4 — Materials, media, emitters, and environments

Define prepared scattering, medium, emitter, and environment contracts while keeping authoring/import outside renderer execution.

## R5 — Visibility and transport

Define query purposes, visibility policy, path state, estimator contracts, and quality tiers. Lower all GPU work through RunenGPU.

## R6 — Radiance cache, history, and reconstruction

Define source-generation validity, bounded world-space cache/history, confidence, update policy, disocclusion, and dynamic-change invalidation.

## R7 — Overlay, color, and presentation intent

Define neutral overlays, color/output, and logical presentation intent while keeping windows and surfaces outside RunenRender.

## R8 — Runenwerk adapter migration and anti-cheating proof

Migrate scene, world, material, SDF, UI, editor, procedural, simulation, and product integrations to public seams. Remove direct WGPU, ECS, Runenwerk, SDF, and UI reach-through from RunenRender.

## RX — External RunenRender clean cutover

Create and populate `dornglut/runen-render`, depend on an exact accepted RunenGPU revision, establish independent conformance, migrate Runenwerk, and delete internal image-formation authority without a dual path.

# Proof categories

Every phase classifies evidence as:

```text
deterministic conformance
boundary integration
visual showcase
benchmark or stress evidence
```

A visually impressive workload cannot replace exact correctness evidence. Performance measurements are not acceptance thresholds until hardware, driver, OS, backend, power state, build mode, workload, and measurement method are bound.

# Offline output boundary

Preferred sequence:

1. Game of Life PNG sequence after G5.
2. Offscreen boids PNG sequence after G6.
3. Procedural sky/SDF/scene sequences after matching RunenRender phases.

Runenwerk owns output clocks, seeds, jobs, bounded in-flight readbacks, filenames, manifests, retries, PNG/EXR encoding, and external video encoding. RunenGPU owns completion/readback facts. RunenRender owns image formation.

# Phase-spec policy

For each bounded phase:

1. verify actual current `main`;
2. inspect declarations, direct/transitive consumers, backend users, tests, examples, benchmarks, diagnostics, and stable formats;
3. write one decision-complete implementation specification;
4. create one owning implementation issue;
5. implement only the authorized slice;
6. run focused and canonical validation on the exact head;
7. critically review the complete diff;
8. merge only with exact-head evidence;
9. publish closeout evidence and update durable state;
10. write the next specification from resulting facts.

Do not prewrite concrete later-phase Rust contracts against unimplemented assumptions.

# Stop conditions

Stop rather than widen scope when:

- an affected value is a stable persisted, replay, network, wire, cache, or external format;
- ownership cannot be separated without an ADR-level decision;
- the phase requires implementing a later phase to remain coherent;
- compatibility aliases, forwarding modules, or duplicate authority appear necessary;
- typed-layout safety cannot be established;
- the current consumer census is materially incomplete;
- a proof requires unrelated renderer/domain architecture as a conformance gate;
- current `main` fails canonical validation for an unrelated reason.

# Parallel work

Allowed:

- read-only inventory and control-flow tracing;
- focused benchmark design without unbound thresholds;
- independent repository work that does not share manifests, owners, or migration seams.

Not allowed:

- parallel implementation of later RunenGPU/RunenRender phases;
- external package creation before its clean-cutover gate;
- compatibility architecture to keep old and new execution paths alive.
