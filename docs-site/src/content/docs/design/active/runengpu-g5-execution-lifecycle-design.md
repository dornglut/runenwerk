---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete architecture for executable logical work, bounded preparation and submission, private command encoding, progress, completion, asynchronous readback, retirement, and final renderer execution cutover.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-14
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g4-context-program-realization-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-g5-execution-lifecycle-investigation.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/specs/pt-runengpu-g5b-execution-lifecycle.ron
  - ../../workspace/specs/pt-runengpu-g5c-renderer-cutover.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Execution Lifecycle Design

## Status and authority

This design binds the G5 decision phase authorized by issue `#284` against accepted G4 main:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

G4 is accepted. This document and its specifications are planning authority only until the planning PR is independently accepted. G5 Rust implementation requires separately activated implementation issues.

The ordered G5 program is:

```text
G5A executable logical work closure
 -> G5B execution lifecycle core
 -> G5C renderer/UI/timing/capture cutover
```

## Mission

G5 completes the future-transferable boundary from accepted logical GPU work to observable GPU execution.

It owns:

- complete executable logical compute/render work contracts;
- exact host-to-GPU Upload and GPU-to-host Readback work operations;
- device-dependent preparation into private realized execution records;
- bounded prepared and in-flight execution capacity;
- private command encoding and queue submission;
- backend-neutral progress;
- one terminal outcome for every accepted submission/readback;
- asynchronous normalized readback;
- shutdown and submission-aware delayed retirement;
- final deletion of the two residual current-render execution seams.

G5 does not own RunenRender image formation, product recovery, reusable surface acquisition/presentation/reconstruction, a task executor, a second command graph, or public backend synchronization objects.

## Durable semantic spine

```text
GpuWorkOperation / GpuWorkFragment
    complete logical GPU work
          |
          v
GpuPreparedWorkGraph
    deterministic order, access, hazard, initialization,
    requirement and provenance authority
          |
          v
GpuContext::prepare(...).await
    device-dependent validation + private G4 realization
    + one prepared-capacity slot
          |
          v
GpuPreparedSubmission
    immutable single-use derived execution state
    context/device-generation bound
          |
          v
GpuContext::submit_prepared(...)
    atomic prepared-slot -> in-flight-slot admission
    + bounded staging + private backend encoding/submission
          |
          +-------------------+
          |                   |
          v                   v
GpuSubmission             GpuReadback
execution outcome         normalized CPU result
          \                   /
           \                 /
            v               v
      progress / cleanup / retirement
            |
            v
   terminal result detaches from context capacity
```

`GpuPreparedSubmission` is not semantic identity and not a second IR. Its correctness derives from the prepared graph, exact G4 contracts, immutable transfer payloads, and the current context/device generation.

## One fact, one owner

- G3 owns operation/access/order/initialization/dependency semantics.
- G4 owns program/interface/layout/pipeline/resource logical contracts and private realization.
- G5 owns executable closure, execution preparation, encoding, submission, progress, transfer completion, and retirement.
- RunenRender owns why a pass exists and what image it intends to form.
- G7 owns reusable surface acquisition, physical presentation, generations, loss and reconstruction.

No G5 type may re-encode renderer pass identity, image-formation semantics, WGPU objects, backend fences, or a duplicate resource-access graph.

# G5A — Executable logical work closure

## Compute

A compute operation becomes logically executable:

```text
GpuComputeOperation
  GpuComputePipelineDescriptor
  GpuRuntimeBindingSet
  dispatch [u32; 3]
```

The runtime binding set reuses accepted G4B typed values. `GpuRuntimeBufferBinding` already contains logical buffer, static offset/size, and optional dynamic offset, so G5 introduces no parallel execution-time dynamic-offset channel.

Compute buffer/texture accesses and requirements derive from pipeline/interface declarations plus runtime resources. Renderer-authored duplicate access truth disappears.

## Render and multi-draw semantics

One `GpuRenderOperation` remains one logical render pass:

```text
GpuRenderOperation
  color/depth attachments
  draws: [GpuRenderDraw]
  timestamp writes

GpuRenderDraw
  GpuRenderPipelineDescriptor
  GpuRuntimeBindingSet
  vertex buffer bindings
  optional index buffer binding
  GpuDrawIntent
  GpuRenderDynamicState
```

This is necessary for the current UI path, which switches multiple pipelines and bindings inside one render pass. Splitting those draws into separate graph nodes would change pass/load-store semantics and add avoidable backend work.

### Vertex/index bindings

`GpuVertexBufferBinding` contains only slot, logical buffer and checked range. Pipeline stride/step/attributes remain pipeline-descriptor authority.

`GpuIndexBufferBinding` contains logical buffer, checked range and `GpuIndexFormat::{Uint16, Uint32}`. Indexed direct/indirect draw intent requires an index binding; non-indexed intent rejects one without a separately accepted semantic.

Existing indirect intent already owns the argument buffer/range and indexed flag. G5 does not add multi-draw-indirect or mesh-shader vocabulary without a demonstrated consumer.

### Dynamic state

The complete normalized G5 vocabulary is:

```text
viewport
scissor
blend constant
stencil reference
```

Each has an explicit semantic default. No draw relies on state accidentally inherited from a previous draw. Private lowering may elide redundant setter calls.

## Runtime binding validation split

One binding authority has two phases:

```text
construction
  exact group/key/cardinality/resource-kind/logical-range checks

GpuContext::prepare
  admitted alignment/format/device-generation checks
  private G4C2 bind-group realization
```

Both consume the same accepted G4B declarations/runtime values. There is no structural/device duplicate descriptor family.

## Operation-derived access truth

After G5A, executable operations derive complete GPU access truth:

- Compute: runtime binding reads/writes;
- Render: attachments, runtime bindings, vertex/index/indirect resources and timestamps;
- Copy/Clear/Resolve/Present: accepted G3 access semantics;
- Upload: exact destination write;
- Readback: exact source read.

Current renderer `lower_caller_accesses` duplication is deleted as each generic operation becomes complete.

## Immutable transfer payload identity

Upload data must not make `GpuWorkOperation` graph ordering/hashing scale with payload byte count.

G5A introduces:

```text
GpuTransferPayloadId
  opaque nonzero process-local identity

GpuTransferPayload
  id
  immutable Arc-backed checked transfer data/layout/provenance
```

For texture transfer the payload reuses/refines accepted `GpuPreparedTextureData`; buffer transfer reuses `PreparedGpuData<TransferData>`.

Rules:

- one payload ID binds to one immutable payload for its lifetime;
- cloning preserves ID and Arc-backed data;
- operation Eq/Ord/Hash may use the runtime payload ID rather than repeatedly hashing/sorting bytes;
- two separately constructed equal byte payloads may have different runtime IDs;
- payload ID is not persistence/replay/wire/cache/content identity;
- a digest may later be diagnostic/dedup evidence only and cannot replace runtime identity or immutable data correctness;
- no renderer-owned payload table or permanent sidecar exists.

## Upload

`GpuWorkOperation::Upload` writes immutable transfer payload data to one exact logical buffer/texture destination.

It is graph-visible because G3 is the sole initialization/hazard authority. A complete upload can therefore satisfy initialization exactly like another initializing write.

The logical operation does not choose physical mechanics. G5B may use a queue write only when its ordering is equivalent; otherwise it uses private staging plus an encoded copy.

## Readback

`GpuWorkOperation::Readback` reads one exact logical buffer/texture region and owns an opaque nonzero process-local `GpuReadbackId`.

The graph does not expose staging resources. G5B privately stages/maps, strips backend texture-row padding, and publishes normalized immutable `GpuReadbackBytes`.

CPU feedback cannot affect later nodes inside the same GPU graph. The consumer observes the result and builds a later submission.

## Logical Present and G7

`GpuPresentOperation` remains G3 logical ordering/presentation intent and source-access truth. It is **not** a WGPU command-encoder operation.

G5 validates/orders the intent and guarantees preceding GPU work is accepted in submission order, but it does not acquire/configure a surface or own a `SurfaceTexture`. The current temporary G7 owner remains responsible for physical surface acquisition/present after successful G5 submission. Later G7 replaces that temporary boundary with the reusable typed surface contract.

G5 cannot absorb G7 just to make the renderer raw-execution census reach zero.

## Sidecar disposition

`RenderGpuWorkSidecar` and `CompiledPassExecutionPlan` cease to own GPU execution semantics in G5A.

RunenRender may retain renderer-local planning/provenance before lowering. After lowering, the prepared RunenGPU graph contains complete generic GPU execution meaning. No permanent `GpuExecutionBindings`, second command list, or node-keyed execution companion is introduced.

# G5B — Execution lifecycle core

## Execution limits

`GpuExecutionLimits` is finite context execution policy with at least:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
max_deferred_retirement_records
```

Prepared capacity is separate because a prepared value pins G4 realization records before queue admission.

There is no context-wide `max_retained_terminal_records` history budget. Once a terminal result is immutable and backend cleanup/retirement for the record is safe, the context detaches its registry reference. Caller-held observation handles may retain terminal result Arcs without backpressuring unrelated future work.

## Preparation

`GpuContext::prepare` is asynchronous because device-dependent G4 realization may be asynchronous.

Preparation:

1. reserves one prepared-submission capacity slot transactionally;
2. validates Running state and context/device generation;
3. validates complete graph/executable contracts;
4. realizes exact G4 resources/program/layout/binding/pipeline records;
5. plans upload/readback staging and pressure requirements;
6. publishes one immutable single-use `GpuPreparedSubmission`.

Failure releases the prepared slot and publishes no partial value.

Physical staging is planned rather than eagerly materialized where possible; in-flight/staging capacity is reserved transactionally at submit admission.

Dropping a prepared value releases its prepared slot and creates no fake submission/cancellation outcome.

## Submit admission

`submit_prepared` is the exact acceptance point.

It atomically transitions:

```text
prepared slot
  -> in-flight slot + required staging/readback/retirement capacity
```

Only after synchronous admission succeeds does it allocate a `GpuSubmissionId` and begin private encoding/submission.

Pressure rejection:

- performs no queue operation;
- allocates no submission ID;
- consumes no in-flight capacity;
- preserves/returns the prepared value and its prepared slot for retry.

There is no hidden accepted-but-unsubmitted queue.

## Submission identity and outcome

`GpuSubmissionId` is opaque, nonzero, monotonically allocated within one live context/device generation and process-local only. It is not a WGPU submission index or backend fence identity.

Every accepted submission has exactly one terminal semantic outcome:

```text
Completed
Failed(GpuSubmissionFailure)
```

Already-submitted GPU work is not advertised as physically cancellable. Observer drop is not cancellation.

## Private encoding

G5B owns the sole private lowering from prepared operations to WGPU execution:

- compute pipeline/bindings/dispatch;
- render attachments/draws/pipelines/bindings/vertex/index/dynamic state/timestamps;
- copy/clear/query resolve;
- Upload physical lowering;
- Readback copy/staging.

Present is not encoded as a GPU command; it is a logical handoff to G7 after accepted submission.

No migrated consumer receives `Device`, `Queue`, `CommandEncoder`, backend pipeline/bind group, mapped range, submission index or generic raw callback.

## Progress

Portable baseline:

```text
GpuContext::progress() -> GpuProgressReport
GpuSubmission::try_outcome()
GpuReadback::try_result()
```

On native WGPU, private progress may invoke nonblocking polling. On WebGPU, browser/event-loop progress remains external and `progress()` drains/publishes callback state. The public contract therefore contains no WGPU `PollType`, backend fence or mandatory blocking wait.

RunenGPU owns no mandatory Tokio/Futures executor, thread pool or implicit immortal progress thread. Optional Future/callback adapters remain wrappers over the same registry and never transfer progress ownership.

## Pressure

`GpuPressure` identifies kind, current occupancy, requested increment, limit and corrective action.

Distinct domains:

```text
prepared submissions
in-flight submissions
upload bytes
readback bytes
pending readback count
deferred retirement
```

Pressure is not silent eviction, hidden queuing, implicit sleep/blocking, validation failure or device loss.

## Submission and readback are separate timelines

A submission can be GPU-complete while readback mapping/materialization is still pending.

Public semantic models remain small:

```text
submission: Submitted -> Completed | Failed
readback:   Pending   -> Ready | Failed
```

Private readback states may distinguish copy-pending/map-pending. Mapped WGPU ranges never escape.

Every accepted readback reaches exactly one terminal result. Parent submission failure structurally terminalizes dependent readbacks where their result can no longer be produced.

## Terminal record detachment

The context registry retains nonterminal or backend-cleanup-pending records only while safety requires.

After terminal result publication and safe cleanup/retirement:

```text
context registry reference -> detached
caller observation Arc     -> may remain
```

A slow caller holding a completed submission/readback cannot consume future execution capacity. If all observers were dropped early, the context still finishes cleanup/terminalization and then discards the record rather than retaining hidden history.

## Upload lifetime and retirement

Upload staging remains charged until backend execution no longer needs it, not merely until API return.

Accepted submissions retain exact G4 realization/staging records until backend execution and dependent readbacks are safe. Logical handle drop can make a record reclaimable but cannot override in-flight references.

Deferred retirement is bounded and participates in pressure.

## Shutdown

Execution state:

```text
Running
ShuttingDown
Closed
```

`begin_shutdown()` idempotently rejects new preparation/submission. Existing prepared values become non-submittable and release their prepared slots when dropped; no submission outcome is invented.

`progress()` remains valid while shutting down and drives accepted submission/readback terminalization, record detachment and retirement.

`Closed` requires accepted work safely terminal/retired or a terminal device/context failure that structurally resolves unresolved records. Product timeout/yield/blocking policy remains Runenwerk host policy.

## Health boundary

G5 reuses the single accepted WGPU health/error-attribution authority. It reports execution/readback terminal facts needed to resolve submissions. G7 owns device/surface reconstruction policy; Runenwerk chooses product recovery.

# G5C — Final consumer cutover

Renderer/UI/timing/capture migrate to the accepted G5B lifecycle.

Renderer no longer owns command encoder creation, queue submission, generic copy/query encoding, upload staging, map/poll loops, mapped ranges, or raw realized-object execution callbacks.

UI preserves one render pass with multiple generic `GpuRenderDraw`s.

Timing becomes:

```text
timestamp writes -> QueryResolve -> Readback -> decode normalized bytes
```

Capture becomes generic copy/readback work; product capture policy/artifact encoding remains outside RunenGPU.

The final G5 execution deletion census is:

```text
CurrentRenderDeviceQueue             0
current_render_device_queue()        0
CurrentRenderExecutionBridge         0
current_render_execution_bridge()    0
renderer direct Device/Queue use     0 for G5 execution
renderer CommandEncoder creation     0 for G5 execution
renderer queue.submit                0
renderer map_async/poll ownership    0 for G5 readback
RenderGpuWorkSidecar execution truth 0
manual duplicate executable accesses 0
```

The separately classified G7 surface acquisition/physical-present boundary is excluded from this G5 execution census and cannot be used as a generic execution escape hatch.

# Error taxonomy

Phase-specific structured errors remain distinct:

```text
GpuExecutionPreparationError
  semantic/device binding mismatch
  stale/foreign generation
  realization failure
  prepared-capacity pressure

GpuSubmitRejection
  stale/closed context
  in-flight/staging pressure
  no submission ID allocated
  preserves retryable prepared value when meaningful

GpuSubmissionFailure
  terminal backend/device/context execution failure
  forced shutdown terminalization
  invariant failure

GpuReadbackFailure
  parent submission failure
  map/normalization failure
  device/context terminal failure
```

Labels/backend text are bounded diagnostics only.

# Concurrency and lock order

1. consumer callback/waker execution occurs outside internal locks;
2. shared WGPU attribution gate remains outer backend-error authority;
3. preparation obtains G4 realization records before inserting G5 lifecycle records;
4. no inverse lifecycle-lock -> G4-registry lock path is permitted;
5. encoding uses already-retained records;
6. completion/map callbacks transition one generation-bound record exactly once, move notification/detachment data out, unlock, then notify/detach;
7. shutdown uses the same state machine rather than a parallel cleanup path.

G5B implementation must bind the exact concrete mutex/registry order once source decomposition is chosen.

# Public ergonomics

Ordinary execution:

```rust
let prepared = gpu.prepare("frame 42", work).await?;
let submission = gpu.submit_prepared(prepared)?;

while submission.try_outcome().is_none() {
    gpu.progress()?;
    host.yield_now();
}
```

Readback:

```rust
let readback_id = work.readback_buffer("result", output_range)?;
let prepared = gpu.prepare("job", work.finish()?).await?;
let submission = gpu.submit_prepared(prepared)?;
let readback = submission.readback(readback_id)?;

while readback.try_result().is_none() {
    gpu.progress()?;
    host.yield_now();
}
```

RunenGPU does not own `host.yield_now()` or the executor/event loop.

# Proof matrix

G5 acceptance requires ordinary proof for at least:

1. operation-derived access equality against superseded renderer duplicate access truth;
2. complete compute pipeline/binding/dispatch execution semantics;
3. multi-draw render pass with pipeline/binding/vertex/index/dynamic-state switching;
4. transfer payload identity is immutable/process-local and graph hashing/order does not traverse large payload bytes;
5. Upload participates in initialization/hazards and cannot reorder across conflicts;
6. Readback participates at an exact graph point and normalizes backend padding privately;
7. prepared capacity is bounded and drop releases exactly one slot;
8. submit pressure preserves prepared work and allocates no submission ID;
9. successful submit atomically converts one prepared slot to one in-flight slot;
10. every accepted submission and readback terminalizes exactly once;
11. submission completion and readback completion may differ without misreporting;
12. terminal observation handles detach from context execution capacity after safe cleanup;
13. callbacks/wakers never execute under internal locks;
14. observer drop does not cancel/discard accepted work;
15. upload/readback/in-flight/retirement pressure is bounded and distinct;
16. shutdown deterministically rejects new work and resolves accepted records;
17. delayed retirement retains required G4 records until safe;
18. native progress works without public WGPU polling types;
19. WebGPU-compatible semantics do not require `Device::poll` to work;
20. logical Present performs no physical surface operation inside G5;
21. one independent non-render consumer uses G5 before renderer cutover;
22. renderer/UI/timing/capture use the same G5 authority after G5C;
23. final source guards prove both temporary G5 execution seams and raw renderer execution are absent.

Environment-dependent proofs report adapter/capability absence separately from successful hardware execution.

# Non-goals

G5 does not add:

- a universal executor/runtime;
- a second public command graph/IR;
- a permanent execution sidecar;
- public WGPU/Naga/mapped-range/submission-index/fence/poll/native-handle types;
- a context-owned unbounded terminal-history ledger;
- multi-queue scheduling;
- aggressive graph optimization/pass fusion;
- G7 surface acquisition/reconstruction;
- RunenRender materials/views/image formation;
- stable persisted execution caches;
- broad external-resource import;
- content-addressed transfer identity promises;
- speculative native interop;
- hardware ray tracing.

# Acceptance and transition

After independent planning acceptance:

```text
activate G5A only
G5B blocked by accepted G5A main
G5C blocked by accepted G5B main
G6 blocked by accepted G5C main
```

Each implementation slice starts from the exact accepted predecessor revision, uses exact-head validation, and deletes predecessor authority within its accepted scope rather than carrying compatibility aliases forward.
