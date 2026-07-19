---
title: RunenRender Extraction Investigation
description: Connector-backed renderer module, graph, WGPU, surface, shader, producer, adapter, diagnostics, and extraction-readiness evidence.
status: active
owner: render
layer: investigation
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/specs/pt-runenrender-r1-identities-errors.ron
---

# RunenRender Extraction Investigation

## Question

Which current renderer responsibilities are reusable neutral renderer contracts,
which belong to a WGPU backend, which remain Runenwerk adapters/product policy,
and what must be repaired before external extraction?

## Verdict

```text
EXTRACTION CANDIDATE                 yes
MOVE engine/src/plugins/render      forbidden
REQUIRED PACKAGES                    runenrender_core, runenrender_wgpu
OPTIONAL PACKAGE                     runenrender_macros after ABI proof
CONNECTOR SEMANTIC INVESTIGATION     substantial, not workflow-complete
COMPLETE FILE/SHADER/CONSUMER MAP    pending local verification
FIRST EXECUTABLE REPAIR              R1 neutral identities and structured errors
EXTERNAL SOURCE MOVEMENT             forbidden
```

The current renderer contains credible graph, resource, pipeline, GPU parameter,
WGPU, surface, shader, and inspection foundations. It is nevertheless one engine
plugin whose authoring and execution paths directly depend on ECS, Runenwerk
lifecycle/time/window state, scene/world/material/SDF/UI/editor features,
filesystem hot reload, diagnostics presentation, and startup policy.

The correct approach is an internal strangler decomposition followed by an
anti-cheating proof, not a directory move.

## Baseline and evidence

Repository: `Crystonix/Runenwerk`

Reviewed published main:

```text
c078bd8609dc407d68269e86a1472c9234932213
```

Evidence:

```text
E2 GitHub package/commit/PR metadata
E3 connector-backed source, manifest, module, and control-flow inspection
```

The connector cannot reliably list every repository file, run Cargo, validate
WGSL, initialize WGPU, execute window/GPU examples, or run benchmarks. Local
verification remains mandatory before implementation activation.

## Current package boundary

Rendering lives inside `engine`, not an independent crate.

The inspected render root exposes:

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

`engine_render_macros` is a separate proc-macro package whose generated paths
target `engine::plugins::render`.

## Current ownership aggregation

The current plugin combines:

1. backend-neutral graph and resource planning;
2. WGPU instance/adapter/device/queue/resources/pipelines/execution;
3. native-window and surface integration;
4. Runenwerk plugin scheduling and frame lifecycle;
5. ECS resource/state extraction;
6. scene/world preparation;
7. material graph compilation and material asset loading;
8. SDF/world residency and raymarch product policy;
9. UI feature preparation and built-in graph validation;
10. editor picking and product features;
11. shader filesystem discovery, polling, and hot reload;
12. diagnostics, capture, artifact export, startup readiness, and frame pacing.

This is Runenwerk integration composition, not one reusable renderer package.

## API and graph findings

Current IDs use Runenwerk `id_macros`:

```text
RenderFlowId
RenderPassId
RenderResourceId
RenderFeatureId
RenderFrameProducerId
```

Current graph/authoring APIs include or depend on:

- ECS resource/state projections;
- host `TypeId` requirements and callbacks;
- fixed-time and catch-up policy;
- product view categories;
- built-in UI composite execution and validation;
- material-scene shader selection;
- product feature IDs and fallback policy;
- `anyhow` and mixed panic/skip behavior.

Target neutral core cannot retain these host/product semantics.

## Frame and producer findings

The current frame model combines potentially neutral frame/view/resource facts
with product selection, main/offscreen product semantics, UI contributions, and
ECS-backed registries.

Target direction:

```text
Runenwerk adapters
  resolve ECS/application/domain state
  build explicit prepared inputs and generic contributions

runenrender_core
  validates and plans explicit work

runenrender_wgpu
  realizes resources/pipelines and executes the plan
```

Renderer plans never reach back into a host world.

## Backend findings

Current WGPU initialization requires a Winit `Arc<Window>`, selecting device and
surface together. This prevents independent headless/offscreen initialization and
mixes host lifetime with backend ownership.

Target split:

```text
Runenwerk host
  Winit windows, event loop, monitor/DPI/resize/visibility policy

runenrender_wgpu
  headless device creation
  optional WGPU surface creation from admitted handles
  configuration/acquire/present/device-loss facts
```

The complete design must prove raw handle lifetime, thread affinity, drop order,
surface retirement, multi-window operation, and device-loss reconstruction.

## Graph classification

The graph is the strongest core candidate but currently contains:

- `BuiltinUiComposite` and UI feature IDs;
- host state projections and `TypeId` callbacks;
- fixed-step regions;
- Runenwerk product view categories;
- product feature variants.

Core graph target:

```text
compute
graphics
fullscreen-raster convenience
copy
present
```

Semantic UI/material/SDF/world/editor validation remains in Runenwerk producers.
Core validates only generic graph/resource/capability contracts.

## Shader and pipeline findings

Current shader authority combines:

- generic identity/source revision;
- filesystem roots and path normalization;
- polling/throttling/force reload;
- last-known-good/application policy;
- Naga/WGSL/WGPU realization;
- tracing and temporary-file tests.

Target split:

```text
runenrender_core
  backend-neutral shader/interface identity and intent where proven

runenrender_wgpu
  WGSL validation, module creation, pipeline realization, backend errors

Runenwerk
  filesystem discovery/watch/reload, asset paths, material translation, fallback policy
```

## GPU layout and macro findings

Current `GpuUniform` and `GpuStorage` derives generate raw layout types, bytemuck
traits, parameter conversion, and paths into `engine::plugins::render`.

A macro package is not mandatory merely because proc macros require a separate
crate. First decide whether the derives remain the correct public API.

If retained:

- WGSL/WGPU layout authority belongs with the WGPU ABI owner, not neutral core;
- generated paths must support dependency renaming;
- padding/alignment/arrays/matrices/nested structures need byte-level proof;
- bytemuck safety requires generated representation invariants;
- external compile-pass/fail tests are mandatory.

## Resource and residency findings

Current generic-looking resource/handle/lifetime infrastructure is mixed with
surface defaults, host type state, material/world/SDF residency, dynamic texture
policy, and saturating arithmetic.

Potential neutral ownership includes validated resource descriptors, generations,
stale-handle behavior, transient lifetime planning, and generic budgets.

Runenwerk retains source reconstruction and domain-specific residency policy.

## Domain adapter findings

The following remain Runenwerk-owned by default:

```text
scene/world extraction
material IR and material asset translation
SDF representation and world residency policy
UI paint/font/atlas conversion
editor picking and editor workflows
procedural camera/population policy
product feature selection and fallback
```

RunenRender accepts generic prepared work only.

## RunenUI relationship

RunenUI and RunenRender are independent peers.

RunenUI owns UI semantics, hit testing, and renderer-neutral paint output. A
future Runenwerk adapter translates accepted paint output into generic
RunenRender contributions. RunenUI may retain standalone backends. Neither
framework depends on the other by default.

## Diagnostics findings

Potentially neutral diagnostics include graph dumps, pass/resource provenance,
backend capabilities, surface/device outcomes, and timing facts.

Runenwerk retains material/SDF/world/editor/product inspection, artifact paths,
overlay presentation, capture export policy, startup readiness, and frame pacing.
Wall-clock/GPU evidence is not deterministic planning authority.

## Durable target decisions

Fixed:

- required package candidates: `runenrender_core` and `runenrender_wgpu`;
- optional `runenrender_macros` only after ABI proof;
- no ECS, Runenwerk, SDF, UI, scene, material-authoring, Winit, or WGPU dependency
  in neutral core;
- no Winit, ECS, Runenwerk, or domain semantics in WGPU package;
- headless WGPU initialization before optional surface attachment;
- no product-specific graph/pass/feature variants;
- explicit prepared inputs and generic producer contributions;
- Runenwerk owns native windows, domain adapters, lifecycle, hot reload, and
  product recovery;
- external extraction blocked until internal public-boundary anti-cheating proof.

Provisional until local evidence:

- exact module/file dispositions;
- exact neutral graph/resource API;
- exact macro retention and package;
- exact shader/interface boundary;
- exact surface target abstraction and thread/drop contract;
- exact diagnostics/capture set;
- exact internal crate/module sequence.

## Repair program

```text
R1 neutral identities, structured errors, and dependency guards
R2 neutral graph and resource descriptors
R3 explicit prepared frame inputs and generic producers
R4 GPU parameter and optional macro ABI conformance
R5 shader descriptor and hot-reload separation
R6 headless WGPU device/resource/pipeline executor
R7 generic surfaces and device-loss contract
R8 generic diagnostics/capture/provenance split
R9 Runenwerk domain/runtime adapter migration
R10 internal public-boundary and performance conformance
```

Only R1 receives a concrete specification now.

## Mandatory local gate

Before activating R1, run:

```text
cargo metadata --format-version 1 --locked
cargo tree -p engine --edges normal,build,dev
find engine/src/plugins/render engine_render_macros assets/shaders engine/examples engine/tests engine/benches -type f | sort
rg -n 'ecs::|crate::runtime|crate::plugins::scene|material_graph|world_sdf|ui_|Ui|NativeWindowId|winit::|wgpu::|anyhow|panic!|unwrap\(|expect\(' engine/src/plugins/render engine_render_macros
rg -n 'plugins::render|RenderFlow|RenderPlugin|PreparedRenderFrame|Gfx' apps domain net adapters engine/tests engine/examples
rg -n 'BuiltinUi|UI_RENDER_FEATURE_ID|TypeId|FixedStep|MainSurface|OffscreenProduct|with_state|uniform_from_state|dispatch_from_state' engine/src/plugins/render
cargo test -p engine --lib --locked
cargo test -p engine --tests --locked
cargo clippy -p engine --all-targets --locked -- -D warnings
python tools/docs/validate_docs.py
pnpm --dir docs-site build
git diff --check
git status --short --branch
```

Also identify shader validation, headless WGPU, surface/window, device-loss, GPU
example, and renderer benchmark commands. Report adapter/device/platform for GPU
evidence.

## Gate result

```text
repository ownership direction     established
connector semantic investigation   substantial, not workflow-complete
local complete inventory           pending
complete design gate               pending
implementation authorization       blocked
external extraction                forbidden
```

## Next safe action

After local verification and owner review, activate exactly R1. Do not implement
R2–R10, create RunenRender, or move renderer source externally during R1.