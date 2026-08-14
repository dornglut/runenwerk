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

What execution, transfer, progress, completion, readback, and retirement authority remains after accepted G4, and what is the smallest future-transferable G5 design that can delete the remaining renderer/raw-WGPU execution seams without duplicating G3 work semantics or absorbing RunenRender image-formation policy?

## Resolved baseline and authorization

The exact accepted G4 planning base is:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

That revision is the guarded squash merge of G4C3 PR `#242`. Automatic push CI `#648` and Documentation Build `#435` succeeded on that exact revision. Issues `#214` and `#188` are accepted and closed.

Issue `#284` authorizes only G5 investigation, design, implementation specifications, and later implementation decomposition. It does not authorize G5 Rust implementation.

## Evidence inspected

The census covered accepted source and authority under:

```text
engine/src/plugins/gpu/api/**
engine/src/plugins/gpu/backend/wgpu/**
engine/src/plugins/render/adapters/gpu_work.rs
engine/src/plugins/render/graph/execution_plan.rs
engine/src/plugins/render/renderer/**
engine/tests/gpu_g4c1_cutover_guards.rs
engine/Cargo.toml
docs-site/src/content/docs/design/active/runengpu-architecture-design.md
docs-site/src/content/docs/workspace/planning/roadmap.md
```

Upstream behavior was checked against the pinned WGPU 27.0.1 source plus WebGPU, Vulkan, and D3D12 synchronization contracts. The comparison is used to reject backend-shaped public APIs, not to authorize a second backend in G5.

## Accepted G4 backend boundary

`WgpuContextState` is the sole private owner of WGPU `Instance`, `Adapter`, `Device`, and `Queue`, health/error attribution, and G4 realization state.

Two temporary G5 migration seams remain.

### `CurrentRenderDeviceQueue`

`engine/src/plugins/gpu/backend/wgpu/state.rs` defines one non-reentrant borrowing loan containing only:

```text
&wgpu::Device
&wgpu::Queue
error-attribution gate guard
```

It owns no G4 realization. Its accepted comment assigns migration/deletion of remaining execution operations to G5.

The G4 structural guard proves production renderer execution has exactly one `current_render_device_queue()` interval and that it begins only after all G4C1/G4C2/G4C3 realization has completed.

### `CurrentRenderExecutionBridge`

`engine/src/plugins/gpu/backend/wgpu/program_binding_realization/current_render_execution_bridge.rs` is the sole lexical bridge for already-realized private objects. It validates and temporarily lends buffers, textures/views, query sets, bind groups, compute/render pipelines, and purpose-typed resource references into encoding callbacks.

It creates no backend object, returns no reusable backend reference, and is a deletion target rather than a permanent public abstraction.

## Current renderer phase two

`engine/src/plugins/render/renderer/render_flow/execute.rs` has a clean G4/G5 phase boundary:

```text
phase 1
  G4 resource/program/layout/bind-group/pipeline realization
  no raw Device/Queue loan

phase 2
  current_render_device_queue()
  create command encoder
  apply staged buffer/texture uploads
  encode realized compute/render/copy/query/timestamp work
  queue.submit(...)
  synchronously drive timing/capture readback helpers
```

The remaining problem is therefore concentrated: one raw orchestration interval plus distributed execution-bridge callbacks. G5 does not need to discover many independent queue owners.

## G3 already owns the GPU work model

`GpuPreparedWorkGraph` is already the immutable deterministic execution authority. It owns prepared nodes, dependencies, topological order, graph-entry initialization, merged requirements, outputs, and diagnostics.

`GpuWorkOperation` already covers:

```text
Compute
Render
Copy
Clear(BufferZero)
Resolve(QueryResolve)
Present
```

The operation contracts already contain dispatch dimensions, attachments/load-store/resolve semantics, direct/indexed/indirect draw intent, copy regions/layouts, query ranges, and logical present source. Copy/clear/render/resolve/present operations derive their resource accesses directly.

G5 must not introduce a second public command graph, executable command list, render-pass IR, or backend-neutral command encoder that repeats these semantics.

## The temporary render sidecar is no longer a valid long-term boundary

`engine/src/plugins/render/adapters/gpu_work.rs` explicitly describes `RenderGpuWorkSidecar` as a temporary G3 bridge. It maps prepared node IDs to `CompiledPassExecutionPlan` payload while declaring the prepared graph authoritative for access, initialization, hazards, requirements, and ordering.

`CompiledPassExecutionPlan` mixes several ownership domains:

- RunenRender planning/provenance (`RenderPassId`, feature IDs, view masks, authoring indices);
- G4 pipeline meaning (program, specialization, raster/vertex state);
- generic execution bindings (bind groups, vertex/index/indirect resources);
- generic copy/query work already represented by G3;
- present/surface meaning deferred to G7.

Moving this enum into RunenGPU would therefore import renderer semantics and duplicate G3/G4 authority.

## G4B now provides the missing backend-neutral execution contracts

G3 predated G4B. Accepted G4B/G4C3 now provide complete logical contracts that can close the G3 execution gap without a permanent sidecar:

- `GpuComputePipelineDescriptor` is a complete backend-neutral compute pipeline contract with semantic equality/hash;
- `GpuRenderPipelineDescriptor` is a complete backend-neutral render pipeline contract with semantic equality/hash and exact vertex/fragment state;
- `GpuRuntimeBindingValue` names typed logical buffer/texture-view/sampler resources by validated binding key;
- `GpuValidatedBindGroupBindings` validates exact layouts, resource types, dynamic/static offsets, ranges, texture format/dimension/sample/storage facts, sampler class, and admitted device alignment/format facts.

The durable clean cutover is therefore to make compute/render `GpuWorkOperation` values logically executable and let G5 preparation privately realize those contracts.

## Required executable logical work closure

### Compute

A compute operation needs exactly:

```text
GpuComputePipelineDescriptor
runtime binding values grouped by logical bind-group layout
dispatch size
```

The runtime binding declarations and pipeline interface are sufficient to derive buffer/texture/sampler resource accesses. Renderer-authored duplicate access lists must disappear.

### Render

One render pass can contain many draws with different pipelines and bindings. Current UI is the decisive proof: one pass switches among rect, stroke, glyph, viewport-embed, and product-surface pipelines; changes bind groups and instance buffers; and sets a scissor per draw.

Splitting those batches into separate graph nodes would alter pass structure and add avoidable load/store/encoder overhead.

The generic logical shape must therefore be:

```text
GpuRenderOperation
  attachments
  draws: [GpuRenderDraw]
  timestamp writes

GpuRenderDraw
  GpuRenderPipelineDescriptor
  runtime binding groups
  vertex buffer bindings by slot/range
  optional index buffer binding + normalized index format
  GpuDrawIntent
  explicit dynamic draw state
```

Dynamic state must include the backend-neutral equivalents of viewport, scissor, blend constant, and stencil reference. Defaults are explicit semantic defaults rather than inherited hidden backend state. Private encoding may elide redundant `set_*` calls.

Normalized index format needs only the portable `Uint16` / `Uint32` vocabulary used by WebGPU/WGPU today. It is logical draw state, not a WGPU type.

## Upload and readback must participate in the graph

An initial submission-only transfer design was rejected during this investigation.

`GpuPreparedWorkGraph` is the sole initialization and hazard authority. Hiding an upload outside the graph could initialize or overwrite a logical resource without G3 seeing the write, forcing either weakened initialization proof or a second transfer-side access authority.

G5A should therefore extend `GpuWorkOperation` with typed transfer operations:

```text
Upload
Readback
```

### Upload semantics

`GpuUploadOperation` is a host-data-to-GPU write at an exact graph point. It uses immutable checked `PreparedGpuData<TransferData>` or a purpose-specific normalized transfer payload plus an exact buffer/texture destination region.

The logical operation does not choose queue-write versus encoded staging-copy mechanics. Backend-private lowering may use a queue write only when its ordering is equivalent; otherwise it uses private staging/copy work. The graph sees one exact destination write either way.

Large transfer bytes must not become stable identity or be repeatedly rehashed as a hidden hot-path cost. Any precomputed content fingerprint is an in-process hashing accelerator only; equality still confirms complete semantic data and no persistence/wire/cache identity is implied.

### Readback semantics

`GpuReadbackOperation` is a GPU-to-host observation point with an exact logical source region and an opaque process-local `GpuReadbackId` allocated by RunenGPU authoring.

The graph records only the source read and readback identity. G5 privately allocates staging, encodes the copy at the operation's exact position, maps only after the submitted work permits it, normalizes/removes backend padding, and publishes `GpuReadbackBytes` asynchronously.

A readback operation does not permit CPU data to influence later nodes in the same GPU graph. CPU feedback is a submission boundary: consume the result, build/prepare subsequent work, then submit again.

## Existing data contracts already support G5 transfers

`PreparedGpuData<TransferData>` is immutable Arc-backed checked transfer data with explicit layout/provenance. `GpuReadbackBytes` is already documented as normalized bytes returned by a future G5 readback operation, with checked layout, optional texture format, and provenance.

G5 should extend these contracts rather than expose raw mapped ranges or renderer-owned staging buffers.

## Progress and completion are not one backend fence API

The pinned WGPU 27 path exposes backend-specific mechanisms:

- queue submission returns a WGPU submission index;
- queue completion callbacks require runtime progress;
- native WGPU progress can be driven through device/instance polling;
- WebGPU progress is browser/event-loop driven and WGPU `Device::poll` is not a universal execution primitive;
- `map_async` completes separately and mapped buffers cannot simultaneously be submitted for GPU access.

WebGPU deliberately gives limited ordering guarantees between different promise classes. Submission completion and mapping/readback completion must therefore remain separate RunenGPU lifecycle facts.

Vulkan and D3D12 reinforce the same abstraction conclusion: queue execution is asynchronous and host-visible completion uses backend-specific synchronization (fences/semaphores or monotonically valued fences). None of those backend objects is an appropriate RunenGPU public contract.

G5 should expose a process-local context/generation-bound `GpuSubmissionId`, structured submission state/outcome, and readback state/outcome. WGPU submission indices, Vulkan fences, D3D12 fence values, Metal completion handlers, and browser promises remain private lowering strategies.

## No hidden executor or universal blocking wait

RunenGPU should not own a Tokio runtime or require one through its public API. Existing Tokio use is synchronization-only and does not justify executor ownership.

The portable baseline is host-driven progress:

```text
GpuContext::progress() -> GpuProgressReport
GpuSubmission::try_outcome()
GpuReadback::try_result()
```

Callbacks/future observation may be layered over the same completion registry, but callers remain responsible for driving the host/event loop. On native, `progress()` may privately poll WGPU. On WebGPU it drains/publishes state while browser progress occurs externally.

A future cannot silently rely on `Device::poll(Wait)` as a universal wake mechanism. A blocking terminal, if a Runenwerk binary needs one, remains host policy outside the standalone portable contract unless separately justified.

## Submission acceptance and pressure

There should be no hidden accepted-but-not-submitted queue.

`GpuContext::prepare(...)` may asynchronously realize the complete work for one context/device generation and return an immutable single-use `GpuPreparedSubmission`. Preparation is inspectable and does not mean the GPU has accepted work.

`submit_prepared` performs bounded admission. On pressure it rejects synchronously and returns/preserves the prepared value for retry. Only successful submission allocates/publishes a `GpuSubmission` whose accepted work must reach exactly one terminal semantic outcome.

Pressure domains must be distinct and bounded at minimum for:

- in-flight submissions;
- upload/staging bytes;
- pending readback bytes/mappings;
- retained completion/readback records;
- delayed-retirement records.

Pressure is neither validation failure nor device loss.

## Completion and readback are separate lifecycles

A submission may be GPU-complete while one or more readback mappings/results are still being materialized. Therefore:

```text
submission lifecycle
  Submitted -> InFlight -> Completed | Failed

readback lifecycle
  PendingGpuCopy -> PendingMap -> Ready | Failed
```

The exact implementation may collapse transient internal states, but public semantics must preserve the distinction.

Dropping a `GpuSubmission` or readback observation handle does not cancel submitted GPU work. There is no claim that already-submitted GPU work is physically cancellable. Dropping or explicitly abandoning a prepared-but-unsubmitted value creates no submission identity and therefore is not a fake `Cancelled` submission outcome.

## Retirement and shutdown

G4 registries use generation-bound Arc records and bounded single-flight publication. G5 must retain every realized resource/program/layout/bind-group/pipeline record needed by an accepted submission until backend execution and dependent readback staging are safe to retire.

Logical last-handle drop cannot imply immediate backend destruction when in-flight submissions still depend on the record.

Shutdown must:

1. reject new preparation/submission admission;
2. stop creating new readback/upload work;
3. continue publishing terminal outcomes for already accepted work where backend progress permits;
4. fail unresolved work structurally on terminal context/device loss or forced closure;
5. release mapped/staging state safely;
6. retire private records only when no accepted work/readback requires them;
7. invoke no consumer callback while an internal lock is held.

G7 still owns surface/device reconstruction policy. G5 reports context/device unavailable/lost facts required to terminate execution; it does not choose product recovery.

## Imported and surface resources

G4C1 intentionally rejects generic `Imported` realization because no concrete import-source contract is accepted, and reserves `SurfaceAcquired` realization for G7.

G5 must not solve this with a broad raw-WGPU resource import API.

- Existing RunenGPU-owned resources reused across work use their actual logical `Gpu*Handle`s.
- Truly external resources require a separately typed import source contract.
- Surface-acquired resources remain G7-owned; G5 consumes only the logical execution resource bound by that future G7 boundary.

## Rejected alternatives

### Permanent `GpuExecutionBindings` sidecar
Rejected. G4B now supplies complete logical pipeline/binding contracts, so a node-keyed permanent sidecar would perpetuate transitional decomposition and duplicate semantic ownership.

### Second backend-neutral command IR
Rejected. G3 already owns operations, accesses, hazards, order, initialization, and requirements.

### Renderer `CompiledPassExecutionPlan` as RunenGPU API
Rejected. It mixes renderer planning/provenance with generic GPU execution.

### Raw fence / WGPU `SubmissionIndex` public API
Rejected. It is backend-shaped and fails WebGPU/backend-neutral semantics.

### Universal `Device::poll` / blocking-wait API
Rejected. Native and WebGPU progress mechanisms differ.

### Hidden unbounded submission or callback queues
Rejected. Pressure must be explicit and bounded.

### Submission-only invisible uploads/readbacks
Rejected. They would bypass G3 initialization/hazard authority or force a second access model.

## Ordered implementation decomposition

Planning should produce three implementation slices:

```text
G5A executable logical work closure
 -> G5B execution lifecycle core
 -> G5C renderer/UI/timing/capture cutover and seam deletion
```

### G5A
Owns complete executable compute/render operation contracts, generic draw state, Upload/Readback operations, operation-derived accesses/requirements, runtime-binding structural validation split, renderer adapter lowering into the generic work model, and deletion of execution-semantic truth from `RenderGpuWorkSidecar`.

No command submission/progress implementation.

### G5B
Owns context/generation-bound prepared submissions, private WGPU encoding for every accepted G5A operation, bounded submission/upload/readback pressure, submission IDs/outcomes, progress registry, asynchronous mapping/readback publication, health integration, delayed retirement, shutdown semantics, and one independent non-render execution proof.

No renderer final cutover yet.

### G5C
Migrates renderer/UI/timing/capture to G5 prepare/submit/progress/readback, deletes temporary renderer execution orchestration, `CurrentRenderDeviceQueue`, `current_render_device_queue()`, `CurrentRenderExecutionBridge`, and `current_render_execution_bridge()`, and proves one private execution authority.

## Investigation verdict

G5 does not need a broader abstraction than G3+G4. It needs to finish their composition.

The durable semantic spine is:

```text
GpuWorkOperation
  complete logical GPU work
      |
GpuPreparedWorkGraph
  deterministic dependency/initialization/hazard authority
      |
GpuContext::prepare
  device-dependent validation + private G4 realization
      |
GpuPreparedSubmission
  derived context/generation-bound executable state
      |
GpuContext::submit_prepared
  bounded admission + private encoding/submission
      |
GpuSubmission / GpuReadback
  progress, terminal outcomes, normalized results, retirement
```

That path removes the transitional renderer sidecar and raw execution bridges without introducing another graph, executor, backend fence API, or renderer-specific RunenGPU contract.
