---
title: RunenRender Decomposition Design
description: Provisional ownership, package candidates, host/backend split, producer seams, and investigation gates required before extracting the Runenwerk renderer.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Decomposition Design

## Status

Repository ownership direction and the requirement for internal decomposition are
fixed. Exact public contracts, package membership, module dispositions, and
implementation phases remain provisional until the complete renderer inventory
and command baseline are verified.

No renderer source movement or broad rewrite is authorized by this document.

## Goal

Create a renderer usable without Runenwerk while retaining Runenwerk-specific
lifecycle, ECS extraction, scene/world/material/SDF/UI/editor adaptation, native
window policy, and product composition in Runenwerk.

## Candidate packages

Required initial candidates:

```text
runenrender_core
runenrender_wgpu
```

A `runenrender_macros` package is optional. It is created only if the current GPU
derives remain justified after public usefulness, safety, package-renaming, and
WGSL/WGPU ABI conformance review.

Do not create separate graph, resource, shader, surface, pipeline, residency, or
inspection packages without independent dependency or release pressure.

## Durable ownership

### runenrender_core candidate

Backend-neutral core may own:

- renderer-local identities;
- graph/pass/resource declarations;
- dependency, access, and capability validation;
- deterministic plan formation;
- generic frame inputs and producer contributions;
- backend-neutral diagnostics and provenance.

Core must not depend on:

```text
WGPU
Winit
Runenwerk
ECS
RunenSDF
RunenUI
scene/world/material-authoring domains
editor or application lifecycle
```

Core must not standardize WGSL/WGPU memory layout as a universal backend-neutral
contract. It may describe generic byte ranges, bindings, and resource intent; the
WGPU ABI owner realizes backend-specific layout.

### runenrender_wgpu candidate

The WGPU package may own:

- instance, adapter, device, and queue;
- headless/offscreen initialization;
- GPU resources, uploads, readback, and caches;
- shader/pipeline realization;
- command encoding and submission;
- WGPU surface creation/configuration/acquisition/presentation;
- surface/device-loss facts and backend diagnostics.

It does not own Winit windows, ECS resources, Runenwerk product state, or domain
semantics.

### Runenwerk integration

Runenwerk retains:

- `RenderPlugin` and application/frame lifecycle;
- native windows, event loop, DPI/monitor/visibility policy;
- ECS/application state projection;
- scene/world/material/SDF/UI/editor adapters;
- shader filesystem discovery and hot reload policy;
- asset path and product feature policy;
- debug overlays, artifact export, startup readiness, and frame pacing;
- integration diagnostics and runtime evidence.

## RunenUI relationship

RunenUI and RunenRender are independent.

RunenUI owns UI semantics, hit testing, and renderer-neutral paint output.
RunenRender owns general render planning and backend execution. A future
Runenwerk-owned adapter translates accepted RunenUI output into generic
RunenRender contributions.

RunenUI may retain lightweight standalone backends. Neither framework depends on
the other by default.

## Current decomposition problem

The current engine renderer combines:

- graph and frame planning;
- WGPU device/resource/pipeline execution;
- native-window surfaces;
- ECS resources and host state projection;
- Runenwerk frame/fixed-time policy;
- scene, material, SDF, UI, world, editor, and product features;
- filesystem shader reload;
- diagnostics, capture, and runtime presentation.

Moving `engine/src/plugins/render` unchanged is forbidden.

## Surface and window boundary

Runenwerk owns native windows and their lifetime. The WGPU backend owns
`wgpu::Surface` realization and presentation. Core owns only renderer-local,
backend-neutral surface/target identity and intent where proven necessary.

The complete design must specify:

- raw-window/display-handle admission and lifetime;
- thread-affinity requirements;
- surface creation and retirement;
- resize/reconfiguration;
- acquire/present terminal outcomes;
- drop order between host windows, surfaces, device, and in-flight frames;
- multi-window and headless operation;
- device-loss reconstruction responsibilities.

No `NativeWindowId`, Winit event, or ECS derive appears in RunenRender public
contracts.

## Generic producer seam

Runenwerk adapters should submit explicit generic rendering work rather than
registering product semantics in renderer core.

The final producer contract may express:

- producer/frame/view/target identity;
- resource declarations and updates;
- draw, dispatch, copy, and pass contributions;
- dependency/order and capability requirements;
- provenance and diagnostics;
- deterministic replacement/removal.

It must not contain ECS entities, UI routes/widgets, SDF fields, material graph
nodes, scene nodes, editor tools, or product feature enums.

## Graph boundary

Neutral graph planning must be testable without WGPU and Runenwerk.

Product-specific graph variants and validation paths such as built-in UI,
material, SDF, world, or editor passes are removed from core. Producers validate
their domain semantics before submitting generic work.

Runenwerk resolves host/ECS state before renderer submission. Core plans do not
reach back into a host world through `TypeId`, callbacks, or fixed-time state.

## Shader, pipeline, and GPU layout boundary

The complete investigation must separate:

```text
backend-neutral shader identity/interface intent
WGSL validation and WGPU module/pipeline realization
Runenwerk filesystem discovery/watch/hot reload policy
Runenwerk material-authoring translation
```

GPU uniform/storage derives are not assumed to be part of core. If retained, they
belong with the owner of the WGSL/WGPU ABI contract and require independent
compile-pass, compile-fail, byte-layout, alignment, and package-renaming proof.

## Resource and synchronization boundary

The final design must define:

- logical identity, generation, and stale-handle behavior;
- create/update/remove admission;
- CPU upload ownership and in-flight retention;
- transient aliasing and lifetime planning;
- cache invalidation and reconstruction;
- queue submission and synchronization;
- memory budget and out-of-memory behavior;
- shutdown and terminal state.

No renderer resource lifetime relies implicitly on ECS lifetime.

## Error and diagnostics policy

RunenRender reports structured facts for invalid plans, unsupported capabilities,
resource failures, shader/pipeline failures, surface outcomes, device loss,
submission/presentation failure, and terminal shutdown.

Runenwerk decides product recovery policy. Deterministic planning evidence is
reported separately from environment-dependent GPU, driver, and window evidence.

## Required investigation output

Before implementation, produce:

- complete module/file/shader/example/test/benchmark inventory;
- complete package/import/consumer map;
- frame, graph, resource, surface, device-loss, and plugin control-flow traces;
- per-module future-owner classification;
- current macro and GPU ABI consumer map;
- headless, surface, shader, GPU, and benchmark command inventory;
- exact move/stay/redesign/delete matrix;
- current Cargo, test, Clippy, shader, GPU, and runtime baseline.

## Sequence

```text
RENDER-001 complete and verify semantic inventory
RENDER-002 close public seam and package design
RENDER-003 decompose internally through small ordered phases
RENDER-004 migrate all Runenwerk producers to public adapter seams
RENDER-005 prove internal anti-cheating, headless/GPU, and performance conformance
RENDER-006 create RunenRender and transfer corrected source
RENDER-007 cut Runenwerk over, delete originals, and close provenance
```

Only RENDER-001 is active after the repository-family charter. The investigation
may record a repair roadmap, but only the next executable phase receives a
concrete implementation specification.

## Anti-cheating gate

Before external extraction, Runenwerk must consume the internally separated
renderer through the same public boundary intended for external users.

Forbidden shortcuts include:

- private reach-through from adapters;
- ECS or Runenwerk types in renderer packages;
- duplicate old and new renderer paths;
- product-specific graph variants;
- direct scene/material/SDF/UI/editor imports in core;
- tests that require the entire engine for neutral planning behavior.

## Stop conditions

Stop before implementation when:

- the file/consumer/runtime inventory is incomplete;
- core versus WGPU versus host ownership remains ambiguous;
- surface lifetime or thread/drop-order contracts are unresolved;
- GPU layout is assigned to neutral core without backend proof;
- product semantics remain in proposed core APIs;
- current main is not green for unrelated reasons;
- the plan requires one broad rewrite or long-lived compatibility layer.

## Definition of done

RunenRender is extracted only when core and WGPU validate independently,
Runenwerk uses public adapter seams only, no product semantics remain in framework
packages, downstream conformance and GPU/runtime evidence pass, Runenwerk consumes
exact revisions, original implementation is removed, and integration validation
is green.