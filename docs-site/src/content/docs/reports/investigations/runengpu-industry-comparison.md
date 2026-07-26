---
title: RunenGPU Industry Architecture Comparison
description: Comparison of RunenGPU's proposed device, workload-graph, and renderer boundaries with WGPU, bgfx, Godot, Unreal, Unity, Filament, and Bevy.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-26
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ./runengpu-public-api-ergonomics-review.md
  - ./runengpu-render-s0-inventory.md
---

# RunenGPU Industry Architecture Comparison

## Purpose

This report compares the intended RunenGPU boundary with established GPU abstraction and render-graph systems. External systems are evidence and design pressure, not Dornglut authority.

The comparison supports a three-layer target:

```text
semantic consumer
    RunenRender or a non-render compute adapter
        -> generic RunenGPU resources and work
            -> RunenGPU context, validation, realization, and execution
                -> WGPU backend
```

The current `RenderFlow` API mixes all three layers plus Runenwerk integration. It is retained as a transitional facade and decomposed rather than copied.

The separate [public API ergonomics review](runengpu-public-api-ergonomics-review.md) constrains how this architecture is exposed to ordinary callers: the work graph is an internal correctness and inspection model, not mandatory public ceremony.

## Compared systems

### WGPU

WGPU is a cross-platform, safe Rust graphics API based on WebGPU. It provides instances, adapters, devices, queues, resources, pipelines, command encoding, surfaces, and validation across Vulkan, Metal, D3D12, OpenGL, WebGL2, and WebGPU environments.

WGPU is the correct first backend for RunenGPU. It does not provide Dornglut-specific ownership, work-fragment composition, source provenance, application lifecycle separation, or a reusable render/non-render consumer contract.

Source: <https://docs.rs/wgpu/latest/wgpu/>

### bgfx

bgfx is a cross-platform, graphics-API-agnostic, bring-your-own-engine rendering library with many production backends. It exposes a mid-level declarative model based on views, resource handles, draw/dispatch submission, sorting, and a dedicated render thread.

Strengths relevant to RunenGPU:

- strong engine independence;
- broad backend coverage;
- practical resource handles and deferred realization;
- multi-threaded encoders and explicit API/render-thread rules.

Limitations relative to the RunenGPU target:

- render-oriented view and draw ordering rather than a general validated resource/work graph;
- less explicit engine-level ownership, provenance, hazard, and import/export contracts;
- backend breadth is mature, but semantic portability is intentionally mid-level rather than strongly typed around Dornglut's workload model.

Sources:

- <https://bkaradzic.github.io/bgfx/overview.html>
- <https://bkaradzic.github.io/bgfx/internals.html>

### Godot RenderingDevice

Godot's `RenderingDevice` is a low-level abstraction over modern graphics APIs. It exposes buffers, textures, shaders, pipelines, compute, render command lists, capabilities, global and local devices, and asynchronous data retrieval. Godot's renderers sit above it.

Strengths relevant to RunenGPU:

- clear renderer-versus-device layering;
- multiple concrete backend drivers;
- direct compute access outside high-level rendering;
- local device support and explicit resource IDs.

Limitations relative to the RunenGPU target:

- relatively low-level and handle-oriented;
- many synchronization and lifetime responsibilities remain manual;
- public driver-resource escape hatches expose backend objects;
- the documented `RenderingDevice` is unavailable in Godot's headless mode;
- it does not itself provide the complete immutable fragment, provenance, and deterministic graph model intended for RunenGPU.

Sources:

- <https://docs.godotengine.org/en/stable/classes/class_renderingdevice.html>
- <https://docs.godotengine.org/en/stable/tutorials/rendering/renderers.html>

### Unreal Render Dependency Graph

Unreal's Render Dependency Graph records render commands into a graph that is compiled and executed. It manages transient resources, subresource transitions, asynchronous compute fences, parallel command recording, pass culling, validation, and graph/memory visualization.

Strengths relevant to RunenGPU:

- mature whole-frame resource and dependency validation;
- automatic transient lifetime and memory aliasing;
- explicit subresource transition handling;
- pass culling, scheduling, parallel recording, and diagnostics.

Limitations relative to the RunenGPU target:

- deliberately renderer-centric and integrated into Unreal's rendering architecture;
- not an independent general GPU framework for arbitrary engines and non-render consumers;
- much more complex than RunenGPU should initially attempt.

Source: <https://dev.epicgames.com/documentation/unreal-engine/render-dependency-graph-in-unreal-engine>

### Unity Render Graph

Unity's Render Graph sits above the Scriptable Render Pipeline. It records render passes and resource use, manages frame resources, establishes synchronization between graphics and compute queues, reduces allocation pressure, and provides a graph viewer.

Strengths relevant to RunenGPU:

- maintainable modular pass authoring;
- resource lifetime and allocation optimization;
- queue synchronization and frame-resource management;
- strong inspection tooling.

Limitations relative to the RunenGPU target:

- tied to Unity's render-pipeline model;
- primarily texture/frame/render-pass oriented;
- unsafe compatibility passes exist for operations outside graph knowledge;
- does not define an engine-independent device and non-render workload framework.

Sources:

- <https://docs.unity.cn/6000.0/Documentation/Manual/urp/render-graph-introduction.html>
- <https://docs.unity.cn/6000.0/Documentation/Manual/urp/render-graph.html>

### Filament FrameGraph

Filament's FrameGraph models resources and passes, with read/write edges, cycle detection, and culling of unreachable nodes. It is intentionally compact and frame-rendering oriented.

Strengths relevant to RunenGPU:

- simple resource/pass dependency model;
- clear read/write edge semantics;
- culling and cycle detection without excessive public complexity.

Limitations relative to the RunenGPU target:

- frame-rendering focus;
- narrower context, capability, imports/exports, asynchronous readback, surface, and device-loss model;
- not intended as a general engine-neutral GPU execution package.

Source: <https://google.github.io/filament/notes/framegraph.html>

### Bevy render graph and renderer

Bevy uses WGPU and provides a render graph/schedule within its render architecture. Its renderer exposes instance, adapter, device, queue, surfaces, command encoders, ECS-integrated schedules, and view queries.

Strengths relevant to Runenwerk:

- Rust and WGPU-native implementation evidence;
- strong ECS integration and modular render systems;
- practical render-device and queue wrappers;
- graph extensibility through nodes and subgraphs.

Limitations relative to the RunenGPU target:

- render graph and execution are closely coupled to Bevy's ECS and render schedule;
- not suitable as the final independent RunenGPU public boundary;
- direct WGPU wrapper types and render-world assumptions would violate RunenGPU's framework independence.

Sources:

- <https://docs.rs/bevy/latest/bevy/render/index.html>
- <https://docs.rs/bevy/latest/bevy/render/renderer/>
- <https://docs.rs/bevy/latest/bevy/render/render_graph/struct.RenderGraph.html>

## Comparative verdict

| Dimension | Strong reference | RunenGPU target | Current position |
|---|---|---|---|
| Backend coverage | bgfx, Godot | WGPU first; additional backends only from demand | Less flexible today |
| Low-level portability | WGPU, Godot, bgfx | Hide backend types where semantic value exists | Sound direction |
| General compute use | WGPU, Godot, bgfx | First-class render and non-render workloads | More general than render-only graphs if proven |
| Resource/work validation | Unreal, Unity | Deterministic generic graph with typed errors | Planned, not yet mature |
| Transient optimization | Unreal, Unity, Filament | Add only after correctness and real pressure | Intentionally less ambitious initially |
| Engine independence | bgfx | Standalone one-package Rust framework | Strong target, not yet proven |
| Rust integration | WGPU, Bevy | Rust-native without ECS coupling | Better independence target than Bevy |
| Tooling and visualization | Unreal, Unity | Structured diagnostics first; visualization later | Worse initially |
| Ergonomics | Unity, Unreal, current RenderFlow | Simple facade plus inspectable advanced path | Must be proven before API freeze |
| Safety and ownership | no single reference | Typed IDs, explicit provenance, structured errors | Potential differentiator |

## Design decisions retained

RunenGPU should retain:

1. **WGPU as the first backend**, not recreate Vulkan/Metal/D3D12 support directly.
2. **A backend-neutral device/execution layer**, similar in placement to Godot RenderingDevice or bgfx.
3. **A generic validated work graph**, informed by Unreal, Unity, and Filament, but usable by compute and rendering consumers.
4. **A separate renderer semantic layer**, so scenes, views, materials, lighting, transport, overlays, and presentation intent remain in RunenRender.
5. **Runenwerk adapters outside both frameworks**, especially ECS projection, windows, source discovery, hot reload, scheduling, recovery, and product policy.
6. **A simple common API over the graph**, so ordinary users submit typed work without manually constructing epochs or invoking validation phases.

## Guardrails against overengineering

RunenGPU must not:

- wrap every WGPU type one-for-one without additional ownership or validation value;
- promise multiple backends before a second implementation exists;
- attempt Unreal-level scheduling, aliasing, multi-queue, and visualization in the initial extraction;
- make a render graph the universal semantic API for simulations, tools, and renderers;
- require ordinary users to understand work graphs, execution epochs, admission, realization, or retirement;
- expose ECS, windows, shader files, scenes, materials, or product policy;
- preserve current `RenderFlow` as a monolithic compatibility authority;
- add an unsafe escape hatch until a concrete consumer and containment rule justify it.

## Recommended initial ambition

The first extraction should be narrower than Unreal RDG and broader than a WGPU wrapper:

```text
required for GX
    normalized capabilities
    logical resources and typed handles
    explicit access and dependencies
    deterministic validation behind a simple public path
    inspectable preparation for tools and tests
    context/device admission
    shader/pipeline realization
    headless compute, upload, readback
    offscreen graphics
    surfaces and structured outcomes
    diagnostics and shutdown

explicitly deferred
    automatic multi-queue scheduling
    aggressive transient memory aliasing
    pass fusion
    backend-independent shader IR
    sparse resources and external interop
    hardware ray tracing as a baseline
    graph visualization UI
```

This target is flexible enough for independent rendering and compute workloads while avoiding the cost of cloning a production AAA render graph before Dornglut has equivalent consumer pressure.
