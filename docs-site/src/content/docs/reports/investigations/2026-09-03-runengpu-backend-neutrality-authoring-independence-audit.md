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
| Scope | Existing RunenGPU public/runtime boundary only; not G8 diagnostics, browser runtime, profiling, reproducibility-provider expansion, or GX extraction |

This report records current implementation evidence and dispositions. It does not create a second architecture authority or authorize implementation beyond the owning GitHub issues.

## Question

Before G8 adds more diagnostics, observability, browser, reproducibility, or operational surface, is the existing RunenGPU boundary already shaped as a portable GPU framework rather than a renamed WGPU API or a host-owned shader authoring toolchain?

The audit asks four narrower questions:

1. Which public concepts are genuinely generic, portable baseline concepts, WGPU-derived but independently implementable concepts, WGPU-specific private implementation details, or future extensions?
2. Does runtime program admission depend on filesystem/module/package/compiler-process or authoring-language policy?
3. Does ordinary/public RunenGPU expose raw backend reach-through or duplicate execution/interface authority?
4. Which concrete findings require a later bounded correction rather than being silently absorbed into this audit?

## Evidence inspected

Primary code evidence on the accepted base:

- `engine/src/plugins/gpu/mod.rs`
- `engine/src/plugins/gpu/api/mod.rs`
- `engine/src/plugins/gpu/api/context.rs`
- `engine/src/plugins/gpu/api/context/descriptor.rs`
- `engine/src/plugins/gpu/api/capability.rs`
- `engine/src/plugins/gpu/api/resource.rs`
- `engine/src/plugins/gpu/api/handles.rs`
- `engine/src/plugins/gpu/api/work_resource_id.rs`
- `engine/src/plugins/gpu/api/program.rs`
- `engine/src/plugins/gpu/api/program/source/identity.rs`
- `engine/src/plugins/gpu/api/program/source/registry.rs`
- `engine/src/plugins/gpu/api/program/analysis.rs`
- `engine/src/plugins/gpu/api/program/interface/binding/stage.rs`
- `engine/src/plugins/gpu/api/ordinary.rs`
- `engine/src/plugins/gpu/api/execution.rs`
- `engine/src/plugins/gpu/api/realization.rs`
- `engine/src/plugins/gpu/api/surface.rs`
- `engine/src/plugins/gpu/backend/mod.rs`
- `engine/src/plugins/gpu/backend/wgpu/execution/retained_continuity.rs`
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

The strongest evidence is structural rather than naming-based:

```text
plugins/gpu
    -> pub mod api
    -> private backend
        -> private wgpu backend
```

`plugins/gpu/mod.rs` exports only `api` publicly, keeps `backend` private, scans GPU source for forbidden renderer/UI/product/backend vocabulary, permits raw `wgpu::` only inside the private WGPU backend, verifies instance/adapter/device creation remains there, and retains tests that deleted prior forwarding/duplicate authorities.

The runtime shader boundary is also correctly split:

```text
external authoring/toolchain owner
    -> canonical WGSL
        -> RunenGPU source admission
            -> private Naga analysis
                -> normalized RunenGPU interface
                    -> private backend realization
```

No reviewed public program contract owns filesystem paths, package/module discovery, compiler subprocesses, WESL/Slang source variants, file watching, reload scheduling, or last-known-good policy.

One material public-authority inconsistency remains: retained continuity/reconstruction observation currently requires `GpuWorkResourceId`, even though the handle contract defines that identity as diagnostic-only and explicitly says ordinary APIs should accept kind-specific handles. That correction is split to blocked follow-up issue `#420`; the internal map key and G7B lifecycle semantics remain valid.

No other material G8-N01/N02 blocker was found in this audit.

## Public-concept classification

### Context, device, adapter, capability, and limits

| Concept family | Classification | Disposition |
|---|---|---|
| `GpuContext`, context/device generation, affinity | genuinely generic | Accepted portable contract. Logical context/generation lifecycle is independent of WGPU. Current private WGPU storage inside the opaque public type is implementation-only. |
| `GpuCapabilityFeature` and requirement/fallback/profile vocabulary | genuinely generic | Accepted. Features express workload needs such as compute, copy, presentation, binding arrays, depth, and timestamps rather than raw backend feature bits. |
| `GpuLimits`, `GpuLimitKind`, alignment and format-role constraints | portable baseline / independently implementable | Accepted. The vocabulary is deliberately closed and tied to work admission requirements rather than exposing backend limit structures. |
| `GpuBackendFamily` | WGPU-derived/industry-shaped but independently implementable | Accepted. Vulkan, Metal, D3D12, OpenGL and browser WebGPU are provider/environment families used for selection evidence and policy, not raw backend object authority. |
| adapter class, power preference, software fallback, portability policy | portable baseline | Accepted. These are admission policy/facts and can be implemented by another backend provider. |

The decisive distinction is that backend family **identity may be public evidence** without backend machinery becoming public authority. The descriptor even documents backend preference as ordering permitted families without treating enumeration as execution authority.

### Resources, formats, usages, and reconstruction

| Concept family | Classification | Disposition |
|---|---|---|
| resource kind, typed handles, lifetime, ownership | genuinely generic | Accepted. These describe logical GPU resources and ownership/lifecycle. |
| `GpuReconstruction` and retained lifecycle facts | genuinely generic | Accepted after G7B. They describe current-state ownership/recovery facts, not WGPU resource recreation calls. |
| memory intent | genuinely generic | Accepted. Device/upload/readback intent is a portable placement/usage distinction without promising backend heaps. |
| buffer/texture usages | portable baseline | Accepted. Uniform/storage/vertex/index/copy/attachment semantics are common GPU contracts and are independently implementable. |
| texture dimensions/formats/capability roles | portable baseline / WGPU-derived but independently implementable | Accepted. The current set is deliberately narrow and normalized; format capability facts remain separate from backend enums. |
| `GpuWorkResourceId` | generic internal/logical identity with diagnostic public observation | Accepted as an internal/map/diagnostic identity; **not accepted as ordinary operational lookup authority**. See finding F1 / issue `#420`. |

The existing handle contract is explicit: handle diagnostic identity has no stable persistence, replay, wire, ABI, or cache meaning; `GpuResourceRef` is for export relationships and diagnostics, and ordinary APIs should accept kind-specific handles.

### Program source, interface, and pipeline

| Concept family | Classification | Disposition |
|---|---|---|
| canonical WGSL admitted source | portable baseline for current RunenGPU | Accepted. WGSL is the current runtime artifact by explicit architecture decision. |
| source owner/key/revision/provenance/digest | genuinely generic process-local artifact identity | Accepted. None are filesystem/package/compiler identities; full canonical WGSL equality remains authoritative. |
| `GpuProgramSourceRegistry` | genuinely generic runtime admission authority | Accepted advanced API. It owns bounded canonical-source consistency, not discovery/reload policy. |
| entry points, compute/vertex/fragment stages | portable baseline | Accepted. Stage values are normalized shader semantics. |
| binding keys/classes/access, layouts, stage visibility, shader I/O | portable baseline / compiler-derived | Accepted. Naga facts are normalized into RunenGPU vocabulary before becoming public. |
| compute/render pipeline contracts | portable baseline | Accepted. They describe GPU pipeline semantics, not WGPU pipeline handles. |
| Naga IR/parser/validator types | WGPU-adjacent compiler implementation detail | Correctly private. `ProgramAnalysis` and Naga values are crate-private; public authority is normalized RunenGPU vocabulary. |

The current source contract therefore satisfies the architectural rule that a frontend may produce WGSL, but WESL/Slang/Naga IR/backend-native modules do not become runtime source variants.

### Work, access, preparation, execution, and readback

| Concept family | Classification | Disposition |
|---|---|---|
| typed work operations, fragments, graph/access/hazard model | genuinely generic | Accepted. One logical work authority remains. |
| ordinary `submit_work` | genuinely generic facade | Accepted. It delegates through `prepare_work_graph -> prepare_submission -> submit_prepared`; it is not a second execution path. |
| prepared submission, submission/readback status, failure classes | genuinely generic | Accepted. Public status/failure semantics do not expose WGPU command buffers/fences/queues. |
| execution pressure policies/stats | genuinely generic operational contract | Accepted. They describe bounded RunenGPU queue/staging/readback authority, not backend queue handles. |
| private `WgpuExecutionState` stored inside opaque public prepared/context values | WGPU-specific private implementation detail | Accepted private implementation. External callers cannot name or access it; a later backend can replace the private representation without changing public semantics. |
| retained continuity/reconstruction observation by diagnostic ID | public-authority mismatch | Bounded correction required; issue `#420`. |

### Realization and cache-facing operational facts

`GpuResourceRealizationPolicy`, program/binding realization policy, realization stats, and structured realization failures are classified as **genuinely generic operational contracts**. They bound RunenGPU-owned lookup authority and classify logical/context/device failures. They do not expose WGPU resources or reinterpret record count as GPU memory/residency.

The word “backend” in normalized failure categories such as backend validation or resource exhaustion is not by itself backend leakage. These categories report where an implementation failed while preserving a stable framework-level class.

### Surface contracts

| Concept family | Classification | Disposition |
|---|---|---|
| `GpuSurfaceTarget` using `raw-window-handle` traits | portable host boundary | Accepted. It deliberately accepts standardized display/window handles, not a WGPU/window-system object. |
| surface identity/generation/affinity | genuinely generic | Accepted. |
| present mode, alpha mode, format/usage capabilities, configuration | WGPU/WebGPU-shaped but independently implementable | Accepted. Availability is negotiated; the types describe swapchain/presentation semantics rather than exposing WGPU surfaces. |
| surface lifecycle errors | genuinely generic normalized lifecycle | Accepted. Foreign/stale/lost/unsupported outcomes are framework-level facts. |

The raw-window-handle dependency is therefore a deliberate portability seam, not raw backend reach-through.

## G8-N01 — authoring-toolchain independence

### Runtime API census

The public program root explicitly states that canonical WGSL admission derives shader-defined facts before private backend realization and that source discovery and authoring-toolchain policy remain outside RunenGPU.

`GpuProgramSourceRegistry` accepts:

```text
GpuProgramSourceIdentity
canonical WGSL text
bounded provenance
```

It does not accept:

```text
filesystem path
module/package graph
frontend/compiler executable
WESL/Slang source variant
watch/reload policy
last-known-good state
build artifact envelope
```

`GpuProgramSourceOwnerId`, key, revision, provenance and digest are process-local/runtime artifact facts. Their documentation explicitly denies filesystem/persistence/wire/cache identity.

### Compiler analysis

`api/program/analysis.rs` uses Naga directly to parse and validate canonical WGSL and then maps Naga facts into private compiler helper enums and public RunenGPU types. `ProgramAnalysis` is crate-private. This is accepted **private compiler implementation**, not authoring-toolchain ownership.

The accepted shader-boundary design explicitly permits a private parser/compiler such as Naga while forbidding Naga IR or generated backend languages from becoming public authority.

### Source-boundary structural proof

`plugins/gpu/mod.rs` contains a repository test that scans the GPU boundary and rejects renderer/UI/ECS/product/source-asset/raw-backend vocabulary, including `shader_asset`, shader asset paths and `include!`, while allowing `wgpu::` only under the private WGPU backend root.

This is stronger than a documentation-only promise: regressions that directly restore those forbidden dependencies fail repository tests.

### Manifest interpretation

`engine/Cargo.toml` is **not** a standalone RunenGPU manifest. The crate also owns current Runenwerk integration, renderer, UI, ECS, asset, product, networking and unrelated examples, so its dependency list cannot be attributed wholesale to RunenGPU.

The manifest does show the expected current GPU implementation dependencies (`wgpu`, `naga`, `raw-window-handle`, data/runtime support) alongside many unrelated host dependencies, but G8 must not create a synthetic `runen-gpu` package solely to manufacture cleaner evidence.

Disposition: source/API authoring independence is proven at the current boundary. The exact minimal standalone dependency manifest remains a **GX extraction proof**, derived from accepted source reachability when `dornglut/runen-gpu` is actually created. This is recorded here rather than pre-creating a future implementation issue.

### G8-N01 verdict

**Pass at the current source/runtime boundary.**

No reviewed public RunenGPU program contract owns authoring frontend selection, filesystem discovery, package resolution, source watching, compiler process orchestration, or reload/LKG policy. Private Naga analysis of canonical WGSL is consistent with the accepted architecture.

The future standalone manifest remains intentionally unmaterialized until GX.

## G8-N02 — backend-neutrality and reach-through audit

### Raw backend reach-through

Current structure:

```text
engine::plugins::gpu::api      public
engine::plugins::gpu::backend  private
backend::wgpu                  private
```

`backend/mod.rs` exports WGPU realization/context/execution types only as `pub(crate)`. Public `api/mod.rs` contains no raw WGPU type export.

The structural GPU test enforces:

- raw `wgpu::` tokens are forbidden outside the private WGPU backend;
- WGPU instance/adapter/device creation remains under the private backend;
- renderer-side public device/queue authority is not restored;
- retired host/surface compatibility bridges remain deleted.

The public surface target uses `raw-window-handle`, not `wgpu::Surface` or a window-system object.

Verdict: **no broad public raw backend reach-through found**.

### Opaque implementation coupling

`GpuContext` currently stores a crate-private `WgpuContextState`. `GpuPreparedSubmission` similarly stores a crate-private weak reference to `WgpuExecutionState`.

This is classified as **WGPU-specific private implementation detail**, not a public-semantic failure:

- fields are not externally accessible;
- callers cannot construct or name the private backend value through the public API;
- public semantics are expressed by context affinity, prepared submission, status and normalized errors;
- a future provider can change the private representation without requiring a public vocabulary change.

G8 does not need a second backend or a trait-object abstraction merely to hide an implementation that is already private.

### Duplicate-authority census

No current evidence of a second ordinary authority was found:

- `submit_work` delegates into the canonical graph/preparation/submission path;
- canonical WGSL compiler analysis produces one effective program interface rather than accepting a second caller-authored reflected interface;
- retained reconstruction uses canonical work/preparation/submission/completion rather than a hidden queue path;
- structural tests assert retired renderer capability/resource/lifetime/forwarding paths remain absent;
- private WGPU realization remains the sole backend implementation owner.

Verdict: **one logical work/execution and one program-interface authority remain**.

### F1 — diagnostic resource identity used as operational retained-state lookup

**Classification:** bounded correction required.

Evidence:

- typed handles document `diagnostic_identity()` as process-local diagnostics only with no persistence/replay/wire/ABI/cache guarantee;
- `GpuResourceRef` documents that ordinary APIs should accept kind-specific handles;
- `GpuWorkResourceId` itself documents its numeric components as diagnostic-only;
- retained reconstruction observation currently accepts `GpuWorkResourceId` directly, and retained continuity observation uses the same lifecycle-map identity pattern.

This is not a backend-specific semantic leak and does not invalidate G7B state ownership. The private retained map may continue using `GpuWorkResourceId`. The problem is that public observation asks callers to promote a diagnostic identity into operational authority.

Disposition: issue `#420` owns the smallest typed observation correction **after #419 is accepted**. It must not redesign retained state, persistence identity, or generation semantics.

### G8-N02 verdict

**Pass with one bounded follow-up finding.**

The existing public concept model is broadly backend-neutral; WGPU-specific state is private. The sole material public-boundary mismatch found is the retained lifecycle lookup shape recorded in `#420`.

## Findings and dispositions

| ID | Finding | Disposition |
|---|---|---|
| F1 | retained lifecycle observation accepts diagnostic `GpuWorkResourceId` as operational lookup authority | **bounded correction required** — issue `#420`, blocked until this audit is accepted |
| F2 | opaque public context/prepared values contain private WGPU state | **private implementation detail** — no correction |
| F3 | backend families, present modes, normalized limits/formats are visibly shaped by current GPU/WebGPU/WGPU concepts | **accepted portable / independently implementable contract** — availability and behavior are normalized, no raw objects exposed |
| F4 | Naga parses/validates canonical WGSL inside the API implementation | **private compiler implementation detail** — accepted by shader-boundary design |
| F5 | public source registry supports source revisions and later admissions | **accepted runtime artifact authority** — it does not own discovery/reload/toolchain policy |
| F6 | `engine/Cargo.toml` contains many non-GPU dependencies | **evidence attribution constraint** — monolithic host manifest is not future RunenGPU package authority; prove minimal manifest at GX rather than inventing it now |
| F7 | actual-browser runtime, diagnostics/observability and reproducibility-provider work remain incomplete | **future G8 slices** — explicitly outside #419, no premature implementation |

## Rejected over-corrections

This audit does **not** justify:

- replacing normalized enums merely because WGPU has similar enum names;
- introducing a second backend only as proof theater;
- adding a public backend trait before a second implementation requires it;
- moving every `impl GpuContext` method physically out of `backend/wgpu` when its semantics are already backend-neutral and its concrete implementation is private;
- creating a standalone `runen-gpu` manifest before GX;
- deleting source revision/provenance because external authoring/reload owners can use them;
- turning Naga into a public interface;
- broadening G8 into diagnostics, profiling, browser automation, reproducibility providers, or extraction.

Those changes would increase abstraction without evidence and risk creating duplicate authority.

## Evidence gaps and limits

1. There is no standalone `dornglut/runen-gpu` package yet, so this audit cannot observe a real standalone Cargo manifest. That is a GX proof obligation, not permission to create one early.
2. This audit does not provide G8-B01 actual-browser runtime proof; Wasm compilation remains separate evidence.
3. This audit does not claim G8-D01/O01/O02/R01 completion.
4. A second backend has not been implemented. Backend neutrality here is established by public vocabulary, privacy, structural reach-through checks, and independent implementability—not by claiming unobserved multi-backend runtime behavior.

## Concrete next decision

Accept #419 as the G8-N01/N02 audit if the report-only delivery passes repository validation and review without new findings.

Then activate **#420 first**, because it is a concrete public-authority inconsistency discovered by the audit. Do not start G8 diagnostics/observability/browser/reproducibility expansion in parallel merely because those proof roles exist. After #420 is accepted on main, reassess the remaining G8 proof matrix and choose the next smallest evidence-driven slice.

## Acceptance checklist for #419

- [x] public concept families classified;
- [x] WGPU-shaped concepts given explicit independent semantic disposition;
- [x] authoring-toolchain/runtime source boundary inspected;
- [x] private Naga use distinguished from authoring ownership;
- [x] raw backend reach-through census completed;
- [x] duplicate-authority census completed;
- [x] material finding split to bounded follow-up `#420`;
- [x] standalone-manifest evidence gap recorded without pre-creating GX work;
- [ ] report delivery reviewed on one unchanged branch head;
- [ ] repository `cargo validate` / docs validation green on that head;
- [ ] diff hygiene and tracked-state cleanliness observed;
- [ ] accepted-main validation observed before closing #419 and activating #420.
