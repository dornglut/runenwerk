---
title: RunenRender Internal Decomposition Execution Plan
description: Dependency-ordered internal repair roadmap from the current Runenwerk renderer plugin to independently conformant RunenRender package boundaries.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ./runenrender-decomposition-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/specs/pt-runenrender-r1-identities-errors.ron
  - ../../workspace/planning/roadmap.md
---

# RunenRender Internal Decomposition Execution Plan

## Purpose

Separate neutral renderer contracts, WGPU backend behavior, and Runenwerk
integration through small dependency-ordered phases before external extraction.

This roadmap records the complete destination. It does not pre-authorize all
phases. Only the next executable phase receives a RON specification.

## Sequence

```text
R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8 -> R9 -> R10
```

Later phases may be investigated in parallel. Implementation cannot skip an unmet
prerequisite or unresolved ownership/safety contract.

## R1 — Neutral identities, errors, and dependency guards

Goal:

- replace renderer-neutral identity dependence on Runenwerk `id`/`id_macros` with
  an internally owned, future-transferable identity contract;
- define structured renderer planning/resource/terminal errors for the touched
  API spine;
- remove saturation/wrap/forging ambiguity from renderer ID allocation;
- establish compile-time/source guards for forbidden core dependencies;
- inventory every current ID/error consumer before graph restructuring.

R1 does not create external packages, redesign graph operations, move WGPU, or
migrate domain producers.

## R2 — Neutral graph and resource descriptors

Goal:

- remove built-in UI/material/SDF/world/editor/product variants;
- remove host `TypeId` callbacks and ECS state projection from compiled plans;
- remove fixed-time/product view semantics;
- define validated generic pass/resource/target/view/capability descriptors;
- retain deterministic planning and structured diagnostics.

Prerequisite: R1 identity and error contracts.

## R3 — Prepared frame inputs and generic producers

Goal:

- define explicit prepared data and ownership/lifetime;
- define producer contribution upsert/replacement/removal;
- move ECS/application state resolution to Runenwerk;
- remove renderer reach-back into host world/resources;
- prove multiple independent producer families through generic contracts.

Prerequisite: R2 graph/resource model.

## R4 — GPU parameters and optional macro ABI

Goal:

- decide whether `GpuUniform`/`GpuStorage` derives remain public;
- separate generic byte/binding intent from WGSL/WGPU layout realization;
- prove supported layout, padding, arrays, matrices, nested structs, generics,
  bytemuck safety, package renaming, and compile failures;
- create a macro package only if retained and justified.

Prerequisite: R3 explicit prepared inputs.

## R5 — Shader and hot-reload boundary

Goal:

- separate shader/interface identity from WGPU realization;
- move filesystem roots/watch/poll/reload and last-known-good policy to Runenwerk;
- keep material-authoring translation in Runenwerk;
- provide structured validation/module/pipeline results.

Prerequisites: R2 descriptors and R4 ABI decision.

## R6 — Headless WGPU executor

Goal:

- initialize instance/adapter/device/queue without a window or surface;
- realize generic resources, shaders, pipelines, uploads, execution, and readback;
- expose backend capabilities and structured failures;
- prove offscreen/headless execution where environment permits.

Prerequisites: R2–R5 neutral contracts.

## R7 — Generic surfaces and device loss

Goal:

- admit host-provided raw handle targets without Winit dependency;
- define handle lifetime, thread affinity, surface generations, resize,
  acquire/present, retirement, drop order, and multi-surface behavior;
- classify surface and device-loss outcomes;
- keep product recovery policy in Runenwerk.

Prerequisite: R6 independent WGPU device.

## R8 — Diagnostics, capture, and provenance split

Goal:

- retain neutral graph/resource/backend provenance and capture facts;
- move product/world/material/SDF/editor inspection and artifact policy to
  Runenwerk;
- separate deterministic planner evidence from environment-dependent timings;
- remove process-global or product presentation authority.

Prerequisites: R2 planning and R6/R7 backend outcomes.

## R9 — Runenwerk adapter migration

Goal:

- migrate scene, world, material, SDF, UI, editor, procedural, runtime, and
  product features to explicit public adapter seams;
- migrate native-window and lifecycle host code;
- remove private reach-through and product-specific graph paths;
- preserve current product behavior through focused and runtime proofs.

Prerequisites: R1–R8 public internal seams.

## R10 — Internal conformance and performance proof

Goal:

- build/test neutral planning without Runenwerk or WGPU;
- build/test WGPU without Runenwerk/Winit and with headless mode;
- prove Runenwerk consumes public seams only;
- prove no duplicate old renderer path remains;
- establish stable/MSRV/Clippy/docs/shader/GPU/surface/runtime/benchmark evidence;
- record exact move/stay/redesign/delete and provenance matrices.

Prerequisites: R1–R9 closed.

R10 completion authorizes external repository-creation planning, not source
transfer by itself.

## Shared invariants

Every phase preserves:

- no Runenwerk/ECS/SDF/UI/scene/material/product semantics in neutral core;
- no Winit, ECS, Runenwerk, or domain semantics in WGPU package target;
- no WGSL/WGPU layout assumptions promoted as universal core semantics;
- structured errors and explicit terminal states;
- deterministic planner evidence separated from GPU/environment evidence;
- no source mirror, compatibility package, or duplicate renderer path;
- no external source movement before R10 acceptance.

## Phase-spec policy

Only R1 has a concrete phase specification now.

After each phase:

1. review delivered source and consumer changes;
2. update the remaining roadmap where facts changed;
3. write the next spec against current main;
4. authorize exactly that phase.

Do not retain R2–R10 RON contracts written against pre-R1 assumptions.

## Parallel work

Allowed during R1:

- read-only module/shader/test/benchmark inventory;
- control-flow diagrams and owner classification;
- GPU/headless/surface command discovery;
- performance-baseline planning.

Forbidden:

- concurrent graph/WGPU/surface/product migrations;
- external package creation or source movement;
- broad renderer rewrite;
- new UI/material/SDF/product feature paths;
- duplicate temporary renderer used outside an unmerged phase branch.

## Final extraction gate

RunenRender repository creation remains blocked until R10 proves:

- independent neutral-core and WGPU package graphs;
- public downstream use;
- headless backend construction;
- accepted surface/device-loss contract;
- Runenwerk public-boundary-only consumption;
- no product semantics or duplicate path in framework candidates;
- complete validation, performance, provenance, and clean-cutover evidence.