---
title: RunenGPU G5 Execution Lifecycle Investigation
description: Exact accepted-main census and cross-backend findings for executable work, command encoding, uploads, readback, progress, pressure, completion, retirement, and final execution cutover.
status: active
owner: gpu
layer: reports
canonical: true
last_reviewed: 2026-08-14
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g5-execution-lifecycle-design.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/specs/pt-runengpu-g5b-execution-lifecycle.ron
  - ../../workspace/specs/pt-runengpu-g5c-renderer-cutover.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Execution Lifecycle Investigation

## Question

What execution, transfer, progress, completion, readback, pressure and retirement authority remains after accepted G4, and what is the smallest future-transferable G5 design that can delete the remaining renderer/raw-WGPU execution seams without duplicating G3 work semantics or absorbing RunenRender/G7 policy?

## Accepted baseline and authorization

Exact accepted G4 base:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

That revision is the guarded squash acceptance of G4C3 PR `#242`; exact push/main CI and Documentation Build succeeded. Issues `#214` and `#188` are accepted/closed.

Issue `#284` authorizes investigation/design/specification only. No G5 Rust implementation is authorized by this report.

## Evidence inspected

Accepted source census covered:

```text
engine/src/plugins/gpu/api/**
engine/src/plugins/gpu/backend/wgpu/**
engine/src/plugins/render/adapters/gpu_work.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/renderer/**
engine/tests/gpu_g4c1_cutover_guards.rs
engine/Cargo.toml
```

Authority census covered the RunenGPU architecture, G3/G4 designs/specifications, repository-family architecture, ADR 0015, and durable roadmap.

Backend behavior was checked against pinned WGPU 27.0.1 and primary WebGPU/Vulkan/D3D12 synchronization semantics. Those comparisons constrain the abstraction; they do not authorize another backend in G5.

# Accepted G4 execution boundary

## One private backend owner

`WgpuContextState` remains sole private owner of WGPU `Instance`, `Adapter`, `Device`, `Queue`, health/error attribution, and G4 realization state.

Two explicitly temporary G5 migration seams remain.

### `CurrentRenderDeviceQueue`

Defined inside `engine/src/plugins/gpu/backend/wgpu/state.rs` as one non-reentrant raw operation loan containing only:

```text
&wgpu::Device
&wgpu::Queue
error-attribution guard
```

It owns no G4 realization. Accepted comments assign migration/deletion of its remaining execution operations to G5.

The accepted G4 structural guard proves production renderer execution has exactly one `current_render_device_queue()` interval and that it begins only after G4C1/G4C2/G4C3 realization.

### `CurrentRenderExecutionBridge`

`engine/src/plugins/gpu/backend/wgpu/program_binding_realization/current_render_execution_bridge.rs` is the sole lexical bridge for already-realized private objects. It validates and temporarily lends buffers, texture views, query sets, bind groups, compute/render pipelines, and purpose-typed resource references into the unchanged renderer encoder.

It creates no backend objects and returns no reusable backend references. It is a deletion target, not a future public API.

## Current renderer phase two

`engine/src/plugins/render/renderer/render_flow/execute.rs` has a clean G4/G5 split:

```text
phase 1
  G4 resource/program/layout/bind-group/pipeline realization
  no raw Device/Queue loan

phase 2
  current_render_device_queue()
  create command encoder
  apply staged uploads
  encode compute/render/copy/query/timestamp/readback work
  queue.submit(...)
  synchronously drive timing/capture readback helpers
```

G5 therefore has a concentrated migration: one raw operation interval plus distributed execution-bridge callbacks, not many independent queue owners.

# G3/G4 already contain almost all logical execution semantics

## G3 is the work authority

`GpuPreparedWorkGraph` already owns immutable deterministic nodes, dependencies, topological order, graph-entry initialization, hazards, requirements, outputs and diagnostics.

`GpuWorkOperation` already covers:

```text
Compute
Render
Copy
Clear(BufferZero)
Resolve(QueryResolve)
Present
```

Its contracts already carry dispatch dimensions, render attachments/load-store/resolve semantics, direct/indexed/indirect draw intent, copy regions/layouts, query ranges and logical present source. Most operations already derive their resource accesses.

A second public command graph/IR would duplicate accepted G3 authority.

## G4B/G4C3 now close the semantic gap G3 intentionally left

G3 was implemented before complete G4 logical contracts existed. Accepted G4 now supplies:

- complete backend-neutral `GpuComputePipelineDescriptor`;
- complete backend-neutral `GpuRenderPipelineDescriptor`;
- exact typed `GpuRuntimeBindingValue` values over logical resources;
- static buffer range plus optional per-binding dynamic offset in `GpuRuntimeBufferBinding`;
- structural/device binding validation and private bind-group realization;
- opaque private pipeline realization.

Therefore G5 does not need a permanent node-keyed execution-binding sidecar. Compute/render operations can become logically executable using accepted G4 descriptors while G5 preparation privately realizes them.

# Temporary renderer sidecar disposition

`RenderGpuWorkSidecar` is explicitly documented as a temporary G3 bridge. It maps prepared node IDs to `CompiledPassExecutionPlan` only because G3 predated G4/G5 execution ownership.

`CompiledPassExecutionPlan` mixes:

- RunenRender provenance/planning (`RenderPassId`, feature IDs, view masks, authoring indices);
- G4 pipeline/program/specialization/raster meaning;
- generic bindings/vertex/index/indirect execution state;
- copy/query work already represented by G3;
- logical Present that physically belongs to the G7 surface boundary.

Moving this enum into RunenGPU would import renderer semantics and duplicate G3/G4 truth. The clean cutover is to make generic work complete and delete sidecar execution authority.

# Executable compute/render closure

## Compute

Required generic semantics reduce to:

```text
GpuComputePipelineDescriptor
runtime binding set
dispatch
```

Runtime binding declarations are sufficient to derive bound-resource accesses and requirements.

## Render

Current UI proves one render pass can legitimately contain multiple draws with different pipelines/bindings/scissors. Splitting each draw into a new render-pass node would alter load/store/pass structure and add avoidable backend overhead.

Therefore the generic shape must support:

```text
GpuRenderOperation
  attachments
  [GpuRenderDraw]
  timestamps

GpuRenderDraw
  pipeline descriptor
  runtime binding set
  vertex buffer slot/range bindings
  optional index buffer binding + normalized index format
  draw intent
  explicit dynamic draw state
```

Dynamic state required by current/future portable render-pass semantics is viewport, scissor, blend constant and stencil reference, each with explicit semantic defaults. The backend may elide redundant setter calls; callers never depend on inherited hidden state.

Existing `GpuDrawIntent::Indirect` already owns indirect argument buffer/range and indexed flag. Multi-draw-indirect/mesh-shader vocabulary is not required by current consumers and remains YAGNI.

# Transfer operations must remain inside the graph

An early possibility was to treat uploads/readbacks only as submission-prefix/suffix concerns. That was rejected after rechecking G3 initialization/hazard authority.

A hidden upload could initialize or overwrite a resource without G3 seeing the write, forcing either weakened initialization proof or a second transfer access model. The correct design is therefore:

```text
GpuWorkOperation::Upload
GpuWorkOperation::Readback
```

Their **logical** source/destination access is graph authority; physical staging/mapping stays private G5 implementation.

## Immutable transfer payload identity — final decision

A second intermediate idea considered hashing transfer bytes and caching a content fingerprint. That is also rejected as the primary operation-identity mechanism.

Reason: `GpuWorkOperation` participates in equality/order/hash. Repeatedly traversing large upload bytes would make graph hashing/sorting scale with payload size; using a process-random digest for ordering would undermine deterministic order; treating a digest as stable identity would also overreach into persistence/content-addressing policy.

Final design:

```text
GpuTransferPayloadId
  opaque nonzero process-local runtime identity

GpuTransferPayload
  id
  immutable Arc-backed checked transfer data/layout/provenance
```

Rules:

- one ID binds to one immutable payload for its lifetime;
- clones preserve ID and Arc-backed bytes;
- operation Eq/Ord/Hash may use payload ID, keeping graph metadata operations bounded independently of byte size;
- separately constructed equal bytes may have distinct payload IDs;
- payload ID is not content identity, persistence identity, replay identity, wire identity or stable cache identity;
- an optional digest may later be diagnostic/dedup evidence only;
- the payload object itself owns the Arc-backed data, so no renderer-owned payload table/sidecar is introduced.

Accepted `PreparedGpuData<TransferData>` and `GpuPreparedTextureData` already provide checked immutable transfer-data/layout building blocks.

## Upload

Upload is one exact logical destination write. A complete upload can satisfy G3 initialization exactly like another complete initializing write.

The logical operation does not choose queue-write versus encoded staging copy. G5 may use queue write only when ordering-equivalent; otherwise it must use private staging/copy without changing graph semantics.

## Readback

Readback is one exact logical source read plus opaque process-local `GpuReadbackId`. Private staging/mapping is not exposed to the graph.

Result is normalized immutable `GpuReadbackBytes`; texture backend row padding is removed before publication.

CPU feedback cannot affect later nodes inside the same submitted graph. The result must be observed before constructing a later submission.

# Logical Present stays distinct from G7 physical presentation

Accepted `GpuPresentOperation` is logical ordering/presentation intent and source-access truth. Current `WgpuCtx` still owns raw `Surface`, `SurfaceConfiguration` and `SurfaceTexture` under the explicitly temporary G7 boundary.

Therefore G5 must not “execute Present” by acquiring/presenting a surface.

G5 validates/orders logical Present and submits all preceding GPU work. After successful G5 submit admission, the current temporary G7 owner may physically present its already-acquired surface texture according to that separate authority. G7 later replaces this raw boundary with reusable typed surface/generation/reconstruction contracts.

The final G5 raw-execution census explicitly excludes separately classified G7 surface-only acquisition/present operations while forbidding that boundary from becoming a generic Device/Queue execution escape.

# Progress/completion abstraction

Pinned WGPU 27 exposes backend-specific facts:

- queue submission returns a submission index;
- completion callbacks require runtime progress;
- native WGPU can drive callbacks/mapping through polling;
- WebGPU underlying progress is browser/event-loop driven;
- `map_async` has a separate mapping lifecycle and mapped resources cannot simultaneously participate in GPU submission.

WebGPU, Vulkan and D3D12 therefore support a monotonic submission-completion abstraction but not one universal public fence/poll object.

G5 should expose:

```text
GpuSubmissionId
GpuSubmission outcome
GpuReadback result
GpuContext::progress()
```

WGPU submission indices/poll types, Vulkan/D3D12 synchronization objects, browser promises and mapped ranges remain private backend mechanisms.

RunenGPU does not own a mandatory Tokio/Futures executor or implicit immortal progress thread. Host/event-loop policy owns when progress is driven.

# Prepared and in-flight capacity — final decision

A later critical review found that bounded in-flight work alone is insufficient: a prepared-but-unsubmitted value can pin G4 realization records.

Final execution-pressure domains include at least:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
max_deferred_retirement_records
```

Preparation reserves exactly one prepared slot transactionally. Failure/drop releases it.

Successful submit atomically converts one prepared slot into one in-flight slot plus required bounded staging/readback/retirement capacity. If submit pressure rejects, it allocates no submission ID and returns/preserves the prepared value **with its prepared slot** for retry.

There is no hidden accepted-but-unsubmitted queue.

# Terminal record retention — final decision

A context-wide retained-terminal-history budget was considered and rejected as unnecessary coupling.

If a completed observation handle kept consuming context execution capacity, a slow inspector could backpressure unrelated future GPU work even though backend cleanup was already safe.

Final rule:

- context registry owns nonterminal or backend-cleanup-pending records;
- exactly-once terminal result is published into immutable shared result state;
- once backend cleanup/retirement for that record is safe, context registry reference detaches;
- caller-held `GpuSubmission`/`GpuReadback` Arcs may retain terminal results without consuming execution pressure;
- if all observers were dropped early, context still terminalizes/cleans the record and then discards it;
- G5 does not create an unbounded global terminal-history ledger.

# Submission/readback lifecycle

Successful submit is the exact semantic acceptance point and allocates `GpuSubmissionId` only after bounded admission succeeds.

Public submission state is intentionally small:

```text
Submitted -> Completed | Failed
```

Backend-specific “physically started” state is not promised.

Readback is separate:

```text
Pending -> Ready(GpuReadbackBytes) | Failed
```

A submission may be GPU-complete while dependent readback mapping/materialization remains pending.

Already-submitted GPU work is not advertised as physically cancellable. Dropping observers does not cancel/discard accepted work. Dropping prepared-but-unsubmitted work creates no submission identity and no fake cancellation outcome.

# Retirement and shutdown

Accepted submissions retain exact G4 realization/staging records until backend execution and dependent readbacks are safe. Logical handle drop cannot destroy an in-flight dependency.

Deferred retirement is bounded.

Execution lifecycle:

```text
Running
ShuttingDown
Closed
```

Shutdown rejects new prepare/submit admission. Existing prepared values become non-submittable and release prepared slots when dropped. `progress()` remains valid while accepted work/readbacks terminalize and records retire/detach.

Product timeout/block/yield/recovery policy remains Runenwerk. G7 owns device/surface reconstruction; G5 reports execution-terminal health facts only.

# Ordered implementation decomposition

Three slices are sufficient and intentionally ordered:

```text
G5A executable logical work closure
 -> G5B complete execution lifecycle core
 -> G5C renderer/UI/timing/capture cutover
```

## G5A

- executable compute/render logical contracts using G4 pipeline/binding types;
- generic multi-draw render state;
- immutable process-local transfer payload identity;
- Upload/Readback operations;
- operation-derived accesses/requirements;
- structural/device binding validation split;
- renderer lowering into generic work;
- delete execution-semantic sidecar/manual duplicate access truth.

No command submission/progress implementation.

## G5B

- finite prepared/in-flight/staging/readback/retirement limits;
- asynchronous prepared submissions;
- atomic bounded submit admission;
- private encoding/submission for accepted operations;
- host-driven progress;
- exactly-once submission/readback terminal state;
- asynchronous normalized readback;
- terminal-record detachment;
- shutdown/delayed retirement;
- one independent non-render compute/readback proof.

Renderer remains temporarily on the existing G5 execution seams.

## G5C

- renderer/UI/timing/capture migrate to accepted G5B APIs;
- generic uploads/readbacks replace renderer staging/map/poll orchestration;
- delete `CurrentRenderDeviceQueue` and accessor;
- delete `CurrentRenderExecutionBridge` and accessor;
- delete renderer raw command encoder/queue submit/map/poll execution ownership;
- preserve separately classified G7 surface-only acquisition/present boundary.

# Rejected alternatives

### Permanent `GpuExecutionBindings` sidecar
Rejected: G4B supplies complete logical pipeline/binding contracts; retaining a node-keyed companion perpetuates transitional decomposition.

### Second backend-neutral command IR
Rejected: G3 already owns operation/access/hazard/order/initialization semantics.

### Move `CompiledPassExecutionPlan` into RunenGPU
Rejected: it mixes renderer planning/provenance with generic GPU execution.

### Hidden submission-only uploads/readbacks
Rejected: they bypass G3 initialization/hazard authority or force a second access model.

### Content digest as transfer operation identity
Rejected: large-byte hashing is avoidable, random digests harm deterministic ordering, and stable digest identity overreaches into persistence/content-addressing policy.

### Raw WGPU/Vulkan/D3D12 fence or poll API
Rejected: backend-shaped and not portable to WebGPU semantics.

### Universal blocking wait or mandatory executor
Rejected: progress ownership differs between native and browser hosts.

### Unlimited prepared values
Rejected: they pin G4 realization records before submission.

### Context-owned terminal history
Rejected: completed observer retention must not backpressure unrelated future execution.

### G5-owned physical surface Present
Rejected: surface acquisition/presentation/generations/reconstruction are G7 authority.

# Investigation verdict

G5 does not need a broader abstraction than G3+G4. It needs to finish their composition and own lifecycle.

The durable spine is:

```text
complete GpuWorkOperation
  -> GpuPreparedWorkGraph
  -> bounded GpuContext::prepare
  -> GpuPreparedSubmission
  -> atomic bounded submit
  -> private execution
  -> GpuSubmission / GpuReadback
  -> progress / cleanup / retirement / detached terminal result
```

This path removes the transitional renderer sidecar and raw execution bridges without introducing another graph, hidden executor, content-addressed transfer scheme, backend fence API, retained terminal-history cache, or renderer-specific RunenGPU contract.
