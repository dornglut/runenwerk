---
title: GPU and Render S0 Identity, Consumer, and Lifecycle Inventory
description: Current identity ownership, allocation, consumer pressure, stable-format risk, shader and macro boundaries, and GPU surface lifecycle findings.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./runengpu-render-s0-inventory.md
  - ./runengpu-render-s0-file-disposition.md
  - ./runenrender-extraction-investigation.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
---

# GPU and Render S0 Identity, Consumer, and Lifecycle Inventory

## Scope and evidence

This report classifies the current identity-like values, direct consumer pressure,
shader and macro ownership, and context/device/surface lifecycle observed at the S0
baseline:

```text
Runenwerk main: 2d8e72dc221535a12b4345c4ff3258a32b6e8c34
primary roots:
    engine/src/plugins/render/**
    engine_render_macros/**
```

The evidence is deterministic source and contract evidence. It does not claim
successful headless GPU creation, offscreen execution, surface presentation,
device-loss recovery, or performance behavior.

## Identity ownership matrix

| Current value | Future owner | Current meaning | Stability classification |
|---|---|---|---|
| `RenderFlowId` | RunenRender | semantic render-flow or render-plan identity | ephemeral runtime identity |
| `RenderPassId` | redesign | conflates a semantic render pass with a future GPU work node | split required; no stable format |
| `RenderResourceId` | RunenGPU | logical work-graph resource identity allocated inside a flow | graph-local runtime identity; G1A candidate |
| `RenderFeatureId` | Runenwerk | product feature registry and selection identity | ephemeral product/runtime identity |
| `RenderFrameProducerId` | RunenRender | render-contribution producer identity used by Runenwerk adapters | ephemeral runtime identity |
| `RenderSurfaceId` | redesign | combines host window mapping and low-level GPU surface identity | split host and GPU identities |
| `RenderFixedStepRegionId` | Runenwerk | fixed-time scheduling region identity | ephemeral engine identity |
| `PreparedFlowInvocationId` | RunenRender | prepared render-plan invocation key | string runtime key; no accepted format |
| `RenderFragmentId` | RunenRender | render contribution fragment identity | stable encoding requires a separate format contract |
| `RenderFragmentPackageId` | RunenRender | render contribution package identity | stable encoding requires a separate format contract |
| `RenderFragmentNamespace` | RunenRender | fragment-label namespace | source/runtime label; not a GPU identity |
| `RenderFeatureContributionCollectorId` | RunenRender | contribution collector/producer identity | ephemeral runtime identity |
| `PassHandle` | RunenRender | typed builder-local semantic pass handle | ephemeral builder handle |
| `StorageArrayHandle<T>` | RunenGPU | typed logical storage-resource handle | ephemeral work-resource handle |
| `UniformHandle<T>` | RunenGPU | typed logical uniform-resource handle | ephemeral work-resource handle |
| `DoubleBufferHandle<T>` | RunenGPU | aggregate of two logical storage-resource handles | ephemeral work-resource aggregate |
| `PipelineKey` | RunenGPU | backend pipeline realization/cache key | ephemeral backend cache key |
| `FlowPassPipelineKey` | redesign | mixes semantic render-pass facts with backend pipeline realization | split render and GPU keys |
| `FlowPassBindGroupKey` | redesign | mixes render-pass pipeline identity and GPU bind-group cache state | split required |
| `RenderGpuCacheHandle` | redesign | mixes renderer residency/cache ownership and GPU realization | ephemeral; owner split required |
| `RenderPreparedFramePreflightCacheKey` | RunenRender | prepared-scene preflight cache key | ephemeral renderer cache key |
| `RuntimeResourceKey` | redesign | mixes logical aliases, renderer targets, and GPU realizations | ephemeral; split required |
| `ShaderHandle` | Runenwerk | source registry/revision/hot-reload handle | runtime registry handle; future GPU shader identity is separate |
| `RenderDynamicTextureTargetKey` | Runenwerk/redesign | product texture-target selection mapped toward render and GPU targets | product key; no accepted stable format |

## Current allocator findings

### `RenderResourceId`

`RenderResourceId` is created through `RenderResourceIdSequence` owned by each
`RenderFlow`. This establishes that it is a logical resource identity scoped to one
work-graph construction context, not:

- a WGPU buffer or texture handle;
- a persisted asset ID;
- a network or replay identity;
- a cross-process cache key;
- a renderer material, provider, view, or target identity.

The current allocation contract still requires correction:

- the generic ID macro exposes raw reconstruction;
- allocation behavior is not expressed through a RunenGPU-owned error type;
- graph ownership is implicit rather than represented by a future-transferable
  contract;
- exhaustion and foreign-graph behavior require focused proof;
- current names incorrectly place the logical GPU resource under renderer authority.

This is why `RenderResourceId -> GpuWorkResourceId` is the bounded G1A candidate.

### Other ID families

The remaining IDs must not be migrated together merely because they share
`api/ids.rs`. Their owners differ:

```text
render semantics     RenderFlowId, RenderFrameProducerId
mixed seam           RenderPassId, RenderSurfaceId
product/host          RenderFeatureId, RenderFixedStepRegionId
gpu work resource    RenderResourceId
```

A shared implementation file is not shared semantic ownership.

## Raw and stable-format risk

S0 found raw construction and diagnostic raw access in current source, including
primary surface constants and test fixtures. It did not find an accepted contract
that makes the current runtime IDs stable persisted, network, replay, wire, or
cross-process cache identities.

Rules for G1A and later phases:

- safe callers cannot construct arbitrary runtime IDs from integers;
- raw values may be exposed only for diagnostics, ordering, hashing, and trace
  correlation;
- raw values carry no persistence or compatibility guarantee;
- a stable source or persisted key must use a separately owned type and versioned
  format;
- no compatibility alias may retain `RenderResourceId` as parallel authority.

Stop G1A immediately if a current consumer intentionally depends on stable raw
`RenderResourceId` encoding.

## Direct consumer inventory

The source scan found 111 direct non-document consumer files and 963 direct
module/type matches.

| Consumer area | Files | Matches | Migration implication |
|---|---:|---:|---|
| `apps/runenwerk_draw` | 6 | 44 | product integration remains Runenwerk-owned |
| `apps/runenwerk_editor` | 35 | 209 | largest application migration surface |
| `domain/**` | 2 | 16 | source domains must publish prepared contributions/workloads rather than depend on internals |
| other `engine/**` source | 15 | 79 | lifecycle and adapter seams must be made explicit |
| `engine/benches/**` | 1 | 40 | benchmark baselines must migrate with accepted contracts |
| `engine/examples/**` | 22 | 160 | public/semi-public example pressure requires mechanical migration |
| `engine/tests/**` | 30 | 415 | largest proof surface and anti-cheating boundary evidence |

The editor, engine tests, and examples dominate migration pressure. This confirms
that copying the current implementation into an external repository would require
a broad compatibility facade or parallel source authority, both forbidden.

## Domain and product reach-back

Current renderer files directly reach into or encode assumptions about:

- ECS resources and component storage;
- Runenwerk frame/fixed-time scheduling;
- native windows and event-loop lifetime;
- scene and world extraction;
- material graph compilation and asset policy;
- SDF world residency and raymarch product behavior;
- UI composite feature validation;
- editor picking and diagnostics;
- procgen population and camera policy;
- cave, world, particle, and VFX source semantics;
- capture, artifact, readiness, pacing, and recovery policy.

These dependencies are not admitted into RunenGPU or RunenRender. They remain
source-domain or Runenwerk adapter responsibilities.

## Shader, pipeline, and macro ownership

Shader ownership is intentionally three-way:

```text
semantic consumer / RunenRender / source domain
    shader meaning, algorithms, entry points, parameters

RunenGPU
    source admission, backend validation, interface realization,
    bind-group and pipeline realization, backend failures

Runenwerk
    filesystem paths, discovery, revision registry, watching,
    hot reload, last-known-good policy, diagnostics presentation
```

Current `shader/**` and pipeline files mix these responsibilities and are retained
or redesigned before movement.

`engine_render_macros` is redesign-before-movement. The current `GpuUniform` and
`GpuStorage` derives generate engine-specific paths and backend ABI behavior. A
separate macro package remains rejected until there is proof for:

- byte layout and alignment;
- arrays, matrices, nested values, and generics;
- compile-pass and compile-fail behavior;
- dependency renaming;
- bytemuck safety;
- at least two independent public consumers.

## Context, device, surface, and shutdown ownership

### RunenGPU

RunenGPU owns the future contracts and backend facts for:

- instance, adapter, device, and queue realization;
- normalized capability admission;
- GPU resources, pipelines, commands, submission, and completion;
- upload and asynchronous readback;
- low-level surface admission, configuration, acquisition, and presentation;
- surface and device outcomes;
- backend shutdown and in-flight work classification.

### RunenRender

RunenRender owns:

- logical render targets and presentation intent;
- color-space and output meaning;
- prepared views and render contributions;
- renderer history and reconstruction policy.

### Runenwerk

Runenwerk owns:

- native window creation and destruction;
- event-loop, DPI, monitor, resize, and visibility policy;
- mapping host windows to GPU surfaces;
- application lifecycle and scheduling;
- product recovery decisions;
- diagnostics presentation and artifact policy.

### Current mixed surface file

The current surface implementation combines WGPU configuration with
`NativeWindowId`, ECS storage, a hand-allocated `RenderSurfaceId`, product
diagnostics, and host mapping. It must be split before movement.

S0 does not establish:

- headless context creation;
- offscreen render conformance;
- complete surface generation and drop-order behavior;
- multi-window presentation;
- device-loss reconstruction;
- GPU timing or memory-budget baselines.

Those are environment-dependent G5-G8 gates, not prerequisites for G1A identity
work.

## Validation and runtime command inventory

Deterministic baseline:

```text
cargo validate
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git status --short --branch
```

Future environment-dependent commands must separately prove:

```text
headless context/device creation
compute upload -> dispatch -> readback
offscreen graphics output
surface configure -> acquire -> present -> reconfigure
multi-window and surface retirement
device loss and reconstruction
capture and runtime examples
GPU benchmark and allocation evidence
```

Unavailable GPU/window evidence must be reported as unavailable, never treated as
passed deterministic evidence.

## G1A boundary

G1A may change only the logical work-resource identity contract and its direct
mechanical consumers. It must not change:

- other current ID families;
- graph node or render-pass semantics;
- WGPU realization;
- shader or pipeline architecture;
- windows or surfaces;
- render transport or provider semantics;
- external repositories;
- product behavior.

The exact G1A specification is written only after this S0 report merges and the
complete `RenderResourceId` use list is rechecked against the new `main`.