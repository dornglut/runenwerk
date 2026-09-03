---
title: RunenGPU Backend-Neutrality and Authoring-Independence Audit
description: Exact-main G8-N01/N02 audit of the RunenGPU public/runtime boundary before further G8 expansion and GX extraction.
status: active
owner: gpu
layer: reports
last_reviewed: 2026-09-03
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-post-g5c-hardening-design.md
  - ../../design/active/runengpu-phase-requirements-proof-matrix.md
  - ../../design/active/runengpu-shader-authoring-artifact-boundary.md
  - ../../workspace/planning/roadmap.md
  - ./runengpu-public-api-ergonomics-review.md
---

# RunenGPU Backend-Neutrality and Authoring-Independence Audit

## Record classification

| Field | Value |
|---|---|
| Evidence class | G8 public-boundary and runtime-source audit |
| Observation date | 2026-09-03 |
| Repository | `dornglut/runenwerk` |
| Parent program | Issue `#167` |
| Owning audit | Issue `#419` |
| Proof roles | `G8-N01` authoring-toolchain independence; `G8-N02` backend-neutrality audit |
| Accepted audit base | `f751c9430664f26d14c88731b4c8686ca247e058` |
| Prior phase | G7B complete through `#415` / PR `#417` |
| Acceptance authority | Owning issue, reviewed PR, repository validation, and accepted-main evidence |
| Scope | Existing RunenGPU public/runtime boundary only; not G8 diagnostics expansion, browser-runtime proof, profiling/operational expansion, reproducibility-provider expansion, or GX extraction |

This report records current implementation evidence and explicit dispositions. It does not create a second architecture authority and does not authorize implementation beyond the owning GitHub issues.

## Question

Before later G8 slices add diagnostics, observability, browser, reproducibility, or operational evidence, is the existing RunenGPU public/runtime boundary already shaped as a portable GPU framework rather than a renamed WGPU API or a host-owned shader authoring toolchain?

The audit asks four questions:

1. How should every externally meaningful public concept family be classified?
2. Does runtime program admission depend on filesystem/module/package/compiler-process or authoring-language policy?
3. Does ordinary/public RunenGPU expose raw backend reach-through or duplicate execution/interface authority?
4. Which concrete findings require a later bounded correction rather than being silently absorbed into this audit?

## Evidence inspected

Primary code evidence on the accepted base includes the public GPU root and the complete externally exported API module families:

- `engine/src/plugins/gpu/mod.rs`
- `engine/src/plugins/gpu/api/mod.rs`
- `engine/src/plugins/gpu/api/access.rs`
- `engine/src/plugins/gpu/api/capability.rs`
- `engine/src/plugins/gpu/api/context.rs` and `api/context/*`
- `engine/src/plugins/gpu/api/copy_compatibility.rs`
- `engine/src/plugins/gpu/api/data.rs`
- `engine/src/plugins/gpu/api/dispatch.rs`
- `engine/src/plugins/gpu/api/errors.rs`
- `engine/src/plugins/gpu/api/execution.rs`
- `engine/src/plugins/gpu/api/graph.rs` and relevant `api/graph/*`
- `engine/src/plugins/gpu/api/handles.rs`
- `engine/src/plugins/gpu/api/operation.rs`
- `engine/src/plugins/gpu/api/ordinary.rs` and ordinary transfer/resource/render helpers through their public exports
- `engine/src/plugins/gpu/api/pipeline_realization.rs`
- `engine/src/plugins/gpu/api/program.rs` and relevant `api/program/*`
- `engine/src/plugins/gpu/api/readback_id.rs`
- `engine/src/plugins/gpu/api/realization.rs`
- `engine/src/plugins/gpu/api/reconstruction.rs`
- `engine/src/plugins/gpu/api/render_execution.rs`
- `engine/src/plugins/gpu/api/render_pass.rs`
- `engine/src/plugins/gpu/api/resource.rs`
- `engine/src/plugins/gpu/api/surface.rs`
- `engine/src/plugins/gpu/api/surface_acquisition.rs`
- `engine/src/plugins/gpu/api/transfer.rs`
- `engine/src/plugins/gpu/api/work.rs`
- `engine/src/plugins/gpu/api/work_resource_id.rs`
- `engine/src/plugins/gpu/backend/mod.rs`
- relevant private `engine/src/plugins/gpu/backend/wgpu/*` execution/continuity realization terminals
- `engine/Cargo.toml`

Accepted semantic evidence:

- `runengpu-architecture-design.md`
- `runengpu-post-g5c-hardening-design.md`
- `runengpu-phase-requirements-proof-matrix.md`
- `runengpu-shader-authoring-artifact-boundary.md`
- `runengpu-public-api-ergonomics-review.md`
- workspace `roadmap.md`

## Executive finding

The current boundary is **directionally backend-neutral and authoring-toolchain independent**. It is not a mechanical public WGPU mirror.

The strongest evidence is structural:

```text
plugins/gpu
    -> public api
    -> private backend
        -> private wgpu implementation
```

`plugins/gpu/mod.rs` publicly exposes the API namespace while keeping backend realization private. Its structural tests reject raw `wgpu::` use outside the private WGPU backend, reject renderer/UI/ECS/product/tool vocabulary at the GPU boundary, keep instance/adapter/device creation private, and retain deletion checks for superseded forwarding/duplicate authorities.

The runtime program boundary is likewise separated:

```text
external authoring/toolchain owner
    -> canonical WGSL
        -> RunenGPU source admission
            -> private compiler analysis
                -> normalized RunenGPU interface
                    -> private backend realization
```

No reviewed public program contract owns filesystem paths, package/module discovery, compiler subprocesses, WESL/Slang source variants, source watching, reload scheduling, or last-known-good policy.

One material public-authority inconsistency remains: retained continuity/reconstruction observation accepts `GpuWorkResourceId`, even though the handle contract defines that identity as diagnostic-only and says ordinary APIs should use typed resource authority. That correction is split to blocked follow-up `#420`. The internal identity/map key and accepted G7B lifecycle semantics remain valid.

No second material G8-N01/N02 correction was found.

## Complete public-concept classification

The table groups related types by one semantic owner. A family is not accepted merely because it resembles WGPU/WebGPU terminology; it is accepted only where the public meaning is independently implementable without exposing the private backend object or mechanism.

### Context admission, capability, device facts, and limits

| Public family | Classification | Disposition |
|---|---|---|
| context identity, device generation, affinity | genuinely generic | Accepted. Logical context/generation lifecycle is independent of WGPU. |
| context descriptors, candidate selection/admission reports, retry/correlation facts | genuinely generic operational/admission contract | Accepted. They expose normalized request/evidence, not adapter objects. |
| capability features, requirements, preferred fallback and profiles | genuinely generic | Accepted. They express workload needs such as compute, copy, presentation, binding arrays, depth and timestamps rather than backend feature bits. |
| normalized limits, alignments, format roles, workload budgets | portable baseline | Accepted. The vocabulary is deliberately closed around current work-admission needs. |
| backend family | industry/backend-shaped but independently implementable | Accepted as selection/evidence policy. Vulkan, Metal, D3D12, OpenGL and browser WebGPU family identity does not expose backend machinery. |
| adapter class, power preference, software fallback and portability policy | portable baseline | Accepted. Another provider can implement the same request/evidence semantics. |
| crate-private `WgpuContextState` stored by opaque `GpuContext` | WGPU-specific private implementation detail | Accepted private representation; not public semantic authority. |

### Resources, typed handles, formats, access, and lifecycle

| Public family | Classification | Disposition |
|---|---|---|
| buffer/texture/view/sampler/query typed handles and `GpuResourceRef` | genuinely generic | Accepted logical resource authority. |
| resource labels/provenance, lifetime, ownership and memory intent | genuinely generic | Accepted. These describe owner/lifecycle/transfer intent without promising backend heaps. |
| durable `GpuReconstruction` contract | genuinely generic lifecycle contract | Accepted after G7B; it describes current-required-content recovery ownership rather than a backend recreation call. |
| buffer/texture usages, dimensions, extents, aspects, view/sampler/query descriptors | portable GPU baseline | Accepted. These are independently implementable GPU resource semantics. |
| normalized texture formats and format capability roles | portable baseline / industry-shaped | Accepted because support is negotiated and the set is deliberately normalized rather than exposing the backend enum. |
| typed buffer/texture accesses, ranges/subresources and access kinds | genuinely generic correctness contract | Accepted. They feed the single work/hazard authority. |
| `GpuWorkResourceId` | generic process-local logical/map identity with diagnostic public observation | Accepted internally and diagnostically; **not accepted as ordinary operational lookup authority**. See F1 / `#420`. |

The handle contract is explicit that diagnostic identities are not persistence, replay, wire, ABI, cache, or content identities. `GpuResourceRef` is appropriate for kind-preserving relationships/inspection; ordinary operational APIs should retain typed resource authority.

### CPU/GPU data, transfer, upload, and readback

| Public family | Classification | Disposition |
|---|---|---|
| `TransferData`, prepared GPU data/schema/layout/endian conversion | genuinely generic CPU↔GPU byte contract | Accepted. These prove bounded byte/layout preparation without exposing backend staging objects. |
| upload sources/operations and buffer/texture copy layouts | portable GPU transfer baseline | Accepted. They describe bytes, destinations and normalized layout constraints. |
| readback targets/operations, returned bytes and typed decoding | portable observation contract | Accepted. Mapping/staging realization stays private. |
| `GpuReadbackId` | genuinely generic process-local correlation identity | Accepted as correlation only; its documentation explicitly denies persistence/replay/wire/cache/content identity. |
| copy compatibility rules | genuinely generic correctness contract | Accepted. Compatibility is checked at the logical resource/format boundary. |

These families do not introduce a filesystem asset loader, persisted artifact identity, or backend buffer-mapping API into RunenGPU.

### Program source, compiler-derived interface, specialization, binding, and pipeline

| Public family | Classification | Disposition |
|---|---|---|
| canonical admitted WGSL source | portable baseline for current RunenGPU | Accepted by explicit architecture decision. |
| source owner/key/revision/provenance/digest | generic process-local runtime artifact identity | Accepted. None are filesystem/package/compiler identities; full canonical WGSL equality remains authoritative. |
| `GpuProgramSourceRegistry` | genuinely generic runtime source-admission authority | Accepted advanced API. It owns bounded source consistency, not discovery/reload policy. |
| entry-point names/stages and selected-program contract | portable baseline | Accepted normalized shader semantics. |
| binding keys/classes/access, layouts, visibility and shader I/O | portable baseline / compiler-derived | Accepted. Compiler facts are normalized before public authority. |
| specialization and runtime-binding contracts | portable baseline | Accepted where represented as normalized typed program/pipeline values rather than backend constants/objects. |
| compute/render pipeline descriptors and render pipeline state | portable GPU pipeline baseline | Accepted. They describe pipeline semantics, not WGPU pipeline handles. |
| Naga parser/validator/IR values and private analysis helpers | compiler-specific private implementation detail | Correctly private. Naga may analyze canonical WGSL, but Naga IR is not public identity or interchange authority. |

A source revision permits an external source owner to publish a later canonical artifact. It does not make RunenGPU responsible for finding files, resolving packages, invoking a frontend compiler, watching sources, or choosing reload/LKG policy.

### Work, operations, render-pass semantics, graph, and dispatch

| Public family | Classification | Disposition |
|---|---|---|
| work fragments, nodes, labels, inputs/imports/outputs/exports | genuinely generic | Accepted. They compose one logical work authority. |
| compute/render/copy/clear/resolve/present/upload/readback operation variants | portable GPU work baseline | Accepted. Operations own normalized intent and derive access/capability facts. |
| direct/indirect dispatch intent and draw intent/ranges | portable GPU execution semantics | Accepted. Indirect argument access is represented through typed resource access rather than a backend command. |
| render attachments, load/store, depth/stencil, vertex/index, primitive/blend/multisample state | portable GPU render-pass/pipeline baseline | Accepted. These are industry/WebGPU-shaped but independently implementable and contain no raw WGPU pass/encoder object. |
| graph preparation, dependency/hazard/initialization validation and prepared work | genuinely generic correctness authority | Accepted. There remains one canonical preparation path. |
| operation/access/authoring/graph errors | genuinely generic structured correctness diagnostics | Accepted. Causes name logical contract failures and may carry diagnostic resource correlation without granting raw identity construction. |
| ordinary authoring helpers and `submit_work` | generic convenience facade | Accepted because they delegate into the same canonical graph/preparation/submission authority rather than creating a second path. |

### Submission, completion, failure, execution pressure, and observation

| Public family | Classification | Disposition |
|---|---|---|
| prepared submission, submission ID/status and readback status | genuinely generic lifecycle contract | Accepted. No public queue, command buffer, fence, semaphore, or callback primitive is exposed. |
| execution lifecycle state, pressure policy and execution stats | genuinely generic operational contract | Accepted. Record/staging/readback counts are RunenGPU authority, not backend residency promises. |
| structured preparation/rejection/failure categories | genuinely generic normalized failure contract | Accepted. Backend-validation/resource-exhaustion categories report failure class without exposing backend error types. |
| retained initialized coverage and opaque content continuity | genuinely generic generation-local lifecycle facts | Accepted G7B authority; they remain independent from reconstruction evidence. |
| private `WgpuExecutionState` referenced inside opaque prepared values | WGPU-specific private implementation detail | Accepted private representation. |
| retained continuity/reconstruction observation by diagnostic ID | public-authority mismatch | **Bounded correction required** in `#420`, after this audit is accepted. |

### Realization and pipeline-realization observation

| Public family | Classification | Disposition |
|---|---|---|
| resource/program/binding realization policies and stats | genuinely generic operational contract | Accepted. They bound RunenGPU lookup authority, not GPU memory/residency. |
| realization and pipeline-realization structured failures | genuinely generic normalized lifecycle/realization diagnostics | Accepted. They expose logical/context/generation/failure classes rather than WGPU handles. |
| program/pipeline realization observations | genuinely generic operational observation | Accepted where identity/facts remain normalized and generation-aware. |
| private realized WGPU resources/layouts/pipelines/bind groups | WGPU-specific private implementation detail | Correctly private under `backend/wgpu`. |

The word `backend` in a normalized failure category is not itself public backend reach-through. The critical question is whether callers receive a stable framework-level fact or a backend object/type; the reviewed public surface does the former.

### Surface target, configuration, acquisition, presentation, and lifecycle

| Public family | Classification | Disposition |
|---|---|---|
| `GpuSurfaceTarget` using `raw-window-handle` traits | portable host boundary | Accepted. It uses standardized display/window handle traits rather than a WGPU/window-system surface object. |
| surface identity/generation/affinity | genuinely generic | Accepted. |
| present/alpha modes, normalized format/usage capabilities and configuration | WebGPU/industry-shaped but independently implementable | Accepted. Availability is negotiated and no WGPU surface/config object is public. |
| acquired surface image/lease and acquisition outcomes | genuinely generic normalized surface lifecycle | Accepted. They preserve foreign/stale/lost/outdated/error distinctions without exposing the backend surface texture. |
| present operation through canonical work | genuinely generic lifecycle/work integration | Accepted. Presentation remains inside the one work/execution authority. |
| surface lifecycle/configuration errors | genuinely generic normalized diagnostics | Accepted. |

### Reconstruction requirement, generation replacement, and recovery evidence

| Public family | Classification | Disposition |
|---|---|---|
| retained reconstruction requirement/state | genuinely generic generation-aware lifecycle evidence | Accepted. Source materialization, deterministic replay, external reimport and permanent-unavailability requirements are owner/recovery facts, not backend recreation commands. |
| explicit device-generation replacement result/error | genuinely generic normalized lifecycle operation | Accepted. It reports generation-aware replacement failure without choosing application recovery policy. |
| reconstruction work preparation | genuinely generic canonical-work integration | Accepted because reconstruction becomes established only through the same preparation/submission/completion authority. |
| automatic reset/replay/retry/degrade/abort policy | outside RunenGPU | Correctly absent. Application/Runenwerk remains the policy owner. |

### Diagnostics, provenance, and correlation

Across the families above, labels, provenance, candidate/submission/readback/context/surface correlation, structured causes, corrections, and point-in-time stats are classified as **genuinely generic diagnostic/operational facts** when they remain bounded and process-local as documented.

Diagnostic IDs do not become persistence or operational authority merely because they are visible in errors or reports. F1 exists precisely because retained lifecycle lookup crossed that boundary.

## G8-N01 — authoring-toolchain independence

### Runtime source/API census

The public program root states that canonical WGSL admission derives shader-defined facts before private backend realization and that source discovery/authoring-toolchain policy remains outside RunenGPU.

`GpuProgramSourceRegistry` accepts:

```text
semantic source identity
canonical WGSL text
bounded provenance
```

It does not accept or own:

```text
filesystem path
module/package graph
frontend/compiler executable
WESL or Slang runtime source variant
watch/reload policy
last-known-good state
persisted build-artifact envelope
```

Source owner/key/revision/provenance/digest are runtime artifact facts, not filesystem/package/compiler identities.

### Private compiler analysis

`api/program/analysis.rs` uses Naga to parse/validate canonical WGSL and maps compiler-known facts into private helper values and normalized public RunenGPU types. `ProgramAnalysis` and Naga values remain crate-private.

This is **compiler-specific private implementation**, not authoring-toolchain ownership. The accepted shader-boundary design explicitly permits a private parser/compiler such as Naga while excluding Naga IR and generated backend languages from public authority.

### Structural boundary proof

`plugins/gpu/mod.rs` contains repository tests that scan the GPU boundary for forbidden renderer/UI/ECS/product/source-asset/raw-backend vocabulary and allow raw `wgpu::` only under the private backend root. It also retains checks against restored renderer device/queue authority and retired forwarding/compatibility paths.

That is stronger than a prose-only boundary: direct regressions in those categories fail repository tests.

### Manifest interpretation

`engine/Cargo.toml` is the current monolithic host/integration manifest, not a standalone RunenGPU package manifest. It contains rendering, UI, ECS, asset, product, networking, image, examples, WGPU/Naga and other dependencies because the `engine` crate owns much more than RunenGPU.

Therefore the manifest cannot truthfully prove either that all listed dependencies belong to RunenGPU or what the exact future standalone dependency set will be.

Disposition: source/API authoring independence is proven at the current RunenGPU boundary. The exact minimal standalone dependency manifest remains a **GX extraction proof**, derived from actual accepted source reachability when `dornglut/runen-gpu` is created. G8 must not create a synthetic package solely to manufacture that evidence early.

### G8-N01 verdict

**Pass at the current source/runtime boundary.**

No reviewed public RunenGPU program contract owns authoring frontend selection, filesystem discovery, package resolution, source watching, compiler-process orchestration, or reload/LKG policy. The future standalone manifest remains intentionally unmaterialized until GX.

## G8-N02 — backend-neutrality, reach-through, and duplicate authority

### Raw backend reach-through

Current structure:

```text
engine::plugins::gpu::api      public
engine::plugins::gpu::backend  private
backend::wgpu                  private
```

`backend/mod.rs` exports WGPU realization/context/execution types only within the crate. Public `api/mod.rs` contains no raw WGPU type export.

Structural tests enforce that raw WGPU usage and instance/adapter/device creation stay under the private backend and that renderer-side public device/queue authority or retired compatibility bridges do not return.

The public surface target uses `raw-window-handle`, not `wgpu::Surface` or a window-system object.

Verdict: **no broad public raw-backend reach-through found**.

### Opaque implementation coupling

`GpuContext` currently contains crate-private WGPU context state; prepared submission state can contain a crate-private weak reference to private WGPU execution state.

This is **WGPU-specific private implementation detail**, not public-semantic leakage:

- callers cannot access or name those private values through the public API;
- public semantics are expressed by normalized context/generation/submission/status/error contracts;
- a future provider may replace the private representation without requiring the public vocabulary to expose WGPU.

G8 does not need a public backend trait or a second backend merely to hide an implementation already hidden by the crate boundary.

### Duplicate-authority census

No second ordinary authority was found:

- ordinary `submit_work` delegates through canonical graph preparation, submission preparation and `submit_prepared`;
- operation variants derive/access the same logical resource and capability model;
- canonical WGSL analysis publishes one effective program interface rather than a second caller-authored reflected interface;
- reconstruction uses canonical work/preparation/submission/completion rather than hidden queue writes;
- surface presentation is represented as canonical work;
- structural tests keep retired renderer capability/resource/lifetime/forwarding paths absent;
- private WGPU realization remains the concrete backend terminal, not a parallel public execution model.

Verdict: **one logical work/execution authority and one effective program-interface authority remain**.

### F1 — diagnostic resource identity used as operational retained-state lookup

**Classification:** bounded correction required.

Evidence:

- typed handles document `diagnostic_identity()` as process-local diagnostics only, with no persistence/replay/wire/ABI/cache guarantee;
- `GpuResourceRef` says ordinary APIs should accept kind-specific handles;
- `GpuWorkResourceId` documents only process-local logical identity and diagnostic numeric components;
- retained continuity/reconstruction observation accepts the ID directly as the public lookup argument.

This does not invalidate the lifecycle semantics or make them backend-specific. The private retained map may continue to key by `GpuWorkResourceId`. The defect is the **public authority shape**: callers are asked to promote a diagnostic identity into an operational lookup argument.

Disposition: `#420` owns the smallest typed-observation correction **after #419 is accepted**. It must not redesign retained state, persistence identity, or generation semantics.

### G8-N02 verdict

**Pass with one bounded follow-up finding.**

The existing public concept model is broadly backend-neutral; WGPU-specific machinery is private. The sole material public-boundary mismatch found is F1 / `#420`.

## Findings and dispositions

| ID | Finding | Disposition |
|---|---|---|
| F1 | retained lifecycle observation accepts diagnostic `GpuWorkResourceId` as operational lookup authority | **bounded correction required** — `#420`, blocked until this audit is accepted |
| F2 | opaque public context/prepared values contain private WGPU state | **private implementation detail** — no correction |
| F3 | backend families, present modes, formats, render-pass and pipeline concepts visibly follow current GPU/WebGPU industry semantics | **accepted portable / independently implementable contract** — support is normalized/negotiated and raw objects remain private |
| F4 | Naga parses/validates canonical WGSL | **compiler-specific private implementation detail** — accepted by shader-boundary design |
| F5 | source registry supports revisions and later admissions | **accepted runtime artifact authority** — discovery/reload/toolchain policy remains external |
| F6 | `engine/Cargo.toml` contains many non-GPU dependencies | **evidence-attribution constraint** — the monolithic host manifest is not future RunenGPU package authority; prove the minimal manifest at GX |
| F7 | prepared data/transfer/readback public contracts describe byte layout, staging intent and decoding | **accepted portable contract** — no backend staging/mapping object is public |
| F8 | reconstruction requirement/state and device-generation replacement are public | **accepted generic lifecycle evidence** — application recovery policy remains external |
| F9 | actual-browser runtime, diagnostics/observability, profiling/operational and reproducibility-provider proof remain incomplete | **future G8 slices** — explicitly outside `#419`; no premature implementation |

## Rejected over-corrections

This audit does **not** justify:

- replacing normalized enums merely because WGPU/WebGPU has similarly named concepts;
- introducing a second backend only as proof theater;
- adding a public backend trait before a second implementation requires it;
- moving every `impl GpuContext` method physically out of `backend/wgpu` when its public semantics are normalized and its concrete state is private;
- creating a standalone `runen-gpu` manifest before GX;
- deleting source revision/provenance because external source owners can use them without ceding toolchain ownership;
- turning Naga into a public interface;
- broadening G8 into diagnostics, profiling, browser automation, reproducibility providers, performance work, or extraction.

Those changes would add abstraction without evidence and risk duplicate authority.

## Evidence gaps and limits

1. There is no standalone `dornglut/runen-gpu` package yet, so this audit cannot observe a real standalone Cargo manifest. That is a GX proof obligation, not permission to create one early.
2. This audit does not provide G8-B01 actual-browser runtime proof; Wasm compilation remains separate evidence.
3. This audit does not claim G8-D01, G8-O01/O02, or G8-R01 completion.
4. A second backend has not been implemented. Backend neutrality here is established by normalized public vocabulary, privacy, structural reach-through checks, and independent implementability—not by claiming unobserved multi-backend runtime behavior.
5. The audit classifies the current public concept families and ownership boundaries; it does not claim every future format, feature, platform, or extension already belongs in the portable baseline.

## Concrete next decision

Accept `#419` as the G8-N01/N02 audit only after this report-only delivery is reviewed on one unchanged head and repository validation/diff hygiene pass.

After merge, observe accepted-main validation before closing `#419`. Then activate **`#420` first**, because it is the concrete public-authority inconsistency discovered by this audit. Do not start diagnostics/observability/browser/reproducibility expansion in parallel merely because later G8 proof roles exist.

After `#420` is accepted on main, reassess the remaining G8 proof matrix and choose the next smallest evidence-driven slice.

## Acceptance checklist for #419

- [x] complete externally meaningful public concept families inventoried/classified;
- [x] WGPU/WebGPU-shaped concepts given explicit independent semantic disposition;
- [x] prepared-data/transfer/readback and generation-replacement/reconstruction surfaces classified;
- [x] authoring-toolchain/runtime source boundary inspected;
- [x] private compiler analysis distinguished from authoring ownership;
- [x] raw backend reach-through census completed;
- [x] duplicate-authority census completed;
- [x] material finding split to bounded follow-up `#420`;
- [x] standalone-manifest evidence gap recorded without pre-creating GX work;
- [ ] report delivery reviewed on one unchanged branch head;
- [ ] repository `cargo validate` / documentation validation green on that head;
- [ ] diff hygiene and tracked-state cleanliness observed;
- [ ] accepted-main validation observed before closing `#419` and activating `#420`.
