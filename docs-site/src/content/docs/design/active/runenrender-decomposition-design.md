---
title: RunenRender Decomposition Design
description: Target ownership, package boundaries, host/surface model, producer seams, and staged internal proof required before extracting the Runenwerk renderer.
status: active
owner: render
layer: engine/render
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../reports/investigations/repository-family-current-state-investigation.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# RunenRender Decomposition Design

## Status

This document fixes the target ownership and decomposition order. It does not
authorize moving `engine/src/plugins/render` into another repository unchanged.

The next implementation authority must follow a complete per-module, shader,
example, benchmark, and test inventory.

## Goal

Create a renderer that can be used independently of Runenwerk while keeping
Runenwerk-specific extraction, lifecycle, product policy, and domain adaptation
in Runenwerk.

The renderer must support conventional raster/compute workloads and generic
producer submissions without requiring ECS, SDF, UI, scene, material-authoring,
editor, or application semantics.

## Candidate repository shape

The target is expected to use two packages initially:

```text
RunenRender/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── runenrender_core/
│   └── runenrender_wgpu/
├── tests/
├── examples/
├── benches/
├── docs/
├── LICENSE-MIT
├── LICENSE-APACHE
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
└── CHANGELOG.md
```

A `runenrender_macros` package is created only if review proves that
`engine_render_macros` is backend-neutral, publicly useful, and cannot remain
ordinary Rust APIs without material usability loss.

Do not create separate graph, shader, resource, pipeline, residency, or surface
crates initially. Package boundaries require independent dependency or release
pressure.

## Package missions

### runenrender_core

Owns backend-neutral contracts:

- render resource and pass identities;
- graph declarations;
- access and dependency descriptions;
- graph validation;
- resource lifetime planning;
- frame requests and validated frame plans;
- generic producer contributions/submissions;
- target and view descriptions;
- backend capability requirements;
- backend-neutral diagnostics and provenance;
- deterministic planning and inspection products.

It must not depend on:

- WGPU;
- Winit;
- Runenwerk;
- ECS;
- RunenSDF;
- RunenUI;
- scene or world domains;
- material-authoring graphs;
- application lifecycle.

### runenrender_wgpu

Owns the conventional WGPU implementation:

- instance, adapter, device, and queue;
- backend capabilities and format selection;
- GPU resource allocation;
- buffer and texture uploads;
- shader module and pipeline realization;
- command encoding and submission;
- WGPU surface configuration, acquisition, presentation, and retirement;
- device-loss and surface-error classification;
- backend caches and backend diagnostics.

It consumes validated `runenrender_core` plans and requests. It does not inspect
Runenwerk ECS resources or product domains.

## Runenwerk-owned integration

Runenwerk retains:

- `RenderPlugin` or its replacement integration plugin;
- application/frame lifecycle scheduling;
- window and event-loop policy;
- mapping native windows to renderer surface handles;
- ECS extraction;
- scene/world preparation;
- material-authoring translation;
- SDF representation and render preparation;
- editor picking and editor-specific render policy;
- debug overlay product policy;
- future RunenUI scene translation;
- product feature selection;
- cross-domain diagnostics and runtime evidence.

## Current-state diagnosis

The current render plugin initializes and schedules a wide product graph:

- scene resources;
- shader, flow, fragment, feature, and frame registries;
- prepared UI, draw, world, cave, detail, procedural-world, material, particle,
  deformation, and wind resources;
- SDF residency and raymarch acceleration;
- world LOD and runtime caches;
- dynamic texture registries;
- pipeline and backend allocators;
- native-window surface registry;
- editor picking and extensive debug/inspection state;
- Runenwerk startup and metrics state;
- Runenwerk schedule sets and a direct ordering dependency on `UiRuntimeSet`.

This is Runenwerk integration composition, not the future renderer core.

The current graph module also exports `validation_builtin_ui`, proving that the
generic graph still contains a product/producer-specific validation path.

## Surface ownership decision

Surface ownership is split explicitly.

### Runenwerk host owns

- native window creation and destruction;
- event loop;
- `NativeWindowId`;
- monitor, DPI, resize, visibility, and window policy;
- selecting which native window maps to which render surface;
- application reaction to unrecoverable presentation failure.

### runenrender_core owns

- renderer-local opaque `RenderSurfaceId` or reviewed replacement;
- generic target size and presentation intent values;
- surface-independent target/view descriptors;
- backend-neutral surface lifecycle reports.

### runenrender_wgpu owns

- `wgpu::Surface`;
- `SurfaceConfiguration` construction;
- format, alpha, usage, present-mode, and frame-latency realization;
- configure/acquire/present;
- recoverable surface errors and backend retirement.

### Runenwerk surface adapter owns

- mapping `NativeWindowId` to renderer-local surface identity;
- providing the native handle required to create a WGPU surface;
- resize/update commands;
- retaining no WGPU resource in generic engine state.

The current type that combines `NativeWindowId`, ECS `Resource` derives,
renderer-local identity, lifecycle, WGPU helpers, and registry diagnostics must be
split. `NativeWindowId` must not appear in RunenRender public contracts.

## ECS independence decision

No RunenRender core or WGPU type derives or requires `ecs::Component` or
`ecs::Resource`.

Runenwerk may wrap renderer values in ECS resources or components inside its
adapter package/module. The renderer accepts ordinary owned/borrowed Rust values
and explicit contexts.

The internal decomposition must make it possible to test graph planning and WGPU
backend construction without creating a Runenwerk `World`.

## Producer seam

Renderer producers submit generic rendering work. They do not register semantic
feature kinds into renderer core.

The exact API follows source/control-flow review, but the contract must express:

- stable producer identity scoped to the host/integration;
- frame and target identity;
- resource declarations or uploads;
- draw/dispatch/copy/pass contributions;
- order/dependency requirements;
- capability requirements;
- diagnostics and provenance;
- deterministic replacement/removal behavior.

The producer seam must not contain:

```text
UI routes or widgets
SDF field semantics
ECS entities
scene nodes
material graph nodes
editor tools
product-specific feature enums
```

Runenwerk adapters translate those domains into generic contributions.

## Graph decision

`runenrender_core` owns generic graph/planning only when a module can be tested
without WGPU and Runenwerk.

Generic graph responsibilities include:

- pass/resource graph formation;
- dependency validation;
- access conflict validation;
- resource lifetime analysis;
- merge/planning rules;
- backend capability requirements;
- deterministic execution-plan output;
- generic diagnostics.

Product-specific built-in validation such as `validation_builtin_ui` is removed
from core. A producer/adaptor validates its own semantic contract before
submitting generic work.

## Shader and pipeline boundary

RunenRender may own backend-generic shader-module and pipeline realization
contracts plus WGPU implementation.

Runenwerk retains material-authoring semantics. The target flow is:

```text
Runenwerk MaterialIr / material graph
    -> Runenwerk material render adapter
    -> generic shader/pipeline/resource requests
    -> RunenRender realization
```

`runenrender_core` must not depend on `material_graph`.

Shader source, reflection, binding layouts, specialization, caching, diagnostics,
and hot-reload ownership must be classified individually. Hot-reload filesystem
or application policy remains Runenwerk unless it is proven backend-library
behavior.

## SDF boundary

RunenRender does not depend on RunenSDF and does not evaluate SDF semantics.

The target flow is:

```text
RunenSDF field/query semantics
    -> Runenwerk SDF render adapter
    -> buffers, textures, shader parameters, passes, and capability requests
    -> RunenRender
```

World LOD, sparse residency, chunk streaming, raymarch acceleration policy, and
SDF product selection remain Runenwerk-owned unless a later renderer-specific
review proves an algorithm is generic GPU residency infrastructure.

## UI boundary

RunenUI is outside this program. RunenRender exposes no RunenUI dependency or UI
semantic API.

A future Runenwerk adapter may convert accepted RunenUI paint output into generic
render contributions. Renderer core must not know buttons, focus, routes,
semantics, UI runtime identities, or font-atlas policy specific to one UI system.

Current names and paths such as `PreparedUiFrameResource`, `UiFontAtlasResource`,
`prepare_ui_feature_resource_system`, `UiRuntimeSet`, and
`validation_builtin_ui` are classified as Runenwerk integration or legacy UI
feature ownership, not RunenRender core.

## Resource lifetime decision

RunenRender owns GPU/resource lifetime only for renderer resources it creates.
Runenwerk owns product/source lifetime.

Contracts must define:

- logical resource identity;
- generation and stale-handle behavior;
- create/update/remove requests;
- CPU source ownership during upload;
- in-flight frame retention;
- aliasing and transient lifetime planning;
- cache invalidation;
- device-loss reconstruction responsibility;
- shutdown order;
- memory budget diagnostics.

No renderer resource may rely on ECS lifetime implicitly.

## Frame and synchronization decision

`runenrender_core` owns deterministic frame-plan formation. `runenrender_wgpu`
owns backend execution and synchronization.

Runenwerk supplies:

- logical frame request;
- target selection;
- producer contributions;
- timing values needed by render algorithms;
- product policy.

The renderer returns structured preparation, submission, presentation, timing,
and failure reports.

Do not expose Runenwerk fixed-time or schedule types in renderer public APIs.

## Device-loss and error policy

Public renderer errors must be structured and classify:

- unsupported capability;
- invalid graph or resource declaration;
- missing/stale resource;
- allocation failure;
- shader or pipeline failure;
- surface lost/outdated/timeout/out-of-memory;
- device lost;
- submission/presentation failure;
- shutdown/terminal state.

RunenRender reports facts and recovery requirements. Runenwerk decides product
policy such as retry, reinitialization, window closure, fallback, or shutdown.

## Diagnostics policy

RunenRender diagnostics use `runenrender.*` codes and contain backend-neutral
identity/provenance where possible.

Runenwerk inspection UI and debug overlays are consumers. They remain outside the
renderer framework.

Wall-clock timing is evidence, not deterministic behavioral authority.

## Macro decision

`engine_render_macros` is not moved automatically.

Each derive must be reviewed for:

- dependency on Runenwerk paths;
- dependency on WGPU/bytemuck layout;
- public usefulness;
- safety invariants;
- compile diagnostics;
- external consumer proof.

If retained, macros move only after the core type/layout contract is accepted.
Otherwise replace them with ordinary traits/builders or keep integration-specific
macros in Runenwerk.

## Internal target layout

Before external extraction, create equivalent internal package/module boundaries
inside Runenwerk:

```text
renderer neutral core
renderer WGPU backend
Runenwerk render host/plugin adapter
Runenwerk ECS/scene adapter
Runenwerk material adapter
Runenwerk SDF/world adapter
Runenwerk editor/debug adapter
Runenwerk legacy/future UI adapter
```

Names follow the complete source inventory. Do not manufacture crates solely to
match this diagram.

## Anti-cheating proof

Runenwerk must consume the internally separated renderer through the same public
boundary intended for external consumers.

Forbidden shortcuts:

- private access from Runenwerk adapters into renderer internals;
- ECS derives inside renderer packages;
- Runenwerk types in renderer signatures;
- product-specific graph variants;
- direct material/SDF/UI imports in renderer core;
- tests that instantiate the whole engine when testing neutral graph behavior.

## Independent conformance

Before extraction, prove:

- graph formation and validation without Runenwerk;
- deterministic plan output;
- resource lifetime and stale identity behavior;
- producer replacement/removal;
- headless/offscreen backend operation where supported;
- WGPU device/resource/pipeline execution;
- surface configuration and recoverable error handling;
- device-loss reconstruction contract;
- multiple surfaces/targets;
- external generic producer;
- stable and MSRV validation;
- representative CPU planning and GPU runtime benchmarks.

GPU evidence must state adapter/device/platform and distinguish deterministic
contract tests from environment-dependent runtime evidence.

## Decomposition phases

### RENDER-001 — Complete semantic inventory

Read every render module, shader, macro, example, benchmark, app integration,
and test. Trace frame preparation, submission, presentation, resource creation,
shader/pipeline compilation, surface lifecycle, and failure control flow.

Classify every item as:

```text
CORE
WGPU_BACKEND
RUNENWERK_HOST
RUNENWERK_ECS_SCENE
RUNENWERK_MATERIAL
RUNENWERK_SDF_WORLD
RUNENWERK_EDITOR_DEBUG
RUNENWERK_UI
REDESIGN
DELETE
```

### RENDER-002 — Decision closure

Finalize public contracts for graph, producer submissions, resources, surfaces,
frames, synchronization, errors, device loss, diagnostics, macros, and threading.

### RENDER-003 — Neutral-core separation inside Runenwerk

- remove ECS and product types from neutral contracts;
- remove product-specific graph validation;
- isolate graph/planning and generic resource contracts;
- add independent tests.

### RENDER-004 — WGPU/backend separation inside Runenwerk

- isolate WGPU realization;
- split host/native-window mapping;
- separate surface creation/config/acquire/present;
- define device-loss reconstruction;
- add backend conformance.

### RENDER-005 — Adapter migration and internal proof

Migrate scene, material, SDF, editor, UI, and plugin composition into Runenwerk
adapters. Runenwerk uses only the intended external boundary.

### RENDER-006 — Repository creation and transfer

Create RunenRender, establish governance/provenance, transfer corrected packages,
and validate independently.

### RENDER-007 — Runenwerk cutover

Pin exact revisions, remove original renderer implementation, migrate all
applications and tests, regenerate lockfile, and run complete CPU/GPU evidence.

### RENDER-008 — Closeout

Record compatibility, provenance, performance, device/platform evidence, deleted
paths, and final ownership.

## Stop conditions

Stop extraction if:

- neutral packages still depend on ECS or Runenwerk;
- `NativeWindowId` appears in renderer public contracts;
- product-specific feature/validation variants remain in core;
- material, SDF, UI, scene, or editor semantics remain in renderer packages;
- resource/device-loss ownership is unresolved;
- Runenwerk cannot consume the internal separation through public seams;
- GPU validation is unavailable for changed backend behavior;
- a source mirror or compatibility renderer would survive the final merge.

## Definition of done

RunenRender is complete only when core and WGPU packages validate independently,
Runenwerk consumes exact revisions through adapters, no Runenwerk or domain
semantic dependency remains in the renderer, all original implementation paths
are removed, and full headless plus GPU integration evidence is green.
