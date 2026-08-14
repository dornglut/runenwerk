---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete architecture for executable logical work, bounded cancellation-safe preparation and submission, private command encoding, progress, completion, asynchronous readback, realization retention, and final renderer execution cutover.
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

G4 is accepted. This document and its specifications are planning authority only until the
planning PR is independently owner-reviewed and accepted. G5 Rust implementation requires
separately activated implementation issues.

The ordered G5 program remains:

```text
G5A executable logical work closure
 -> G5B complete surface-independent execution lifecycle
 -> G5C renderer/current-host integration and final execution cutover
```

## Mission

G5 completes the future-transferable boundary from accepted logical GPU work to observable GPU
execution without creating a second command IR or moving rendering/surface/product policy into
RunenGPU.

It owns:

- complete executable logical compute/render work contracts;
- graph-visible host-to-GPU Upload and GPU-to-host Readback operations;
- device-dependent preparation into private realized execution records;
- bounded cancellation-safe preparation and submit admission;
- private command encoding and queue submission;
- backend-neutral progress and exactly-once terminal outcomes;
- normalized asynchronous readback;
- safe retention/release of accepted G4 realization records;
- shutdown and terminal-record detachment;
- final deletion of the two residual current-render execution seams.

It does **not** own RunenRender image formation, reusable G7 surfaces, product recovery, a task
executor, process-global context, a second work graph, or public backend synchronization objects.

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
       progress / cleanup / Arc release
            |
            v
  terminal result detaches from context capacity
```

`GpuPreparedSubmission` is not another semantic IR. Its correctness derives from the prepared
G3 graph, exact G4 contracts, immutable transfer payloads, and one context/device generation.

## One fact, one owner

- **G3:** operation/access/order/initialization/dependency semantics.
- **G4:** program/interface/layout/pipeline/resource contracts and private realization registries.
- **G5:** executable closure, execution preparation, encoding, submission, progress, transfer
  completion, staging lifetime, and safe release timing.
- **RunenRender:** why work exists and what image it intends to form.
- **current-host/G7 boundary:** acquired presentation surface mechanics before reusable G7 exists.
- **G7:** reusable surface capability, identity/generation, acquisition/presentation, loss and
  reconstruction.

No G5 type may re-encode renderer pass identity, image-formation semantics, WGPU objects, backend
fences, or a duplicate resource-access graph.

# G5A — Executable logical work closure

## Operation value semantics

Accepted G3 simple operation types happened to derive `PartialOrd/Ord/Hash`. Those traits are not
part of graph ordering authority: prepared graph determinism comes from fragment/node/resource
identities and dependency topology.

G5A therefore binds:

```text
executable GpuWorkOperation aggregates
    semantic PartialEq / Eq
    no fabricated operation-level PartialOrd / Ord / Hash
```

Do not add total ordering to G4 pipeline/program/runtime-binding contracts merely to preserve an
accidental derive. Lower-level IDs, ranges and scalar enums may retain natural ordering where it
has an independent use.

If implementation finds a concrete accepted consumer that truly requires operation ordering or
hashing, stop and classify that use. Labels, pointers, naked hashes and backend object addresses
cannot become semantic ordering authority.

## Compute

A compute operation becomes logically executable:

```text
GpuComputeOperation
  GpuComputePipelineDescriptor
  GpuRuntimeBindingSet
  GpuDispatchSize
```

The runtime binding set reuses accepted G4B typed values. `GpuRuntimeBufferBinding` already owns
logical buffer, static offset/size and optional dynamic offset; G5 introduces no parallel dynamic
offset channel.

Compute accesses derive from G4 interface declarations plus exact runtime resources. Accepted G4B
storage-buffer `ReadOnly/ReadWrite` and storage-texture `ReadOnly/WriteOnly/ReadWrite` modes are
sufficient to derive bound-resource hazards without a renderer-authored duplicate access list.

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

This is required by the current UI path, which switches pipelines, bind groups, instance buffers
and scissors inside one render pass. Splitting each draw into a graph node would change pass/load-
store semantics and add backend work.

### Vertex and index bindings

`GpuVertexBufferBinding` owns slot + logical buffer + checked range. Pipeline stride, step mode and
attributes remain pipeline-descriptor authority.

`GpuIndexBufferBinding` owns logical buffer + checked range + RunenGPU
`GpuIndexFormat::{Uint16, Uint32}`. Direct/indexed draw ranges and index element alignment are
validated where knowable without reading GPU-produced contents. Existing indirect intent remains
the authority for its argument buffer/range/indexed flag; G5A does not speculate multi-draw or
mesh-shader vocabulary.

### Dynamic state

Dynamic state is complete per draw rather than inherited accidentally:

```text
optional viewport override
optional scissor override
optional blend-constant override
explicit stencil reference
```

Normalized values are:

- `GpuViewport`: finite canonical f32 bits, signed-zero normalization, positive width/height,
  checked `0 <= min_depth <= max_depth <= 1`;
- `GpuScissorRect`: checked integer x/y/width/height with positive extent and attachment bounds;
- `GpuBlendConstant`: four finite canonical f64 components with signed-zero normalization;
- stencil reference: `u32`.

No zero-to-one blend-constant clamp is invented unless a later accepted backend-neutral rule
requires it. Defaults are full attachment viewport/scissor, transparent-zero blend constant, and
zero stencil reference. Private lowering may skip redundant setters, but semantic state is still
complete.

## Operation-derived access truth

After G5A the operation is the one executable access owner:

- Compute: runtime binding reads/writes;
- Render: attachments, runtime bindings, vertex/index/indirect resources, timestamps;
- Copy/Clear/Resolve/Present: accepted G3 semantics;
- Upload: exact destination write;
- Readback: exact source read.

Construction validates the **complete** derived access set for impossible overlapping roles inside
one operation, including runtime bindings versus attachments or other incompatible writes.
Renderer `lower_caller_accesses` duplication is deleted.

## Immutable transfer payload value versus record identity

Large upload data must not create a fake operation ordering/hash contract.

```text
GpuTransferPayloadId
    opaque nonzero process-local record/correlation identity

GpuTransferPayload
    id
    immutable Arc-backed checked transfer data/layout/provenance
```

Rules:

- clones preserve record ID and immutable data;
- one ID can never be rebound to another value;
- semantic `PartialEq/Eq` compares checked payload value semantics and **excludes** the ID;
- independently constructed byte-identical payloads therefore remain semantically equal;
- an `is_same_record`-style predicate distinguishes clones of one payload record from equal
  independently constructed values;
- executable operations do not require Hash/Ord, so graph preparation never hashes/sorts payload
  bytes as an ordering mechanism;
- the ID is not content, persistence, replay, wire, cache or cross-process identity;
- an optional digest may later be diagnostic/dedup evidence only.

No renderer-owned payload table or permanent sidecar exists.

## Upload

`GpuWorkOperation::Upload` writes one immutable transfer payload to an exact logical buffer/texture
region. It is graph-visible so G3 remains the sole initialization/hazard authority.

Physical lowering is private G5B policy. Queue-write lowering is permitted only when it is proven
ordering-equivalent to the exact graph node position—for example a graph-prefix upload with no
preceding dependency that would be bypassed. Otherwise private staging plus encoded copy preserves
the node position.

## Readback

`GpuWorkOperation::Readback` reads an exact logical buffer/texture region and owns an opaque
process-local `GpuReadbackId` for its result effect.

Physical staging/mapping is private. Texture results are normalized to tight logical rows in
`GpuReadbackBytes`; backend padding and mapped ranges never escape.

CPU feedback does not affect later nodes in the same GPU graph. Consumers observe a result and
build a later submission.

## Logical Present and SurfaceAcquired resources

`GpuPresentOperation` remains logical ordering/presentation intent, not a WGPU command.

`SurfaceAcquired` logical resources remain distinct from ordinary G4C1-realizable resources. G5A
does not manufacture an imported-resource surrogate or sampled/storage exception. Their already-
accepted attachment/copy/present roles remain logical requirements for the pre-G7 current-host
integration; physical surface capability/acquisition/presentation remains outside reusable G5.

## Sidecar disposition

`RenderGpuWorkSidecar` and `CompiledPassExecutionPlan` cease to own GPU execution semantics in
G5A. RunenRender may retain planning/provenance before lowering, but the prepared RunenGPU graph
contains complete generic GPU execution meaning after lowering.

No permanent `GpuExecutionBindings`, second command list or node-keyed execution companion is
introduced.

# G5B — Complete surface-independent lifecycle

## Execution capacity

`GpuExecutionLimits` is finite context policy with at least:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

`max_prepared_submissions` counts **both active async prepare reservations and published prepared
values**. That prevents unbounded concurrent preparation from bypassing pressure while waiting on
G4 realization.

There is no duplicate G5 realization-retirement record budget and no context-wide terminal-history
budget.

## Cancellation-safe preparation

`GpuContext::prepare` is async because G4 realization may be async.

Preparation:

1. reserves one prepared-capacity slot through an RAII token;
2. checks Running state and context/device generation;
3. validates complete executable graph semantics;
4. obtains exact G4 resource/program/layout/binding/pipeline realization Arcs;
5. records unresolved SurfaceAcquired requirements without raw surface objects;
6. plans upload/readback staging needs;
7. publishes one immutable `GpuPreparedSubmission`.

Normal error, future cancellation/drop, or abandoned owned G4 single-flight realization releases
owned reservations. No partial prepared value or submission ID is published.

Dropping a published prepared value releases its prepared slot and creates no fake cancellation
outcome.

## Submit admission and irreversible acceptance

`submit_prepared` is the public surface-independent admission terminal.

Before acceptance it rejects and preserves prepared work for:

- stale/closed context;
- already-consumed state;
- in-flight/upload/readback pressure;
- unresolved `SurfaceAcquired` requirements (`SurfaceBindingRequired`).

Pre-acceptance rejection performs no queue action and allocates no `GpuSubmissionId`.

Successful admission atomically converts prepared capacity to in-flight/staging capacity and only
then allocates a nonzero monotonic process-local `GpuSubmissionId`.

**ID allocation is the irreversible semantic acceptance point.** Any later encoding, backend
validation, queue, health or device failure terminalizes that exact accepted submission once. It
does not roll back into a submit rejection.

There is no hidden accepted-but-unsubmitted queue.

## Private encoding

G5B privately encodes surface-independent:

- compute;
- ordinary render attachments/draws;
- copy/clear/query resolve;
- Upload;
- Readback staging/copy.

The encoder receives retained G4 records, not renderer raw callbacks. `Device`, `Queue`,
`CommandEncoder`, backend pipelines/bind groups, mapped ranges, submission indices and poll types
remain private.

G5B does not acquire/bind/present `SurfaceAcquired` resources. It records their unresolved typed
requirements for G5C.

## Submission and readback lifecycles

Public state remains intentionally small:

```text
submission: Submitted -> Completed | Failed
readback:   Pending   -> Ready | Failed
```

Submission GPU completion and readback mapping/materialization are separate facts. A submission may
be `Completed` while a readback remains `Pending`.

Every accepted submission/readback terminalizes exactly once. Parent submission failure
terminalizes dependent readbacks when their result can no longer be produced.

Observer drop is not cancellation and never discards accepted work.

## Progress

Portable baseline:

```text
GpuContext::progress() -> GpuProgressReport
GpuSubmission::try_outcome()
GpuReadback::try_result()
```

Native private implementation may perform nonblocking WGPU polling. On WebGPU, browser/event-loop
progress remains external and `progress()` drains/publishes callback state. The public contract
contains no WGPU poll/fence promise and no universal blocking wait.

RunenGPU owns no mandatory Tokio/Futures executor, thread pool or immortal native progress thread.
Optional Future/callback adapters wrap the same lifecycle records and invoke observers outside
locks.

## G4 realization retention, not a second retirement owner

Accepted G4 realized handles already retain `Arc<Record>`, and G4 registries own bounded lookup
capacity plus lookup-only collection.

G5 therefore delays reclamation by retaining the exact G4 Arcs in preparing/prepared/in-flight/
readback state as required. When execution/readback safety allows, it drops those Arcs; the G4
registry then remains the sole authority that may collect lookup-only realization records under
its own capacity policy.

Private G5 staging belongs directly to in-flight/readback lifecycle records and is dropped when
safe under upload/readback/count pressure.

No second G5 resource/program/pipeline retirement registry, identity or
`max_deferred_retirement_records` counter exists.

## Terminal record detachment

The context registry retains nonterminal or backend-cleanup-pending lifecycle records only while
safety requires. After terminal result publication and safe staging/Arc release:

```text
context registry reference -> detached
caller observation Arc     -> may remain
```

A caller may keep an immutable terminal result indefinitely without consuming future context
execution capacity. If all observers disappear early, the context still terminalizes/cleans the
accepted record and then discards it rather than retaining hidden history.

## Shutdown

Execution state is:

```text
Running -> ShuttingDown -> Closed
```

`begin_shutdown()` is idempotent and rejects new prepare/submit admission. In-progress prepares
cannot publish after shutdown/stale-generation detection; RAII cancellation releases their slots.
Prepared-but-unsubmitted values become non-submittable and release capacity on drop.

`progress()` remains valid while shutting down. `Closed` requires accepted work/readbacks to be
terminal and private staging/lifecycle cleanup safe, or a terminal context/device failure that has
structurally resolved unresolved records.

Product timeout/yield/blocking/recovery policy remains Runenwerk/G7 policy.

## Lock order

The accepted WGPU error-attribution/health owner remains single authority.

Rules:

1. shared error-attribution gate remains outer backend-attribution lock;
2. preparation obtains G4 realization records before publishing G5 lifecycle records;
3. no inverse lifecycle-lock -> G4-registry lock acquisition path;
4. prepared-slot RAII Drop does not invert that hierarchy;
5. encoding operates on already-retained records;
6. completion/map callbacks update one lifecycle record, move notification/detachment work out,
   unlock, then notify/drop;
7. shutdown uses the same state machine rather than a parallel cleanup hierarchy.

G5B implementation must document the concrete mutex/registry order once source decomposition is
chosen.

## Independent proof

G5B proves the reusable boundary before renderer migration using one surface-independent non-render
compute workload with exact deterministic output and asynchronous readback. It may not use
RunenRender or current-render bridges.

# G5C — Renderer/current-host integration and final cutover

## Why a current-host surface terminal is required

Accepted G4 intentionally does **not** realize an acquired presentation surface as an ordinary
G4C1 resource, but current rendering still needs the acquired surface for accepted attachment/copy/
present roles. G5C must therefore remove renderer raw execution without pretending G7 already
exists.

## Owner-local pre-G7 surface composition

No new public/reusable surface API or third object-reference bridge is introduced.

The existing current-host surface owner composes with G5 through one owner-local lexical terminal:

```text
GpuPreparedSubmission with SurfaceAcquired requirements
        |
        v
reserve G5 submit capacity
(no GpuSubmissionId; reservation cannot escape)
        |
        v
acquire current-host surface lease(s)
        |
        v
validate logical resource identity
+ context affinity
+ configured format/extent
+ allowed attachment/copy role
        |
        v
commit the same G5 submit transaction
+ allocate GpuSubmissionId
        |
        v
private G5 WGPU encoding / queue submit
        |
        v
current-host owner physical present
```

If acquisition/binding fails before commit, provisional capacity is released, already-acquired
leases are dropped without presentation, prepared work is preserved/returned, and no submission ID
exists.

After ID commit, later execution failure terminalizes the accepted submission normally.

Raw `Surface`, `SurfaceTexture`, acquired `Texture`/`TextureView` remain inside the current-host and
private-WGPU integration boundary. They cannot be stored in logical work, cached as G4 realization,
used for sampled/storage shader binding, returned to renderer code, or exposed through a generic
callback.

Physical `present()` remains current-host/G7 migration ownership. G5 `Completed` means GPU
execution completion, not compositor/display completion.

The current-host owner maps logical `SurfaceAcquired` GPU resource identities to its own surface
slots; renderer-specific `RenderSurfaceId` does not enter reusable RunenGPU contracts.

This terminal is explicit G7 deletion inventory. G7 later replaces it with reusable typed surface
capability/generation/acquisition contracts rather than G5 generalizing it first.

## Consumer cutover

Renderer/UI/timing/capture migrate to the accepted G5 lifecycle:

- renderer stops creating command encoders and submitting queues;
- UI lowers one multi-draw render pass without splitting it;
- timing becomes timestamp writes -> query resolve -> readback -> decode;
- capture becomes generic copy/readback work;
- staged uploads become graph-visible Upload operations;
- primitive/examples/benches use public typed G5 where ordinary execution is the subject.

RunenRender retains image-formation/planning policy; capture/timing interpretation remains outside
RunenGPU.

## Final deletion census

At G5C acceptance:

```text
CurrentRenderDeviceQueue             0
current_render_device_queue()        0
CurrentRenderExecutionBridge         0
current_render_execution_bridge()    0
renderer direct Device/Queue use     0 for G5 execution
renderer CommandEncoder creation     0 for G5 execution
renderer queue.submit                0
renderer map_async/poll ownership    0 for G5 readback
renderer raw acquired surface handoff 0
RenderGpuWorkSidecar execution truth 0
manual duplicate executable accesses 0
```

Raw current-host surface mechanics may remain only inside the separately classified pre-G7 owner
and private WGPU lexical integration described above. That exception cannot become a generic G5
execution/resource escape.

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

A prepared value that requires a current-host surface is intentionally not accepted by the reusable
surface-independent submit terminal before G7; the Runenwerk current-host integration supplies that
binding privately in G5C.

# Error taxonomy

```text
GpuExecutionPreparationError
  semantic/device binding mismatch
  stale/foreign generation
  realization failure
  prepared-capacity pressure

GpuSubmitRejection
  stale/closed context
  in-flight/staging pressure
  SurfaceBindingRequired
  no submission ID allocated
  retryable prepared value preserved where meaningful

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

Current-host surface acquisition/configuration/presentation errors remain on the pre-G7 surface
boundary when they occur before G5 submission commit.

# Proof matrix

G5 planning requires implementation specs to prove at least:

1. complete operation-derived access truth matches superseded renderer duplicate access truth;
2. operation determinism does not depend on fabricated `Ord/Hash` over executable descriptors;
3. transfer payload semantic equality is distinct from record identity;
4. dynamic viewport/blend values have canonical finite equality semantics;
5. compute pipeline/binding/dispatch execution is complete;
6. one render pass supports multi-draw pipeline/binding/vertex/index/dynamic-state switches;
7. Upload participates in initialization/hazards and queue-write lowering never violates graph order;
8. Readback participates at an exact graph point and normalizes backend padding privately;
9. in-progress prepare plus published prepared capacity is bounded and cancellation-safe;
10. submit pressure/surface-binding rejection creates no submission ID and preserves prepared work;
11. every post-ID failure terminalizes the accepted submission exactly once;
12. submission completion and readback completion may differ;
13. terminal observations detach from context execution capacity after safe cleanup;
14. G5-held G4 realization Arcs prevent premature collection and no second retirement registry exists;
15. callbacks/wakers execute outside internal locks;
16. observer drop does not cancel/discard accepted work;
17. shutdown deterministically rejects new work and resolves accepted lifecycle records;
18. native progress works without public WGPU polling types;
19. WebGPU-compatible semantics do not depend on `Device::poll` having effect;
20. G5B independent non-render proof uses no renderer/surface escape;
21. current-host surface pre-commit failure creates no submission and leaks no capacity/lease;
22. surface binding validates exact logical identity/affinity/format/extent/allowed role;
23. sampled/storage SurfaceAcquired binding remains rejected before G7;
24. representative renderer/UI/timing/capture use the same G5 authority after G5C;
25. final source guards prove both temporary G5 execution seams and renderer raw execution are absent.

Environment-dependent proof reports unavailable adapter/surface/capability separately from successful
hardware execution.

# Non-goals

G5 does not add:

- a universal executor/runtime;
- a second public command graph/IR;
- a permanent execution sidecar;
- public WGPU/Naga/mapped-range/submission-index/fence/poll/native-handle/surface types;
- fabricated total ordering over program/pipeline semantics;
- content-addressed transfer identity;
- a second realization-retirement registry;
- a context-owned unbounded terminal-history ledger;
- multi-queue scheduling;
- aggressive graph optimization/pass fusion;
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
validation, and deletes predecessor authority within its accepted scope rather than carrying
compatibility aliases forward.