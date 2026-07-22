---
title: GPU and Render S0 Inventory
description: Complete current-state inventory verdict, ownership findings, disposition totals, unknowns, and first bounded RunenGPU candidate.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./runengpu-render-s0-file-disposition.md
  - ./runengpu-render-s0-identity-consumer-lifecycle.md
  - ./runenrender-extraction-investigation.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
---

# GPU and Render S0 Inventory

## Status

S0 is complete for tracked source ownership and direct consumer classification at
Runenwerk baseline:

```text
source baseline: 2d8e72dc221535a12b4345c4ff3258a32b6e8c34
inventory workflow run: 29858157586
primary source roots:
    engine/src/plugins/render/**
    engine_render_macros/**
```

The inventory establishes a bounded first G1 candidate. It does not authorize G1
implementation until a separate exact implementation specification is reviewed.

Environment-dependent GPU behavior remains unverified by S0. That is expected:
S0 is a source/contract inventory, not a GPU runtime proof.

## Inventory coverage

The generated inventory and manual review account for:

| Evidence | Count |
|---|---:|
| Tracked repository files scanned | 2,659 |
| Primary render/macro files | 174 |
| Primary lines | 53,585 |
| Related GPU/render/shader files outside the primary tree | 305 |
| Direct non-document source consumer files | 111 |
| Direct source-type/module matches | 963 |
| Identity-like declarations | 23 |
| Active WGSL assets | 23 |
| Render/GPU tests, examples, and benchmarks | 74 |

Every primary file has one exact disposition in
[GPU and Render S0 File Disposition](runengpu-render-s0-file-disposition.md).

## Core verdict

The current renderer is not one transferable package.

```text
current engine render tree
├── low-level GPU execution substrate
├── render/image-formation semantics
├── Runenwerk lifecycle and product policy
├── source-domain adapters
└── mixed files that must be split or replaced
```

Disposition:

| Target | Files |
|---|---:|
| Move to RunenGPU after internal proof | 25 |
| Move to RunenRender after internal proof | 10 |
| Stay in Runenwerk | 60 |
| Stay with another domain | 28 |
| Redesign before movement | 50 |
| Delete after replacement | 1 |

Only 35 of 174 primary files are substantially aligned with one future framework
owner. Fifty files contain the actual decomposition work. Copying the current tree,
renaming it, or creating speculative package mirrors is therefore rejected.

## RunenGPU source candidates

The strongest current RunenGPU candidates are:

```text
backend device/execution/format/resource/pipeline/WGPU context mechanics
gpu_primitives
GPU value/parameter ABI
resource descriptors, imports, usages, and lifetimes
selected pipeline keys/cache/specialization mechanics
resource lifetime validation
```

These candidates still live under engine paths and use current render vocabulary.
They move only after G1-G8 establish the future public boundary internally.

The current surface file is not directly movable. It combines:

- WGPU surface configuration;
- Runenwerk `NativeWindowId`;
- ECS resource storage;
- host-window-to-surface mapping;
- a hand-allocated `RenderSurfaceId`;
- product lifecycle diagnostics.

It must split into a RunenGPU low-level surface contract and a Runenwerk host
mapping/recovery adapter.

## RunenRender source candidates

The strongest current RunenRender candidates are limited to:

```text
prepared view/context facts
contribution and fragment registration/validation
renderer contribution diagnostics
renderer detail/quality semantics
```

Most current frame, graph, pipeline, renderer, and residency files are not directly
movable because they combine:

- semantic render passes and product features;
- WGPU resources/pipelines/command encoding;
- ECS state and Runenwerk runtime scheduling;
- UI, SDF, world, material, editor, and procedural behavior;
- capture, readiness, artifact, and recovery policy.

The future RunenRender implementation must be formed from accepted prepared-scene,
provider, material, transport, overlay, and RunenGPU contracts rather than copied
from the current renderer directory.

## Runenwerk-retained ownership

Runenwerk retains current files that own:

- plugin and frame lifecycle;
- ECS/domain extraction and submission;
- native windows and product surface mapping;
- product feature registry and selection;
- UI/editor/world/cave/VFX integration;
- shader source discovery, registry, revision, and hot reload;
- material-authoring compilation and handoff;
- capture, artifacts, inspection, startup readiness, frame pacing, and recovery;
- procedural authoring/lowering and runtime product policy.

These are not framework extraction failures. They are the intended integration and
product layer.

## Other-domain ownership

Twenty-eight current files remain with source owners or their Runenwerk adapters:

- material graph/source compilation;
- procgen authoring, population, camera, lowering, and validation;
- cave/world rendering preparation;
- particle/VFX source semantics.

Those domains prepare render contributions or GPU workloads. Their source
semantics do not move into RunenGPU or RunenRender.

## Graph verdict

The current graph must be split conceptually:

```text
RunenRender
    semantic render plan
    views, providers, quality, visibility, transport, overlay intent
        -> lowers to

RunenGPU
    work nodes
    resources, accesses, dependencies, hazards, lifetimes, capabilities
```

Current graph files mix both levels and include fixed-step state, feature IDs,
host `TypeId` state, and built-in UI validation. With the exception of resource
lifetime validation, the current graph is redesign-before-movement.

`graph/validation_builtin_ui.rs` is deleted after neutral overlay validation
replaces the special case.

## Shader and macro verdict

Current shader ownership is three-way:

```text
RunenRender/domain consumer
    shader meaning and source product

RunenGPU
    source admission, backend validation, binding/pipeline realization

Runenwerk
    filesystem discovery, revisions, watching, reload, last-known-good policy
```

The current `shader/**` files combine these roles and are retained or redesigned;
they are not copied wholesale.

`engine_render_macros` is also redesign-before-movement. `GpuUniform` and
`GpuStorage` currently generate engine-specific paths and ABI behavior. A future
macro package is not accepted until external compile-pass/fail, layout, alignment,
nested-type, dependency-renaming, and bytemuck-safety requirements prove it is
necessary.

## Consumer pressure

Direct source consumers are concentrated in:

| Consumer group | Files | Matches |
|---|---:|---:|
| `apps/runenwerk_draw` | 6 | 44 |
| `apps/runenwerk_editor` | 35 | 209 |
| `domain` | 2 | 16 |
| `engine other source` | 15 | 79 |
| `engine/benches` | 1 | 40 |
| `engine/examples` | 22 | 160 |
| `engine/tests` | 30 | 415 |

The editor, engine tests, and examples are the dominant migration surface.
Therefore external extraction before internal anti-cheating proof would create
either a broad compatibility layer or a duplicate runtime path.

## Identity verdict

The current five IDs in `api/ids.rs` do not share one owner:

```text
RenderFlowId           RunenRender semantic plan identity
RenderPassId           mixed; split render semantic pass from GPU work node
RenderResourceId       RunenGPU logical work-resource identity
RenderFeatureId        Runenwerk product/feature identity
RenderFrameProducerId  RunenRender contribution producer identity,
                       allocated/used by Runenwerk adapters
```

Additional handles and keys are classified in the identity report.

No accepted declaration currently establishes these IDs as persisted, replay,
network, cache, or wire identities. The generated stable-format heuristic produced
false positives from historical documents, caches, and generic serialization
imports. Runtime IDs remain ephemeral until a separate format owner explicitly
accepts stable encoding.

## Surface and lifecycle verdict

Current lifecycle evidence is incomplete for runtime proof but sufficient for
ownership:

- context/device/queue creation belongs to RunenGPU;
- window/event-loop and native-handle lifetime belong to Runenwerk;
- low-level surface admission/configure/acquire/present belongs to RunenGPU;
- logical target/color/presentation intent belongs to RunenRender;
- window-to-GPU-surface mapping, resize policy, diagnostics presentation, and
  recovery belong to Runenwerk.

S0 found no accepted headless initialization proof, offscreen conformance,
device-loss proof, or complete drop-order evidence. Those are mandatory G5-G7
execution gates, not blockers to G1 identity work.

## Validation evidence

Deterministic current evidence includes:

- tracked source and Cargo metadata inventory;
- complete primary-file disposition;
- identity/raw-allocation scan;
- shader/macro and direct-consumer scan;
- test/example/benchmark command inventory;
- the repository `cargo validate` baseline on the documentation branch.

Environment-dependent evidence still required later:

```text
headless context creation
compute upload/dispatch/readback
offscreen graphics
surface acquire/present/reconfigure
multi-window behavior
device loss and reconstruction
GPU timings and allocation/performance baselines
```

These belong to G5-G8 and must be reported separately from deterministic contract
tests.

## First bounded G1 candidate

S0 recommends exactly one first implementation slice:

### G1A — Split the logical GPU work-resource identity

Replace the current graph-local `RenderResourceId` ownership with a future
RunenGPU-owned logical work-resource identity and structured allocation errors.

Required scope:

```text
current owner:
    engine/src/plugins/render/api/ids.rs
    RenderResourceIdSequence allocation path
    typed resource handles
    resource descriptors/import/usages/lifetimes
    graph and renderer consumers of the logical ID

new semantic owner:
    internal future-transferable RunenGPU contract

mechanical consumer migration:
    RenderResourceId -> GpuWorkResourceId
    no compatibility alias
```

Required behavior:

- non-zero opaque value;
- fallible allocation with explicit exhaustion;
- no wrapping, saturation, or panic-based success path;
- deterministic allocation within one work-graph builder;
- safe code cannot forge arbitrary IDs;
- no persistence or cross-context claim;
- typed handles continue to carry only the logical resource identity and their
  owned metadata;
- foreign graph use is rejected where the current graph boundary can prove it;
- public errors are structured and string-independent.

Explicit non-scope:

```text
RenderFlowId
RenderPassId
RenderFeatureId
RenderFrameProducerId
RenderSurfaceId
WGPU device/resource creation
graph redesign
surface/window lifecycle
shader/pipeline architecture
external repository creation
RunenRender implementation
```

Why this is the correct first slice:

1. `RenderResourceId` is allocated per `RenderFlow`, proving it is a logical
   work-graph resource rather than a persisted asset or concrete WGPU handle.
2. It is the clearest GPU-owned identity in the mixed `api/ids.rs` file.
3. The migration corrects ownership without requiring WGPU, window, or transport
   changes.
4. It creates the first future-transferable RunenGPU contract consumed by the
   existing renderer, satisfying internal-boundary proof rather than copying code.
5. It exposes allocation and foreign-graph assumptions before resource descriptor
   and hazard work in G2/G3.

## G1A prerequisites and stop conditions

Before writing the exact G1A implementation spec, inspect the complete
`RenderResourceId` allocation/use list and name every changed file. Stop if:

- any use treats the ID as a source asset, persisted, replay, network, cache, or
  cross-process identity;
- allocation cannot be made fallible without changing unrelated render behavior;
- graph-local ownership cannot be represented without introducing a compatibility
  alias;
- the slice requires WGPU resource realization, surface, shader, or renderer
  execution changes;
- unrelated baseline failures exist.

The G1A implementation specification must be a separate issue/PR based on current
`main` after this S0 report merges.

## S0 completion verdict

```text
file inventory                 complete
direct consumer inventory      complete
identity classification        complete for current declarations
shader/macro ownership         complete for S0
surface/lifecycle ownership    complete; runtime proof deferred
validation command inventory   complete
file disposition               complete
bounded G1 candidate           identified
Rust implementation            not started
external source movement       forbidden
```
