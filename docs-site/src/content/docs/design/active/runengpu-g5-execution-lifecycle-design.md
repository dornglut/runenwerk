---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete architecture for executable logical work, private command encoding, bounded submission, progress, completion, asynchronous readback, retirement, and final renderer execution cutover.
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

G4 is accepted. G5 implementation is not authorized by this document alone. The implementation specifications must be independently accepted before bounded implementation issues are activated in order.

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
- exact host-to-GPU upload and GPU-to-host readback work operations;
- device-dependent preparation into private realized execution records;
- private command encoding and queue submission;
- bounded submission, staging, mapping, completion, and retirement pressure;
- backend-neutral progress;
- one terminal outcome for every accepted submission/readback;
- asynchronous normalized readback;
- shutdown and submission-aware delayed retirement;
- final deletion of the two residual G4 execution compatibility seams.

G5 does not own RunenRender image formation, product recovery, surface acquisition/reconstruction, a task executor, or a second command graph.

## Durable semantic spine

```text
GpuWorkFragment / GpuWorkOperation
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
          |
          v
GpuPreparedSubmission
    immutable single-use derived execution state
    context/device-generation bound
          |
          v
GpuContext::submit_prepared(...)
    bounded admission + private backend encoding/submission
          |
          +-------------------+
          |                   |
          v                   v
GpuSubmission             GpuReadback
execution outcome         normalized CPU result
          \                   /
           \                 /
            v               v
        progress / retirement
```

`GpuPreparedSubmission` is not semantic identity and not a second IR. Its correctness derives from the prepared graph plus exact G4 logical descriptors and the current context/device generation.

## Core authority rule

There must be one representation of each semantic fact.

- G3 owns operation/access/order/initialization/dependency semantics.
- G4 owns program/interface/layout/pipeline/resource logical contracts and private realization.
- G5 owns execution preparation, encoding, submission, progress, transfer completion, and retirement.
- RunenRender owns why a pass exists and what image it intends to form.
- G7 owns reusable surface acquisition/presentation/reconstruction contracts.

No G5 type may re-encode a renderer pass ID, material/view semantics, WGPU pipeline object, WGPU command buffer, backend fence, or a duplicate resource-access graph.

# G5A — Executable logical work closure

## Compute operation

The durable compute contract becomes logically executable:

```rust
GpuComputeOperation {
    pipeline: GpuComputePipelineDescriptor,
    bindings: GpuRuntimeBindingSet,
    dispatch: [u32; 3],
}
```

The exact Rust field organization may use private inner records/builders, but the semantic facts are fixed.

`GpuRuntimeBindingSet` is backend-neutral un-realized binding input. Construction performs context-free structural checks; G5 preparation performs device-dependent checks using `GpuRuntimeBindingDeviceFacts` and then realizes bind groups through G4C2.

The compute operation derives all buffer/texture accesses and capability requirements from:

- pipeline interface declarations;
- typed runtime binding resources;
- storage/read/write binding classes;
- dispatch/pipeline requirements.

A caller may not provide a second manual access list for the same bound resources.

## Render operation and draw model

A render operation owns one logical render pass with any number of draws:

```rust
GpuRenderOperation {
    color_attachments: Vec<GpuColorAttachment>,
    depth_stencil_attachment: Option<GpuDepthStencilAttachment>,
    draws: Vec<GpuRenderDraw>,
    timestamp_writes: Vec<GpuTimestampWrite>,
}
```

Each draw is self-contained logical draw state:

```rust
GpuRenderDraw {
    pipeline: GpuRenderPipelineDescriptor,
    bindings: GpuRuntimeBindingSet,
    vertex_buffers: Vec<GpuVertexBufferBinding>,
    index_buffer: Option<GpuIndexBufferBinding>,
    intent: GpuDrawIntent,
    dynamic_state: GpuRenderDynamicState,
}
```

### Vertex buffer binding

A vertex binding contains:

```text
slot: u32
buffer: GpuBufferHandle
range: GpuBufferRange
```

Construction validates:

- `Vertex` usage;
- nonempty in-bounds range;
- one binding per slot;
- slot is declared by the selected pipeline's vertex-input state;
- every pipeline-required vertex slot is supplied exactly once before preparation succeeds.

The pipeline descriptor remains authoritative for stride, step mode, and attribute layout. The draw does not duplicate them.

### Index buffer binding

Indexed direct or indexed indirect draw intent requires exactly one index binding:

```text
buffer: GpuBufferHandle
range: GpuBufferRange
format: GpuIndexFormat::{Uint16, Uint32}
```

Non-indexed draw intent rejects an index binding unless a separately accepted future feature gives it semantics.

The binding validates `Index` usage, format alignment, nonempty range, and complete bounds. WGPU `IndexFormat` does not enter the public API.

### Dynamic draw state

The complete G5 dynamic vocabulary is:

```text
viewport
scissor
blend constant
stencil reference
```

Each field is an explicit normalized value with an explicit semantic default:

- viewport default = complete attachment extent and depth range `[0, 1]`;
- scissor default = complete attachment extent;
- blend constant default = transparent zero;
- stencil reference default = `0`.

Defaults are resolved from logical attachment state before backend encoding. A draw never depends on state accidentally inherited from a previous draw. Private lowering may skip redundant backend `set_*` calls.

The first implementation may only exercise scissor beyond defaults, but the complete vocabulary prevents a later API break when viewport/blend/stencil consumers arrive.

## Binding structural/device validation split

Existing `GpuValidatedBindGroupBindings` mixes context-free structural compatibility with device-dependent alignment/format capability checks. G5A separates those concerns without creating two authorities:

```text
GpuRuntimeBindingSet
    construction:
      exact group/binding keys
      exact fixed-array cardinality
      logical resource kind
      buffer/texture logical descriptor compatibility

GpuContext::prepare
    device-dependent validation:
      admitted alignments
      format capabilities
      generation/context realization
      private bind-group realization
```

Both phases derive from the same G4B layout/interface declarations.

## Access derivation

After G5A, every operation derives complete access truth.

Compute derives accesses from runtime bindings.

Render derives:

- attachment reads/writes/resolves;
- runtime binding reads/writes;
- vertex/index reads;
- indirect argument reads;
- query timestamp writes.

Copy/clear/resolve/present retain their accepted G3 derivation.

Upload/readback add exact transfer accesses.

`GpuWorkFragmentBuilder` may permit extra explicit accesses only for a separately named future semantic not representable by operations. Current renderer `lower_caller_accesses` is not such a case and must be deleted during G5A cutover.

## Upload operation

Host-to-GPU data is first-class logical work:

```rust
GpuUploadOperation::Buffer {
    destination: GpuBufferRegion,
    data: GpuTransferPayload,
}

GpuUploadOperation::Texture {
    destination: GpuTextureCopyRegion,
    data: GpuTextureTransferPayload,
}
```

The public names may refine, but the semantics are fixed.

A transfer payload wraps immutable checked transfer data and provenance. It is not a WGPU staging object.

Operation access is one exact destination write. Graph initialization simulation treats a complete upload write exactly like any other initializing write.

Physical lowering is private:

- queue write is allowed only when it is ordering-equivalent to the logical operation;
- otherwise G5 uses a private staging buffer plus encoded copy;
- backend optimization may not reorder the upload across a conflicting node.

Large data bytes are semantic data but not stable identity. Runtime hashing may use a cached content fingerprint followed by full equality; the fingerprint is never persistence/replay/wire/cache authority.

## Readback operation

GPU-to-host observation is first-class logical work:

```rust
GpuReadbackOperation::Buffer {
    id: GpuReadbackId,
    source: GpuBufferRegion,
}

GpuReadbackOperation::Texture {
    id: GpuReadbackId,
    source: GpuTextureCopyRegion,
}
```

`GpuReadbackId` is opaque, nonzero, process-local runtime correlation identity. It survives graph composition without becoming persisted identity. Duplicate IDs in one prepared graph are rejected.

The operation derives one exact source read. It does not expose a staging resource to graph callers.

Private G5 lowering allocates bounded staging, encodes the transfer at the operation's exact graph position, and maps/materializes only after backend execution permits it.

`GpuReadbackBytes` is the normalized result. Texture readback removes backend row padding and reports the logical tight layout/format.

CPU feedback cannot alter later nodes inside the same graph. Such feedback requires a later submission.

## Sidecar disposition

`RenderGpuWorkSidecar` and `CompiledPassExecutionPlan` cease to own execution semantics in G5A.

RunenRender may retain renderer-local planning/provenance data required before lowering, but after lowering the prepared RunenGPU graph contains complete GPU execution meaning.

No node-keyed permanent `GpuExecutionBindings` companion is introduced.

# G5B — Execution lifecycle core

## Preparation API

Target public shape:

```rust
let prepared = gpu.prepare("frame 42", work).await?;
inspect(prepared.graph());
inspect(prepared.diagnostics());
```

The exact convenience overloads may accept one fragment, one prepared graph, or an iterable of composable work values, but all paths converge on one preparation terminal.

Preparation:

1. validates context/device availability and generation;
2. validates the complete prepared graph and executable operation contracts;
3. realizes every referenced logical resource through G4C1;
4. validates/realizes programs, layouts, bind groups, and pipelines through G4B/G4C2/G4C3;
5. plans private upload/readback staging within configured bounds;
6. retains all exact private records needed for execution;
7. produces one immutable single-use `GpuPreparedSubmission`.

Preparation never creates a backend queue submission and therefore never allocates a `GpuSubmissionId`.

## Prepared submission semantics

`GpuPreparedSubmission` is:

- context/device-generation bound;
- single-use for successful submission;
- inspectable through normalized graph/diagnostic facts;
- non-persistent;
- not a backend command buffer;
- not guaranteed reusable across generations;
- safe to drop without creating a fake cancellation outcome.

It may hold private derived encoded metadata or realized records. Those are non-authoritative and can be rebuilt from the logical source while the generation remains valid.

## Bounded submit API

Target shape:

```rust
match gpu.submit_prepared(prepared) {
    Ok(submission) => { /* accepted */ }
    Err(rejection) => {
        let prepared = rejection.into_prepared();
        inspect(rejection.pressure());
        // caller may retry later
    }
}
```

Successful admission is the exact semantic point at which work becomes accepted and receives a `GpuSubmissionId`.

There is no hidden pending-submission queue before this point.

## Submission identity

`GpuSubmissionId` is:

- opaque and nonzero;
- context/device-generation bound;
- monotonically allocated within one live generation;
- process-local only;
- diagnostic/correlation identity, not stable persisted identity;
- independent of WGPU `SubmissionIndex` or any backend fence object.

## Private encoding

G5 owns the sole private lowering from prepared operations to WGPU command encoding.

It handles:

- compute pipeline/bind groups/dispatch;
- render attachments, pipelines, bindings, vertex/index state, dynamic state, direct/indexed/indirect draws, timestamps;
- all accepted copy operations;
- buffer zero;
- query resolve;
- Upload physical lowering;
- Readback private copy/staging setup;
- logical present only through the separately accepted G7-bound surface terminal when G7 is available; G5 does not take surface acquisition policy.

RunenRender does not receive a `CommandEncoder`, `Device`, `Queue`, WGPU pipeline, WGPU bind group, mapped range, or callback terminal.

## Submission state

Public semantic state is intentionally small:

```text
Submitted
Completed
Failed
```

`InFlight` may exist as an internal or diagnostic refinement but does not need to be a distinct promise to callers because backend APIs differ on when work has physically started.

A successful submit obtains exactly one terminal `GpuSubmissionOutcome`:

```text
Completed
Failed(GpuSubmissionFailure)
```

Failure categories distinguish at minimum:

- context/device unavailable or lost;
- backend execution/validation failure attributable to the accepted submission;
- forced shutdown before terminal completion can be proven;
- internal invariant violation represented as bounded structured failure rather than panic across the API boundary.

Already-submitted work is not advertised as physically cancellable.

## Progress contract

Portable baseline:

```rust
let report = gpu.progress()?;
```

`GpuProgressReport` is normalized, bounded evidence such as:

```text
submissions completed
submissions failed
readbacks advanced/ready/failed
records retired
pressure relieved
context/device terminal facts observed
```

It contains no WGPU poll/fence values.

On native WGPU the backend may call nonblocking `Device::poll`/equivalent internally. On WebGPU the browser/event loop owns underlying progress; `progress()` drains and publishes completed callback state.

RunenGPU owns no mandatory Tokio/runtime executor and spawns no implicit immortal progress worker in the baseline design.

Observation APIs may support callback/future adapters, but:

- registering an observer does not transfer progress ownership;
- callbacks run outside internal locks;
- observer queues/wakers are bounded or one-per-observation handle;
- dropping observers does not discard accepted work;
- portable correctness remains available through `progress()` plus `try_*` state observation.

## Pressure model

`GpuExecutionLimits` is part of context execution policy and bounds at least:

```text
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
max_retained_terminal_records
max_deferred_retirement_records
```

Limits are nonzero and validated. Defaults are finite and conservative rather than effectively unlimited.

Structured `GpuPressure` records:

```text
kind
current
requested
limit
corrective action
```

Kinds remain distinct for submission count, upload bytes, readback bytes, readback count, terminal record retention, and deferred retirement.

Pressure is a submit/prepare admission result, not a silent queue, sleep, data drop, device loss, or validation failure.

## Readback lifecycle

For each accepted `GpuReadbackId`, submission publishes one observation handle:

```text
Pending
Ready(GpuReadbackBytes)
Failed(GpuReadbackFailure)
```

Internal states may distinguish GPU-copy pending and map pending.

Readback completion is not the same as submission execution completion. A submission may be `Completed` while a dependent readback is still `Pending`.

Mapping rules:

- staging buffers are private;
- map callbacks publish into RunenGPU-owned state;
- mapped ranges never escape;
- bytes are copied/normalized into immutable `GpuReadbackBytes` before unmap/retirement;
- texture padding is removed before publication;
- every accepted readback reaches exactly one terminal Ready/Failed result unless its containing submission itself fails, in which case it terminates as a structured dependent failure.

## Upload lifecycle

G5 accounts for physical staging lifetime, not only API call lifetime.

Queue-write staging or private upload buffers remain charged to upload pressure until the backend submission has advanced far enough that the backend no longer requires their bytes/records.

Different physical upload strategies may coexist under one logical Upload operation when they preserve exact graph order and result semantics.

## Retirement

Each prepared/accepted submission retains the exact G4 realization records and private staging records it needs.

Retirement is safe only after:

- backend execution no longer references the record;
- dependent mapping/readback no longer references the record;
- no other live realization handle/submission references it.

Logical handle drop may make a registry record reclaimable but cannot override in-flight submission references.

Retirement queues are bounded and participate in pressure reporting.

## Shutdown

G5 introduces an explicit non-reentrant shutdown state in `GpuContext` execution authority:

```text
Running
ShuttingDown
Closed
```

`begin_shutdown()`:

- atomically rejects new prepare/submit admission;
- preserves existing accepted completion records;
- initiates backend-progress/retirement observation;
- is idempotent.

`progress()` remains valid during `ShuttingDown`.

The context reaches `Closed` when all accepted work/readbacks are terminal and private retirement is complete, or when a terminal context/device failure forces unresolved records to structured failure and safe backend release.

A blocking host terminal is not required by the future-transferable G5 API. Runenwerk binaries may drive `progress()` until closed according to product shutdown policy.

## Health and device-loss boundary

G5 reuses the one accepted G4 WGPU health owner and error-attribution gate. It does not install duplicate device-loss or uncaptured-error observers.

G5 converts terminal health facts into submission/readback failures and shutdown progress.

G7 owns device/surface generations after replacement, reconstruction facts, surface loss, and recovery mechanics. Runenwerk chooses product recovery/retry policy.

# G5C — Final consumer cutover

## Renderer cutover

Renderer lowering produces complete executable RunenGPU work and then uses only:

```text
GpuContext::prepare
GpuContext::submit_prepared
GpuContext::progress
GpuSubmission state/outcome
GpuReadback results
```

Renderer code no longer owns:

- command encoder creation;
- queue submission;
- raw buffer/texture copy encoding;
- WGPU timestamp/query resolve encoding;
- WGPU map/poll loops;
- raw pipeline/bind-group/resource lexical terminals.

RunenRender continues to own preparation of scene/render semantics and lowering into the generic work contracts.

## UI cutover

The existing one-pass UI batch lowers to one `GpuRenderOperation` containing multiple `GpuRenderDraw`s. Pipeline switches, bind groups, vertex buffers, and scissors become generic draw semantics, preserving one render pass.

No UI-specific execution terminal enters RunenGPU.

## Timing and capture cutover

Timing becomes generic:

```text
render timestamp writes
 -> QueryResolve operation
 -> Readback operation
 -> GpuReadbackBytes decode
```

Capture becomes generic copy/readback operations. Renderer capture policy still chooses what/when to capture and how to interpret/encode product artifacts.

## Required deletion census

Final accepted G5 must reach:

```text
CurrentRenderDeviceQueue             0
current_render_device_queue()        0
CurrentRenderExecutionBridge         0
current_render_execution_bridge()    0
renderer direct Device/Queue use     0 for G5 execution
renderer CommandEncoder creation     0 for G5 execution
renderer queue.submit calls          0
renderer map_async/poll ownership    0 for G5 readback
RenderGpuWorkSidecar execution truth 0
manual duplicated caller accesses    0 for executable bindings
```

The temporary G7 surface bridge may remain and is not counted as G5 execution reach-through.

# Error taxonomy

G5 errors remain structured by phase:

```text
GpuExecutionPreparationError
  semantic/device binding mismatch
  stale/foreign context generation
  realization failure
  transfer/readback preparation failure
  preparation pressure

GpuSubmitRejection
  stale/closed context
  structured pressure
  no submission ID allocated
  returns prepared value where retry is meaningful

GpuSubmissionFailure
  terminal backend/device/context execution failure
  forced shutdown terminalization
  invariant failure

GpuReadbackFailure
  parent submission failure
  map failure
  normalization failure
  device/context terminal failure
```

Labels and backend text are bounded diagnostics only.

# Concurrency and lock order

G5 extends the accepted single WGPU attribution gate discipline.

Rules:

1. no consumer callback/waker is invoked while a G5 registry or G4 realization lock is held;
2. acquire the shared attribution gate before backend operations that can publish WGPU errors;
3. never acquire G4 realization registry locks after a G5 completion/retirement lock in an order that can invert the realization path;
4. move terminal records out of locked state before invoking observers;
5. one submission record transitions to terminal state exactly once using compare/guarded mutation;
6. mapping callbacks publish only through generation-bound readback records;
7. shutdown uses the same state machine rather than a second cleanup path.

The G5B implementation spec must bind the exact lock-order table after the source decomposition is selected.

# Public ergonomics

Ordinary path:

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

RunenGPU intentionally does not own `host.yield_now()` or an executor. A Runenwerk app/event loop, CLI helper, or downstream runtime owns that policy.

A later ergonomic future adapter may be added only if it preserves explicit progress ownership and does not introduce an implicit runtime requirement.

# Proof matrix

G5 acceptance requires ordinary automated proof for at least:

1. complete operation-derived access equality versus formerly duplicated renderer access truth;
2. executable compute binding compatibility and exact pipeline realization;
3. multi-draw render pass with pipeline/binding/vertex/index/dynamic-state switches;
4. Upload participates in initialization/hazard order and cannot be physically reordered across conflicts;
5. Readback participates as an exact source read and captures the point-in-graph contents;
6. prepared submission rejects stale/foreign context generation before backend encoding;
7. submit pressure allocates no submission ID and preserves retryable prepared work;
8. every accepted submission transitions to exactly one terminal outcome;
9. every accepted readback transitions to exactly one Ready/Failed result;
10. submission completion and readback completion can occur at different times without misreporting either;
11. no callback/observer fires while internal locks are held;
12. drop of observation handles does not cancel/discard accepted work;
13. upload/readback/staging pressure is bounded and structurally distinguishable;
14. shutdown rejects new admission and deterministically terminalizes/drains existing records;
15. delayed retirement retains resources/pipelines until submissions/readbacks are safe;
16. native WGPU progress works through private polling without exposing WGPU polling types;
17. WebGPU-compatible design does not require `Device::poll` to have effect;
18. one independent non-render compute consumer executes through G5 before renderer cutover;
19. renderer/UI/timing/capture execute through the same G5 authority after G5C;
20. final source guards prove both temporary G5 seams and renderer raw execution are absent.

Environment-dependent adapter tests must report adapter absence separately from successful hardware execution.

# Non-goals

G5 does not add:

- a universal executor/runtime;
- a second public command graph/IR;
- a raw backend command encoder callback;
- public WGPU, Naga, mapped-range, submission-index, fence, semaphore, or native handle types;
- multi-queue scheduling;
- graph pass fusion/aliasing/reordering beyond accepted dependency semantics;
- surface acquisition/reconstruction policy;
- RunenRender materials/views/image formation;
- stable persisted execution caches;
- broad external-resource import;
- speculative unsafe/native interop;
- hardware ray tracing.

# Acceptance and transition

Planning acceptance authorizes implementation issue creation only.

After the planning PR is independently accepted:

```text
activate G5A only
G5B blocked by accepted G5A main
G5C blocked by accepted G5B main
```

Each implementation slice must begin from the exact accepted predecessor revision, prove exact-head CI and Documentation Build, and delete the predecessor authority it replaces within its accepted scope.

G6 remains blocked until G5C is accepted and the final G5 execution boundary is proven on accepted main.
