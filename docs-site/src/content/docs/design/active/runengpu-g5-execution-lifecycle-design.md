---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete architecture for executable logical work, bounded cancellation-safe preparation and submission, private command encoding, progress, completion, asynchronous readback, realization retention, current-host surface integration, and final renderer execution cutover.
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
- device-dependent preparation through accepted G4 realizations;
- finite cancellation-safe prepared/in-flight/staging capacity;
- private command encoding and queue submission;
- backend-neutral progress and exactly-once terminal outcomes;
- normalized asynchronous readback;
- safe retention/release timing for accepted G4 realization records;
- shutdown and terminal-record detachment;
- final deletion of the two residual current-render execution seams.

G5 does not own:

- RunenRender image formation or product policy;
- reusable G7 surface capability/identity/generation/loss/reconstruction;
- a process-global GPU context;
- a mandatory executor/thread pool/background progress service;
- a second work graph/command IR;
- public WGPU/native synchronization or raw backend objects.

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
    one RAII prepared-capacity reservation
    + device-dependent G4 realization
          |
          v
GpuPreparedSubmission
    immutable single-use derived execution state
    context/device-generation bound
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
from the accepted G3 graph, G4 contracts, immutable transfer payloads, and one exact context/device
generation.

## One fact, one owner

- **G3:** operations, access, initialization, hazards, dependencies and prepared order.
- **G4:** context/device facts, programs, interfaces, layouts, logical resources and private
  realization registries.
- **G5:** executable closure, preparation, encoding, submission, progress, transfer completion,
  staging lifetime and safe release timing.
- **RunenRender:** why work exists and what image it intends to form.
- **current-host/G7 boundary:** physical acquired presentation-surface mechanics before reusable G7.
- **G7:** reusable surface capability, identity/generation, acquisition/presentation, loss and
  reconstruction.

# G5A — Executable logical work closure

## Operation value semantics

Simple accepted G3 operation payloads currently derive `PartialOrd/Ord/Hash`, but prepared graph
determinism does not depend on operation comparison. It comes from fragment/node/resource
identities and dependency topology.

Executable G5A operation aggregates therefore retain semantic `PartialEq/Eq` without manufacturing
operation-level `PartialOrd/Ord/Hash` over complex G4 pipeline/program contracts.

Do not add a fake total order through labels, pointers, naked hashes or backend object addresses.
If implementation finds a real accepted operation-order/hash consumer, stop and classify it.

## Compute

```text
GpuComputeOperation
  GpuComputePipelineDescriptor
  GpuRuntimeBindingSet
  GpuDispatchSize
```

The runtime binding set reuses accepted G4B typed values. `GpuRuntimeBufferBinding` already owns
logical buffer, static offset/size and optional dynamic offset; G5 adds no second offset channel.

Compute resource accesses derive from one accepted interface/runtime-binding authority. Existing
G4B storage-buffer `ReadOnly/ReadWrite` and storage-texture `ReadOnly/WriteOnly/ReadWrite` modes
carry enough semantic information to replace renderer-authored duplicate bound-resource access
lists.

## Render and multi-draw

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

This is required by the current UI path, which switches pipelines, bind groups, instance buffers
and scissors inside one render pass. Splitting those draws into separate graph nodes would change
pass/load-store semantics and add avoidable backend work.

### Render-pass compatibility

G5A derives one logical pass signature from the operation attachments:

```text
effective render extent
common attachment sample count
ordered color attachment formats
optional depth/stencil format
```

Every active color/depth attachment in the pass must have the same effective render extent and
sample count. Existing G3 resolve validation still requires matching resolve extent/format and a
single-sampled resolve target.

Every `GpuRenderDraw` pipeline must match the same pass signature exactly for:

- ordered color-target count/formats;
- depth/stencil presence and format;
- multisample sample count.

Pipeline blend/write/primitive state may differ between draws because it remains draw-pipeline
state rather than render-pass compatibility state.

The accepted G4 render model has no multiview or 3D depth-slice contract. G5A does not invent one.
A demonstrated current requirement for either is a separate semantic-extension stop condition,
not permission to forward WGPU types.

### Vertex/index bindings

`GpuVertexBufferBinding` owns slot + logical buffer + checked range. Pipeline stride, step mode and
attributes remain G4 pipeline-descriptor authority.

`GpuIndexBufferBinding` reuses the already accepted RunenGPU
`GpuIndexFormat::{Uint16, Uint32}`; G5 does not define another index-format enum. It owns logical
buffer + checked range + index format and validates usage/alignment/direct indexed range coverage
where knowable without reading GPU-produced contents.

Existing indirect draw intent remains authority for its argument buffer/range/indexed flag.
Multi-draw-indirect and mesh-shader vocabulary remain outside G5A until a consumer requires them.

### Dynamic state

Each draw has complete effective dynamic state rather than accidentally inheriting a previous draw:

```text
optional viewport override
optional scissor override
optional blend-constant override
explicit stencil reference
```

Normalized value rules:

- **viewport:** finite canonical f32-bit x/y/width/height/min/max depth, signed-zero normalized,
  nonnegative width/height, `0 <= min_depth <= max_depth <= 1`;
- **scissor:** checked u32 x/y/width/height, zero area allowed, checked arithmetic, rectangle inside
  the common logical render extent;
- **blend constant:** four finite canonical f64 components with signed-zero normalization; no
  invented 0..1 clamp;
- **stencil reference:** `u32`.

Zero-area viewport/scissor are valid no-rasterization state. G5A does not require viewport x/y to
lie within the attachment and does not encode backend device limits. G5B preparation checks the
pinned backend's maximum viewport size/position rules against admitted device facts.

Defaults are full common render extent for viewport/scissor, transparent-zero blend constant and
zero stencil reference. The private encoder may elide redundant setter calls without changing
semantic state.

## Complete operation-derived access truth

After G5A:

- Compute derives runtime binding reads/writes;
- Render derives attachment/resolve/binding/vertex/index/indirect/timestamp accesses;
- Copy/Clear/Resolve/Present retain accepted G3 semantics;
- Upload derives one exact destination write;
- Readback derives one exact source read.

Construction validates the complete derived access set for incompatible overlapping roles,
including runtime bindings versus attachments and multiple write-capable bindings. Renderer
`lower_caller_accesses` duplication is deleted.

## Transfer payload value versus record identity

```text
GpuTransferPayloadId
    opaque process-local record/correlation identity

GpuTransferPayload
    id
    immutable Arc-backed checked transfer value
```

Rules:

- clones preserve record ID and immutable data;
- one ID is never rebound to another payload;
- semantic `PartialEq/Eq` compares checked payload value semantics and excludes the ID;
- independently constructed byte-identical payloads remain semantically equal;
- an `is_same_record`-style predicate distinguishes clones of one record from equal independent
  records;
- operation-level Hash/Ord is not required, so graph scheduling never needs to hash/sort upload
  bytes;
- payload ID is not persistence/content/replay/wire/cache/cross-process identity;
- any future digest is diagnostics/dedup evidence only.

## Upload

`GpuWorkOperation::Upload` is one exact graph-visible destination write. It can satisfy G3
initialization and participates in ordinary hazard/order semantics.

Physical lowering is private G5B policy. Queue write is allowed only when proven equivalent to the
exact graph node position; otherwise private staging plus an encoded copy preserves order.

## Readback

`GpuWorkOperation::Readback` is one exact graph-visible source read plus opaque process-local
`GpuReadbackId`.

Private staging/mapping does not enter logical work. Texture results normalize to tightly packed
logical rows in `GpuReadbackBytes`; mapped ranges/backend row padding never escape.

CPU feedback cannot affect later nodes in the same submission. Consumers observe a result before
constructing later work.

## Present and SurfaceAcquired logical resources

`GpuPresentOperation` remains logical order/presentation intent, not a command-encoder operation.

`SurfaceAcquired` resources remain distinct from ordinary G4C1 realizations. G5A does not create an
import surrogate or shader sampled/storage exception. Their already-accepted attachment/copy/
present roles remain typed logical requirements for the pre-G7 current-host integration.

## Sidecar deletion boundary

`RenderGpuWorkSidecar`/`CompiledPassExecutionPlan` stop owning GPU execution semantics in G5A.
RunenRender may retain planning/provenance before lowering, but complete generic execution meaning
exists in the G3/G5 work graph afterward.

No permanent `GpuExecutionBindings`, second command list or prepared-node execution companion is
introduced.

# G5B — Complete surface-independent execution lifecycle

## Finite execution capacity

`GpuExecutionLimits` contains at least:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

`max_prepared_submissions` counts **active asynchronous prepare reservations plus published
prepared submissions**. Waiting on G4 realization cannot bypass pressure.

There is no duplicate G5 realization-retirement budget and no context-wide terminal-history
budget.

## Cancellation-safe preparation

`GpuContext::prepare`:

1. reserves one prepared slot through RAII before asynchronous realization;
2. checks Running state and exact context/device generation;
3. validates complete executable graph semantics and device-dependent facts;
4. obtains exact accepted G4 realization Arcs for ordinary resources/programs/bindings/pipelines;
5. records unresolved `SurfaceAcquired` attachment/copy/present requirements without raw surface
   objects;
6. plans bounded upload/readback staging needs;
7. publishes one immutable `GpuPreparedSubmission`.

Normal error, future cancellation/drop, shutdown/stale-generation detection or abandoned owned G4
single-flight realization releases owned reservations. No partial prepared value or submission ID
is published.

A published prepared value retains the same prepared-capacity slot until drop or successful submit
conversion.

## Submit admission and irreversible acceptance

Reusable `submit_prepared` rejects before acceptance for stale/closed state, already-consumed
prepared work, in-flight/staging pressure, or unresolved `SurfaceAcquired` requirements.

Pre-acceptance rejection:

```text
no queue operation
no GpuSubmissionId
no in-flight slot
prepared semantics/slot preserved when retry is meaningful
```

Successful admission atomically converts prepared capacity to in-flight/staging capacity and only
then allocates a nonzero monotonic process-local `GpuSubmissionId`.

**Submission-ID allocation is the irreversible semantic acceptance point.** Any later encoding,
backend validation, queue, health or device failure terminalizes that exact accepted ID once; it
never rolls back into a submit rejection.

There is no hidden accepted-but-unsubmitted queue.

## Private surface-independent encoding

G5B privately encodes ordinary realized:

- compute;
- render;
- copy/clear/query resolve;
- Upload;
- Readback staging/copy.

The encoder receives retained G4 records, not renderer raw callbacks. `Device`, `Queue`,
`CommandEncoder`, WGPU pipelines/bind groups/resources, mapped ranges, submission indices and poll
objects remain private.

G5B does not acquire/bind/present `SurfaceAcquired` resources. Ordinary submit returns typed
`SurfaceBindingRequired` and preserves the prepared value when such requirements remain unresolved.

## Submission and readback state

```text
submission: Submitted -> Completed | Failed
readback:   Pending   -> Ready | Failed
```

Submission GPU completion and readback mapping/materialization are separate facts. One may be
`Completed` while a dependent readback remains `Pending`.

Every accepted submission/readback terminalizes exactly once. Parent submission failure
terminalizes dependent readbacks when their result can no longer be produced. Observer drop is not
execution cancellation.

## Progress

Portable baseline:

```text
GpuContext::progress() -> GpuProgressReport
GpuSubmission::try_outcome()
GpuReadback::try_result()
```

Native private implementation may perform nonblocking WGPU polling. Browser/WebGPU progress remains
browser/event-loop driven; `progress()` drains/publishes callback state. Public G5 contains no WGPU
poll/fence promise or universal blocking wait.

RunenGPU owns no mandatory Tokio/Futures executor, thread pool or immortal progress thread.
Optional Future/callback adapters wrap the same records and invoke observers outside locks.

## G4 realization retention, not another retirement owner

Accepted G4 realized handles already retain `Arc<Record>`, and G4 registries own bounded lookup/
cache capacity plus lookup-only collection.

G5 delays reclamation simply by retaining the exact G4 Arcs required by active prepare/prepared/
in-flight/readback state. When execution/readback safety allows, G5 drops those references; G4
remains the sole realization lookup/capacity/collection authority.

Private G5 staging is owned directly by the in-flight/readback record using it and is bounded by
upload/readback/count pressure.

No second resource/program/pipeline retirement registry, duplicate realization identity or
`max_deferred_retirement_records` counter exists.

## Terminal record detachment

After one terminal immutable result is published and private staging/G4-reference cleanup is safe:

```text
context registry reference -> detached
caller observation Arc     -> may remain
```

A caller may retain terminal results without consuming future execution capacity. If observers
vanish early, the context still terminalizes/cleans accepted work and then discards the internal
record rather than retaining hidden history.

## Health, shutdown and lock order

The accepted G4 WGPU device-loss/uncaptured-error/error-attribution authority remains the sole
health owner.

Execution state is:

```text
Running -> ShuttingDown -> Closed
```

Shutdown rejects new prepare/submit admission, prevents in-progress prepare publication, leaves
`progress()` valid while accepted work/readbacks resolve, and does not fabricate cancellation for
already accepted work.

Concrete implementation must preserve:

1. shared backend error-attribution gate outside attributable backend operations;
2. G4 realization acquisition before published G5 lifecycle-record insertion;
3. no inverse G5-lifecycle-lock -> G4-registry lock path;
4. RAII prepared-slot Drop without inverse locking;
5. encoding over already-retained records;
6. callbacks updating one record, moving notification/detachment work out, unlocking, then
   invoking observers;
7. shutdown through the same state machine rather than a parallel cleanup hierarchy.

## Independent proof

G5B proves the reusable boundary first with one non-render, surface-independent compute workload
using typed prepare/submit/progress/readback and exact deterministic output. It cannot use
RunenRender, current-render bridges or raw WGPU escape hatches.

# G5C — Renderer/current-host integration and final cutover

## Hidden surface problem resolved

Accepted G4 intentionally keeps acquired presentation surfaces outside ordinary G4C1 realization,
while current rendering still uses them for already-existing attachment/copy/present roles. G5C
must therefore eliminate renderer raw execution without pretending reusable G7 already exists.

## Owner-local current-host surface composition

No new public/reusable surface API, broad import path or third generic object-reference bridge is
introduced.

For current-host surface-backed prepared work:

```text
prepared work with typed SurfaceAcquired requirements
        |
        v
reserve G5 submit capacity
(no submission ID; reservation cannot escape)
        |
        v
resolve logical SurfaceAcquired IDs -> current-host slots
acquire all required leases in deterministic logical-ID order
        |
        v
validate complete binding set
context affinity
logical identity
configured format/extent
allowed attachment/copy/present role
        |
        v
commit the same G5 submit transaction
allocate GpuSubmissionId
        |
        v
private G5 WGPU encoding / Queue::submit
        |
        v
current-host owner may attempt SurfaceTexture::present
```

Any pre-commit acquisition/binding failure drops already acquired leases without present, releases
provisional capacity, preserves/returns prepared work and creates no submission ID.

After commit, any later encoding/queue/device failure terminalizes the accepted submission once.
If encoding fails before `Queue::submit` returns, acquired leases are dropped without present.

For attachment use the private encoder receives only the matching acquired `TextureView`; for copy
source/destination it receives only the matching acquired `Texture`. Raw surface objects are
lexical owner-local values: they are not stored in logical work, cached as G4 realization, returned
to renderer code, persisted or passed through a generic callback. Sampled/storage shader use stays
forbidden before G7.

After `Queue::submit` returns, the current-host owner may perform its existing `present()` attempt.
That attempt is not evidence that `GpuSubmission` has completed and not evidence of compositor/
display completion. An asynchronous device/execution failure may be observed after a present
attempt; G5 execution outcome and pre-G7 presentation-attempt fact remain separate and do not
retroactively rewrite each other.

Renderer-specific `RenderSurfaceId` does not enter reusable RunenGPU contracts; the current-host
owner maps logical `SurfaceAcquired` identities to its own surface slots.

This owner-local terminal is explicit G7 deletion inventory. G7 later replaces it with reusable
surface capability/generation/acquisition/loss/recovery contracts.

## Consumer cutover

Renderer/UI/timing/capture migrate onto the same accepted G5 authority:

- renderer stops creating command encoders and submitting queues;
- UI lowers one multi-draw render pass with compatible pipelines without splitting it;
- timing becomes timestamp writes -> query resolve -> readback -> policy decode;
- capture becomes copy/readback -> normalized bytes -> Runenwerk artifact policy;
- staged uploads become graph-visible Upload operations;
- primitive/examples/benches use typed G5 when ordinary execution is the subject.

RunenRender keeps image-formation/planning semantics. Runenwerk keeps event-loop/yield/blocking,
shader discovery/hot reload, product recovery and artifact persistence/presentation policy.

## Final deletion census

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

Raw current-host surface mechanics may remain only in the separately classified pre-G7 owner and
private WGPU lexical integration above. That exception cannot become a generic G5 execution/
resource escape.

# Public ergonomics

Surface-independent execution:

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

Prepared work requiring current-host surfaces intentionally cannot use the reusable surface-
independent submit terminal before G7; G5C supplies that binding privately for the current host.

# Error taxonomy

```text
GpuExecutionPreparationError
  semantic/device binding mismatch
  render attachment/pipeline incompatibility
  stale/foreign generation
  realization failure
  prepared-capacity pressure

GpuSubmitRejection
  stale/closed context
  in-flight/staging pressure
  SurfaceBindingRequired
  no submission ID allocated
  prepared work preserved where retryable

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

Current-host surface acquisition/configuration/binding failures before commit remain current-host/
pre-G7 surface failures and create no G5 submission.

# Proof matrix

Implementation specifications must prove at least:

1. operation-derived access truth matches superseded renderer duplicate access truth;
2. graph determinism does not depend on fabricated operation Ord/Hash;
3. transfer payload semantic equality is distinct from record identity;
4. render attachments have one common extent/sample count;
5. every draw pipeline exactly matches pass color/depth/sample compatibility;
6. zero-area viewport/scissor are valid while float state has canonical finite equality;
7. device-specific viewport limit/position validation occurs during preparation;
8. compute pipeline/binding/dispatch execution is complete;
9. one render pass supports multi-draw pipeline/binding/vertex/index/dynamic-state switches;
10. Upload participates in initialization/hazards and queue-write lowering never violates graph order;
11. Readback participates at an exact graph point and normalizes backend padding privately;
12. active prepare plus published prepared capacity is finite and cancellation-safe;
13. submit pressure/surface-binding rejection creates no submission ID and preserves prepared work;
14. every post-ID failure terminalizes the accepted submission exactly once;
15. submission completion and readback completion may differ;
16. terminal observations detach from execution capacity after safe cleanup;
17. G5-held G4 realization Arcs prevent premature collection and no second retirement registry exists;
18. callbacks/wakers execute outside internal locks;
19. observer drop does not cancel/discard accepted work;
20. shutdown rejects new work and resolves accepted lifecycle records deterministically;
21. native progress works without public WGPU polling types;
22. WebGPU-compatible semantics do not depend on `Device::poll` having effect;
23. G5B independent non-render proof uses no renderer/surface escape;
24. current-host pre-commit surface failure creates no submission and leaks no capacity/lease;
25. surface binding validates exact logical identity/affinity/format/extent/allowed role;
26. attachment uses only private acquired view and copy uses only private acquired texture;
27. sampled/storage `SurfaceAcquired` use remains rejected;
28. encoding failure before queue submit creates Failed and no present attempt;
29. post-submit async failure and current-host present attempt remain separate facts;
30. representative renderer/UI/timing/capture use the same G5 authority after G5C;
31. final source guards prove both temporary execution seams and renderer raw execution are absent.

Environment-dependent proofs report unavailable adapter/surface/capability separately from
successful hardware execution.

# Non-goals

G5 does not add:

- a universal executor/runtime;
- a second command graph/IR;
- a permanent execution sidecar;
- public WGPU/Naga/mapped-range/submission-index/fence/poll/native-handle/surface types;
- fabricated ordering over program/pipeline semantics;
- content-addressed transfer identity;
- a second realization-retirement registry;
- a context-owned terminal-history ledger;
- multi-queue scheduling;
- aggressive graph optimization/pass fusion;
- multiview or 3D depth-slice rendering without a separate accepted extension;
- multi-draw-indirect or mesh-shader semantics without demonstrated demand;
- reusable G7 surface identity/generation/capability/recovery;
- RunenRender materials/views/image formation;
- stable persisted execution caches;
- broad external-resource import;
- speculative native interop;
- hardware ray tracing.

# Acceptance and transition

After independent **owner review and planning acceptance**:

```text
activate G5A only
G5B blocked by accepted G5A main
G5C blocked by accepted G5B main
G6 blocked by accepted G5C main
```

Each implementation slice starts from the exact accepted predecessor revision, uses exact-head
validation, and deletes predecessor authority inside its accepted scope rather than carrying
compatibility aliases forward.