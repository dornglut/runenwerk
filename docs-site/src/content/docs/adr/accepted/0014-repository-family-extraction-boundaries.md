---
title: Repository Family Extraction Boundaries
description: Accepted repository-level clean-cutover decision for RunenSDF, RunenECS, RunenGPU, and RunenRender while retaining Runenwerk integration ownership and independent RunenUI authority.
status: accepted
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-21
related_designs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../design/active/runenecs-extraction-boundary-design.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ./0015-separate-gpu-execution-from-rendering.md
related_roadmaps:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
---

# ADR 0014: Repository Family Extraction Boundaries

## Amendment

ADR 0015 amends this ADR's original RunenRender backend/package decision.

The original decision selected a renderer-owned WGPU backend. The accepted target
is now:

```text
RunenGPU
  runengpu_core
  runengpu_wgpu

RunenRender
  runenrender_core
  runenrender_gpu

RunenRender -> RunenGPU
```

All clean-cutover, Runenwerk integration, RunenUI independence, versioning,
provenance, and conformance rules in this ADR remain authoritative.

## Decision

Create independent framework repositories through governed clean cutovers:

- `Crystonix/runen-sdf`;
- `Crystonix/runen-ecs`;
- `Crystonix/runen-gpu`;
- `Crystonix/runen-render`.

Runenwerk remains the integration and product repository. Framework repositories
must not depend on Runenwerk. Integration translation, application lifecycle,
product policy, and cross-domain composition remain in Runenwerk.

RunenUI remains an independent peer repository governed separately:

- RunenUI does not depend on Runenwerk or RunenRender by default;
- RunenUI core/runtime do not require RunenGPU;
- RunenRender does not depend on RunenUI;
- Runenwerk may later translate accepted renderer-neutral RunenUI output into
  RunenRender overlay work;
- standalone RunenUI backends may exist without becoming RunenRender authority.

This ADR does not select RunenUI APIs or authorize RunenUI implementation.

## Dependency direction

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk adapters/integration --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Direct framework dependencies require accepted evidence of independent value and
correct ownership. `RunenRender -> RunenGPU` is the accepted lower-level exception.
It is not introduced to avoid a Runenwerk domain adapter.

## Extraction order

Tracks may progress at different maturity levels:

1. RunenSDF: correct and prove the numerical/API boundary, then extract through its
   bounded standalone-transfer and cutover phases.
2. RunenECS: close safety, scheduler, spatial, reflection, messaging, change, and
   replication ownership before source movement.
3. RunenGPU: complete the GPU/render inventory, decompose and prove general GPU
   execution internally, then extract and cut Runenwerk over.
4. RunenRender: decompose image formation on the accepted RunenGPU boundary, prove
   public seams through Runenwerk consumption, then extract.

RunenGPU precedes RunenRender because the current renderer plugin combines general
GPU execution with rendering, windows/surfaces, ECS, scene, material, SDF, UI,
editor, diagnostics, and runtime integration.

## Clean cutover

Every completed extraction must:

- preserve source provenance and licensing;
- establish independent validation and public downstream conformance;
- pin Runenwerk to an exact revision or exact prerelease version;
- migrate all active consumers;
- delete the original Runenwerk implementation in the completed cutover;
- remove temporary migration seams;
- leave no compatibility package, forwarding namespace, source mirror, submodule,
  moving-branch dependency, or writable parallel authority.

GPU/render cutovers additionally leave no duplicate device/queue/resource
namespace or renderer execution path.

Temporary duplication may exist only on an unmerged extraction branch.

## Ownership decisions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary,
numerical policy, spatial bounds, composition, capabilities, and CPU reference
queries. It does not own Runenwerk geometry, world streaming, ECS, rendering, GPU,
materials, or product policy.

Field samples and queries distinguish values from algorithmically safe tracing
steps and expose exact-distance capability explicitly.

### RunenECS

RunenECS owns ECS semantics, not a permanently fixed storage implementation.
General spatial indexing, engine lifecycle, rendering extraction, GPU execution,
networking, replay, and world policy remain outside ECS core.

Scheduler ownership is divided between neutral scheduling, ECS integration, and
Runenwerk frame/tick policy. Messaging and change-journal facilities remain
provisional until consumer evidence proves their independent ECS role.

### RunenGPU

RunenGPU owns general GPU execution:

- runtime identities and capabilities;
- resources, access, initialization, and lifetimes;
- bounded GPU work descriptions and validation;
- shader/pipeline admission and backend realization;
- compute/render/copy execution;
- headless operation, uploads, readback, completions;
- low-level surfaces and device outcomes;
- GPU diagnostics and provenance.

Required candidates are `runengpu_core` and `runengpu_wgpu`.

RunenGPU does not own image formation, materials, transport, simulations, SDF,
UI, ECS, windows/event loops, or product recovery.

### RunenRender

RunenRender owns image formation:

- prepared render scenes and contributions;
- views and logical targets;
- providers/interactions;
- materials, media, emitters, and environments;
- visibility, light transport, caches, reconstruction;
- overlays, output color intent, and render diagnostics;
- render-specific lowering into RunenGPU work.

Required candidates are `runenrender_core` and `runenrender_gpu`.

RunenRender does not own a WGPU device, queue, surface, generic resource system,
ECS extraction, authoring, simulations, SDF mathematics, UI semantics, windows,
or product selection.

### Runenwerk

Runenwerk retains:

- application and frame lifecycle;
- native windows and event-loop policy;
- GPU context configuration and product capability selection;
- ECS/domain extraction;
- scene/world/material/SDF/UI/editor/simulation adapters;
- shader source discovery and hot-reload policy;
- product feature/quality selection;
- diagnostics presentation, artifacts, and recovery.

## Shared infrastructure

Do not create a universal `RunenCore`, meta-framework, universal ID crate, or
universal diagnostics crate.

RunenGPU is not a shared semantic core. It owns only GPU execution.

Each repository owns identities and values whose invariants it defines. Adapters
map them explicitly. Diagnostics use repository namespaces and preserve upstream
identity.

## Versioning and formats

Before stable publication, cross-repository dependencies use exact revisions or
exact prerelease versions. Moving branches are forbidden.

Persisted source, artifact, trace, replay, cache, and wire formats require an
owner, identifier/version, validation, and migration policy. Rust API versioning
does not implicitly version persisted data. Runtime GPU/render handles are not
stable formats.

## Consequences

- Parallel investigation is allowed, but implementation gates differ by track.
- Shared workspace and planning files have one merge owner at a time.
- RunenSDF remains the first extraction-workflow proof.
- RunenECS source movement remains blocked by safety and ownership work.
- RunenGPU external extraction is blocked until internal shared-consumer and
  public-boundary proof.
- RunenRender implementation/extraction is ordered after the accepted RunenGPU
  boundary and cutover.
- Existing code location is evidence, not permanent ownership.
- Connector inspection cannot satisfy command-validation gates.

## Rejected alternatives

Rejected:

- extracting current directories immediately;
- one repository containing SDF, ECS, GPU, rendering, and UI;
- Git submodules or source mirrors;
- a universal shared-core repository;
- long-lived compatibility packages;
- keeping general compute inside RunenRender;
- renaming all rendering to RunenGPU;
- duplicate GPU contexts per domain;
- making RunenUI depend on RunenRender solely for Runenwerk integration;
- moving Runenwerk product policy into framework cores.

## Fitness functions

The program succeeds only when:

- each framework validates independently;
- Runenwerk consumes through one-way public dependencies;
- independent downstream consumers use public APIs;
- framework repositories contain no Runenwerk assumptions;
- adapters translate rather than duplicate algorithms;
- RunenGPU serves rendering and non-render compute through one public execution
  boundary;
- RunenRender uses RunenGPU rather than direct WGPU ownership;
- original Runenwerk implementations are removed after cutover;
- no dependency cycle, duplicate execution path, source mirror, or compatibility
  authority remains;
- provenance, licensing, compatibility, and current documentation are complete.
