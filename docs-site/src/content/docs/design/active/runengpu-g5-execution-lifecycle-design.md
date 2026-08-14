---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete architecture for executable logical work, static bind-group realization plus dynamic execution state, bounded cancellation-safe preparation and submission, private command encoding, progress, completion, asynchronous readback, realization retention, current-host surface integration, and final renderer execution cutover.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-14
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g4-context-program-realization-design.md
  - ./runengpu-g4c2-presentation-surface-binding-boundary.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-g5-execution-lifecycle-investigation.md
  - ../../reports/investigations/2026-08-14-runengpu-g5-critical-review.md
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

G4 is accepted. This document and its implementation specifications remain planning authority only
until PR `#285` passes owner review and is accepted. G5 Rust implementation requires separately
activated child issues from the accepted predecessor revision.

Ordered delivery:

```text
G5A executable logical work closure
 -> G5B complete surface-independent execution lifecycle
 -> G5C renderer/current-host integration and final execution cutover
```

## Mission

G5 completes the future-transferable boundary from accepted logical GPU work to observable GPU
execution without creating a second command IR or moving image-formation, surface-lifecycle or
product policy into RunenGPU.

G5 owns:

- complete executable logical compute/render work;
- graph-visible Upload and Readback operations;
- one logical runtime-binding model with static physical realization and per-use dynamic state;
- device-dependent preparation through accepted G4 realizations;
- finite cancellation-safe prepared/in-flight/staging capacity;
- private command encoding and queue submission;
- backend-neutral progress and exactly-once terminal outcomes;
- normalized asynchronous readback;
- safe retention/release timing for accepted G4 realization records;
- shutdown and terminal-record detachment;
- final deletion of the two residual current-render execution seams.

G5 does not own RunenRender image formation, reusable G7 surfaces, a process-global GPU context, a
mandatory executor, a second work graph, or public raw backend/synchronization objects.

# Durable semantic spine

```text
GpuWorkOperation / GpuWorkFragment
    complete logical GPU work
          |
          v
GpuPreparedWorkGraph
    deterministic node/resource identity
    + order/access/hazard/initialization/requirements
          |
          v
GpuContext::prepare(...).await
    RAII prepared-capacity reservation
    + G4 realization
    + prepared dynamic binding-use state
          |
          v
GpuPreparedSubmission
    immutable single-use derived execution state
          |
          v
GpuContext::submit_prepared(...)
    atomic prepared -> in-flight admission
    + submission ID only after acceptance
          |
          +----------------------+
          |                      |
          v                      v
GpuSubmission                GpuReadback
GPU execution outcome        normalized CPU result
          \                      /
           \                    /
            v                  v
       progress / cleanup / G4 Arc release
            |
            v
 terminal result detaches from context capacity
```

`GpuPreparedSubmission` is derived execution state, not another semantic IR. Its correctness comes
from the accepted G3 graph, G4 contracts, immutable transfer/binding values, and one exact context/
device generation.

## One fact, one owner

- **G3:** operations, access, initialization, hazards, dependencies and prepared order.
- **G4:** context/device facts, programs, interfaces, layouts, logical resources and private
  realization registries.
- **G5:** executable closure, dynamic binding use, preparation, encoding, submission, progress,
  transfer completion, staging lifetime and safe release timing.
- **RunenRender:** why work exists and what image it intends to form.
- **current-host/G7 boundary:** physical acquired presentation-surface mechanics before reusable G7.
- **G7:** reusable surface capability, identity/generation, acquisition/presentation, loss and
  reconstruction.

# G5A — Executable logical work closure

## Operation value semantics

Simple accepted G3 operation payloads currently derive `PartialOrd/Ord/Hash`, but prepared graph
determinism comes from fragment/node/resource identity and topology, not operation comparison.

Executable G5A aggregates therefore retain semantic `PartialEq/Eq` without manufacturing a total
operation `Ord/Hash` over complex G4 pipeline/program/binding contracts. A discovered real operation
ordering/hash consumer is a stop condition for classification; labels, pointers and naked hashes are
not semantic ordering authority.

## Compute

```text
GpuComputeOperation
  GpuComputePipelineDescriptor
  GpuRuntimeBindingSet
  GpuDispatchSize
```

## Render and multi-draw

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

One render operation remains one logical render pass. Current UI proves that several compatible
pipelines/bind groups/instance buffers/scissors may switch inside one pass without splitting
load/store semantics.

### Render-pass compatibility

G5A derives one logical pass signature:

```text
effective render extent
common attachment sample count
ordered color attachment formats
optional depth/stencil format
```

Every active color/depth attachment shares effective extent/sample count. Existing G3 resolve
validation remains authoritative for matching resolve extent/format and a single-sampled resolve
target.

Every draw pipeline matches the same pass signature for ordered color-target count/formats,
depth/stencil presence/format and sample count. Blend/write/primitive state remains draw-pipeline
state and may differ.

Accepted RunenGPU has no multiview or 3D depth-slice render contract. Demonstrated demand for either
requires a separate semantic extension rather than raw WGPU passthrough.

### Vertex/index bindings

`GpuVertexBufferBinding` owns slot + logical buffer + checked range. Pipeline stride/step/attributes
remain G4 pipeline authority.

`GpuIndexBufferBinding` reuses accepted `GpuIndexFormat::{Uint16, Uint32}` and owns logical buffer +
checked range + format. G5 does not define another index-format type. Existing indirect intent owns
its argument buffer/range/indexed flag; multi-draw-indirect/mesh-shader vocabulary remains YAGNI.

### Dynamic render state

Each draw has complete effective state:

- viewport: finite canonical f32 bits, signed-zero normalized, nonnegative width/height,
  `0 <= min_depth <= max_depth <= 1`;
- scissor: checked u32 rectangle, zero area allowed, inside common render extent;
- blend constant: four finite canonical f64 values, signed-zero normalized, no invented 0..1 clamp;
- stencil reference: `u32`.

Zero-area viewport/scissor are valid. G5A does not impose device-specific viewport size/position
limits; G5B preparation validates those against admitted backend/device facts.

Defaults are full common render extent, transparent-zero blend constant and zero stencil reference.
Private encoding may elide redundant setters but semantic state is never inherited accidentally.

## Runtime binding set and dynamic offsets

`GpuRuntimeBindingSet` is the one logical binding-use aggregate for a compute invocation or render
draw. It owns complete typed G4 runtime values across the pipeline layout, including each buffer's
static offset/size and optional **u64 logical dynamic offset**.

The logical dynamic offset participates in semantic equality and in the effective buffer access
range used by G3/G5 hazard validation. It is execution-use state, not physical bind-group object
identity.

This distinction matters because accepted G4C2 currently keys bind-group realization by complete
runtime values, while WGPU bind-group creation uses only the buffer's static offset/size and applies
dynamic offsets later at `set_bind_group`. Leaving dynamic offset in the physical key would create
multiple identical backend bind groups for invocations that differ only by dynamic offset and could
consume bounded G4 registry capacity unnecessarily.

G5A therefore refines physical bind-group realization:

```text
GpuRuntimeBindingSet                 // full logical invocation semantics
       |
       +--> complete access/hazard semantics using effective offset
       |
       +--> private static bind-group projection
              layout
              resource identity
              texture/sampler facts
              buffer static offset/size
              NO dynamic offset
                    |
                    v
              GpuRealizedBindGroup   // physical backend object
```

The G4C2 single-flight key uses full equality over that static projection, never a naked hash.
Requests differing only by dynamic offset reuse one physical record. Static resource/offset/size or
layout changes still split realization.

`GpuRealizedBindGroup` remains a physical handle. The G5A clean cutover removes the public
`GpuRealizedBindGroup::values()` accessor because one shared physical record cannot truthfully own
one invocation's dynamic offsets. Logical values remain inspectable through `GpuRuntimeBindingSet`;
no compatibility alias returns a first-request or synthetic value set.

G5 keeps logical dynamic offset as u64. The pinned WGPU 27 backend uses `DynamicOffset = u32`, so
G5B preparation performs a checked u64->u32 conversion after logical alignment/effective-range
validation. Values not representable by the backend fail structurally before encoding; the reusable
logical contract is not globally narrowed merely for the first backend.

For each bind-group slot, prepared private execution state contains:

```text
GpuRealizedBindGroup
+ ordered checked backend dynamic-offset slice
```

The slice contains exactly one value per dynamic buffer declaration in canonical layout binding
order. Private compute/render encoding passes it to each `set_bind_group`. Dynamic offsets are never
recovered from the realized record or a renderer side channel.

## Complete operation-derived access truth

After G5A:

- Compute derives runtime binding reads/writes using effective dynamic ranges;
- Render derives attachment/resolve/binding/vertex/index/indirect/timestamp accesses;
- Copy/Clear/Resolve/Present retain accepted G3 semantics;
- Upload derives one exact destination write;
- Readback derives one exact source read.

The complete derived set is checked for incompatible overlap, including bindings versus attachments
and multiple write-capable bindings. Renderer `lower_caller_accesses` duplication is deleted.

## Transfer payload value versus record identity

```text
GpuTransferPayloadId
    opaque process-local record/correlation identity

GpuTransferPayload
    id
    immutable Arc-backed checked transfer value
```

Semantic `PartialEq/Eq` compares checked payload value and excludes record ID. Clones preserve ID
and value; `is_same_record`-style inspection distinguishes one shared record from equal independent
records. Operation Hash/Ord is not required, so scheduling does not hash/sort upload bytes.

Payload ID is not content/persistence/replay/wire/cache/cross-process identity; any future digest is
diagnostics/dedup evidence only.

## Upload and Readback

`GpuWorkOperation::Upload` is one exact graph-visible destination write and participates in G3
initialization/hazard order. Queue-write lowering is allowed only when proven equivalent to the
exact graph position; private staging/copy is required otherwise.

`GpuWorkOperation::Readback` is one exact graph-visible source read plus process-local
`GpuReadbackId`. Private staging/mapping remains hidden; texture results normalize to tight logical
rows in `GpuReadbackBytes`. CPU feedback cannot affect later nodes in the same submission.

## Present and SurfaceAcquired resources

`GpuPresentOperation` remains logical order/presentation intent, not a command-encoder operation.
`SurfaceAcquired` resources remain outside ordinary G4C1 realization and outside sampled/storage
shader binding. Their accepted attachment/copy/present roles remain typed requirements for the
pre-G7 current-host integration.

## Sidecar deletion boundary

`RenderGpuWorkSidecar`/`CompiledPassExecutionPlan` stop owning GPU execution semantics in G5A.
RunenRender may retain planning/provenance before lowering, but complete generic execution meaning
exists in the G3/G5 work graph afterward. No permanent execution companion or second command list
survives.

# G5B — Complete surface-independent execution lifecycle

## Finite capacity

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

Prepared capacity counts active async prepare reservations plus published prepared submissions.
There is no duplicate G5 realization-retirement budget and no context-owned terminal-history
budget.

## Cancellation-safe preparation

`GpuContext::prepare` reserves one prepared slot through RAII before asynchronous realization,
validates complete logical/device facts, obtains exact G4 realizations, prepares each physical
bind-group use plus its checked ordered dynamic-offset slice, records unresolved SurfaceAcquired
requirements, plans transfer staging and publishes one immutable `GpuPreparedSubmission`.

Error, cancellation/drop, shutdown/stale generation or abandoned owned realization releases owned
reservations. No partial prepared value or submission ID appears.

For WGPU, dynamic offset preparation additionally verifies that each logical u64 offset is aligned,
within the effective logical buffer range, representable as u32, and ordered exactly by the bind-
group layout's dynamic buffer declarations.

## Submit admission and irreversible acceptance

Reusable submit rejects before acceptance for stale/closed state, already-consumed work, pressure or
unresolved `SurfaceAcquired` requirements. Pre-acceptance rejection performs no queue action, no
submission-ID allocation and preserves retryable prepared work.

Successful admission atomically converts prepared capacity to in-flight/staging capacity and then
allocates a nonzero monotonic process-local `GpuSubmissionId`.

**ID allocation is irreversible semantic acceptance.** Any later encoding/backend/queue/health/
device failure terminalizes that accepted ID once. There is no hidden accepted-but-unsubmitted
queue.

## Private encoding

G5B privately encodes ordinary realized compute/render/copy/clear/query/Upload/Readback work.
At every compute/render `set_bind_group`, it passes the prepared physical bind group plus that
invocation's ordered checked dynamic-offset slice. The backend bind group itself does not contain
those offsets.

`Device`, `Queue`, `CommandEncoder`, WGPU pipelines/bind groups, `DynamicOffset`, mapped ranges,
submission indices and poll types remain private. G5B does not acquire/bind/present SurfaceAcquired
resources; reusable submit returns `SurfaceBindingRequired` when they remain unresolved.

## Submission/readback, progress and shutdown

```text
submission: Submitted -> Completed | Failed
readback:   Pending   -> Ready | Failed
```

Submission GPU completion and readback materialization remain distinct. Every accepted record
terminalizes exactly once; observer drop is not cancellation.

Portable progress is `GpuContext::progress()` plus nonblocking observation. Native implementation
may privately poll WGPU; browser/WebGPU progress remains event-loop driven. RunenGPU owns no
mandatory executor/thread/background progress service.

Execution state is `Running -> ShuttingDown -> Closed`; shutdown rejects new admission while
`progress()` remains valid for accepted lifecycle cleanup.

## G4 realization retention

G5 retains exact G4 `Arc<Record>` handles required by active prepare/prepared/in-flight/readback
state. Releasing those references when safe returns reclamation authority to existing bounded G4
registries. No second resource/program/pipeline/bind-group retirement registry exists.

Terminal immutable result state detaches from context execution capacity after safe staging/G4-ref
cleanup; caller observation handles may then live independently without backpressuring future work.

## Independent proof

G5B proves the reusable boundary first with a non-render, surface-independent compute/readback
workload. It must include at least one dynamic buffer binding executed with two distinct valid
offsets that reuse one physical bind-group record while producing correct per-use results.

# G5C — Renderer/current-host integration and final cutover

## Current-host surface composition

Accepted G4 keeps acquired presentation resources outside ordinary G4C1 realization while current
rendering still needs accepted attachment/copy/present roles. G5C removes renderer raw execution
without pretending reusable G7 already exists.

No public/reusable surface API or third generic bridge is introduced. For current-host surface work:

```text
prepared work with SurfaceAcquired requirements
 -> reserve G5 submit capacity; no ID
 -> resolve logical SurfaceAcquired IDs to current-host slots
 -> acquire all required leases in deterministic logical-ID order
 -> validate affinity + identity + format/extent + allowed role
 -> commit and allocate GpuSubmissionId
 -> private G5 encode / Queue::submit
 -> current-host owner may attempt SurfaceTexture::present
```

Pre-commit failure drops already-acquired leases without present, releases provisional capacity,
preserves prepared work and creates no ID.

Attachment use receives only the matching acquired `TextureView`; copy receives only the matching
acquired `Texture`, both lexically in the private current-host/WGPU integration. They never become
logical work, G4 realization, renderer authority or generic callback data. Sampled/storage use stays
forbidden.

If encoding fails after ID commit but before `Queue::submit` returns, the accepted submission fails
and no present is attempted. After `Queue::submit` returns, current-host may attempt present. That
attempt is not GPU-completion or display/compositor-completion evidence; a later asynchronous GPU
failure and the earlier present attempt remain separate facts.

This owner-local terminal is explicit G7 deletion inventory.

## Consumer cutover and deletion census

Renderer/UI/timing/capture migrate onto accepted G5:

- renderer no longer creates command encoders or submits queues;
- UI remains one compatible multi-draw render pass;
- timing uses timestamp writes -> resolve -> readback -> policy decode;
- capture uses copy/readback -> normalized bytes -> Runenwerk artifact policy;
- staged uploads become graph-visible Upload work;
- ordinary primitive/examples/benches use typed G5.

At G5C acceptance:

```text
CurrentRenderDeviceQueue                0
current_render_device_queue()           0
CurrentRenderExecutionBridge            0
current_render_execution_bridge()       0
renderer Device/Queue for G5 execution  0
renderer CommandEncoder creation        0
renderer queue.submit                    0
renderer map_async/Device::poll owner   0 for G5 readback
renderer raw acquired surface handoff   0
RenderGpuWorkSidecar execution truth    0
manual duplicate executable accesses    0
```

Raw current-host surface mechanics may remain only inside the separately classified pre-G7 owner
and private lexical WGPU integration.

# Public ergonomics

```rust
let prepared = gpu.prepare("frame 42", work).await?;
let submission = gpu.submit_prepared(prepared)?;

while submission.try_outcome().is_none() {
    gpu.progress()?;
    host.yield_now();
}
```

Readback follows the same lifecycle through `submission.readback(id)` and `try_result()`. RunenGPU
does not own `host.yield_now()` or the executor/event loop.

# Error taxonomy

```text
GpuExecutionPreparationError
  semantic/device binding mismatch
  dynamic-offset alignment/backend representability mismatch
  render attachment/pipeline incompatibility
  stale/foreign generation
  realization failure
  prepared-capacity pressure

GpuSubmitRejection
  stale/closed context
  in-flight/staging pressure
  SurfaceBindingRequired
  no submission ID allocated

GpuSubmissionFailure
  failure after accepted submission-ID commit
  backend/device/context execution failure
  forced shutdown terminalization
  invariant failure

GpuReadbackFailure
  parent submission failure
  map/normalization failure
  device/context terminal failure
```

# Proof matrix

Implementation specifications prove at least:

1. operation-derived access truth replaces renderer duplicate access truth;
2. graph determinism does not depend on fabricated operation Ord/Hash;
3. transfer payload equality is distinct from record identity;
4. render attachments share extent/sample count and every draw pipeline matches pass signature;
5. zero-area viewport/scissor are valid and float state has canonical equality;
6. dynamic logical buffer offsets affect effective access ranges but do not split physical bind-group realization;
7. WGPU dynamic offsets are checked for u32 representability and encoded in canonical binding order;
8. no public realized-bind-group accessor conflates one physical record with one invocation's dynamic values;
9. compute and compatible multi-draw render execution are complete;
10. Upload participates in initialization/hazards and queue-write lowering preserves graph order;
11. Readback occurs at an exact graph point and strips backend padding;
12. active prepare plus published prepared capacity is finite/cancellation-safe;
13. pre-ID rejection preserves work and every post-ID failure terminalizes exactly once;
14. submission completion and readback completion may differ;
15. terminal observations detach from execution capacity after safe cleanup;
16. G5-held G4 Arcs prevent premature collection without a second retirement registry;
17. callbacks/wakers execute outside internal locks;
18. shutdown resolves accepted lifecycle state deterministically;
19. native/WebGPU-compatible progress exposes no backend poll/fence type;
20. independent G5B proof uses no renderer/surface escape and exercises dynamic offsets;
21. current-host pre-commit surface failure creates no submission and leaks no lease/capacity;
22. surface attachment/copy binding is exact and sampled/storage use stays rejected;
23. GPU outcome and current-host present attempt remain separate facts;
24. representative renderer/UI/timing/capture use the same G5 authority after G5C;
25. final source guards prove temporary bridges and renderer raw execution are absent.

# Non-goals

No universal executor, second command IR, permanent execution sidecar, public WGPU/Naga/native
objects, second public dynamic-offset channel, fabricated program/pipeline ordering, content-addressed
transfer identity, second realization-retirement registry, global terminal-history ledger,
multi-queue scheduler, speculative graph optimizer, multiview/3D depth-slice/multi-draw/mesh-shader
extension without demand, reusable G7 surface model, RunenRender image semantics, broad import API or
speculative native interop is authorized.

# Acceptance and transition

After independent **owner review and planning acceptance**:

```text
activate G5A only
G5B blocked by accepted G5A main
G5C blocked by accepted G5B main
G6 blocked by accepted G5C main
```

Each implementation slice starts from the exact accepted predecessor revision and deletes
superseded authority rather than carrying compatibility aliases forward.