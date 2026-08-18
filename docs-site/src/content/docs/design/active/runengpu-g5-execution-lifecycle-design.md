---
title: RunenGPU G5 Execution Lifecycle Design
description: Decision-complete post-G4R design for executable logical GPU work, bounded execution preparation, backend-neutral submission/progress/readback, G7A surface sequencing, and the final renderer cutover.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-18
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g3-access-work-graph-design.md
  - ./runengpu-g3r-definite-initialization-correction.md
  - ./runengpu-g4-context-program-realization-design.md
  - ./runengpu-g4c2-presentation-surface-binding-boundary.md
  - ./runenrender-decomposition-design.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../reports/investigations/runengpu-g5-execution-lifecycle-investigation.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Execution Lifecycle Design

## Authority

This design is the fresh G5 planning candidate owned by issue `#284` and derived from exact accepted
post-G3R/G4R main:

```text
d7afaba20a27901e7a6bc4d6d75e6c761c7cbf86
```

The earlier WGPU-27 G5 planning PR `#285` is superseded historical evidence. It is not a predecessor,
merge base or implementation authority.

This document binds durable G5 architecture. The owning GitHub issue owns activation and current
state. The companion G5A RON file is subordinate handoff detail for the first bounded implementation
slice only.

No G5 Rust implementation is authorized by this document alone.

## Mission

RunenGPU is the backend-neutral logical semantics layer for GPU resources and execution. G5 extends
that boundary from validated/realized work through observable execution while keeping physical WGPU
realization private.

The target stack is:

```text
Runenwerk / domain adapters
        |
        v
RunenRender image-formation semantics          independent non-render consumers
        |                                             |
        +--------------------+------------------------+
                             v
                  RunenGPU logical GPU semantics
                  resources / programs / work
                  access / initialization / execution
                             |
                             v
                  private physical realization
                             |
                             v
                           WGPU 30
                             |
                Vulkan / Metal / D3D12 / WebGPU / ...
```

G5 must not turn RunenGPU into a public WGPU command wrapper.

## Ordered delivery

The durable roadmap remains:

```text
accepted G4R
    -> G5A executable logical work closure
    -> G5B reusable surface-independent execution lifecycle
    -> G7A minimal durable surface foundation
    -> G5C final renderer/UI/timing/capture execution cutover
    -> G6 breadth/scale/offscreen/cost proof
    -> G7B complete loss/generation/reconstruction
    -> G8 conformance and zero reach-through
    -> GX standalone transfer
```

Only G5A may be activated directly after this planning design is accepted.

# 1. Governing semantic laws

## 1.1 One logical work authority

`GpuWorkOperation`, `GpuWorkFragment` and `GpuPreparedWorkGraph` remain the sole authority for:

- logical GPU operations;
- logical resources;
- exact G3R initialization requirements/effects;
- access and hazard intent;
- dependencies and deterministic prepared order;
- operation-derived capability requirements.

G5 adds execution-complete operation payloads. It does not create another public command graph,
backend command buffer IR, execution DAG or renderer sidecar with duplicate GPU semantics.

Private WGPU command encoders and implementation-local encode plans are derived realization state and
never semantic authority.

## 1.2 Logical semantics stay above physical realization

Public RunenGPU types may describe:

```text
GpuComputeOperation
GpuRenderOperation
GpuUploadOperation
GpuReadbackOperation
GpuPreparedSubmission
GpuSubmissionId
GpuSubmissionStatus
GpuReadbackId
GpuReadbackStatus
```

They must not expose or make correctness depend on:

```text
wgpu::Device
wgpu::Queue
wgpu::CommandEncoder
wgpu::CommandBuffer
wgpu::SubmissionIndex
wgpu::PollType
wgpu::BufferView
wgpu::Surface
Vk*/MTL*/D3D12* synchronization handles
```

## 1.3 G3R remains initialization authority

G5 does not reintroduce `write access => initialized` inference.

```text
access/hazard envelope
!=
exact initialization requirement
!=
operation-guaranteed initialization effect
```

Uploads, readbacks, copies, clears, query resolve and attachment semantics participate in the same
G3R preparation model. Execution convenience may not bypass it.

## 1.4 Accepted is not physically submitted

The public submission fact is RunenGPU admission:

```text
Accepted
    -> Completed
    -> Failed
```

`Accepted` does **not** mean a backend queue submission call has already succeeded. Internal physical
states may exist for diagnostics, but they are not portable public lifecycle states.

A `GpuSubmissionId` is allocated only at the irreversible RunenGPU acceptance point. After that ID is
published, every later synchronous or asynchronous failure terminalizes that accepted submission
exactly once.

# 2. G5A — executable logical work closure

G5A completes logical operation meaning but does not own queue submission or progress.

## 2.1 Compute operation

The target semantic shape is directionally:

```text
GpuComputeOperation
  pipeline: GpuComputePipelineDescriptor
  bindings: GpuRuntimeBindingSet
  dispatch: GpuDispatchIntent
  timestamp_writes
```

Exact field names may follow current API decomposition, but every compute operation must contain all
information required for backend-neutral execution after private realization.

### Dispatch intent

Replace the direct-only/nonzero-only dispatch shape with:

```text
GpuDispatchIntent
  Direct { x: u32, y: u32, z: u32 }
  Indirect { arguments: GpuBufferHandle, offset: u64 }
```

Direct dispatch rules:

- each dimension is checked against the admitted execution limit;
- zero is valid;
- if any dimension is zero, the shader invocation grid is empty;
- empty direct dispatch produces no shader resource access and no shader initialization effect;
- explicit operation-owned timestamp commands remain meaningful even when the shader dispatch is
  empty;
- private encoding may elide an empty dispatch while preserving explicit non-shader command
  semantics.

Indirect dispatch rules:

- the argument buffer has an exact 12-byte read range beginning at the checked offset;
- the offset obeys accepted indirect-buffer alignment semantics;
- the operation derives an indirect read access;
- the operation requires `GpuCapabilityFeature::IndirectExecution`;
- runtime argument values are backend/GPU data and are not inspected by host planning as semantic
  truth.

## 2.2 One normalized indirect-execution capability

Clean-cut:

```text
GpuCapabilityFeature::IndirectDraw
    -> GpuCapabilityFeature::IndirectExecution
```

The accepted backend admission fact already comes from WGPU `INDIRECT_EXECUTION`, which covers both
indirect drawing and dispatching. Keeping draw-only naming or adding a parallel dispatch capability
would misrepresent one physical capability as two independent facts.

All existing indirect draw requirements migrate to `IndirectExecution`. No deprecated alias,
forwarding enum variant or compatibility translation survives G5A.

## 2.3 Render operation and draws

One `GpuRenderOperation` remains one logical render pass:

```text
GpuRenderOperation
  color attachments
  optional depth/stencil attachment
  draws: [GpuRenderDraw]
  timestamp writes
```

Each `GpuRenderDraw` owns complete effective draw execution state:

```text
GpuRenderDraw
  pipeline: GpuRenderPipelineDescriptor
  bindings: GpuRuntimeBindingSet
  vertex buffers
  optional index buffer
  draw intent
  viewport
  scissor
  blend constant
  stencil reference
```

This supports multiple compatible pipelines/bindings/draws inside one pass without fabricating
extra attachment load/store boundaries.

### Pass signature

Preparation derives one render-pass signature from attachments:

- effective extent;
- sample count;
- ordered color formats;
- optional depth/stencil format.

Every draw pipeline must be compatible with that pass signature before backend encoding. Pipeline
blend/primitive/write state remains pipeline/draw state and may differ between draws where the pass
signature allows it.

## 2.4 Render usage compatibility versus draw-local aliasing

G5A must model the WebGPU-portable semantic distinction explicitly.

### Pass-wide usage compatibility

One render pass is one usage scope. Across all draws/state uses in the pass:

- read/input/constant/storage-read/attachment-read combinations must remain compatible;
- storage/storage use may coexist across different draws;
- attachment/attachment use may coexist only where accepted attachment-region rules permit it;
- a resource cannot be both incompatible read-like and write-like usages in the same pass;
- writable attachment usage cannot alias an incompatible sampled/storage use in the pass.

### Draw-local writable binding aliasing

For each draw's effective pipeline/bindings:

- overlapping buffer bindings are rejected when either effective binding is writable;
- overlapping texture subresources are rejected when either effective binding is writable;
- disjoint ranges/subresources remain valid;
- dynamic offsets participate in the effective range before this check.

This replaces the current broader rule that effectively rejects any overlapping write-capable
access across the render operation.

## 2.5 Vertex and index binding semantics

`GpuVertexBufferBinding` owns:

- vertex slot;
- logical buffer handle;
- checked byte range.

Pipeline stride, step mode and attributes remain accepted G4 pipeline state.

`GpuIndexBufferBinding` owns:

- logical buffer handle;
- checked byte range;
- accepted `GpuIndexFormat`.

No second index-format vocabulary is added.

## 2.6 Dynamic render state

Each draw carries complete effective dynamic state so semantics do not depend on hidden backend state
inheritance.

Required initial state:

- viewport with finite canonical values and `0 <= min_depth <= max_depth <= 1`;
- scissor rectangle checked against effective render extent;
- finite canonical blend constant;
- stencil reference `u32`.

Zero-area viewport/scissor are valid no-rasterization cases. Private realization may elide redundant
state setters, but the logical value remains explicit.

## 2.7 Color clear values

Clean-cut `GpuColorClearValue` from normalized-color policy to generic attachment clear semantics:

- four finite `f64` components;
- canonicalize signed zero/NaN rejection consistently with other semantic floating values;
- no generic `[0, 1]` restriction;
- the selected attachment format owns target-format conversion/admission;
- depth clear remains separately constrained to its depth semantic domain.

This does not add speculative texture formats. It removes an invalid semantic ceiling from an
existing generic value.

## 2.8 Runtime binding set

`GpuRuntimeBindingSet` is the complete logical binding-use aggregate for one compute invocation or
render draw. It preserves accepted G4 typed runtime binding values and provides deterministic
pipeline-layout slot ordering.

It owns semantic values such as:

- logical resource identity;
- static buffer offset/size;
- optional logical `u64` dynamic offset;
- texture/view/sampler identity;
- effective access intent derived from the accepted interface.

## 2.9 Dynamic offsets are per-use execution state

The physical bind group does not own dynamic offsets.

Target decomposition:

```text
GpuRuntimeBindingSet
      |
      +--> logical effective access range
      |       static offset + dynamic offset + size
      |
      +--> static physical binding projection
              layout
              resource identities
              static buffer offsets/sizes
              texture/sampler/view facts
              NO dynamic offset
                      |
                      v
              GpuRealizedBindGroup
```

G5A changes the private G4C2 bind-group key and record to use the static projection. Requests that
differ only by dynamic offsets reuse one physical bind group.

Remove physical-record APIs that claim one invocation's complete runtime values are properties of the
realized bind group. Logical binding values remain inspectable from the logical use.

G5B later prepares ordered backend dynamic offsets per bind-group use and performs private narrowing
to the WGPU offset domain.

## 2.10 Complete operation-derived accesses

After G5A the generic work operation derives all GPU accesses needed for correctness:

- Compute: runtime binding accesses + indirect arguments + timestamps;
- Render: attachments/resolves + runtime bindings + vertex/index/indirect buffers + timestamps;
- Copy/Clear/Resolve/Present: accepted G3/G3R semantics;
- Upload: exact destination write;
- Readback: exact source read.

Renderer-authored duplicate generic GPU access declarations are removed as each operation becomes
complete. Domain/renderer semantic provenance may remain, but not duplicate GPU access truth.

## 2.11 Upload operation

Add graph-visible immutable Upload work:

```text
GpuUploadOperation
  destination range/region
  immutable checked payload
```

Properties:

- participates in G3 hazards and exact initialization effects;
- payload lifetime/value is owned independently of WGPU staging;
- payload diagnostic record identity is not content identity, cache identity or persistence identity;
- physical upload strategy is private G5B realization.

## 2.12 Readback operation

Add graph-visible Readback work:

```text
GpuReadbackOperation
  source range/region
  GpuReadbackId
```

Properties:

- participates in G3 read hazards/requirements;
- CPU-visible result is asynchronous and cannot feed a later node in the same logical submission;
- result bytes are normalized to logical/tight data rather than exposing mapped backend ranges;
- submission completion and readback materialization remain distinct facts.

## 2.13 Execution-required normalized limits

G5A extends normalized limit vocabulary only where accepted logical execution needs device facts.
The first required additions include at least:

- maximum compute workgroups per dimension;
- maximum bind-group count;
- maximum combined bind-group + vertex-buffer count where required by the backend-neutral contract.

Existing vertex-buffer, attachment, binding and dynamic-alignment facts remain authoritative.

Rule:

> if a device-dependent constraint is knowable before private encoding and can affect acceptance of an
> accepted RunenGPU operation, normalize/admit it at the owning semantic boundary instead of waiting
> for a backend validation failure.

Do not mirror unrelated WGPU limits merely for completeness.

## 2.14 G5A renderer transition

G5A makes generic GPU work execution-complete. Current renderer planning structures may remain for
RunenRender semantics/provenance, but they must stop being independent GPU execution authority.

Temporary raw execution before G5C may reference the generic operation values by prepared node
identity. It may not preserve a second pipeline/binding/draw/access description and reconcile them at
runtime.

# 3. G5B — reusable surface-independent execution lifecycle

G5B consumes accepted G5A and gives `GpuContext` bounded execution ownership.

## 3.1 Execution policy and capacities

One context-local execution policy owns finite independent capacities. Initial categories:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

These are distinct from:

- G4 realization-record capacity;
- logical resource byte size;
- physical GPU memory/residency;
- renderer/product budgets.

Pressure is structured and inspectable. No hidden unbounded queue is introduced.

## 3.2 Execution preparation

Directionally:

```text
GpuPreparedWorkGraph
        |
        v
GpuContext::prepare_submission(...).await
        |
        v
GpuPreparedSubmission
```

Preparation:

1. reserves one prepared slot through cancellation-safe RAII;
2. validates context/device-generation affinity;
3. validates all G5A execution limits and dynamic offsets;
4. obtains/retains required G4 resource/program/binding/pipeline realizations;
5. plans immutable upload/readback staging requirements;
6. creates ordered private dynamic-offset slices;
7. publishes one single-use prepared ticket only after the record is complete.

No submission ID exists during preparation.

## 3.3 Prepared ownership and revocation

Caller-held `GpuPreparedSubmission` is a single-use ticket, not the owner of context execution state.
The context owns the prepared record and its G4/staging reservations.

The ticket may hold only weak/revocable context linkage needed for drop notification/diagnostics.
Holding a prepared value alone must not keep the context/device/queue or prepared capacity alive.

Dropping an active ticket releases its record when the context still exists.

`begin_shutdown()` can revoke every still-prepared record and release its capacity even if callers
continue holding ticket values.

## 3.4 Submit result and Rust ownership

Pre-acceptance rejection must return ownership of the prepared value:

```text
submit_prepared(prepared)
  -> Accepted(GpuSubmission)
  -> Rejected(GpuSubmitRejection { prepared, reason })
```

Exact Rust spelling may use `Result`, but the ownership law is mandatory.

Retryable pressure rejection does not force reconstruction/re-realization of prepared work.
Nonretryable stale/revoked reasons remain explicit on the returned value.

## 3.5 Irreversible acceptance sequence

Submit admission is ordered:

```text
validate prepared ticket/context/generation/state
 -> reserve/convert finite in-flight + staging capacity
 -> remove prepared record from retryable prepared authority
 -> allocate/publish GpuSubmissionId
 -> state = Accepted
 -> private encode/queue work
```

Before ID publication:

- no queue action;
- no logical Upload side effect;
- no accepted submission record.

After ID publication:

- the call cannot return a pre-acceptance rejection;
- encode, queue, device-health or callback failure terminalizes that ID as `Failed` exactly once;
- success eventually terminalizes as `Completed` exactly once.

## 3.6 Private encoding

G5B privately encodes accepted G5A operations with accepted G4 realizations.

For each compute/render binding use it provides:

```text
GpuRealizedBindGroup
+ exact ordered checked dynamic-offset slice
```

Logical offsets remain `u64`; private backend narrowing is checked before encoding.

No raw backend object becomes public or renderer-owned.

## 3.7 Transactional Upload lowering

Initial accepted implementation uses encoded staging copies for Upload.

Reason:

- WGPU queue writes are staged before the next queue submit;
- a failure can otherwise leave physical writes to be flushed by later logical work;
- that would violate exactly-one logical submission ownership and G3R initialization/hazard truth.

A future private queue-write optimization is allowed only if a separate proof demonstrates that all
staged writes are transactionally coupled to the same accepted submission and cannot survive its
failure/rejection boundary.

The public Upload contract does not change if the private strategy changes.

## 3.8 Submission observation

Public state:

```text
GpuSubmissionStatus
  Accepted
  Completed
  Failed(structured failure)
```

No public `Submitted`, fence, backend queue index or native poll state.

Observation handles may outlive context cleanup after terminal state, but detached terminal state
must not hold backend execution capacity or G4 realization retention.

## 3.9 Readback observation

Readback state is independent:

```text
GpuReadbackStatus
  Pending
  Ready(GpuReadbackBytes)
  Failed(structured failure)
```

A submission can be `Completed` while a readback mapping/result callback is still `Pending`.
Submission failure terminalizes dependent readbacks as failed.

Mapped WGPU ranges never escape. Texture readback removes physical row padding and exposes the exact
logical bytes/metadata requested by the operation.

## 3.10 Progress

`GpuContext::progress()` is nonblocking and backend-neutral.

It may:

- privately poll native WGPU where useful;
- drain/observe completion and map callbacks;
- terminalize submission/readback state;
- release staging and retained G4 references that are now safe;
- advance graceful shutdown toward `Closed`.

On browser WebGPU, callbacks are event-loop driven. `progress()` must not claim to synchronously wait
for GPU completion there.

RunenGPU owns no mandatory Tokio/Futures runtime, worker thread or immortal polling service.

## 3.11 Graceful shutdown

Context execution state:

```text
Running
 -> ShuttingDown
 -> Closed
```

`begin_shutdown()`:

- is idempotent;
- rejects new preparation and new submission acceptance;
- revokes all unaccepted prepared records;
- keeps accepted submissions/readbacks progressable and observable;
- reaches `Closed` after accepted work is terminal and execution-owned resources are detached.

Shutdown does not transfer product recovery policy into RunenGPU.

## 3.12 Last-context Drop

Last `GpuContext` Drop is abrupt owner loss, not implicit graceful shutdown.

Required public consequences:

- no new execution can occur;
- prepared records are revoked/released;
- every still-nonterminal accepted submission/readback observation becomes a structured
  `ContextDropped`/equivalent terminal failure exactly once;
- no public claim is made that already issued hardware work was synchronously cancelled;
- detached terminal observations may remain alive without device/queue or G4 realization authority.

This makes resource ownership deterministic even when callers skip `begin_shutdown()`.

## 3.13 G4 realization retention

Prepared/in-flight state retains exact accepted G4 realization records only while required.

G5 does not create a second resource/program/pipeline retirement registry. When execution no longer
needs a record, it releases its existing `Arc` retention and existing G4 bounded registries regain
reclamation authority.

## 3.14 G5B proof workload

Acceptance requires a genuine non-render, headless proof using the same public logical model:

```text
Upload input
 -> compute
 -> compute using a second dynamic offset but the same physical bind group
 -> Readback
 -> progress to terminal submission/readback
 -> verify result
```

The proof must demonstrate:

- two valid dynamic offsets reuse one physical bind-group record;
- no renderer API is involved;
- G3R initialization/hazard semantics remain authoritative;
- pressure rejection preserves prepared ownership;
- cancellation/drop releases prepared capacity;
- exactly-once terminal outcomes;
- readback completion distinct from submission completion.

Native exact-head proof is required. Wasm compilation/conformance is mandatory. A real browser
WebGPU lifecycle smoke is required where repository infrastructure can execute it; if infrastructure
cannot, that limitation is recorded and no stronger browser-runtime claim is made.

# 4. G7A — minimal durable surface foundation

G7A is deliberately between G5B and G5C.

## 4.1 Why G7A exists here

Current Runenwerk owns raw WGPU `Surface` objects and a `CurrentHostSurfaceBridge`. Migrating renderer
execution before defining reusable surface identity/acquisition/presentation would force either:

- a disposable G5-only surface API;
- a broad raw-surface escape hatch;
- or current Winit/product policy into RunenGPU.

All are rejected.

## 4.2 Minimum G7A facts

G7A owns only the generic facts required for reusable presentation execution:

- opaque `GpuSurfaceId`;
- context/device-generation affinity;
- surface generation/lease identity sufficient to reject stale acquired images;
- normalized capabilities and admitted configuration;
- acquisition result categories;
- acquired-image identity/role facts;
- presentation acceptance/result facts.

It keeps physical surface objects private.

## 4.3 G7A exclusions

G7A does not own:

- Winit windows/event loops;
- application resize/minimize/visibility policy;
- product retry/fallback/recovery choice;
- full device replacement/reconstruction;
- renderer image-formation semantics;
- persisted surface identity.

Full loss/generation/reconstruction conformance remains G7B.

# 5. G5C — final renderer execution cutover

G5C consumes accepted G5B and G7A.

## 5.1 Migration

Migrate current renderer/UI/timing/capture paths so all generic GPU operations lower into the same
G5 logical/execution model.

Current renderer structures may still own:

- scene/view/material/image-formation decisions;
- render-plan provenance;
- feature/fallback/product policy;
- capture selection and product artifact policy.

They no longer own:

- generic uploads;
- raw command encoding;
- generic compute/render/copy execution;
- queue submission;
- GPU completion/progress;
- mapped readback mechanics;
- generic surface acquire/present mechanics.

## 5.2 Required deletion

The final G5C accepted tree contains zero definitions/call sites for:

```text
CurrentRenderDeviceQueue
current_render_device_queue()
CurrentRenderExecutionBridge
current_render_execution_bridge()
```

All purpose-typed terminals that exist solely to lend G4 realized WGPU objects into renderer
execution are deleted with the bridge.

No replacement compatibility alias, broad closure bridge, `raw_device()`, `raw_queue()`,
`as_wgpu_*`, forwarding module or second backend path is allowed.

## 5.3 Surface cutover

Current-host WGPU surface ownership/configuration/acquire/present moves behind accepted G7A generic
surface contracts. Runenwerk keeps the Winit/window/event-loop and product recovery side.

G5C does not reopen G7A or add a renderer-private second surface model.

# 6. Failure taxonomy

Failures are structured by owner and lifecycle stage.

Initial categories include:

```text
logical work invalid
context/generation mismatch
execution limit rejected
prepared revoked/consumed
pre-acceptance pressure
shutdown/closed
backend encode/validation contract failure
backend queue/device failure
device/context lost
readback mapping/result failure
surface binding/acquisition/present failure (after G7A)
context dropped
```

Diagnostic backend strings may accompany a category but never determine semantic category, retry
policy, persistence identity or equality.

# 7. Determinism and identity

- G3 prepared order remains deterministic for identical logical input.
- Physical realization/cache order is not promoted into semantic order.
- `GpuSubmissionId`, `GpuReadbackId`, prepared-ticket IDs and transfer-record IDs are process-local
  opaque operational identities.
- None are wire, persistence, replay, content, cache or cross-process identity.
- Submission acceptance order is explicit owner-local execution order, not a universal Runenwerk
  logical clock.

# 8. Pressure and memory semantics

Keep these dimensions separate:

```text
logical resource size
G4 realization-record count
G5 prepared count
G5 in-flight submission count
G5 upload staging bytes
G5 readback staging bytes
physical backend allocation/residency
```

RunenGPU may observe/report each where it owns the fact. It must not claim exact physical GPU memory
residency from logical or staging counts.

# 9. Portability

WGPU is the first private backend, not RunenGPU's public ceiling.

G5 contracts therefore use:

- normalized capability/limit facts;
- backend-neutral execution lifecycle;
- backend-neutral progress/observation;
- backend-neutral readback bytes;
- explicit unsupported capability rejection.

Do not expose WGPU native-only transition/barrier APIs, raw HAL access, backend fences or browser
promise objects through G5.

A future specialized/native interoperability contract requires separate accepted evidence and cannot
be a generic raw escape hatch.

# 10. Validation and conformance

## G5A

At one unchanged reviewed feature head require:

- focused logical operation/access/initialization tests;
- direct zero-dispatch proof;
- indirect compute/draw capability and access proof;
- render pass-wide usage versus draw-local alias proofs;
- color clear format-admission proofs;
- dynamic-offset effective-range and static-realization-key proofs;
- normalized execution-limit admission proofs;
- renderer duplicate GPU-semantic deletion guards;
- `cargo validate`;
- `git diff --check`;
- Documentation Build;
- exact-head CI and complete diff review.

## G5B

Require:

- headless non-render compute/upload/readback proof;
- cancellation-safe prepared reservation tests;
- retryable submit rejection returning prepared ownership;
- exact acceptance/terminalization tests;
- shutdown/revocation/last-context-drop tests;
- staging/readback pressure tests;
- native progress/runtime proof;
- wasm/conformance evidence and browser runtime evidence where infrastructure permits;
- no raw backend public API;
- `cargo validate`, `git diff --check`, Documentation Build, exact-head CI and complete diff review.

## G7A

Require independent surface capability/identity/generation/acquire/present design and proof before
G5C activation.

## G5C

Require representative renderer, UI, timing and capture paths through G5/G7A plus structural zero
counts for every deleted raw execution seam.

# 11. Explicit non-goals

G5 does not authorize:

- mesh shaders/tasks;
- ray tracing/query APIs;
- sparse resource APIs;
- multi-queue or queue-family public policy;
- raw native backend passthrough;
- broad resource aliasing/placed heaps;
- persisted pipeline/backend caches;
- video/tensor/distributed-GPU systems;
- RunenRender image-formation semantics;
- ECS/world/SDF semantics;
- application scheduling;
- complete G7B reconstruction;
- standalone repository extraction.

Those require their own evidence gates.

# 12. Acceptance and next activation

This planning design is decision-complete only if owner review confirms that implementation does not
need to invent execution ownership, lifecycle, pressure, progress, transfer, surface sequencing or
portability policy mid-slice.

After planning acceptance, activate exactly one bounded implementation issue:

```text
G5A — executable logical work closure
```

Do not activate G5B, G7A, G5C or GX simultaneously. Each consumes accepted predecessor authority and
must use a fresh exact-main census before implementation.
