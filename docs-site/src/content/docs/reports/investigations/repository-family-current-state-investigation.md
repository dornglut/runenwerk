---
title: Repository Family Current-State Investigation
description: Current source, dependency, ownership, and extraction-readiness evidence for RunenSDF, RunenECS, RunenGPU, RunenRender, and Runenwerk integration.
status: active
owner: workspace
layer: investigation
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ./runenrender-extraction-investigation.md
  - ../../workspace/planning/active-work.md
---

# Repository Family Current-State Investigation

## Investigation question

Which Runenwerk subsystems can become independent repositories, what boundaries
must change first, and which work may proceed safely in parallel?

## Baseline and evidence

Repository: `Crystonix/Runenwerk`

GPU/render split review baseline:

```text
8de096259eab30f8d67672010df9190970d0bfc4
```

Evidence:

```text
E2 GitHub repository, commit, and pull-request metadata
E3 connector-backed source, manifest, module, and control-flow inspection
```

This investigation did not run local Cargo, shader, GPU, surface, runtime, or
benchmark commands. Command validation remains mandatory in each activated phase.

Source classification establishes ownership direction and track order. It does
not authorize structural implementation or source movement.

## Workspace candidates

| Candidate | Current location | Current readiness |
|---|---|---|
| RunenSDF | `domain/sdf`; external transfer handled by its separate track | boundary corrected; standalone transfer active elsewhere |
| RunenECS | `domain/ecs`, `domain/ecs_macros`, parts of `domain/scheduler` | ownership/safety repair roadmap recorded; no implementation authorization |
| RunenGPU | general GPU portions inside `engine/src/plugins/render` and macros | architecture accepted on branch; complete S0 local inventory pending |
| RunenRender | render semantics/realization inside `engine/src/plugins/render` | architecture revised; implementation waits for RunenGPU boundary |
| RunenUI | external `Crystonix/runen-ui` | independent workstream |

## RunenSDF status

The in-workspace boundary correction completed through PR #116:

- removed Runenwerk geometry leakage;
- introduced validated SDF-owned bounds and rays;
- separated signed value from optional conservative safe step;
- made exact-distance capability explicit;
- added structured construction, sampling, gradient, and query errors;
- migrated package tests.

The standalone transfer and later Runenwerk cutover remain the separate SDF
track's authority. This GPU/render branch changes no SDF source or lifecycle fact
without rebase and reconciliation.

## RunenECS status

Current ECS aggregation still includes storage/query/runtime plus spatial,
reflection, messaging, ownership, change, and lifecycle concerns. Scheduler
ownership also spans generic scheduling, ECS integration, and Runenwerk frame
policy.

The accepted repair sequence remains independent. No ECS Rust implementation is
authorized by the GPU/render split.

## Current renderer/GPU aggregation

Rendering is not an independent package. The engine render root includes:

```text
api
backend
composition
features
frame
gpu_primitives
graph
inspect
material_compiler
params
pipelines
procedural
renderer
residency
resource
shader
runtime
texture_upload
plugin
```

It combines at least:

1. general GPU identities, resources, pipelines, and execution;
2. WGPU instance/adapter/device/queue ownership;
3. low-level surfaces and presentation;
4. render graph, views, targets, providers, and image formation;
5. Runenwerk plugin scheduling and ECS resources;
6. scene/world preparation;
7. material IR-to-WGSL compilation;
8. SDF/world residency and raymarch product features;
9. UI preparation and rasterization;
10. editor, cave, procedural, VFX, deformation, and wind features;
11. shader filesystem polling and reload;
12. diagnostics, capture, artifacts, startup, and frame pacing.

Moving the directory unchanged is forbidden.

## RunenGPU finding

General GPU execution has independent value beyond rendering.

The current backend already owns WGPU devices, queues, resources, compute/render
pass execution, uploads/readback-related paths, and surfaces, but it is coupled to
renderer IDs, ECS resources, Winit windows, product surface policy, and generic
`anyhow` failures.

Correct target:

```text
runengpu_core
  identities, capabilities, resources, access/lifetimes, work graphs,
  submissions/readback, validation, neutral diagnostics

runengpu_wgpu
  WGPU instance/adapter/device/queue/resources/pipelines/commands,
  headless execution, low-level surfaces, backend outcomes
```

RunenGPU does not own rendering or domain algorithms.

## RunenRender finding

Image formation remains a separate reusable framework.

Correct target:

```text
runenrender_core
  prepared scene/contributions, views/targets, providers/interactions,
  materials/media, emitters/environments, transport/history/overlays/presentation

runenrender_gpu
  render-specific realization, visibility, transport, caches,
  reconstruction, overlay/output lowering into RunenGPU work
```

RunenRender does not own direct WGPU execution.

## Accepted dependency

```text
RunenRender -> RunenGPU
```

This is a lower-level framework dependency, not a Runenwerk domain adapter.
Non-render consumers may use RunenGPU without RunenRender.

## Current identity problem

Current identities are not correctly separated:

```text
RenderFlowId
RenderPassId
RenderResourceId
RenderFeatureId
RenderFrameProducerId
RenderSurfaceId
```

Evidence indicates mixed roles:

- semantic render flow/pass/resource;
- backend resource/work ownership;
- built-in product feature selection;
- producer/contribution identity;
- native-window mapping;
- WGPU surface lookup.

The old RunenRender R1 cannot safely rewrite these all as renderer-local IDs.
Every family must first be classified as GPU-owned, renderer-owned,
Runenwerk-owned, retired, or blocker.

## Current control-flow problem

Frame preparation currently reaches into:

- ECS world/resources through `WorldMut`, `TypeId`, and `Any`;
- scene manager state;
- shader filesystem polling;
- native-window lifecycle;
- fixed-time/product selection;
- UI and viewport bindings;
- feature-specific prepared resources.

Submission currently reaches into:

- WGPU surfaces and renderer internals;
- UI font atlas;
- product diagnostics and capture export;
- startup readiness and frame pacing;
- scene/time/product state.

Target:

```text
Runenwerk/domain adapters
    prepare explicit immutable inputs

RunenRender
    compose and plan image formation

RunenGPU
    validate and execute GPU work
```

Neither framework reaches back into the host world.

## RunenUI disposition

RunenUI is independent.

It owns semantic UI, layout, accessibility, hit testing, text shaping, and
renderer-neutral paint output. Initial integration remains a Runenwerk adapter to
RunenRender overlay contributions. Optional GPU-backed UI rendering does not make
RunenUI core depend on RunenGPU.

## Parallel-work decision

Allowed concurrently:

```text
RunenSDF    separately authorized transfer work
RunenECS    read-only investigation/design
RunenGPU    architecture and S0 local inventory
RunenRender read-only semantic design
RunenUI     independent workstream
```

Serialized or coordinated:

- shared manifests and lockfile;
- root architecture and canonical planning summaries;
- current render/GPU identity files;
- WGPU ownership changes;
- external transfers and clean cutovers.

## Corrected extraction sequence

```text
RunenSDF separate track

S0 GPU/render inventory
    -> G1-G9 internal RunenGPU proof
    -> external RunenGPU transfer/cutover
    -> R1-R8 internal RunenRender proof
    -> external RunenRender transfer/cutover
    -> reusable adapters
    -> advanced field-ray renderer
```

## Rejected extraction attempt

Commit `b5e9624c...` remains historical evidence of an incomplete extraction that
removed source before completing dependency, consumer, lockfile, test, asset, and
authority cutover. It is not an implementation base.

## Remaining evidence before G1

- full local file inventory;
- complete consumer/import map;
- current ID, allocator, raw reconstruction, and stable-use map;
- shader/pipeline/macro inventory;
- surface/window/device/drop-order trace;
- Cargo/test/Clippy/docs baseline;
- headless GPU, shader, surface, device-loss, and benchmark command inventory;
- exact file-level move/stay/redesign/delete matrix.

## Gate result

```text
repository ownership direction      decision-complete
module-family GPU/render inventory  substantial through connector evidence
file-level/local command inventory  pending
RunenGPU G1 implementation          not authorized
RunenGPU external transfer          forbidden
RunenRender implementation          blocked by RunenGPU boundary
RunenRender external transfer       forbidden
```

## Conclusion

The repository family should not extract the current renderer as one package.
General GPU execution and image formation have distinct invariants and consumers.
The safe path is internal RunenGPU proof, clean RunenGPU cutover, then RunenRender
decomposition and extraction over that accepted boundary.
