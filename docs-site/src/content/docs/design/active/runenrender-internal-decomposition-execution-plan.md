---
title: RunenGPU and RunenRender Decomposition Execution Plan
description: Dependency-ordered internal decomposition, conformance, external transfer, and clean-cutover roadmap for GPU execution and rendering.
status: active
owner: workspace
layer: framework/gpu-render
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./runengpu-architecture-design.md
  - ./runenrender-decomposition-design.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/specs/pt-runengpu-g1-identities-errors.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU and RunenRender Decomposition Execution Plan

## Purpose

Replace the earlier renderer-only decomposition with two ordered tracks:

```text
RunenGPU
    general GPU execution

RunenRender
    image formation over RunenGPU
```

The plan records the complete destination. It does not pre-authorize all phases.
Only the next executable phase receives a current RON specification.

## Superseded plan

The previous RunenRender R1-R10 plan is superseded before implementation because
it assigned generic GPU resource planning, WGPU execution, surfaces, and device
lifecycle to RunenRender.

The unimplemented `PT-RUNENRENDER-R1` identity phase must not be activated.
Current `Render*Id` values require ownership classification first.

## Sequence

```text
S0 ownership and command inventory
    -> G1 -> G2 -> G3 -> G4 -> G5 -> G6 -> G7 -> G8 -> G9
    -> GX external RunenGPU transfer and Runenwerk cutover
    -> R1 -> R2 -> R3 -> R4 -> R5 -> R6 -> R7 -> R8
    -> RX external RunenRender transfer and Runenwerk cutover
    -> adapters
    -> advanced renderer roadmap
```

RunenSDF extraction may continue in parallel. Shared manifests, lockfiles, and
canonical planning summaries have one merge owner at a time.

# 1. S0 — Complete GPU/render ownership and command inventory

## Goal

Produce a complete move/stay/redesign/delete map before structural implementation.

## Required source inventory

Inspect:

```text
engine/src/plugins/render/**
engine_render_macros/**
assets/shaders/**
render tests, examples, and benchmarks
window/surface integration
all WGPU consumers
all renderer public consumers
```

Every module and relevant type is classified as:

```text
RunenGPU core
RunenGPU WGPU backend
RunenRender core
RunenRender GPU realization
Runenwerk adapter
Runenwerk product policy
domain-owned
retire
unresolved blocker
```

## Identity inventory

For each current ID, record:

```text
current declaration and allocator
all consumers
raw reconstruction
persistence/replay/network/cache use
future semantic owner
future runtime/stable-key distinction
migration risk
```

At minimum:

```text
RenderFlowId
RenderPassId
RenderResourceId
RenderFeatureId
RenderFrameProducerId
RenderSurfaceId
```

## Required command baseline

Before G1 activation, run and report:

```text
cargo metadata --format-version 1 --locked
cargo tree -p engine --edges normal,build,dev
rg --files engine/src/plugins/render engine_render_macros assets/shaders engine/tests engine/examples engine/benches
rg -n 'wgpu::|winit::|RawWindowHandle|Surface|Device|Queue|CommandEncoder|ComputePass|RenderPass' engine apps domain adapters
rg -n 'Render(Flow|Pass|Resource|Feature|FrameProducer|Surface)Id|try_from_raw|\.raw\(' engine apps domain adapters
rg -n 'plugins::render|RenderPlugin|PreparedRenderFrame|SurfaceFrameSubmission|GpuUniform|GpuStorage' engine apps domain adapters
rg -n 'TypeId|ecs::|World|Resource|material_graph|world_sdf|ui_|Ui|editor|procedural' engine/src/plugins/render
cargo +stable fmt --all --check
cargo test -p engine --lib --locked
cargo test -p engine --tests --locked
cargo clippy -p engine --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
```

## Exit gate

- complete file and consumer inventory;
- complete identity and allocator inventory;
- complete shader/pipeline/macro inventory;
- surface and device lifecycle trace;
- current focused and broad validation facts;
- performance-baseline command inventory;
- no unresolved ownership item required by G1.

# 2. RunenGPU internal decomposition

RunenGPU is formed and proven inside Runenwerk before external transfer.

## G1 — GPU identities, structured errors, and dependency guards

Goal:

- introduce only GPU-owned runtime identities;
- add explicit owner-scoped allocation;
- define structured identity/exhaustion/foreign-context failures;
- classify and mechanically migrate current GPU-owned ID consumers;
- install forbidden-dependency guards.

Do not mechanically rename all `Render*Id` values.

Excluded:

- graph/resource redesign;
- WGPU movement;
- surfaces;
- shader ABI;
- renderer semantic identities;
- external repository source.

## G2 — Resource descriptors, ownership, and lifetimes

Goal:

- define backend-neutral buffer/texture/sampler/resource intent;
- define initialization, access ranges, imports/exports, and lifetime classes;
- define stale and foreign handle behavior;
- define transient epoch retirement and readback ownership;
- prove validation with CPU-only tests.

Prerequisite: G1.

## G3 — Bounded GPU work-plan model

Goal:

- define compute/render/copy/clear/resolve/present nodes;
- define immutable work fragments;
- compose fragments deterministically;
- validate access hazards, initialization, cycles, capabilities, and lifetimes;
- keep domain and renderer semantics outside the graph.

Prerequisite: G2.

## G4 — Shader, pipeline, and GPU ABI boundary

Goal:

- separate domain shader meaning from GPU admission and WGPU realization;
- distinguish logical parameters, bytes, bindings, and backend layout;
- move filesystem/watch/reload policy out of the GPU framework boundary;
- decide whether existing derives survive;
- create a macro package only after ABI and compile conformance.

Prerequisite: G3.

## G5 — Headless WGPU executor

Goal:

- initialize instance/adapter/device/queue without a window or surface;
- realize buffers and compute pipelines;
- upload, dispatch, submit, complete, and read back;
- expose normalized capabilities and structured outcomes.

Required proof: one non-render compute workload.

Prerequisites: G1-G4.

## G6 — Offscreen graphics and copy execution

Goal:

- add textures and render pipelines;
- support offscreen targets, clear/resolve/copy, and image readback;
- prove compute-written data consumed by a render workload;
- keep rendering semantics in the consumer.

Prerequisite: G5.

## G7 — Surfaces and device lifecycle

Goal:

- admit host-provided window/display handles without Winit dependency;
- define surface generations, configure/acquire/present, resize facts, retirement,
  thread/drop-order constraints, and multi-surface behavior;
- define device-loss and out-of-memory facts;
- keep product recovery in Runenwerk.

Prerequisite: G6.

## G8 — Shared-consumer and anti-cheating proof

Required consumers:

```text
one renderer workload
one non-render compute workload
```

Prove:

- one shared context and submission authority;
- no consumer-owned competing device/queue/resource namespace;
- no private reach-through;
- no renderer or domain meaning in RunenGPU;
- current product behavior preserved.

Prerequisites: G1-G7.

## G9 — Internal conformance and performance

Goal:

- build/test neutral core without WGPU;
- build/test WGPU backend without Runenwerk/Winit/domains;
- prove headless, offscreen, surface, readback, shutdown, and failure contracts;
- prove Runenwerk public-boundary-only consumption;
- establish stable/MSRV/Clippy/docs/GPU/runtime/benchmark evidence;
- record final move/stay/redesign/delete and provenance matrices;
- prove no duplicate GPU path.

Prerequisites: G1-G8.

# 3. GX — External RunenGPU transfer and clean cutover

External source movement begins only after G9 acceptance.

## GX1 — Standalone transfer

- create/populate `Crystonix/runen-gpu` from accepted internal packages;
- preserve provenance and licensing;
- validate packages independently;
- prove public downstream use;
- identify the exact accepted revision;
- do not cut Runenwerk over in the same unreviewed step.

## GX2 — Runenwerk cutover

- pin the exact external revision;
- migrate all active consumers;
- delete internal GPU package source in the completed cutover;
- remove temporary paths and private reach-through;
- prove no duplicate GPU execution path;
- record compatibility and provenance closeout.

RunenRender implementation phases below require the external RunenGPU cutover.

# 4. RunenRender internal decomposition

Read-only planning may occur earlier. Structural implementation waits until the
RunenGPU boundary is accepted so it does not create another temporary GPU layer.

## R1 — Renderer semantic identities and errors

Goal:

- introduce renderer-only scene/view/target/provider/instance/material/emitter/
  contribution/history identities;
- keep GPU resources, pipelines, submissions, and surfaces RunenGPU-owned;
- define structured renderer errors and dependency guards;
- migrate only classified renderer-semantic consumers.

## R2 — Prepared render scene and contribution lifecycle

Goal:

- define immutable prepared scenes and contributions;
- define producer insertion/replacement/removal/retirement;
- define views, logical targets, providers, materials/media, emitters, overlays,
  changes, and provenance;
- remove ECS/application reach-back;
- prove at least two independent producer families.

Prerequisite: R1.

## R3 — Semantic render planning

Goal:

- separate render planning from GPU work planning;
- remove product-specific, UI-specific, material-specific, SDF-specific, world,
  and editor variants from renderer core;
- replace host callbacks/TypeId/fixed-time projections with prepared input;
- lower generic render intent toward RunenGPU work descriptions.

Prerequisite: R2.

## R4 — Render GPU realization over RunenGPU

Goal:

- realize current render targets, shaders, pipelines, visibility, overlays, and
  presentation through `runengpu_core`;
- remove direct WGPU device, queue, surface, and resource ownership from renderer;
- preserve current offscreen and presented behavior.

Prerequisites: R3 and external RunenGPU cutover.

## R5 — Material, shader, and reload separation

Goal:

- retain renderer material evaluation and shader-interface meaning;
- use RunenGPU for shader/pipeline realization;
- keep material-authoring translation and filesystem/hot-reload policy in
  Runenwerk/domain owners;
- define structured last-known-good/product policy outside framework cores.

Prerequisite: R4.

## R6 — Logical targets, overlays, and presentation

Goal:

- separate logical targets/output color intent from RunenGPU surface resources;
- preserve headless/offscreen operation;
- define overlay composition without UI semantic access;
- keep windows and recovery in Runenwerk.

Prerequisite: R5.

## R7 — Runenwerk adapter migration and parity

Goal:

- migrate scene, world, material, SDF, UI, editor, procedural, debug, and product
  paths to explicit prepared contributions;
- remove private reach-through and product-specific framework paths;
- preserve current product behavior with focused/runtime proofs.

Do not begin broad field-ray/GI feature development in this parity phase.

Prerequisites: R1-R6.

## R8 — Internal renderer conformance and performance

Goal:

- build/test renderer core without Runenwerk/WGPU/Winit/ECS/SDF/UI;
- prove GPU realization uses RunenGPU only;
- prove headless offscreen rendering;
- prove deterministic contributions and current behavior parity;
- prove public-boundary-only consumption and no duplicate renderer path;
- establish performance and provenance baselines.

Prerequisites: R1-R7.

# 5. RX — External RunenRender transfer and clean cutover

After R8:

1. transfer `runenrender_core` and `runenrender_gpu` to `Crystonix/runen-render`;
2. validate standalone and through a public downstream consumer;
3. identify the exact accepted revision;
4. pin Runenwerk to that revision;
5. migrate adapters;
6. delete internal framework source;
7. remove compatibility paths;
8. prove no duplicate renderer path;
9. close provenance and compatibility evidence.

# 6. Post-extraction adapters

## RunenUI

Blocked until RunenUI exposes an accepted renderer-neutral paint protocol.
Initial ownership remains Runenwerk integration:

```text
RunenUI paint output
    -> Runenwerk adapter
    -> RunenRender overlay contribution
```

Neither framework core depends on the other.

## RunenSDF

Initial ownership remains Runenwerk integration:

```text
RunenSDF field contract
    -> Runenwerk adapter
    -> RunenRender provider
    -> optional RunenGPU realization
```

A bridge package is extracted only after another independent host needs it.

## Simulations

Fluid, wind, vegetation, fire, procedural generation, and other domains may
contribute RunenGPU work and RunenRender providers through explicit adapters.
Their algorithms and state remain domain-owned.

# 7. Advanced renderer roadmap

Only after both clean cutovers stabilize:

```text
F1 reference implicit solid renderer
F2 shells, fibers, liquids, and volumes
F3 unified many-light direct transport
F4 sparse directional radiance cache
F5 lobe-separated reconstruction and bounded history
F6 preview-to-reference quality tiers
F7 endless-world transport horizons and summaries
F8 explicit material/transport/display stylization
F9 character, vegetation, water, and world production proofs
F10 tooling, authoring integration, profiling, and hardening
```

# 8. Phase-spec policy

Only `PT-RUNENGPU-G1` may receive the next implementation specification after S0
command and ownership gates close.

After every phase:

1. review delivered source and consumer changes;
2. update remaining ownership and roadmap facts;
3. write the next spec against current main;
4. authorize exactly that phase;
5. record closeout before the next activation.

Do not prewrite later phase specs against assumptions.

# 9. Parallelism

Allowed:

- RunenSDF external transfer in its separate thread;
- read-only GPU/render inventory;
- RunenGPU architecture and G1 planning;
- independent RunenUI work;
- benchmarks and command discovery.

Serialized or coordinated:

- shared root/planning files;
- Cargo manifests and lockfile;
- renderer/GPU identity source;
- WGPU device/resource ownership;
- source transfers and cutovers.

Forbidden:

- duplicate GPU implementations;
- external RunenGPU source before G9;
- `runenrender_wgpu` extraction;
- broad renderer rewrite in the old plugin;
- direct RunenUI/RunenRender core dependency;
- adapter extraction before both public sides stabilize.

# 10. Immediate next action

The current architecture branch closes ownership and records connector evidence.
Before G1 implementation activation, complete the local S0 command inventory and
update the G1 spec with exact files and consumers.

The next implementation phase is:

```text
PT-RUNENGPU-G1
GPU identities, structured errors, and dependency guards
```

Start by extracting the RunenGPU boundary internally, not by copying code to the
external repository.
