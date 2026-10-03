---
title: Runenwerk Render Runtime Decomposition Investigation
description: Exact-current investigation of the retained Runenwerk render integration/runtime after the standalone RunenRender cutover, including source census, ownership, lifecycle, structural debt, and bounded refactor disposition.
status: active
owner: engine
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
related_docs:
  - ./2026-10-03-runenwerk-wider-render-integration-source-census.md
  - ./2026-10-03-runenwerk-render-runtime-dependency-census.md
  - ./2026-10-03-runenwerk-render-runtime-source-census.md
  - ../../engine/reference/plugins/render/architecture.md
  - ../../engine/reference/plugins/render/render-target-architecture.md
  - ../../engine/roadmaps/render-final-architecture-migration.md
  - ../../design/accepted/runenrender-decomposition-design.md
  - ../../design/accepted/feature-owned-render-contributions-design.md
  - ../../design/accepted/render-execution-graph-compiler-maturity-design.md
  - ../../design/accepted/render-fragment-data-driven-maturity-design.md
  - ../../design/accepted/render-production-readiness-and-inspection-design.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
---

# Runenwerk Render Runtime Decomposition Investigation

## Purpose

Issue #1138 investigates the retained Runenwerk render integration/runtime after the
standalone RunenRender authority cutover.

The investigation answers a narrower question than "should the renderer be rewritten?":

> Does the current physical decomposition of `engine::plugins::render` still match the
> accepted Runenwerk / RunenRender / RunenGPU ownership model, and what bounded structural
> corrections are justified by current source rather than by directory aesthetics?

This report is point-in-time investigation evidence. Current source and tests remain current
behavior authority. Accepted ADRs/designs remain durable architecture authority. GitHub issues
own activation and sequencing.

## Exact evidence baseline

The investigation re-resolved the live default branch before drawing conclusions:

```text
RUNENWERK_MAIN=8d0bad6c7e4c24f2579bd89c3f797946af85dba3
RUNEN_RENDER_PIN=b3076bb30ab3c43a180f656b4902c79c482f11ae
RUNEN_GPU_PIN=789b430fdefeda89bfe59de86d548618b8f8ab9a
RUNEN_SHADER_VIA_RUNEN_RENDER=406f8165da92caa2d296b3a2f774ba534279870d
```

The Runenwerk baseline is the accepted RX cutover commit from PR #1137. The standalone
RunenRender implementation and conformance authority is therefore external at the exact pin above.
Runenwerk retains product/runtime integration.

The investigation read, among other current owners:

```text
AGENTS.md
ARCHITECTURE.md
Cargo.toml
engine/Cargo.toml
engine/src/plugins/render/**
engine/tests/render_cutoff_guard.rs
apps/runenwerk_render_lab/**
apps/runenwerk_arena/**
apps/runenwerk_editor/** direct render consumers
apps/runenwerk_draw/** direct render consumers
engine/src/plugins/ui/** render publication paths
engine/src/plugins/world/** render adapters

docs-site/src/content/docs/workspace/documentation-structure.md
docs-site/src/content/docs/guidelines/dependency-rules.md
docs-site/src/content/docs/guidelines/programming-principles.md
docs-site/src/content/docs/engine/reference/plugins/render/**
docs-site/src/content/docs/engine/roadmaps/render-final-architecture-migration.md
docs-site/src/content/docs/engine/roadmaps/fully-featured-renderer-roadmap.md
docs-site/src/content/docs/design/accepted/runenrender-decomposition-design.md
docs-site/src/content/docs/design/accepted/feature-owned-render-contributions-design.md
docs-site/src/content/docs/design/accepted/render-execution-graph-compiler-maturity-design.md
docs-site/src/content/docs/design/accepted/render-fragment-data-driven-maturity-design.md
docs-site/src/content/docs/design/accepted/render-production-readiness-and-inspection-design.md
docs-site/src/content/docs/adr/accepted/0015-separate-gpu-execution-from-rendering.md
```

Search was used for discovery and consumer finding. Ownership conclusions were checked against the
owning source/designs. The central source census was not inferred from search results: it was
resolved from the exact recursive Git tree for the accepted commit.

## Executive disposition

The investigation does **not** justify another renderer extraction, a rewrite, or a replacement of
the accepted prepare/compile/execute model.

The durable authority model is healthy:

```text
Runenwerk
  host, ECS/domain/product integration, prepared-frame orchestration,
  product presentation policy and renderer-facing adapters
        |
        v
RunenRender
  reusable renderer-semantic scene/request/admission/result authority
        |
        v
RunenGPU
  generic physical GPU resources/work/admission/realization/submission
```

The main defect is more specific:

> the retained Runenwerk integration/runtime has accumulated obsolete pre-cutover residue,
> compatibility-only public surface, broad re-exports, and several implementation responsibilities
> inside large convergence modules even though its semantic authority boundaries are mostly sound.

The correct response is a **sequenced internal decomposition and cleanup**, not an ontology rewrite.

The first delivery should remove proven obsolete compatibility/backend residue and correct
post-cutover documentation truth. Larger physical decomposition must serialize with active
RunenUI #1110 and continue the already accepted feature-contribution migration instead of replacing
it.

# 1. Exact central source census

At the accepted Runenwerk revision, the exact recursive Git tree contains:

```text
engine/src/plugins/render/**
184 Rust files
2,442,976 Rust source bytes
```

A separate module-content census of those files gives approximately **68k physical Rust lines**.
The line figure is descriptive, not a quality threshold; the file count and byte count above are
the exact Git-tree census.

Top-level concentration from the exact tree:

| Group | Rust files | Source bytes |
| --- | ---: | ---: |
| `renderer` | 27 | 588,626 |
| `inspect` | 33 | 404,334 |
| `runtime` | 7 | 225,685 |
| `graph` | 12 | 217,188 |
| `frame` | 10 | 195,955 |
| `features` | 18 | 133,279 |
| `adapters` | 5 | 133,184 |
| `api` | 8 | 125,266 |
| `material_compiler` | 12 | 106,986 |
| `gpu_primitives` | 6 | 50,217 |
| `procedural` | 8 | 47,552 |
| `composition` | 6 | 46,225 |
| `residency` | 3 | 35,961 |
| `shader` | 5 | 33,274 |
| render-root files | 7 | 32,220 |
| `backend` | 7 | 29,982 |
| `resource` | 4 | 16,542 |
| `pipelines` | 3 | 14,024 |
| `params` | 3 | 6,476 |

The size is therefore not caused by one accidental giant file. It is a substantial engine-runtime
subsystem with several independently meaningful stages.

Important large implementation units include the RenderFlow API, GPU-work adaptation, graph
validation/preflight, prepared contribution/frame formation, runtime prepare/submit/diagnostics,
renderer preparation, and render-flow execution. Their size is supporting evidence only; the
findings below depend on ownership and dependency behavior.

## 1.1 Wider rendering footprint

The linked
[wider render integration source census](./2026-10-03-runenwerk-wider-render-integration-source-census.md)
records the exact accepted-revision source inventory outside the central Render tree using a
reproducible inclusion rule: all Render Lab Rust plus path-scoped direct Runenwerk Render /
RunenRender / RunenGPU consumers in the required integration, app, example, and test areas.

At the reviewed revision:

```text
central engine/src/plugins/render/**        184 Rust files / 2,442,976 bytes
wider Runenwerk render integration          124 Rust files / 2,720,874 bytes
---------------------------------------------------------------------------
combined concrete Runenwerk render scope    308 Rust files / 5,163,850 bytes
```

The wider 124-file inventory breaks down as:

| Area | Rust files |
| --- | ---: |
| Render Lab — all Rust | 9 |
| Arena — direct Render integration | 5 |
| Editor — direct Render integration | 34 |
| Draw — direct Render integration | 4 |
| Engine UI — direct Render integration | 4 |
| Engine World — direct Render integration | 5 |
| Other Engine host/scene/debug render-facing | 7 |
| Engine examples/proofs — direct Render/RunenGPU | 21 |
| Engine tests — direct Render/RunenRender/RunenGPU | 35 |

The wider inventory deliberately does not count whole app/test/example directories. Neighboring
files without a direct render/framework edge are excluded. Every retained path is resolved to an
immutable blob SHA in the appendix.

Standalone RunenRender and standalone RunenGPU remain separate repositories and are not included in
the 308-file Runenwerk count.

# 2. Current lifecycle and data-flow map

The accepted runtime shape is real in current source:

```text
app/domain/product/ECS state
        |
        | RenderPrepare
        v
flow registry sync
feature registry sync
shader-product polling/revision capture
surface snapshot
prepared view/invocation formation
feature contribution collection
dynamic target/upload snapshot
product-selection snapshot
        |
        v
PreparedRenderFrame
        |
        | immutable frame-scoped boundary
        v
RenderSubmit
        |
        +-- revision-check compiled flow registry
        +-- consume shader registry revision/product data
        +-- consume renderer/debug/inspection policy
        +-- consume deterministic RunenRender contributions
        |
        v
Gfx / Renderer execution runtime
        |
        +-- RenderFlow resource/pipeline/work realization
        +-- RunenRender semantic execution where requested
        +-- RunenGPU work/surface/submission/readback
        |
        v
derived timing/capture/provenance/readiness evidence
```

`engine/tests/render_cutoff_guard.rs` explicitly guards the cut. Submit must not restore old
live-extraction behavior such as `RenderFrameDataRegistry`, submit-time uniform projection,
viewport-binding discovery, shader polling, or the retired `PreparedRenderWorkPlan` sidecar.

The submit path still reads runtime resources. That is not itself a boundary violation. In
particular, current submit checks prepared flow-registry revision against the current registry
before consuming compiled plans. The relevant architectural question is whether submit re-discovers
domain/product truth after the cut. Current evidence does not show that regression.

# 3. Current authority map

| Concern | Current semantic owner | Runenwerk render role |
| --- | --- | --- |
| renderer-semantic scene/request/result | RunenRender | project/adapt and consume public contracts |
| generic GPU resources/work/program admission/execution | RunenGPU | translate validated render execution into public RunenGPU contracts |
| source/world/material/UI/gameplay truth | owning domain/framework/app | consume prepared projections only |
| frame extraction and render scheduling | Runenwerk Engine Render | owner |
| RenderFlow authoring/compiler/preflight | Runenwerk Engine Render | owner |
| native window/render-surface correlation | Runenwerk host/runtime + Render integration | owner of correlation/policy, not physical GPU semantics |
| render feature contribution packet/validation/fallback gate | Runenwerk Engine Render | owner |
| product fallback legality/freshness/rebuild | owning producer/product domain | not Render |
| renderer inspection/capture/readiness evidence | Runenwerk Engine Render | derived evidence only |
| shader filesystem watching/reload/product last-good policy | Runenwerk | product policy only |
| reusable shader-source/toolchain/canonical-artifact semantics | RunenShader family authority | separate cross-repository boundary; not reassigned here |

# 3.1 Exact source inventory and source-level dependency census

The immutable source inventory is recorded in the linked
[exact source census](./2026-10-03-runenwerk-render-runtime-source-census.md).
It lists all 184 Rust files with byte size and immutable blob SHA at the reviewed revision.

The linked [lexical dependency census](./2026-10-03-runenwerk-render-runtime-dependency-census.md)
independently covers the same 184/184 files with zero omissions or duplicates and records, from
exact immutable file bytes, each file's blob SHA, explicit cross-top-level Render dependencies,
direct framework dependencies, direct peer-plugin dependencies, and lexical public-item count.
It explicitly distinguishes this exact lexical reference graph from a compiler-resolved Rust type
graph.

Summarizing that per-file census at the top-level owner boundary gives the following materially
relevant edges:

| Consuming group | Explicit dependency groups observed | Architectural interpretation |
| --- | --- | --- |
| `api` | `graph`, `renderer`, `gpu_primitives`, `procedural` | `api -> renderer` is compatibility debt; compiler/procedural helper dependencies are authoring/compiler integration. |
| `composition` | `api`, `graph` | Accepted fragment-to-RenderFlow/compiler seam. |
| `features` | `api`, `frame`, `backend` | Feature packet/host integration; current UI/backend edge is active migration pressure. |
| `frame` | `api`, `features`, `backend` | Prepared-frame boundary plus legacy feature strangler and surface identity. |
| `graph` | `api`, `composition`, `features`, `resource` | Accepted compiler ownership. |
| `inspect` | `graph`, `features`, `renderer`, `pipelines`, `shader` | Derived inspection/evidence projections over execution state. |
| `renderer` | `api`, `graph`, `frame`, `features`, `inspect`, `pipelines`, `shader`, `backend`, `adapters` | Main execution convergence point; later decomposition candidate. |
| `runtime` | `frame`, `renderer`, `inspect`, `backend`, `pipelines` | Prepare/submit orchestration. |
| `procedural` | `api`, `gpu_primitives` | Authoring/lowering path. |
| `backend` | `graph`, `pipelines` | Residual host/capability wrapper; partly obsolete after RunenGPU. |

This summary is backed by the complete per-file lexical appendix. Same-owner relative references
are intentionally omitted from the top-level summary because they do not change owner direction;
the appendix remains the exact file-by-file evidence surface.

## 3.2 Standalone framework dependency classification

Direct dependency census inside `engine/src/plugins/render/**` at the reviewed revision:

| Dependency | Files | Disposition |
| --- | ---: | --- |
| `runen_gpu::` | 68 | Expected physical/render-execution vocabulary and integration. |
| `runen_render::` | 5 | Narrow semantic admission/execution/prepared-payload integration. |
| `runen_shader::` | 0 | No direct central Render dependency. |
| `wgpu::` | 0 | No retained raw WGPU authority in central Render. |
| `naga::` | 2 | Local shader/program validation in material compiler and GPU-primitive planning; separate shader-authority seam, not GPU-device ownership. |

The five direct RunenRender files classify as:

| File | Role | Disposition |
| --- | --- | --- |
| `frame/packet.rs` | Carries prepared RunenRender scene/request/input/availability DTOs across the frame boundary. | Legitimate semantic projection. |
| `renderer/mod.rs` | Owns RunenRender execution state/evidence. | Legitimate runtime integration. |
| `renderer/render_flow/execute.rs` | Calls `admit_render`, prepares execution, binds output destinations, records submission lifecycle. | Legitimate semantic execution integration. |
| `renderer/setup.rs` | Progresses execution lifecycle and exposes derived temporal evidence. | Legitimate derived execution/evidence integration. |
| `runtime/frame_submit.rs` | Direct RunenRender imports occur in tests only. | No production authority edge. |

The 68 RunenGPU users fall into four owner-correct classes:

1. **authoring/compiler vocabulary** — `api`, `graph`, `frame`, `composition`,
   `resource`, `pipelines`, and `procedural` use backend-neutral RunenGPU ids, formats,
   usages, binding keys, resource references, or work descriptors;
2. **explicit translation** — `adapters` maps render compiler/execution facts into RunenGPU work,
   resources, and capabilities;
3. **physical realization** — `renderer`, current `backend`, `gpu_primitives`, and one
   `runtime` path own Runenwerk-side orchestration over public RunenGPU execution contracts;
4. **derived evidence** — `inspect` projects RunenGPU-backed provenance/resource/timing facts.

No inspected call establishes duplicate generic device/resource/submission authority in Runenwerk.

The dependency appendix also records the separate owner-crate concentration check: `runen_ecs`
is the broad integration substrate (27 central files), while domain-specific direct coupling is
concentrated in `product` (9), `world_sdf` (3), `world_ops` (1), `runen_spatial` (6),
`ui_render_data` (11), `ui_text` (2), `ui_math` (1), and `material_graph` (9). There is no
direct central Render dependency on `runen_input` or `runen_net`. These edges are classified by
authority in the module matrix below rather than conflated with RunenRender/RunenGPU framework
ownership.

The dependency appendix also records the direct semantic-owner overlay that sits beside the
RunenRender/RunenGPU execution boundary. Positive reviewed-reference sets include:

| Owner | Central Render files | Meaning |
| --- | ---: | --- |
| RunenECS | 27 | ECS substrate for resources/components/scheduling, not renderer semantics. |
| RunenSpatial | 6 | chunk/spatial identity for prepared world/residency integration. |
| `product` | 9 | prepared selection/residency/authority/freshness/query facts; product truth stays external. |
| `material_graph` | 9 | material compiler input authority; generic Render does not own material truth. |
| `ui_render_data` | 11 | renderer-neutral UI publication payloads. |
| `ui_text` / `ui_math` | 2 / 1 | UI text/proof realization contracts. |
| `world_sdf` / `world_ops` | 3 / 1 | World-owned payload/identity/generation inputs consumed by derived render integration. |

This reinforces the vertical-feature finding: direct dependency is legitimate where Render consumes
a prepared/owner-defined contract; it becomes a defect only when Render recreates the owner's
semantic policy or source truth.

## 3.3 Top-level owner, lifecycle, authority, and consumer matrix

The exact file/dependency appendices answer **what exists and what it references**. The matrix below
answers **why each retained top-level package exists** and whether it owns source truth, prepared
state, derived execution state, product policy, or evidence.

| Area | Module visibility / public pressure | Primary change reason | Lifecycle | Authority class | Maintained consumer class | Active scope pressure | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| root integration (`app_ext`, `plugin`, `gpu_context_policy`, `native_host`, `readiness`, `texture_upload`) | mixed private/`pub(crate)` modules with selected public re-exports | app/plugin composition, GPU-context policy, native host attachment, readiness, upload integration | startup + prepare/submit + host lifecycle | Runenwerk integration/product policy; derived runtime state | apps, engine runtime, tests | #1110 may overlap plugin/host-facing publication | Keep composition root; split only proven residual host debt later. |
| `api` | public module and broad root re-export; high app/example pressure | declarative RenderFlow/resource/pass authoring | authoring + compile input | Runenwerk render authoring contract | Arena, Draw, Editor, Render Lab, examples/tests | R1 compatibility cut | Keep; remove downward compatibility dependency. |
| `composition` | public module; advanced/fragment consumers | flow/fragment registry, validation, merge/hot-reload integration | authoring promotion + prepare sync | derived accepted render descriptions; no product truth | render runtime, fragment examples/tests | none material found | Keep accepted fragment/compiler seam. |
| `graph` | public module; advanced/bench/test consumers | static validation, planning, execution-plan compilation, prepared preflight | compile + preflight | derived compiler authority over render execution validity | runtime, renderer, Render Lab/bench/inspection | none material found | Keep; static and prepared validation remain distinct. |
| `frame` | public module; strong app integration pressure | immutable prepared frame, invocations, targets, selections, contribution boundary | RenderPrepare -> RenderSubmit cutoff | prepared projection, not source truth | Arena, Draw, Editor, Render Lab, runtime/tests | feature strangler crosses it | Keep public prepared-frame integration contract. |
| `features` | public module; mixed generic + vertical contracts | feature descriptors, collector/fallback gates, renderer-facing producer adapters | prepare | prepared/derived integration; producer semantics remain external | World, UI, Editor, runtime | #1110 UI; accepted collector migration; #1124 SDF disposition | Keep framework; prune unused scaffold separately after recheck. |
| `runtime` | public module but mainly engine-internal scheduling consumers | frame prepare/submit transactions, dynamic targets/uploads, diagnostics orchestration | RenderPrepare + RenderSubmit | orchestration over prepared/derived state | RenderPlugin, renderer, tests | #1110 may overlap frame submit | Keep; later split orchestration stages only by change reason. |
| `renderer` | public module; root exposes `Gfx`/`Renderer`; concrete `Renderer` has little external pressure | physical render execution, RunenRender integration, RunenGPU realization, UI/material preparation, caches | submit/execution | derived execution state; no product truth | engine host/state, Render Lab, runtime | #1110 UI realization | Keep now; later decompose convergence points. |
| `backend` | public module with active surface identity consumers | surface/native correlation plus residual pre-RunenGPU scaffolding | host + execution support | mixed Runenwerk host integration and obsolete derived scaffold | Editor/Arena/Render Lab/runtime | later R2 only | Do not delete wholesale; re-home retained host contracts and delete only proven residue later. |
| `adapters` | public module but predominantly internal execution pressure | translate render facts/work into RunenGPU contracts | compile/submit realization | translation only | renderer/render-flow execution | none material found | Keep explicit translation; later public-surface narrowing may hide internals. |
| `gpu_primitives` | public module; procedural/compiler use | renderer-specific scan/compaction/counter/draw-plan building over RunenGPU vocabulary | planning + execution preparation | derived render algorithm/plans | procedural path, tests | none material found | Keep unless later RunenGPU extraction pressure is independently proved. |
| `resource` | public module | dynamic/transient resource descriptions and usage contracts | authoring + compile + realization | render execution descriptions | graph/runtime/renderer | none material found | Keep. |
| `pipelines` | public module; mostly renderer/inspection use | pipeline keys/cache metadata | execution | derived cache state | renderer, inspection | later backend residue cleanup | Keep canonical cache owner; remove alias-only backend mirror later. |
| `residency` | public module | render GPU cache identities/budget/residency bookkeeping | prepare + execution | derived renderer cache state | render integration | no active writer found | Keep where consumed; do not confuse with product residency intent. |
| `procedural` | public module; example/authoring pressure | procedural visual descriptions, camera, population lowering/validation | authoring + planning | prepared/derived render contracts | examples, API, GPU primitives | no active writer found | Keep. |
| `params` | public module | typed GPU parameter/value authoring | authoring | value/encoding contract | public render authors | no active writer found | Keep. |
| `shader` | public module + selected root exports | shader product discovery, filesystem/watch/reload, revisions, last-good diagnostics | startup + prepare polling | Runenwerk product/runtime policy, not reusable RunenShader semantics | Editor/runtime/renderer/inspection | RunenShader family boundary | Keep Runenwerk policy subset; stop on reusable-toolchain pressure. |
| `material_compiler` | public module; Editor consumers | material product -> render shader/resource compilation and validation | authoring/asset preparation | specialized Runenwerk material/render integration | Editor material lab/runtime | #842 semantic owner | Keep structurally stable while #842 remains open. |
| `inspect` | public module; extensive example/tool/test use | read-only diagnostics, capture, provenance, readiness, production evidence | prepare/submit aftermath + tooling | derived evidence only | apps/tools/examples/tests | #1058 consumes Render Lab evidence but no central writer at final recheck | Keep semantically read-only; move neutral DTOs only if later cycle pressure requires. |

This matrix is the semantic counterpart to the exact lexical dependency appendix. It also explains
why a physical source edge is not automatically an ownership violation: the relevant test is whether
the dependency reaches an owner for its contract or recreates that owner's authority.

# 4. Findings

## RRA-001 — Standalone authority cutover is structurally sound

**Classification:** keep.

The accepted RunenRender and RunenGPU authority split is visible in current Cargo/source:

- Runenwerk consumes exact standalone revisions;
- RunenRender depends on RunenGPU and RunenShader;
- central Runenwerk render code has no current direct `wgpu::` runtime API ownership;
- Runenwerk uses public RunenRender and RunenGPU contracts.

There is no evidence supporting another framework extraction or moving the whole
`engine::plugins::render` tree into RunenRender.

## RRA-002 — The retained runtime is objectively large, but size is not the defect

**Classification:** evidence.

The exact 184-file / 2,442,976-byte census proves the retained runtime is a substantial subsystem.
It does not prove that any particular responsibility is misplaced.

The strongest maintainability findings come from obsolete surfaces, dependency direction, change
reasons, and migration state below.

## RRA-003 — The prepared-frame cut is intact

**Classification:** keep and strengthen.

Current prepare/submit source and `render_cutoff_guard.rs` support the accepted law:

```text
mutable owner state -> RenderPrepare -> PreparedRenderFrame -> RenderSubmit
```

No rewrite of this boundary is justified. Future refactors must preserve it mechanically.

## RRA-004 — `RenderFrameDataRegistry` is compatibility-only public residue and creates a real layering inversion

**Classification:** delete through clean cutover.

Current facts:

- `RenderFrameDataRegistry` lives in `renderer/frame_bindings.rs`;
- its source comment calls it a projection helper for adapter APIs/tests and says active submission
  uses `PreparedRenderFrame`;
- `api/bindings.rs` and `api/flow.rs` import it downward from `renderer`;
- `RenderFlow::project_uniforms(...)` exposes it publicly;
- examples/tests retain compatibility coverage and already contain `#[allow(deprecated)]`;
- the type/method itself is not actually marked `#[deprecated]`;
- canonical docs call it compatibility-only;
- the final migration roadmap identifies `renderer/frame_bindings.rs` as cleanup surface;
- cutoff tests prohibit it from active submit.

This is not merely old naming. The app-facing authoring layer depends on a compatibility type
physically owned by the execution implementation.

**Disposition:** migrate the remaining projection tests/examples to a non-public test/helper path or
the real prepared-flow projection path, delete the public compatibility API and
`renderer/frame_bindings.rs`, and do not add a forwarding alias.

## RRA-005 — `backend/` contains proven post-RunenGPU residue

**Classification:** mixed delete + move/rename.

Current evidence:

### Delete candidates

`backend/pipeline_cache.rs` contains only two aliases:

```text
BackendPipelineCacheStats = PipelineCacheStats
BackendPipelineCacheResource = PipelineCacheResource
```

Exact-symbol search found no maintained consumers.

`BackendResourceAllocatorResource` and its `TextureResourceEntry` /
`BufferResourceEntry` types are only found in their definition plus plugin initialization.
No maintained allocation consumer was found. Current renderer resource realization uses RunenGPU
contracts and renderer-owned derived runtime caches instead.

These are dead/degenerate infrastructure candidates and should be removed, with an exact-current
consumer search repeated immediately before deletion.

### No-op gate

`backend/execution.rs::ensure_compiled_pass_is_supported` accepts every current
`CompiledPassExecutionPlan` variant unconditionally. It is not currently a capability owner.

If no separately owned future behavior is active at delivery time, delete the no-op gate and call
the real typed compiler/preflight capability checks directly. Do not preserve a placeholder
backend abstraction.

### Misnamed retained host integration

`backend/wgpu_ctx.rs::WgpuCtx` contains RunenGPU objects (`GpuContext`, public surface handles
and configurations) and no direct WGPU authority. The name is false after GX/RX cutover.

`backend/surface.rs` owns Runenwerk logical render-surface/native-window correlation and lifecycle,
which is host/presentation integration rather than generic GPU backend authority.

**Target:** retire `backend/` as a catch-all. Retained surface/context integration should live
under an explicit Runenwerk render-host/presentation boundary with backend-neutral names. The
precise implementation may use names equivalent to:

```text
render/host/surface.rs
render/host/gpu_context.rs
render/host/native.rs
```

The current `WgpuCtx` should become a backend-neutral RunenGPU integration name such as
`RenderGpuContext` only if its exact responsibilities remain the same after the move.

No compatibility aliases are justified.

## RRA-006 — Feature contribution central growth is real but already an accepted migration seam

**Classification:** continue accepted migration; do not redesign it in this investigation.

Current source still has central legacy `PreparedFeaturePayload` variants and feature-specific
prepared resources. Central preparation/hashing therefore still knows several concrete feature
payloads.

That is not an accidental discovery. The accepted Feature-Owned Render Contributions design
explicitly identifies the same friction and intentionally introduced the registered typed collector
path beside the legacy variants.

Current source has the registered collector infrastructure and registered feature payload support,
while legacy variants remain.

**Disposition:** continue the accepted strangler sequence in bounded feature migrations. Do not
replace it with a new generic plugin bag, do not remove all legacy variants in one unproven change,
and do not move product truth into Render. Once every maintained legacy payload is migrated and
equivalence is proven, remove the compatibility variants in a clean cut.

## RRA-007 — The large `Renderer` is a convergence point, but its UI split is currently owned by #1110

**Classification:** split later, serialized with active work.

`renderer/mod.rs` and neighboring files currently combine:

- RunenGPU context/execution integration;
- render-flow runtime resources;
- pipeline realization/cache state;
- dynamic target cache;
- RunenRender execution state/evidence;
- UI shader/pass/glyph realization;
- material preparation;
- timing/capture/provenance/preflight snapshots.

This is more than one reason to change.

However, active RunenUI issue #1110 explicitly owns the UI publication/realization cut and lists
`renderer/mod.rs`, `renderer/prepare.rs`, `runtime/frame_submit.rs`, `features/ui/**`, and
`plugin.rs` in its delivery surface.

**Disposition:** do not preemptively move UI realization while #1110 is active. Re-resolve the
post-#1110 source and then split execution code by demonstrated change reason. The expected
direction is to keep the generic renderer executor free of reusable UI semantics and retain only
Runenwerk-specific UI-to-render adaptation.

## RRA-008 — Inspection is physically coupled to execution, but no second truth path was found

**Classification:** keep semantics; consider contract placement during later decomposition.

The `inspect/` tree is large and renderer execution imports timing/capture/provenance contracts
from it, while inspection history imports renderer timing DTOs.

This is a physical coupling. It is not evidence that inspection owns source/product truth.

The accepted production-readiness design explicitly assigns read-only renderer execution
diagnostics/capture/readiness to this subsystem. Current evidence remains consistent with that
model.

**Disposition:** do not split `inspect` merely because it is large. If execution/inspection
cycles obstruct later decomposition, move only the neutral execution-evidence/control contracts to
a boundary module and leave report/history/projection logic in `inspect`. No new truth authority.

## RRA-009 — Some apparent cycles are intentional accepted design, not cleanup targets

**Classification:** keep unless the owning design changes.

Two important examples:

### Fragment composition / graph compiler

The accepted Render Fragment design deliberately places fragment descriptors/registry under
`composition/` and fragment merge under `graph/merge.rs`, with merged flows going through the
normal compiler. The resulting physical references between composition and graph are not sufficient
evidence to move merge solely to make an acyclic folder diagram.

### Feature/frame migration

The accepted collector design deliberately spans `features`, `frame`, `runtime`, and
`inspect` during migration. The current coexistence is temporary by design.

A structural audit must therefore distinguish architectural dependency laws from the impossibility
of enforcing a perfect folder DAG inside one crate during an accepted migration.

## RRA-010 — Static validation and prepared-frame preflight are distinct accepted compiler stages

**Classification:** keep.

The presence of both `graph/validation.rs` and `graph/prepared_validation.rs` is not proven
duplication. The accepted compiler design explicitly defines:

```text
static flow validation/planning
!=
prepared-frame execution preflight
```

The first validates authored topology/resources/passes. The second validates frame-scoped aliases,
dynamic targets, invocation/history scopes, feature gates and capabilities.

Individual duplicate checks may still be refactored when found, but the two-stage architecture is
intentional.

## RRA-011 — The root render facade is broad, but not all prepared/compiler contracts are internal

**Classification:** narrow selectively after compatibility cleanup.

`render/mod.rs` wildcard-reexports authoring, compiler, feature, frame, procedural, residency,
resource and runtime surfaces, and additionally reexports `Gfx`, `Renderer`, timing types, and
the compatibility `RenderFrameDataRegistry`.

The consumer audit corrects an earlier overgeneralization:

- `RenderPlugin`, `AppRenderExt`, and `RenderFlow` are ordinary maintained app-facing contracts;
- `PreparedRenderFrame` is also a real maintained integration contract with Arena, Draw, Editor,
  Render Lab, benches, examples, and tests;
- `Gfx` is an advanced host/runtime contract used by Render Lab and engine host integration;
- `Renderer` itself has no maintained app consumer found beyond engine state;
- `CompiledRenderFlowPlan` has narrow maintained advanced consumers (Render Lab temporal quality,
  one editor architecture guard, benches/examples);
- `RenderExecutionGraphPreparedReport` and prepared-preflight report types are currently
  inspection/test/bench surfaces;
- `RenderFrameDataRegistry` is only used by repository examples/tests and is excluded from active
  submission.

**Disposition:** preserve ordinary authoring and prepared-frame integration ergonomics. Remove only
proven compatibility surface first. Later public-facade narrowing must be type-by-type and
consumer-backed; it must not hide `PreparedRenderFrame` merely because it is execution-adjacent.

## RRA-012 — Canonical/current render docs contain post-cutover factual drift

**Classification:** correct.

Confirmed stale claims include:

- `render-final-architecture-migration.md` says the Runenwerk renderer owns all `wgpu`
  artifacts/runtime caches;
- `render-target-architecture.md` says the renderer/backend side owns all `wgpu` runtime
  objects;
- `public-api-reference.md` still documents the deleted predecessor
  `PreparedRenderWorkPlan` and `CompiledRenderFlowPlan::structural_work()`;
- public reference still calls the RunenGPU-backed host wrapper `WgpuCtx`.

These claims conflict with current source, cutoff guards, and accepted RunenGPU/RunenRender
authority.

**Disposition:** correct current/reference docs as part of the first post-investigation cleanup.
Do not rewrite historical evidence merely because terminology changed later.

## RRA-013 — Shader/product policy is a separate authority seam

**Classification:** stop/split boundary.

Current Runenwerk shader registry responsibilities are filesystem discovery, watching, reload
scheduling, product revisions, and product-facing diagnostics. Those remain legitimate
Runenwerk-owned product policy.

Reusable shader-source/toolchain and canonical-artifact semantics now have separate RunenShader
family authority. Material WGSL generation and broader source-toolchain evolution therefore cannot
be silently reassigned during a render-directory cleanup.

If the cleanup uncovers a missing reusable RunenShader contract, open that work under the owning
cross-repository authority. Do not solve it with a Runenwerk render compatibility layer.

## RRA-014 — Vertical feature placement contains both valid adapters and unused scaffold

**Classification:** mixed keep / active migration / delete-candidate / separately owned disposition.

A complete feature-tree review distinguishes four cases.

### Generic feature contribution framework — keep

`features/mod.rs`, the typed collector registry, descriptors, contribution status/fallback gates,
and registered payload contracts are renderer-owned execution integration. Their current coexistence
with legacy central payload variants is the accepted strangler from RRA-006.

### Active producer adapters — keep within current owner

- particle/VFX and world-visual collectors are registered and translate prepared producer resources
  into generic render contribution packets;
- world runtime cache is actively consumed by World -> Render bridges and remains derived GPU/render
  cache state;
- SDF residency/page/brick/clipmap state is actively produced by the World bridge and consumed as
  renderer integration/residency evidence.

These are not dead merely because their semantic vocabulary is product-specific.

### UI realization/publication — active #1110 migration seam

`features/ui/**` includes publication ordering, prepared `UiFrame` transport, font-atlas
realization, output proof, and renderer submission integration. Active issue #1110 owns the current
RunenUI publication cut. No structural move from this investigation may preempt its accepted write
set.

### Unused scaffold — delete candidates after exact-current revalidation

Current search found no maintained consumers for:

- `CaveRenderVisibilityResource`;
- `DetailCellPayload` / `DetailPreparedCellResource`;
- `WorldLodBand`, `WorldLodPolicyResource`, `WorldLodSelectionResource` beyond plugin
  initialization for the latter resources;
- `EditorPickingResultResource` beyond plugin initialization. The sibling `EditorGizmoAxis`, `EditorPickingHit`, and `EditorPickingTarget` contracts are actively consumed by Editor and are **not** deletion candidates.

The LOD scaffold is especially misleading because accepted renderer-scale architecture assigns
**semantic LOD policy** outside Render while allowing renderer-owned derived visibility/LOD
execution structures. Since the current threshold policy has no maintained consumer, it should not
be preserved as future-facing authority.

These delete candidates are separate evidence from the post-RunenGPU `backend/` residue and must
be revalidated immediately before any delivery.

## RRA-015 — SDF raymarch scaffolding is superseded as acceleration authority, not simply dead

**Classification:** retain integration evidence where consumed; do not redesign under #1138.

Completed investigation #1124 already owns the detailed disposition:

- `RenderSdfResidencyResource` / clipmap records: retain as Runenwerk
  integration/residency evidence, not generic RunenRender acceleration authority;
- current `RenderSdfRaymarchAccelerationResource`: superseded as acceleration authority;
- current "distance mips": placeholder/proof-only and superseded for real hierarchy claims;
- current tile/depth candidate lists: placeholder/proof-only and superseded as spatial candidate
  acceleration;
- generic `RenderFieldSemanticInput`: retain;
- generic future query acceleration belongs to standalone RunenRender.

Therefore #1138 must not absorb SDF acceleration redesign or delete still-consumed integration
evidence under a generic cleanup label.

# 5. Dependency-direction assessment

The current source has several physical bidirectional references inside the single render crate
tree. The important conclusion is not "all cycles are forbidden."

| Relationship | Current interpretation |
| --- | --- |
| `api <-> renderer` | **Defect** where caused by compatibility-only `RenderFrameDataRegistry`; delete that dependency. |
| `composition <-> graph` | **Accepted** fragment/compiler seam; no move justified solely for folder acyclicity. |
| `frame <-> features` | **Accepted migration seam** while collector strangler remains incomplete. |
| `renderer <-> inspect` | **Instrumentation coupling**; semantically acceptable while evidence remains derived/read-only, but candidate contract-placement cleanup later. |

This distinction prevents a cosmetic re-layout from replacing accepted semantics.

# 5.1 Inspection responsibility classification

The 33-file `inspect/` tree is large but remains semantically derived/read-oriented. Source search
found no production `ResMut`, `world.resource_mut`, or resource-insertion path inside inspection;
the only resource insertion hit is a test fixture. Apparent `commands` hits are benchmark-command
fields in evidence requests, not engine command mutation.

The inspection modules classify as:

| Class | Modules / examples | Disposition |
| --- | --- | --- |
| Runtime diagnostics and budgets | `budgets`, `pipeline_fallback`, `timings`, `gpu_residency` | Keep derived. |
| Prepared/execution projections | `plan`, `prepared_frame`, `resource_inspector`, `producer`, `query_snapshot`, `frame_history` | Keep read-only. |
| Capture/debug inspection | `capture`, `texture_preview`, `texture_view`, `graph_dump`, `pass_provenance` | Keep tooling/evidence. |
| Product-family production evidence | `material_*`, `scale_*`, `sdf_*`, `temporal_*`, `product_visual_evidence`, `ray_query` | Keep as evidence while current owning designs require them; do not promote to product truth. |
| Aggregate readiness/reporting | `readiness`, `report`, `artifacts`, `config` | Keep aggregate/read-only. |

No second execution or product-truth path was found. Later physical decomposition may move neutral
execution-evidence DTOs out of `inspect` if needed to remove import cycles, but semantic ownership
does not need redesign.

# 5.2 Large-module cohesion review

Large-file size is not itself a defect. The current large units classify as:

| File | Current change reasons | Disposition |
| --- | --- | --- |
| `api/flow.rs` | RenderFlow authoring/resources/builders plus compatibility uniform projection and embedded tests. | Mostly cohesive authoring; delete compatibility projection, do not split builder API by LOC. |
| `adapters/gpu_work.rs` | Render work composition, RunenGPU translation, control ordering, timing/capture/present nodes, extensive correctness tests. | Cohesive translation subsystem; consider test extraction only if maintenance pressure warrants. |
| `graph/validation.rs` | Static RenderFlow validation rules. | Large but cohesive; keep one compiler-validation owner. |
| `graph/prepared_validation.rs` | Prepared-frame preflight, cache keys/hashing, runtime guards. | Cohesive second validation phase; keep distinct from static validation. |
| `frame/contributions.rs` | Generic contribution container plus many legacy feature/material payload families and codecs. | Genuine mixed convergence/migration file; shrink as feature collector strangler completes. |
| `renderer/prepare.rs` | UI preparation/batching, material GPU realization, uploads, feature hashing. | Multiple independent reasons to change; split after #1110/material writer constraints resolve. |
| `runtime/frame_prepare.rs` | Frame/surface/view/invocation extraction, feature collection, input projection, fallback policy. | Orchestration hub; later split by preparation stage after collector migration. |
| `renderer/render_flow/execute.rs` | Batch realization, RunenRender admission/execution, pass execution, capture, present/provenance. | Execution convergence point; later split by execution stage once current boundaries stabilize. |
| `runtime/frame_submit.rs` | Multi-surface transaction/orchestration, frame history, retry/defer behavior, attachment validation, diagnostics tiering. | Large but mostly submit-transaction owner; extract validation/history helpers only when change pressure justifies. |
| `runtime/frame_diagnostics.rs` | Delayed capture/probe/diff transaction lifecycle and bounded evidence retention. | Large but cohesive diagnostics transaction subsystem. |

# 5.3 Public consumer disposition

The root facade should be cleaned by proven consumer class rather than by module aesthetics:

| Surface | Maintained consumer class | Disposition |
| --- | --- | --- |
| `RenderPlugin`, `AppRenderExt` | Arena, Draw, Editor, Render Lab, examples/tests | Keep ergonomic public entrypoints. |
| `RenderFlow` and ordinary authoring builders | Apps, examples, Render Lab, benches/tests | Keep public. |
| `PreparedRenderFrame` and prepared request/invocation contracts | Arena, Draw, Editor, Render Lab, benches/tests | Keep as explicit Runenwerk integration contract. |
| `Gfx` | Render Lab + engine native-host/state integration + tests | Keep advanced host/runtime contract for now; reassess placement, not existence. |
| `Renderer` | Engine state only among exact `render::Renderer` consumers found | Candidate to stop root re-exporting after consumer-proofed cleanup. |
| `CompiledRenderFlowPlan` | Narrow advanced Render Lab/bench/guard consumers | Candidate for explicit `graph` namespace rather than root wildcard. |
| preflight/inspection report types | Tests/bench/inspection | Prefer owner submodule surface. |
| `RenderFrameDataRegistry` | Examples/tests only | Delete; no compatibility alias. |

# 6. Target decomposition

No new crate or repository is justified. The target remains one Engine Render complicated
subsystem with explicit internal stages.

The durable conceptual target is:

```text
authoring
  api / procedural authoring / parameter contracts
      |
      v
composition + compiler
  fragment composition
  static graph validation
  planning/execution-plan formation
      |
      v
prepared frame boundary
  views/invocations/targets
  generic contribution packets
  RunenRender semantic contributions
      |
      v
prepare/submit orchestration
  ECS extraction only before cutoff
  immutable frame submission
      |
      v
execution runtime
  RunenRender integration
  RunenGPU work/resource/pipeline realization
      |
      v
host/presentation integration
  native window <-> render surface correlation
  RunenGPU context/surface attachment
      |
      v
derived observability
  timing/capture/provenance/readiness/reporting
```

Feature-specific and product-specific adapters attach to those stages without becoming a second
authority.

A physical steady-state directory may converge toward:

```text
render/
  api/                 ordinary declarative authoring
  composition/         fragment/source composition contracts
  graph/               render execution compiler and preflight
  frame/               prepared-frame and contribution boundary
  features/            render-owned feature adapters/collectors
  runtime/             prepare/submit scheduling and transactions
  renderer/            generic execution runtime; retained name until a rename earns its churn
  host/                surface/native/RunenGPU presentation integration
  inspect/             derived inspection/readiness/reporting
  shader/              Runenwerk product discovery/watch/reload policy
  material_compiler/   current specialized Runenwerk material->render product compiler
  adapters/            explicit RunenGPU translation only where still needed
  ...
```

This investigation does **not** authorize a wholesale `renderer -> execution` rename. The current
name is ambiguous after standalone RunenRender extraction, but a mass rename would produce large
consumer churn without changing ownership. First remove false backend/WGPU terminology and split
real responsibilities. Re-evaluate the remaining name after those changes.

# 7. Keep / move / split / rename / delete disposition

| Current area | Disposition | Reason |
| --- | --- | --- |
| `api/` | Keep; narrow public facade | Correct authoring owner; remove compatibility dependency on renderer. |
| `composition/` | Keep | Accepted fragment/source composition owner. |
| `graph/` | Keep | Accepted compiler/static+prepared validation owner. |
| `frame/` | Keep | Correct immutable prepare/submit boundary. |
| `features/` | Keep, continue collector migration | Current central legacy growth is known migration debt. |
| `runtime/` | Keep; later split large systems by coherent stage | Correct scheduler/orchestration owner. |
| `renderer/` | Keep now; split by change reason after #1110 | Execution convergence point; active UI writer overlaps. |
| `inspect/` | Keep | Correct derived evidence owner; no truth-path violation found. |
| `shader/` | Keep product-policy subset | Filesystem/watch/reload policy is Runenwerk-owned. |
| `material_compiler/` | Keep for now | #842 and shader-authority evolution own further semantic changes. |
| `backend/wgpu_ctx.rs` | Move + rename | Contains RunenGPU host/context integration, not WGPU authority. |
| `backend/surface.rs` | Move to host/presentation boundary | Runenwerk surface/window correlation. |
| `backend/formats.rs` | Move with surface host policy | Only supports surface-format selection. |
| `backend/pipeline_cache.rs` | Delete | Alias-only, no maintained consumers found. |
| `backend/resource_allocator.rs` | Delete if exact-current search remains empty | Defined + initialized but no maintained consumer found. |
| `backend/execution.rs` | Delete if still all-pass no-op | No current capability decision. |
| `renderer/frame_bindings.rs` | Delete after test/example migration | Compatibility-only registry excluded from active runtime. |
| root wildcard reexports | Narrow in later bounded slice | Current facade obscures owner submodules. |
| UI realization embedded in renderer | Split only after #1110 | Active RunenUI cut owns the boundary now. |

# 8. Interaction with active work

## #1110 — RunenUI renderer publication

At the final investigation recheck, #1110 remains open, but its named branch is 0 commits ahead / 13 behind accepted `main` and no open #1110 PR was found. It is therefore not a current changed-file writer at this instant. It remains an accepted scope owner that must be re-resolved before a cleanup claims overlapping files.

Potential overlap includes:

```text
features/ui/**
renderer/mod.rs
renderer/prepare.rs
runtime/frame_submit.rs
plugin.rs
```

Do not perform broad physical moves across those files if #1110 has reactivated a writer by the time implementation starts. If it remains writer-free, record that exact-current fact in the successor issue/PR rather than treating this point-in-time branch state as a permanent exemption.

## #842 — material compiler capability admission

At the final recheck #842 is open, with no matching branch or open PR found. It remains the semantic material/compiler scope owner. Do not reorganize or redefine its semantics under render cleanup; re-resolve before touching material-compiler source.

## #977 — world/SDF authority documentation

The current open PR is documentation-only and does not overlap the new investigation report path.
Do not use render cleanup to revise world/SDF semantic ownership.

## #1058 — Render Lab native trace recording

Render Lab is a product/validation consumer. At the final recheck #1058 has stale/diverged branches but no open PR; their changed files are automation/input-only and do not overlap central render source. Preserve Render Lab's public inspection/execution needs during API cleanup and re-resolve before publication; do not move trace semantics into generic renderer state.

# 9. Architecture-document disposition

No new ADR is required for the findings above because the durable ownership law is unchanged.

Required documentation work is correction, not a new decision:

- update post-cutover RunenGPU ownership wording in current render architecture/reference pages;
- delete current-reference documentation for retired `PreparedRenderWorkPlan` /
  `structural_work()`;
- rename documentation references when `WgpuCtx` is replaced;
- describe `RenderFrameDataRegistry` as removed after its clean cutover rather than retaining a
  compatibility section;
- preserve historical reports/closeouts as point-in-time evidence.

If a future change would move reusable shader semantics to/from RunenShader, change RunenRender
semantics, or establish a public cross-crate render ABI, that requires separately owned
architecture work.

# 10. Migration sequence

The safe sequence is not a mass tree move, and the first delivery must own one coherent boundary.

## R1 — remove the compatibility projection inversion

First successor only:

1. remove public `RenderFrameDataRegistry` and `RenderFlow::project_uniforms(...)` compatibility
   projection;
2. migrate repository-owned examples/tests to the maintained prepared/projection path or a private
   test helper;
3. delete `renderer/frame_bindings.rs` and the root compatibility re-export;
4. update current/reference documentation for that removed API;
5. strengthen cutoff/source guards so the compatibility path cannot return.

This is the smallest confirmed architectural defect: app-facing authoring currently depends
downward on a compatibility type physically owned by `renderer`, while active submit already
excludes it.

R1 must not reorganize host/backend code merely because the same investigation found that debt.

## R2 — post-RunenGPU host/backend residue cleanup

Only after R1 is accepted and current state is re-audited:

- delete alias-only/dead/no-op backend scaffolding if exact-current evidence still proves it unused;
- move retained surface/RunenGPU/native-host integration out of false WGPU/backend naming;
- correct current/reference ownership wording associated with that move.

This is a distinct physical-host boundary and therefore a separate change reason from R1.

## R3 — complete accepted feature collector migration

Continue the existing Feature-Owned Render Contributions design one bounded feature at a time.
After all maintained legacy feature payloads are migrated and equivalence is proven, delete central
compatibility variants/helpers in a clean cut.

This phase is not authorized by R1 or R2 merely because all three improve structure.

## R4 — post-#1110 execution decomposition

Re-census `renderer/`, `runtime/`, and `features/ui/` after the RunenUI publication state is
resolved.

Split only responsibilities that remain independently changing, likely including:

- UI-specific realization;
- generic RenderFlow execution;
- RunenRender semantic contribution execution;
- resource/pipeline realization;
- execution-evidence publication.

Do not create generic service layers or compatibility facades.

## R5 — public surface narrowing

Once internal paths are stable:

- preserve ordinary authoring and prepared-frame integration contracts;
- remove obsolete root wildcard reexports only where consumer evidence supports it;
- keep advanced compiler/execution/inspection APIs under explicit owner submodules where useful;
- update all repository-owned consumers in the same clean cut.

# 11. Validation requirements for delivery

Every structural delivery must preserve current semantic behavior and exact framework pins unless
its own issue explicitly owns an update.

Minimum families for R1:

```text
cargo test -p engine --test render_cutoff_guard
cargo test -p engine --test render_flow_v2
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Add focused source/compile guards for the deleted compatibility names. R1 does not change host or GPU execution, so it must not invent a native-GPU evidence requirement merely because later R2 work may do so.

# 12. Rejected alternatives

## Rewrite the renderer

Rejected. No authority/correctness evidence requires it, and it would discard a functioning
prepare/compiler/execution architecture.

## Move the 68k Runenwerk tree into standalone RunenRender

Rejected. Most retained code owns Runenwerk integration, product surfaces, host lifecycle,
features, diagnostics and RunenGPU translation rather than reusable renderer semantics.

## Create more crates immediately

Rejected. Current pressure is internal cohesion/compatibility residue, not independent reuse,
release/versioning, ABI or build-cost pressure.

## Make every folder dependency acyclic

Rejected. Some current cross-references are accepted compiler/composition or strangler-migration
seams. Dependency direction must follow ownership, not a cosmetic directory DAG.

## Keep aliases for compatibility

Rejected for repository-owned cleanup. The repository prefers clean cutovers and current consumers
can migrate atomically. Temporary coexistence remains only where an accepted design explicitly
requires proof before removal, such as the feature collector strangler.

# 13. Next gate

After the completeness correction recorded as INV-COMP-001, this report contains the required
source inventory, framework dependency classification, vertical-feature disposition, inspection
classification, cohesion review, and public-consumer map.

The investigation candidate is decision-complete for **one first successor shape**, subject to
acceptance of this report on `main` and exact-current writer re-resolution.

The next implementation issue, created only after this investigation is accepted, should own
exactly:

> remove the compatibility-only `RenderFrameDataRegistry` projection path and the resulting
> `api -> renderer` layering inversion, migrate repository-owned consumers, update current docs,
> and guard against predecessor reintroduction, without changing active frame submission semantics.

It must not include the separately evidenced backend/host rename, feature collector migration,
SDF acceleration work, RunenUI publication, material semantics, or broader renderer decomposition.

After that delivery is accepted, re-audit current state before activating another structural slice.

The later R2-R5 items above are architecture sequence/disposition only, not pre-created
implementation backlog. Premature issue #1140 was closed as not planned after INV-COMP-001 and is
not delivery authority.

## Result

The current Runenwerk rendering architecture is **semantically healthier than its directory
structure**.

The reusable renderer/GPU authority cuts are correct. The prepared-frame boundary and two-stage
compiler model are correct. The central runtime is large because it is a genuine complicated
subsystem.

The actionable defects are narrower and concrete:

- compatibility-only public projection API still points from authoring into renderer implementation;
- post-RunenGPU backend residue and false WGPU naming remain;
- feature contribution migration is only partially complete by accepted design;
- the renderer remains a multi-responsibility convergence point pending active RunenUI work;
- the flat public facade obscures owner submodules;
- current/reference docs still describe predecessor GPU/work-plan ownership.

The long-term fix is therefore a clean, sequenced internal decomposition that preserves the
accepted semantic architecture instead of replacing it.
