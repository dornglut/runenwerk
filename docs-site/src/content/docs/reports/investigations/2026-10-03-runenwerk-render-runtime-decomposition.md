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
  - ../../../../ARCHITECTURE.md
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

The central 184-file tree excludes:

- Render Lab;
- Arena presentation;
- Editor viewport/material/render integration;
- Draw GPU/render integration;
- Engine render examples;
- Engine render integration tests;
- UI-to-render publication;
- world-to-render adapters;
- standalone RunenRender;
- standalone RunenGPU.

For scale context only, the exact Git tree contains:

```text
apps/runenwerk_render_lab/**  9 Rust files / 314,532 bytes
engine/tests/**              79 Rust files / 794,338 bytes
engine/examples/**           39 Rust files / 181,457 bytes
```

Those directories include concerns beyond this investigation's central runtime and must not be
blindly counted as render implementation. Direct render consumers exist across Arena, Editor, Draw,
Render Lab, UI publication, and world integration.

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

## RRA-011 — The root render facade is broader than ordinary consumers require

**Classification:** narrow after compatibility cleanup.

`render/mod.rs` wildcard-reexports authoring, compiler, feature, frame, procedural, residency,
resource and runtime surfaces, and additionally reexports `Gfx`, `Renderer`, timing types, and
the compatibility `RenderFrameDataRegistry`.

Current ordinary app consumers are concentrated on higher-level contracts such as `RenderPlugin`,
`RenderFlow`, product target/frame requests, feature publication, and inspection controls.
Compiler/execution internals are mainly consumed internally, by tests/benchmarks, and by advanced
Render Lab proof paths.

**Disposition:** after compatibility deletion and active writer reconciliation, keep ordinary
authoring/runtime contracts ergonomic but require advanced compiler/execution/inspection APIs to be
reached through their owning submodules. Do not preserve wildcard aliases solely to avoid import
updates in repository-owned consumers.

This is an API cleanup and needs a complete consumer diff before delivery.

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

Treat as serialization barrier for files it owns, especially:

```text
features/ui/**
renderer/mod.rs
renderer/prepare.rs
runtime/frame_submit.rs
plugin.rs
```

Do not perform broad physical moves across those files until #1110 either lands, closes, or its
owner explicitly releases the paths.

## #842 — material compiler capability admission

Do not reorganize or redefine material compiler semantics underneath this active issue. Structural
work may record the boundary, but semantic material/compiler changes remain #842-owned.

## #977 — world/SDF authority documentation

The current open PR is documentation-only and does not overlap the new investigation report path.
Do not use render cleanup to revise world/SDF semantic ownership.

## #1058 — Render Lab native trace recording

Render Lab is a product/validation consumer. Preserve its public inspection/execution needs during
API cleanup; do not move trace semantics into generic renderer state.

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

The safe sequence is not a mass tree move.

## R1 — post-cutover residue cleanup

After re-resolving #1110 writer ownership:

1. remove compatibility-only `RenderFrameDataRegistry` public projection path and migrate
   repository-owned tests/examples;
2. remove proven dead/alias/no-op `backend` residue;
3. move retained surface/RunenGPU host integration out of the false backend/WGPU namespace;
4. correct current/reference documentation truth;
5. add/strengthen source guards so deleted compatibility names cannot return.

This slice should change no rendering semantics.

## R2 — complete accepted feature collector migration

Continue the existing Feature-Owned Render Contributions design one bounded feature at a time.
After all maintained legacy feature payloads are migrated and equivalence is proven, delete central
compatibility variants/helpers in one clean cut.

This phase is not authorized by R1 merely because both improve structure.

## R3 — post-#1110 execution decomposition

Re-census `renderer/`, `runtime/`, and `features/ui/` after the RunenUI publication cut.

Split only responsibilities that remain independently changing, likely including:

- UI-specific realization;
- generic RenderFlow execution;
- RunenRender semantic contribution execution;
- resource/pipeline realization;
- host/presentation integration;
- execution-evidence publication.

Do not create generic service layers or compatibility facades.

## R4 — public surface narrowing

Once internal paths are stable:

- remove obsolete root wildcard reexports;
- keep ordinary app authoring ergonomic;
- keep advanced compiler/execution/inspection APIs under explicit owner submodules;
- update all repository-owned consumers in the same clean cut.

# 11. Validation requirements for delivery

Every structural delivery must preserve current semantic behavior and exact framework pins unless
its own issue explicitly owns an update.

Minimum families for R1:

```text
cargo test -p engine --test render_cutoff_guard
cargo test -p engine --test render_flow_v2
cargo test -p engine render_runtime_inspect
cargo validate
git diff --check
CI=true pnpm --dir docs-site build
```

Add focused tests for any moved surface/host context and compile-fail/source guards for deleted
compatibility names.

Host-backed GPU validation is required only where the slice changes host/surface execution. Do not
fabricate unavailable native evidence.

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

This investigation is decision-complete for **R1 only**.

The next implementation issue should own exactly:

> retire proven post-RunenGPU/RunenRender compatibility/backend residue, move the retained
> surface/RunenGPU host integration to backend-neutral ownership, and reconcile current render
> documentation, without changing renderer semantics.

That issue must re-resolve active #1110 before claiming `plugin.rs` or `renderer/mod.rs`.
If #1110 remains an active writer on required files, R1 is blocked/serialized rather than
parallelized through conflicting edits.

R2-R4 remain sequenced architecture outcomes, not pre-created implementation backlog.

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
