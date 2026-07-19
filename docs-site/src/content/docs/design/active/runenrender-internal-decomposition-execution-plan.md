---
title: RunenRender Internal Decomposition Execution Plan
description: Dependency-ordered implementation plan that converts the accepted RunenRender target into ten bounded internal repairs before external extraction.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ./runenrender-decomposition-design.md
  - ../../reports/investigations/runenrender-complete-semantic-inventory.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../workspace/specs/pt-runenrender-r1-identities-errors.ron
  - ../../workspace/specs/pt-runenrender-r2-neutral-graph-resources.ron
  - ../../workspace/specs/pt-runenrender-r3-prepared-inputs-producers.ron
  - ../../workspace/specs/pt-runenrender-r4-gpu-params-macros.ron
  - ../../workspace/specs/pt-runenrender-r5-shader-boundary.ron
  - ../../workspace/specs/pt-runenrender-r6-headless-wgpu.ron
  - ../../workspace/specs/pt-runenrender-r7-surfaces-device-loss.ron
  - ../../workspace/specs/pt-runenrender-r8-diagnostics-split.ron
  - ../../workspace/specs/pt-runenrender-r9-adapter-migration.ron
  - ../../workspace/specs/pt-runenrender-r10-internal-conformance.ron
---

# RunenRender Internal Decomposition Execution Plan

## Purpose

This plan creates the renderer boundary inside Runenwerk before external source
movement. Every phase spec is planning-only until local inventory, baseline
validation, prior closeout, and explicit activation pass.

## Dependency order

```text
R1 identities/errors
  -> R2 neutral graph/resources
  -> R3 prepared inputs/producers
  -> R4 GPU params/macros
  -> R5 shader boundary
  -> R6 headless WGPU executor
  -> R7 surfaces/device loss
  -> R8 diagnostics split
  -> R9 Runenwerk adapter migration
  -> R10 public-boundary-only conformance
  -> external repository transfer
```

Implementation does not skip dependencies. Later phases may be investigated in
parallel only without source changes or shared-file conflicts.

## R1 — Neutral identities, errors, and dependency map

Primary scope:

```text
engine/src/plugins/render/api/ids.rs
engine/src/plugins/render/api/handles.rs
engine/src/plugins/render/graph/diagnostics.rs
engine/src/plugins/render/resource/lifetime.rs
new internal neutral renderer module/crate manifest only if activation proves it enforces dependency direction
focused identity/error/dependency-guard tests
```

Outcomes:

- renderer-local opaque IDs with explicit allocators/namespaces;
- no dependency on Runenwerk ID macros in the target neutral layer;
- no global atomic canonical identity;
- exhaustion is structured;
- renderer error taxonomy replaces public `anyhow`/panic lookup paths in the
  bounded scope;
- exact current dependency classification becomes architecture tests.

R1 must not redesign graph operations, WGPU, surfaces, producers, shaders, or
product adapters.

## R2 — Neutral graph and resource descriptors

Primary scope:

```text
engine/src/plugins/render/graph/**
engine/src/plugins/render/resource/**
engine/src/plugins/render/api/passes.rs
engine/src/plugins/render/api/resources.rs
focused graph/resource tests
```

Outcomes:

- generic compute/graphics/fullscreen/copy/present operations;
- remove built-in UI/product feature variants and validation;
- remove ECS/host TypeId state requirements;
- remove fixed-time regions and Runenwerk product views;
- explicit validated resource descriptors and lifetimes;
- deterministic graph plan and structured validation errors;
- neutral graph tests require no engine world or WGPU.

## R3 — Explicit prepared frame inputs and producer contributions

Primary scope:

```text
engine/src/plugins/render/frame/**
engine/src/plugins/render/api/flow.rs
engine/src/plugins/render/api/dispatch.rs
engine/src/plugins/render/composition/fragments.rs
engine/src/plugins/render/composition/fragment_validation.rs
new internal neutral frame/producer modules where required
focused producer/frame tests
```

Outcomes:

- explicit prepared uniform/storage/texture/buffer/draw/dispatch inputs;
- no callbacks into ECS/application state from compiled plans;
- generic producer identity, provenance, upsert/replacement/removal;
- explicit view/target/flow invocations without product semantics;
- Runenwerk fixed-time/product selection expands inputs before renderer call.

## R4 — GPU parameters and renderer macros

Primary scope:

```text
engine/src/plugins/render/params/**
engine_render_macros/**
focused layout tests and downstream compile fixtures
```

Outcomes:

- accepted GPU uniform/storage traits and validated layout descriptors;
- macros target the neutral renderer API with package-rename support;
- WGSL/WGPU ABI conformance for supported values;
- unsupported layouts fail precisely at compile time;
- no engine paths in generated code.

## R5 — Shader descriptors versus filesystem hot reload

Primary scope:

```text
engine/src/plugins/render/shader/**
engine/src/plugins/render/pipelines/** only descriptor/key boundary
engine/src/plugins/render/composition/hot_reload.rs
asset/shader integration call sites required by the split
focused shader descriptor/revision/reload tests
```

Outcomes:

- neutral shader identity/source/interface/revision descriptors;
- filesystem paths/watch/polling and last-known-good policy stay Runenwerk-owned;
- WGPU realization remains deferred to R6;
- invalid shader/flow returns structured errors rather than silent skip;
- no material/SDF/UI semantic shader kinds in neutral contracts.

## R6 — Headless WGPU device/resource/pipeline executor

Primary scope:

```text
engine/src/plugins/render/backend/device.rs
engine/src/plugins/render/backend/wgpu_ctx.rs excluding surface/window coupling
engine/src/plugins/render/renderer/** generic WGPU execution only
engine/src/plugins/render/pipelines/** WGPU realization/cache
engine/src/plugins/render/resource/** backend realization hooks
headless/offscreen backend tests
```

Outcomes:

- configurable WGPU instance/adapter/device/queue initialization without window;
- generic buffer/texture/sampler/bind-group/shader/pipeline realization;
- compute/graphics/copy/offscreen execution and readback;
- structured backend initialization/resource/pipeline/submission errors;
- no Winit, ECS, product feature, UI, material, SDF, scene, or editor dependency in
  the target backend boundary.

## R7 — Generic surface target and device-loss contract

Primary scope:

```text
engine/src/plugins/render/backend/surface.rs
surface-related portions of backend/wgpu_ctx.rs
Runenwerk native-window mapping/host integration paths found by local inventory
focused multi-surface/error/device-loss tests
```

Outcomes:

- Runenwerk owns Winit windows and NativeWindowId;
- renderer core owns opaque SurfaceId/reports;
- WGPU backend owns `wgpu::Surface`, configuration, acquire, present, retirement;
- backend accepts generic WGPU/raw-window-handle-compatible surface targets;
- structured outdated/lost/timeout/out-of-memory/device-lost outcomes;
- explicit reconstruction requirements and product policy separation.

## R8 — Generic diagnostics, capture, and provenance split

Primary scope:

```text
engine/src/plugins/render/inspect/**
engine/src/plugins/render/graph/diagnostics.rs
backend/core report modules
Runenwerk debug/capture/export consumers
focused deterministic report tests
```

Outcomes:

- generic graph/pass/resource/backend/provenance/capture facts separated;
- SDF/material/world/product/editor evidence remains Runenwerk-owned;
- no filesystem artifact path or UI presentation policy in renderer packages;
- wall-clock data remains observational;
- diagnostics use structured `runenrender.*` identity.

## R9 — Migrate Runenwerk domain and runtime adapters

Primary scope:

```text
engine/src/plugins/render/features/**
engine/src/plugins/render/material_compiler/**
engine/src/plugins/render/procedural/**
engine/src/plugins/render/residency/**
engine/src/plugins/render/runtime/**
engine/src/plugins/render/texture_upload.rs
engine/src/plugins/render/plugin.rs
scene/world/material/SDF/editor/UI/app integration call sites and focused tests
```

Outcomes:

- scene/material/SDF/UI/editor/product/runtime code submits only generic public
  renderer work;
- no product/domain semantic imports in neutral/WGPU packages;
- Runenwerk lifecycle, shader file loading, KTX2 paths, startup/pacing, and product
  policy remain outside renderer packages;
- no private reach-through into renderer internals;
- current application behavior is preserved.

## R10 — Internal package anti-cheating and performance proof

Primary scope:

```text
internal renderer package manifests and dependency guards
public downstream producer test package
core/macros/WGPU conformance tests
headless examples
GPU runtime examples/tests
benchmarks and proof/closeout docs
```

Outcomes:

- internal packages have the exact dependency direction intended externally;
- graph/core tests need no engine or GPU;
- WGPU tests need no engine or Winit;
- external producer uses public APIs only;
- Runenwerk uses public seams only;
- stable/MSRV/headless/GPU validation passes;
- representative planning/backend benchmark baseline exists;
- exact repository transfer inventory is complete.

R10 does not create RunenRender or delete original source.

## Shared constraints

Every repair:

- starts from current merged `main` after truthful prior closeout;
- has one bounded PR and one phase spec;
- does not create the external repository or Git dependency;
- does not leave compatibility aliases, source mirrors, duplicate renderers, or
  product-specific variants in the target neutral boundary;
- does not use private reach-through to satisfy Runenwerk adapters;
- reports deterministic tests separately from environment-dependent GPU evidence;
- records adapter/device/platform for GPU runs;
- updates current docs when public behavior changes.

## Activation gate

Before R1 activation, run the complete local inventory and baseline from the
renderer investigation. Local evidence that changes ownership or uncovers
additional product coupling must update Markdown authority before activating an
implementation spec.
