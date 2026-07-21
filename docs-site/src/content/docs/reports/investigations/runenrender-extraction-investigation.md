---
title: RunenGPU and RunenRender Boundary Investigation
description: Connector-backed current ownership findings and remaining S0 evidence required to separate GPU execution from image formation.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU and RunenRender Boundary Investigation

## Question

Which current `engine/src/plugins/render` responsibilities are:

- general GPU execution owned by RunenGPU;
- image-formation semantics owned by RunenRender;
- Runenwerk host/product/integration responsibility;
- another domain's responsibility;
- redesign or deletion candidates?

## Verdict

```text
RUNENGPU CANDIDATE                 yes
RUNENRENDER CANDIDATE              yes
MOVE CURRENT RENDER DIRECTORY      forbidden
INITIAL PUBLIC PACKAGES            runen-gpu, runen-render
DIRECT WGPU OWNER                  RunenGPU only
FIRST EXECUTABLE WORK              none; S0 inventory first
EXTERNAL SOURCE MOVEMENT           forbidden
```

The current plugin combines general GPU execution, image formation, domain
adaptation, native host integration, and product policy. It cannot be extracted as
one repository or repaired by renaming.

The required dependency direction is:

```text
RunenRender -> RunenGPU
```

The correct approach is two internal public-boundary proofs followed by separate
clean cutovers.

## Evidence status

The durable findings below come from connector-backed source, manifest, module,
and control-flow inspection originally recorded against the current renderer
family before the architecture correction. Workflow and planning infrastructure
changed afterward, but no GPU/render Rust cutover has merged.

Evidence currently supports ownership correction. It does not satisfy the complete
S0 file/consumer/command inventory.

Not yet verified by this investigation:

- complete recursive file and shader listing;
- every downstream consumer/import;
- current `cargo validate` with any proposed implementation;
- headless/offscreen/surface/device-loss behavior;
- GPU adapter/driver evidence;
- renderer benchmarks;
- exact move/stay/redesign/delete disposition.

## Current package boundary

Rendering remains inside the `engine` package. The inspected render root exposes
families such as:

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

`engine_render_macros` is a separate proc-macro package whose generated paths target
`engine::plugins::render`.

Source grouping is evidence, not target ownership.

## Current ownership aggregation

The current implementation combines:

1. WGPU instance, adapter, device, queue, resources, pipelines, and submission;
2. generic-looking graph/resource/access/lifetime planning;
3. native-window surface creation and presentation;
4. Runenwerk plugin scheduling and frame lifecycle;
5. ECS resource and state extraction;
6. scene/world preparation;
7. material graph compilation and asset loading;
8. SDF/world residency and ray-march product policy;
9. UI preparation and built-in composite behavior;
10. editor picking and product features;
11. shader filesystem discovery, polling, and hot reload;
12. diagnostics, capture, artifact export, startup readiness, and frame pacing.

This is integration composition, not one reusable framework boundary.

## Ownership classification

### RunenGPU candidates

Potential RunenGPU responsibility:

- device/context/queue creation and backend facts;
- normalized capability admission;
- buffers, textures, views, samplers, query resources, and pipelines;
- access, initialization, lifetime, hazard, and retirement validation;
- generic compute/render/copy/clear/resolve/present workload execution;
- shader/module/pipeline realization;
- staging upload and asynchronous readback;
- command encoding, submission, completion, and shutdown;
- low-level surface creation/configuration/acquisition/presentation;
- structured GPU/surface/device outcomes and timings.

These candidates must be purged of renderer, ECS, scene, world, material, SDF, UI,
editor, and product meaning before extraction.

### RunenRender candidates

Potential RunenRender responsibility:

- prepared render scenes and deterministic contributions;
- renderer views and logical targets;
- provider/instance/interaction contracts;
- material, medium, emitter, and environment rendering semantics;
- visibility/provider-query policy;
- light transport and estimator policy;
- radiance caches and bounded history;
- reconstruction, overlay, color, and presentation intent;
- renderer diagnostics/provenance;
- lowering into RunenGPU work fragments.

These candidates must not own WGPU, windows, ECS extraction, source authoring,
field/SDF mathematics, UI semantics, or product recovery.

### Runenwerk-retained responsibility

Runenwerk retains:

- application/plugin/frame lifecycle;
- native windows, event loop, DPI, monitor, resize, and visibility policy;
- ECS and domain extraction;
- scene/world/material-authoring/SDF/UI/editor/simulation adapters;
- shader filesystem discovery, revision, watch, reload, and last-known-good policy;
- product feature, quality, fallback, artifact, startup, and recovery policy;
- integration diagnostics and runtime evidence.

### Other-domain responsibility

The following remain with their semantic owners:

```text
SDF values, bounds, and query safety       RunenSDF/domain SDF
ECS lifecycle and query semantics          RunenECS/domain ECS
UI state/layout/hit testing/accessibility  RunenUI
material authoring graph/source            material-authoring owner
scene/world authoritative state            scene/world owners
simulation algorithms/state                simulation owners
```

Adapters lower accepted facts into RunenGPU or RunenRender inputs.

## Identity findings

Current runtime concepts include identities such as:

```text
RenderFlowId
RenderPassId
RenderResourceId
RenderFeatureId
RenderFrameProducerId
```

Current IDs use Runenwerk identity infrastructure and are consumed across graph,
resource, producer, feature, and runtime paths.

These names do not prove one semantic owner. S0 must classify every identity as:

```text
RunenGPU runtime identity
RunenRender semantic identity
Runenwerk producer/product identity
source/persisted identity
delete or redesign
```

The old renderer-first identity phase is unsafe because it could encode GPU and
Runenwerk concepts into RunenRender.

S0 must also inspect:

- allocators and sequences;
- safe raw constructors and `.raw()` consumers;
- hashing/ordering assumptions;
- persistence, replay, network, cache, trace, and artifact uses;
- stale-handle or generation requirements;
- foreign-context requirements.

## Graph findings

The current graph contains potentially reusable planning mechanics but also host
and product semantics:

- built-in UI composite behavior;
- UI and product feature IDs;
- host `TypeId` requirements and callbacks;
- fixed-time regions;
- product view categories;
- material-scene shader selection and fallback policy.

Target separation:

```text
RunenRender
    semantic render plan and quality/visibility/transport intent
        -> RunenGPU work fragments

RunenGPU
    compute/render/copy/clear/resolve/present nodes
    resource accesses and dependencies
    hazard/capability/lifetime validation
```

Neither graph reaches back into ECS or host state.

## Frame and producer findings

The current frame model mixes potentially reusable view/resource facts with
product selection, main/offscreen product semantics, UI contributions, and
ECS-backed registries.

Target:

```text
Runenwerk adapters
    resolve source state
    -> immutable RenderContribution values

RunenRender
    compose PreparedRenderScene
    -> deterministic render plan
    -> RunenGPU work fragments
```

Producer identities are not raw ECS entity identities. Replacement/removal and
retirement are explicit.

## Backend and surface findings

Current WGPU initialization selects device and native surface together from a
Winit window. This prevents clean headless initialization and couples host lifetime
to backend execution.

Target:

```text
Runenwerk host
    windows/event loop/handles/DPI/resize/visibility/recovery

RunenGPU
    headless context
    optional surface realization from admitted handles
    configuration/acquire/present/device outcomes

RunenRender
    logical target, color, compositing, presentation intent
```

S0 must trace:

- raw handle admission and lifetime;
- thread affinity;
- surface creation/retirement;
- resize/reconfiguration;
- acquire/present outcomes;
- window/surface/device/in-flight drop order;
- multi-window and headless paths;
- device-loss reconstruction ownership.

## Resource and residency findings

Current resource infrastructure mixes generic-looking handles/lifetimes with
surface defaults, host type state, material/world/SDF residency, dynamic texture
policy, and saturating arithmetic.

Potential RunenGPU ownership:

- validated resource descriptors;
- context/generation/stale-handle behavior;
- imports/exports;
- access and hazard validation;
- transient lifetime planning;
- generic budgets and backend allocation facts.

Potential RunenRender ownership:

- render-provider acceleration and cache keys;
- render-specific realization requirements;
- history/radiance/reconstruction validity.

Runenwerk/domain owners retain authoritative source and reconstruction policy.

## Shader and pipeline findings

Current shader authority combines:

- generic identity/source revision;
- filesystem roots and path normalization;
- polling/throttling/forced reload;
- last-known-good application policy;
- material translation;
- Naga/WGSL/WGPU realization;
- tracing and temporary-file tests.

Target split:

```text
consumer/RunenRender
    shader meaning and source product

RunenGPU
    admission, backend validation, module/binding/pipeline realization

Runenwerk
    filesystem discovery/watch/reload, source revision, material translation,
    product fallback and user diagnostics
```

Logical parameters, byte representation, binding representation, and WGPU layout
must remain distinct.

## Macro findings

Current `GpuUniform` and `GpuStorage` derives generate layout types, conversions,
bytemuck traits, and hard-coded engine render paths.

A macro package is not accepted by default. S0 must determine:

- every macro consumer;
- whether the derives are still the correct public API;
- byte layout, padding, alignment, arrays, matrices, nested structures, and
  bytemuck safety;
- dependency renaming and external compile-pass/fail behavior;
- whether ordinary traits/builders are sufficient.

If retained, backend ABI mechanics belong with RunenGPU, while renderer parameter
meaning remains with RunenRender or the source owner.

## RunenUI findings

RunenUI and RunenRender remain independent.

RunenUI owns semantic UI, state, focus, accessibility, layout/style/text, hit
testing, and renderer-neutral paint output.

A future Runenwerk bridge may translate paint output into a RunenRender overlay
contribution. RunenRender must not access widget state or perform UI hit testing.
RunenUI core does not depend on RunenGPU.

## RunenSDF findings

RunenSDF standalone transfer is complete at revision
`d52badefc640d6dc6dcdd40268af3aea1bb8eefe`. Current Runenwerk `main` does not yet
record a completed internal clean cutover.

RunenSDF remains backend-neutral. Rendering and GPU realization consume explicit
adapter products that preserve field capabilities and numerical safety.

## Diagnostics findings

Potential RunenGPU diagnostics:

- backend/adapter/capabilities;
- resources, workloads, hazards, submissions;
- shader/pipeline realization;
- uploads/readbacks/timings;
- surface/device/terminal outcomes.

Potential RunenRender diagnostics:

- prepared scene/contribution validity;
- provider/material/emitter/transport admission;
- quality degradation;
- cache/history/reconstruction/overlay validity;
- mapping to RunenGPU work provenance.

Runenwerk retains user presentation, artifact paths, capture/export policy,
startup readiness, frame pacing, and recovery.

## S0 required output

Before any G1 specification, produce:

1. complete recursive file, shader, macro, test, example, benchmark, and artifact
   inventory;
2. complete Cargo/import/downstream consumer graph;
3. identity/allocator/raw-use/stable-format classification;
4. graph/resource/frame/producer control-flow traces;
5. context/device/queue/surface/window/drop/shutdown trace;
6. shader/pipeline/reload/macro ABI inventory;
7. domain and product reach-back inventory;
8. headless/offscreen/surface/device-loss/runtime/benchmark command inventory;
9. exact move/stay/redesign/delete matrix;
10. first bounded G1 candidate and stop conditions;
11. current `cargo validate` plus separately reported environment-dependent evidence.

## Stop conditions

Stop before implementation when:

- any file or active consumer is unclassified;
- runtime IDs may be persisted/transmitted without a separate stable format;
- GPU versus renderer versus Runenwerk ownership remains ambiguous;
- surface/window/device/drop order remains unknown;
- macro safety/ABI cannot be separated;
- the first slice requires graph, WGPU, renderer, adapter, and product changes at
  once;
- a speculative package or compatibility layer is proposed;
- current main is not green for unrelated reasons.

## Conclusion

The current source contains credible reusable GPU and rendering foundations, but
its ownership is combined. The durable next action is S0—not renderer identity
implementation, directory movement, or external repository population.
