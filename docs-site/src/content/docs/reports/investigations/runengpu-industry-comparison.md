---
title: RunenGPU Industry Architecture Comparison
description: Evidence-based comparison of RunenGPU and RunenRender with WGPU, bgfx, Godot, Unreal, Unity, Filament, Bevy, rend3, CUDA, and OptiX, including recurring weaknesses, ownership lessons, and reevaluation gates.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../workspace/planning/roadmap.md
  - ./runengpu-public-api-ergonomics-review.md
  - ./runengpu-proof-workload-strategy.md
  - ./runengpu-runenrender-application-domain-fit.md
  - ./runengpu-render-s0-inventory.md
---

# RunenGPU Industry Architecture Comparison

## Purpose

This report compares the intended RunenGPU and RunenRender boundaries with current
GPU abstractions, render graphs, renderers, and vendor-specific compute/ray systems.
External systems are evidence and design pressure, not Dornglut authority.

The target layering remains:

```text
Runenwerk adapters and host policy
    -> RunenRender semantic image planning
        -> RunenGPU generic resources and work
            -> WGPU first backend
```

Independent non-render consumers may use RunenGPU without RunenRender.

## Research method

### Evidence hierarchy

Claims are ranked by evidence strength:

1. current official API/reference documentation;
2. current official architecture and migration documentation;
3. maintainer-authored issue reports or accepted project design notes;
4. reproducible current-source evidence in Runenwerk;
5. user reports and ecosystem discussion;
6. inference, stated explicitly and never presented as established fact.

### Classification

Every weakness is classified as one or more of:

```text
structural limitation
    follows from the system's ownership or design

implementation defect
    bug, regression, missing validation, or incomplete feature

ecosystem/tooling friction
    documentation, examples, integrations, debugging, or adoption cost

platform/backend limitation
    inherited from drivers, APIs, browsers, hardware, or operating systems

Runen-introduced risk
    complexity or failure mode created by Dornglut's own abstraction
```

A competitor bug is not evidence that Runen's architecture is superior. It is useful
only when it exposes a contract Runen must normalize, diagnose, or deliberately
inherit.

## Current first-backend facts

WGPU is the correct first backend because it supplies safe Rust access to WebGPU and
native Vulkan, Metal, D3D12, OpenGL, and WebGL2-family paths. It already owns adapter
selection, devices, queues, resources, pipelines, command encoding, surfaces,
validation, and backend portability.

RunenGPU must not recreate those backends.

Current WGPU facts that matter operationally:

- `Device::poll` drives resource cleanup and mapping callbacks on native backends;
- `PollType::Wait` can block, while WebGPU explicit polling is a no-op because the
  browser event loop drives callbacks;
- a queue-empty observation can be stale immediately when other threads can submit;
- pipeline cache reuse depends on compatible WGPU version/cache format, adapter,
  backend, and driver acceptance;
- driver/backend shader compilation can exhibit platform-specific stalls;
- device loss, out-of-memory, format support, and experimental features remain
  backend outcomes;
- ray tracing and several advanced features remain experimental and are not a stable
  portable baseline.

RunenGPU therefore adds ownership, normalization, validated composition, progress,
pressure, recovery facts, and diagnostics. It does not promise to remove driver or
platform variability.

Sources:

- <https://docs.rs/wgpu/latest/wgpu/>
- <https://docs.rs/wgpu/latest/wgpu/struct.Device.html>
- <https://docs.rs/wgpu/latest/wgpu/type.PollType.html>
- <https://docs.rs/wgpu/latest/wgpu/struct.PipelineCache.html>
- <https://github.com/gfx-rs/wgpu/issues/4589>
- <https://github.com/gfx-rs/wgpu/issues/7443>

## Compared systems

## WGPU directly

### Strengths

- smallest Rust-native dependency and conceptual surface;
- current cross-platform backend coverage;
- direct access to new WGPU features;
- excellent fit for a bounded application or one renderer;
- no additional graph or ownership overhead.

### Recurring weaknesses and complaints

- applications must design their own logical resource ownership, provenance,
  lifetimes, work composition, readback, recovery, and diagnostics;
- progress and callback behavior differs between native and web;
- asynchronous mapping still requires application-level completion orchestration;
- shader/pipeline compilation cost can be backend-specific and disruptive;
- low-level validation errors may not identify the product operation or source-domain
  value that caused the failure;
- device/surface loss and reconstruction remain application architecture problems.

### Classification

Mostly structural and platform/backend limitation. Shader or correctness regressions
are implementation defects, not justification for another abstraction by themselves.

### Runen decision

Use WGPU as the first backend. Direct WGPU is the strongest alternative and the
mandatory narrow performance comparison. If RunenGPU cannot demonstrate reusable
correctness or a second non-render consumer, direct WGPU should win.

## bgfx

bgfx is a mature API-agnostic bring-your-own-engine rendering library. It uses views,
handles, deferred resource commands, sort keys, a dedicated render-thread model, and
multi-threaded encoders.

### Strengths

- mature backend and platform breadth;
- explicit API/render-thread contract;
- configurable bounded encoders, views, transient buffers, and frame latency;
- practical immediate handles with deferred realization;
- strong engine independence;
- extensive tooling and shader/resource utilities.

### Recurring weaknesses and trade-offs

- view/sort-key ordering is rendering-oriented rather than a general resource hazard
  graph;
- submission order may not equal execution order except in sequential view modes;
- fixed/configurable limits are explicit but require product-level pressure handling;
- thread-affinity rules are essential and can be violated by integration code;
- semantic ownership, source provenance, reconstruction, and cross-fragment imports
  remain outside the library;
- C/C++ tooling and conventions are less natural for a Rust-native typed framework.

### Runen lesson

Retain explicit limits, thread/progress ownership, deferred realization, and broad
engine independence. Do not adopt view IDs or sort keys as general dependency
authority.

Sources:

- <https://bkaradzic.github.io/bgfx/overview.html>
- <https://bkaradzic.github.io/bgfx/internals.html>
- <https://bkaradzic.github.io/bgfx/bgfx.html>

## Godot RenderingDevice

Godot's RenderingDevice provides a low-level abstraction for buffers, textures,
shaders, pipelines, compute, render lists, capabilities, local devices, and explicit
resource IDs.

### Strengths

- clean renderer-versus-device placement;
- direct compute outside high-level rendering;
- multiple production backend drivers;
- explicit local-device and resource-lifecycle APIs;
- integrated engine diagnostics and feature discovery.

### Recurring weaknesses and trade-offs

- resource and synchronization responsibilities remain relatively manual;
- resource IDs require deliberate lifetime management;
- driver-resource escape hatches expose backend objects;
- local RenderingDevice support has platform/headless constraints;
- integration is engine-specific rather than a standalone Rust framework;
- immutable fragment composition, provenance, and deterministic generic graph
  authority are not the primary public model.

### Runen lesson

Keep device placement and compute independence. Avoid public raw-driver reach-through
and require structured lifecycle/ownership contracts.

Sources:

- <https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html>
- <https://docs.godotengine.org/en/stable/tutorials/rendering/renderers.html>

## Unreal Render Dependency Graph

Unreal RDG records commands into a graph and manages transient resources,
subresource transitions, asynchronous compute fences, pass culling, parallel command
recording, validation, aliasing, and RDG Insights visualization.

### Strengths

- mature dependency and transient-lifetime validation;
- subresource-aware transitions;
- memory aliasing and pass culling;
- parallel setup/recording;
- rich graph and memory diagnostics;
- explicit external-resource import/extraction.

### Recurring weaknesses and trade-offs

- deeply renderer/RHI and Unreal integrated;
- setup/execute split and side-effect restrictions demand discipline;
- immediate/unsafe-style escape paths disable optimization and parallelism;
- transient aliasing increases debugging complexity and makes uninitialized-content
  bugs more visible;
- traces and graph diagnostics can generate large volumes of data;
- complexity is appropriate for AAA pressure but excessive as an initial RunenGPU
  target.

### Runen lesson

Adopt declared access, deterministic validation, inspectable dependencies, and later
subresource/lifetime evidence. Defer aliasing, pass fusion, multi-queue scheduling,
and graph UI until real pressure exists.

Source:

- <https://dev.epicgames.com/documentation/unreal-engine/render-dependency-graph-in-unreal-engine>

## Unity Render Graph

Unity's current Scriptable Render Pipeline uses Render Graph for pass authoring,
resource lifetime, synchronization, allocation reduction, and graph inspection.
Unity documents compatibility mode but states that the non-Render-Graph path is no
longer developed or improved.

### Strengths

- modular pass authoring and frame-resource management;
- resource lifetime and allocation optimization;
- queue synchronization;
- graph viewer and frame-debug tooling;
- strong integration with the rendering product.

### Recurring weaknesses and trade-offs

- migration from compatibility paths creates ecosystem churn;
- unsafe passes are required for operations outside graph knowledge;
- render-pipeline integration and texture-centric assumptions dominate;
- graph correctness can be bypassed at the cost of optimization and insight;
- not an independent non-render GPU framework.

### Runen lesson

Do not preserve a compatibility mode after cutover. Reject graph bypasses unless a
specific contained backend operation is accepted. Keep the common path ergonomic so
users do not demand an unsafe alternative merely to avoid ceremony.

Sources:

- <https://docs.unity3d.com/Manual/urp/render-graph.html>
- <https://docs.unity3d.com/Manual/urp/compatibility-mode.html>

## Filament FrameGraph and renderer

Filament is a compact, efficient physically based renderer with a focused FrameGraph.
Its graph models resource/pass reads and writes, cycles, culling, lifetimes,
load/store inference, and imported/exported resources.

### Strengths

- smaller and clearer graph than AAA engine systems;
- practical mobile/desktop renderer;
- strong conventional PBR image formation;
- resource/pass semantics without mandatory public complexity;
- efficient implementation discipline.

### Recurring weaknesses and trade-offs

- renderer-centric, with textures dominating resource use;
- import/export and subresource features add substantial subtlety;
- not a generic compute/simulation execution framework;
- conventional renderer architecture does not target RunenRender's heterogeneous
  field/provider goals.

### Runen lesson

Prefer compact correctness authority and avoid abstracting beyond current consumers.
Filament is a strong substitute if conventional PBR rendering satisfies accepted
proofs.

Sources:

- <https://google.github.io/filament/>
- <https://google.github.io/filament/notes/framegraph.html>

## Bevy renderer

Bevy uses WGPU and performs rendering in a separate `SubApp`, with extraction from the
main world and optional pipelined/parallel rendering.

### Strengths

- Rust/WGPU-native production evidence;
- explicit extract/render-world separation;
- modular schedules and renderer plugins;
- practical device, queue, pipeline-cache, and recovery-aware resource patterns;
- extensive examples and community adoption.

### Recurring weaknesses and trade-offs

- renderer and graph are coupled to Bevy ECS schedules and render-world assumptions;
- direct WGPU wrapper types are common;
- extraction can duplicate state and introduce latency/complexity;
- pipeline specialization and shader compilation can create application hitches;
- unsuitable as the independent RunenGPU boundary without importing Bevy ownership.

### Runen lesson

Keep explicit prepared-data extraction, but place it in Runenwerk adapters. RunenGPU
must remain useful without ECS, and RunenRender's prepared scene must not become a
hidden second world.

Sources:

- <https://docs.rs/bevy/latest/bevy/render/index.html>
- <https://docs.rs/bevy_render/latest/bevy_render/struct.RenderPlugin.html>

## rend3

rend3 is a Rust renderer built on WGPU with a renderer, instruction processing, and a
render graph. It offers PBR-oriented functionality while remaining more customizable
than a full engine.

### Strengths

- Rust-native and WGPU-based;
- much less work than building a full renderer;
- useful PBR, skybox, shadow, tonemapping, and graph infrastructure;
- appropriate for applications needing a customizable conventional renderer.

### Limitations for Dornglut

- renderer-first rather than generic GPU-framework-first;
- public architecture exposes WGPU-level concepts;
- mesh/PBR assumptions are stronger than the intended provider/field-first direction;
- does not establish the Runen family ownership, recovery, and cross-consumer model.

### Runen decision

A primary substitute for a modest Rust renderer. RunenRender must prove meaningful
value beyond a tailored rend3-style renderer.

Sources:

- <https://docs.rs/rend3/latest/rend3/>
- <https://docs.rs/rend3/latest/rend3/types/struct.Renderer.html>

## CUDA and OptiX

CUDA and OptiX provide maximum NVIDIA-specific compute and programmable ray-tracing
capability.

### Strengths

- mature compute ecosystem, profiling, libraries, and multi-GPU support;
- direct access to NVIDIA hardware capabilities;
- OptiX programmable RTX traversal and ray pipelines;
- excellent fit for controlled NVIDIA-only products or research.

### Limitations for Dornglut

- vendor lock-in;
- no WebGPU/browser path;
- product portability and fallback become separate architectures;
- graphics presentation and non-NVIDIA support require additional systems;
- not a suitable portable public baseline.

### Runen lesson

Keep compute-based field traversal as the portable baseline. Consider vendor-specific
acceleration only as a contained internal backend specialization with explicit
fallback and proof.

Sources:

- <https://docs.nvidia.com/cuda/cuda-c-programming-guide/>
- <https://developer.nvidia.com/rtx/ray-tracing/optix>

## Ranked alternatives for Dornglut's target

The ranking assumes Dornglut wants a reusable Rust GPU framework, an independent
representation-neutral renderer, non-render compute consumers, headless/offscreen
work, and field/procedural content.

| Rank | Alternative | Best case | Why it does not fully replace the target |
|---|---|---|---|
| 1 | Direct WGPU plus a custom renderer | Lowest complexity and fastest access to backend features | Recreates ownership, graph, progress, recovery, and diagnostics locally as needs grow. |
| 2 | WGPU plus rend3 or another Rust renderer | Conventional PBR application with moderate customization | Renderer assumptions and public WGPU exposure do not provide the intended generic framework split. |
| 3 | Bevy renderer/full Bevy | ECS-centric game or tool where Bevy ownership is acceptable | Couples rendering and GPU execution to Bevy schedules/worlds. |
| 4 | Filament | Efficient conventional PBR across platforms | Does not provide general compute/work composition or field/provider architecture. |
| 5 | Godot | Complete engine/product delivery | Replaces Runenwerk ownership and is not a reusable Rust framework family. |
| 6 | bgfx plus custom renderer | Broad C/C++ backend/platform coverage | Mid-level render ordering and C++ ecosystem do not supply Runen's typed work/lifecycle model. |
| 7 | Unreal or Unity | Shipping conventional games and high-end production rendering quickly | Full-engine adoption replaces the project rather than supplying reusable framework boundaries. |
| 8 | CUDA/OptiX | NVIDIA-only compute/ray product | Portability and browser/native cross-platform goals are lost. |

For a conventional game or visualization product, Unreal, Unity, Godot, Bevy,
Filament, or rend3 may be the better engineering decision. The Runen split is
justified only by Dornglut's reusable-framework and representation goals.

## Comparative matrix

| Dimension | Strong reference | Runen target | Current risk |
|---|---|---|---|
| Backend breadth | bgfx, WGPU | WGPU first; second backend only from evidence | Backend-neutrality may remain theoretical. |
| Generic compute | WGPU, CUDA, Godot | First-class independent non-render consumer | Renderer may remain the only real consumer. |
| Resource/work correctness | Unreal, Unity | Typed G3 preparation with no bypass | Validation overhead or excessive ceremony. |
| Simple public path | bgfx, Filament | One-call submit over the same prepared authority | Advanced graph concepts may leak. |
| Engine independence | bgfx | Standalone one-package Rust framework | Runenwerk assumptions may survive adapters. |
| Conventional renderer maturity | Unreal, Unity, Filament | RunenRender remains representation-neutral | Tooling and features will lag for years. |
| Field/volume/procedural flexibility | custom/CUDA/OptiX research systems | Narrow provider capabilities without mandatory source meshes | Provider model may become a universal-trait abstraction. |
| Diagnostics | Unreal, Unity | Structured facts first; UI later | Capture volume and schema stability can expand scope. |
| Progress and pressure | bgfx explicit frame/limits | Explicit G5 progress and quotas | Current ad hoc waits and unbounded queues. |
| Recovery | Bevy/Godot engine integration | Framework facts, Runenwerk policy | Device-loss support may remain untested. |
| Cache compatibility | WGPU backend facts | Versioned derived cache with strict keys | Incorrect reuse or no measurable benefit. |
| Performance | direct APIs and mature engines | Narrow direct-WGPU comparisons | Abstraction/dispatch overhead may erase value. |

## Weakness-to-contract ledger

| External lesson | Runen addresses | Runen inherits | New Runen risk | Owner/phase | Required evidence |
|---|---|---|---|---|---|
| Native/web progress differs | One explicit normalized progress model | Browser/native callback mechanics | Deadlock, reentrancy, or hidden blocking | RunenGPU G5 | Native and WebGPU completion tests; callback lock audit. |
| Shader/pipeline compilation hitches | Cache facts, cold/warm diagnostics, async/product policy separation | Driver/compiler cost | Complex cache with false confidence | G4/G6, Runenwerk policy | Cold/warm timing; incompatible cache rejection; direct baseline. |
| Fixed limits and transient pressure | Structured quotas and pressure outcomes | Hardware memory/limit ceiling | Overly conservative defaults or unbounded fallback | G5/G8 | Saturation tests and high-water diagnostics. |
| Graph bypasses undermine optimization | No compatibility/unsafe common path | Some backend-specific work may remain unavailable | Users forced to fork or reach through | G3/G4/G8 | Source guards; contained specialization review. |
| Transient aliasing complicates debugging | Defer aliasing; preserve initialization diagnostics | Memory pressure | Missing later performance opportunity | Post-G8 only | Separate accepted performance design. |
| ECS render worlds duplicate state | Prepared values/contributions, not a mirror world | Adapter copy cost | Full rebuilds and one-frame latency | RunenRender R1/R2 | Incremental versus full preparation proof. |
| Driver/device loss invalidates resources | Generations and reconstruction reports | Actual driver/device failure | Product policy leaks into framework | G7, Runenwerk | Loss simulation and reconstructability matrix. |
| Large traces/captures create cost/privacy issues | Namespaced versioned capture facts, bounded growth | External tool behavior | Capture becomes persistence authority accidentally | G8, Runenwerk | Schema/version/redaction and size-bound tests. |
| Vendor-specific RT/compute outperforms portable paths | Optional contained specialization | Hardware asymmetry | Backend leakage and two semantic paths | Later accepted design | Same semantic proof plus fallback and no public raw handles. |
| Mature engines have superior tooling | Structured diagnostic foundation | Years of ecosystem investment | Building tools before core correctness | G8/R8 then tools | Tooling issue justified by observed diagnosis friction. |

## Runen-introduced risks

### Duplicate WGPU without semantic value

Every RunenGPU wrapper must add ownership, validation, portability, lifecycle,
testing, or diagnostic value. One-for-one wrappers are rejected.

### Two frameworks, two maintenance burdens

RunenGPU and RunenRender are justified only if both remain independently coherent and
RunenGPU has a real non-render consumer.

### Provider over-generalization

RunenRender must use narrow capabilities. A universal provider trait requiring all
query families is prohibited.

### Prepared-scene copy and rebuild cost

Incremental insert/replace/remove and changed-region facts are mandatory before the
scene architecture stabilizes.

### Graph and dispatch overhead

Preparation, validation, allocation, and dispatch overhead must be measured against a
narrow direct-WGPU path. Architectural cleanliness is not performance evidence.

### Smaller ecosystem and tooling

Runen will initially be worse than mature engines for debuggers, profilers, examples,
asset tooling, and known workarounds. Documentation must state this directly.

## Decisions retained

1. WGPU remains the first backend.
2. RunenGPU is broader than a WGPU wrapper and narrower than Unreal RDG.
3. RunenRender owns image formation and depends on RunenGPU.
4. Runenwerk owns ECS extraction, windows, shader-file policy, product recovery,
   capture persistence, offline sequencing, and artifact encoding.
5. G3 graph semantics remain accepted and unchanged by operational hardening.
6. Multi-queue scheduling, aliasing, pass fusion, graph visualization, universal
   shader IR, broad interop, and hardware ray tracing remain deferred.
7. Existing mature systems remain valid substitutes and must be reevaluated at phase
   gates.

## Strategic reevaluation and kill criteria

Reconsider or stop the RunenGPU split if:

- no independent non-render consumer exists by G6;
- ordinary consumers require raw WGPU access;
- G3/G5 correctness value cannot be demonstrated with simpler direct WGPU code;
- measured CPU/memory overhead is material and cannot be reduced without bypassing
  validation;
- backend-neutrality repeatedly fails because public contracts encode WGPU details;
- progress, pressure, or recovery requires product policy inside RunenGPU.

Reconsider or stop the RunenRender split if:

- procedural/SDF terrain, volume, and incremental-scene proofs do not benefit from
  provider abstraction;
- a rend3/Filament-style renderer satisfies all accepted proofs with less ownership;
- providers require one universal trait or pervasive runtime branching;
- prepared-scene updates systematically rebuild the whole scene;
- optional history/current-frame quality goals cannot share one coherent transport
  family;
- the renderer becomes the only meaningful RunenGPU consumer and non-render use does
  not materialize.

## Final verdict

Assuming correct implementation, RunenGPU plus RunenRender is the strongest strategic
architecture for Dornglut's intended reusable, field/procedural, compute-and-render
platform. It is not the universally best renderer or engine choice.

The strongest competitor is direct WGPU with a smaller custom renderer. The Runen
split earns its cost only through reusable validation, independent consumers,
representation-neutral image formation, operational contracts, clean ownership, and
measured acceptable overhead.
