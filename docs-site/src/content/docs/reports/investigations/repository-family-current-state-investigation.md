---
title: Repository Family Current-State Investigation
description: Current source, dependency, ownership, and extraction-readiness evidence for RunenSDF, RunenECS, RunenGPU, and RunenRender.
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
  - ../../workspace/planning/roadmap.md
---

# Repository Family Current-State Investigation

## Question

Which Runenwerk subsystems are independently useful framework candidates, what is
their current state, what boundaries remain incorrect, and which work may proceed
safely in parallel?

## Current repository family

```text
product       repository                 package       crate
RunenSDF      Crystonix/runen-sdf        runen-sdf     runen_sdf
RunenECS      Crystonix/runen-ecs        runen-ecs     runen_ecs
RunenGPU      Crystonix/runen-gpu        runen-gpu     runen_gpu
RunenRender   Crystonix/runen-render     runen-render  runen_render
RunenUI       Crystonix/runen-ui         runen-ui      runen_ui
Runenwerk     Crystonix/runenwerk        workspace      integration/product
```

RunenSDF and RunenUI exist as independent repositories. RunenECS, RunenGPU, and
RunenRender require the accepted internal-proof and clean-cutover sequences before
Runenwerk source movement.

## Evidence basis and limits

This investigation is grounded in:

- GitHub repository, branch, commit, issue, and pull-request state;
- source, manifest, module, and documentation inspection;
- exact-head CI and documentation validation from completed workflow cleanup;
- accepted RunenSDF standalone conformance evidence;
- connector-backed GPU/render ownership inspection.

It does not yet provide the complete GPU/render S0 file and consumer inventory, GPU
adapter/driver evidence, headless/offscreen/surface/device-loss runtime evidence,
or renderer performance baselines.

Source classification establishes ownership and sequencing. It does not authorize
implementation by itself.

## Program-state summary

| Candidate | Current location/state | Readiness | Next allowed work |
|---|---|---|---|
| RunenSDF | standalone repository accepted at `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`; `domain/sdf` still present on current Runenwerk `main` | standalone complete; Runenwerk cutover separate | exact consumer audit and separately reviewed clean cutover |
| RunenECS | `domain/ecs`, `domain/ecs_macros`, parts of `domain/scheduler` | internal ownership/safety repair required | bounded R1 implementation after current-main review |
| RunenGPU | GPU execution mixed into `engine/src/plugins/render` and related packages | architecture accepted; S0 incomplete | read-only S0 inventory only |
| RunenRender | image formation mixed with GPU/host/domain/product code | architecture accepted; depends on RunenGPU extraction | read-only S0 inventory only |
| RunenUI | independent repository/workstream | governed separately | RunenUI roadmap; no Runenwerk ownership claim |

## RunenSDF

### Completed

The in-workspace boundary correction completed through Runenwerk PR `#116`.

The standalone repository transfer completed through Runenwerk PR `#118` and
`Crystonix/runen-sdf` PR `#1` at:

```text
d52badefc640d6dc6dcdd40268af3aea1bb8eefe
```

Accepted standalone ownership includes:

- field values and samples;
- conservative tracing steps and exact-distance capabilities;
- validated bounds and rays;
- primitives, composition, transforms, gradients, normals, and queries;
- numerical policy and CPU reference conformance.

RunenSDF does not own Runenwerk geometry, world streaming, ECS, rendering, GPU,
materials, UI, or product policy.

### Remaining Runenwerk state

Current Runenwerk `main` still contains `domain/sdf`. No merged Runenwerk evidence
currently records the clean-cutover deletion as complete.

The next decision must inventory every code, test, manifest, adapter, document, and
persisted consumer, then either:

- pin the accepted external revision for real active consumers and delete the
  internal authority; or
- delete the internal package without adding an unused external dependency.

No forwarding package, source include, mirror, alias, submodule, or branch
dependency may survive.

## RunenECS

### Current aggregation

Current ECS-related source includes:

- entity/component/resource/world lifecycle;
- storage and queries;
- commands and deferred structural mutation;
- systems and access declarations;
- reflection and macros;
- geometry-dependent spatial indexing;
- configured scheduling/runtime reports;
- messaging, work queues, change extraction, ownership, telemetry, and integration
  surfaces.

### Boundary problems

- geometry/spatial policy is exposed through ECS ownership;
- scheduler versus ECS-local scheduling ownership requires explicit separation;
- reflection and macro contracts require independent public proof;
- unsafe query/SystemParam boundaries require review;
- messaging, networking, replay, ownership, and change extraction mix reusable and
  Runenwerk-specific semantics;
- moving current directories unchanged would freeze accidental aggregation.

### Next work

The accepted repair order remains R1–R9. R1 covers identity and structured errors.
No external source movement is authorized before the internal proof sequence and
standalone conformance complete.

## Current GPU/render aggregation

The current `engine/src/plugins/render` and related macro/shader/runtime code
combine:

1. WGPU instance, adapter, device, queue, resources, pipelines, and submission;
2. generic-looking graph, resource, access, and lifetime planning;
3. native-window surface creation and presentation;
4. Runenwerk plugin/frame/time lifecycle;
5. ECS and domain extraction;
6. scene/world preparation;
7. material graph compilation and asset preparation;
8. SDF/world residency and product rendering policy;
9. UI and editor integration;
10. shader filesystem discovery and hot reload;
11. diagnostics, capture, artifacts, startup, frame pacing, and recovery policy.

Moving this directory unchanged is forbidden.

## RunenGPU findings

### Target ownership

RunenGPU owns:

- GPU contexts and execution epochs;
- normalized capabilities and requirements;
- resources, views, pipelines, and query resources;
- initialization, access, lifetimes, hazards, imports/exports, and retirement;
- compute/render/copy/clear/resolve/present workloads;
- shader/pipeline admission and WGPU realization;
- uploads, asynchronous readback, submission, completion, and shutdown;
- low-level surfaces and backend/device outcomes;
- GPU diagnostics, timings, and provenance.

### Target exclusions

RunenGPU does not own renderer semantics, field/simulation algorithms, ECS, UI,
world/editor/product meaning, native-window/event-loop policy, shader file
watching, or product recovery.

### Readiness

```text
ARCHITECTURE: accepted
S0 INVENTORY: incomplete
IMPLEMENTATION SPEC: none
RUST IMPLEMENTATION: forbidden until S0 review
EXTERNAL TRANSFER: forbidden
```

## RunenRender findings

### Target ownership

RunenRender owns:

- prepared render scenes and deterministic contributions;
- views and logical targets;
- providers, instances, and interactions;
- materials, media, emitters, and environments;
- visibility and provider-query policy;
- transport, quality, radiance caches, history, and reconstruction;
- overlays, color, and presentation intent;
- lowering render plans into RunenGPU workloads;
- renderer diagnostics and provenance.

### Target exclusions

RunenRender does not own WGPU directly, general GPU execution, ECS extraction,
field/SDF mathematics, UI semantics/hit testing, native windows, shader file
watching, or Runenwerk lifecycle/product recovery.

### Readiness

```text
ARCHITECTURE: accepted
DEPENDENCY: RunenRender -> RunenGPU
S0 INVENTORY: incomplete
RUNENGPU CUTOVER: prerequisite
IMPLEMENTATION SPEC: none
EXTERNAL TRANSFER: forbidden
```

## Identity problem

Current `Render*Id` values span graph, resources, features, producers, and runtime
paths. Their names and location do not prove one semantic owner.

S0 must classify every identity and allocator as:

```text
RunenGPU runtime identity
RunenRender semantic identity
Runenwerk producer/product identity
source/persisted identity
delete or redesign
```

It must inspect raw constructors/accessors, hashing/order assumptions, generation
and stale-handle needs, and persistence/replay/network/cache/trace/artifact uses.

The old renderer-first identity phase is unsafe and retired.

## Required GPU/render sequence

```text
S0 complete inventory
-> G1-G8 internal RunenGPU proof
-> GX external RunenGPU clean cutover
-> R1-R8 internal RunenRender proof on RunenGPU
-> RX external RunenRender clean cutover
-> reusable adapter review
-> advanced renderer work
```

RunenGPU precedes RunenRender because rendering depends on general GPU execution.
Advanced renderer work must not harden accidental ownership before foundational
cutovers.

## S0 required evidence

S0 must provide:

- complete file, shader, macro, test, example, benchmark, and artifact inventory;
- complete Cargo and downstream consumer graph;
- identity, allocator, raw-use, and stable-format classification;
- graph/resource/frame/producer control-flow trace;
- context/device/queue/surface/window/drop/shutdown trace;
- shader/pipeline/reload/macro ABI ownership map;
- domain/product reach-back inventory;
- headless/offscreen/surface/device-loss/runtime/benchmark command inventory;
- exact move/stay/redesign/delete matrix;
- current `cargo validate` and separately reported environment-dependent evidence;
- one bounded first implementation candidate and stop conditions.

## RunenUI disposition

RunenUI remains an independent peer.

It owns semantic UI, state/actions, focus/accessibility, layout/style/text, hit
testing, and renderer-neutral paint output.

A future Runenwerk bridge may translate accepted paint output into a RunenRender
overlay contribution. RunenRender does not access widget state or perform UI hit
testing. RunenUI does not depend on RunenGPU by default.

## Parallel-work decision

Allowed in parallel:

- read-only GPU/render S0 inventory;
- independently scoped RunenECS repair work that does not share manifests,
  identities, or lifecycle ownership;
- RunenUI work in its repository;
- separately reviewed RunenSDF consumer/cutover investigation;
- unrelated product work with no shared architecture files.

Must be serialized or explicitly rebased:

- root workspace manifests and lockfiles;
- GPU/render identity and lifecycle changes;
- framework dependency direction;
- canonical repository-family architecture;
- external source movement or deletion.

## Rejected extraction attempt

Commit `b5e9624c...` remains historical evidence of deletion before complete
workspace, dependency, consumer, test, asset, and authority cutover. It is not an
implementation base.

PR `#119` is also superseded: it correctly identified `RunenRender -> RunenGPU`,
but used stale repository state, speculative multi-package targets, and retired
planning authorities.

## Conclusion

RunenSDF standalone transfer is complete; its Runenwerk clean cutover remains a
separate decision. RunenECS requires ordered repair. RunenGPU and RunenRender have
accepted ownership architecture, but S0 is the only safe next action. No GPU or
renderer implementation or external source movement is authorized yet.
