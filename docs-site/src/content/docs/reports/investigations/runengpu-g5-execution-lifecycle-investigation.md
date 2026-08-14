---
title: RunenGPU G5 Execution Lifecycle Investigation
description: Exact accepted-main census and cross-backend findings for executable work, render-pass compatibility, transfers, progress, completion, realization retention, current-host surface integration, and final execution cutover.
status: active
owner: gpu
layer: reports
canonical: true
last_reviewed: 2026-08-14
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g5-execution-lifecycle-design.md
  - ../../design/active/runengpu-g4c2-presentation-surface-binding-boundary.md
  - ./2026-08-14-runengpu-g5-critical-review.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/specs/pt-runengpu-g5b-execution-lifecycle.ron
  - ../../workspace/specs/pt-runengpu-g5c-renderer-cutover.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Execution Lifecycle Investigation

## Question

What execution, transfer, progress, completion, readback, pressure, realization-lifetime and
current-host-surface authority remains after accepted G4, and what is the smallest
future-transferable G5 design that can delete the remaining renderer/raw-WGPU execution seams
without duplicating G3 work semantics or absorbing RunenRender/G7 policy?

## Accepted baseline and authorization

Exact accepted G4 base:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

Issue `#284` authorizes investigation, design and implementation specification only. No G5 Rust
implementation is authorized by this report or the planning PR.

## Evidence inspected

Accepted source census covered at least:

```text
engine/src/plugins/gpu/api/work.rs
engine/src/plugins/gpu/api/graph/**
engine/src/plugins/gpu/api/program/**
engine/src/plugins/gpu/api/context.rs
engine/src/plugins/gpu/api/realization.rs
engine/src/plugins/gpu/backend/wgpu/**
engine/src/plugins/render/adapters/gpu_work.rs
engine/src/plugins/render/graph/**
engine/src/plugins/render/renderer/**
engine/tests/gpu_g4c1_cutover_guards.rs
```

Authority census covered RunenGPU/G4/G4C2 designs and specs, repository-family architecture, ADR
0015 and the durable roadmap. Pinned WGPU 27.0.1 source was inspected for render-pass attachment,
pipeline compatibility and dynamic-state validation; WebGPU/Vulkan/D3D12 synchronization models
were used only to test portability of the lifecycle vocabulary, not to authorize another backend.

The later [G5 Critical Review](2026-08-14-runengpu-g5-critical-review.md) rechecked the initial
planning conclusions against exact accepted source and corrected several intermediate ideas. This
report records both the investigation path and the final result; explicitly rejected intermediate
ideas below are not competing authority.

# Accepted G4 execution boundary

## One private backend owner

`GpuContext` owns one private `WgpuContextState`. That state remains the sole private owner of WGPU
`Instance`, `Adapter`, `Device`, `Queue`, shared device-health/error-attribution state and accepted
G4 realization states. There is no process-global GPU context or accepted executor/thread owner.

## Two temporary G5 seams

### `CurrentRenderDeviceQueue`

One non-reentrant raw operation loan contains only borrowed `Device`/`Queue` plus the shared
error-attribution guard. It owns no G4 realization and is explicit G5 deletion inventory.

### `CurrentRenderExecutionBridge`

The G4C3 lexical bridge validates and lends already-realized resources, query sets, bind groups and
compute/render pipelines into the unchanged renderer encoder. It creates no backend object and
returns no reusable backend authority. It is explicit G5 deletion inventory.

## Current renderer phase two

Accepted renderer structure is already cleanly split:

```text
phase 1
  G4 resource/program/layout/bind-group/pipeline realization

phase 2
  current_render_device_queue()
  command encoder
  staged uploads
  compute/render/copy/query/readback encoding
  queue.submit(...)
  timing/capture readback orchestration
```

G5 therefore replaces one central raw orchestration interval plus purpose-typed execution bridge
callbacks rather than many independent queue owners.

# G3/G4 already contain the semantic spine

## G3 is graph authority

`GpuPreparedWorkGraph` owns immutable nodes, dependencies, deterministic topological order,
graph-entry initialization, hazards, requirements, outputs and diagnostics.

Existing `GpuWorkOperation` covers:

```text
Compute
Render
Copy
Clear(BufferZero)
Resolve(QueryResolve)
Present
```

G3 already carries dispatch dimensions, attachments/load-store/resolve semantics, direct/indexed/
indirect draw intent, copy regions/layouts, query ranges and logical present source. Most current
operations already derive accesses. A second public command graph/IR would duplicate accepted
authority.

## Operation ordering was not graph authority

Simple G3 operation payloads happen to derive `PartialOrd/Ord/Hash`, but `GpuWorkNode` requires
value equality only and graph preparation orders through node/resource identity plus dependencies.

Accepted G4 compute/render pipeline descriptors provide semantic equality/hash but no total
ordering over program/pipeline meaning.

Final G5A therefore keeps semantic `PartialEq/Eq` for executable operation aggregates and removes
operation-level `PartialOrd/Ord/Hash` where no real consumer requires them. G4 descriptors do not
gain artificial ordering through labels, pointers or naked hashes.

## G4 closes the execution-semantic gap G3 intentionally left

Accepted G4 supplies:

- complete `GpuComputePipelineDescriptor` and `GpuRenderPipelineDescriptor`;
- typed program interfaces and binding declarations;
- storage read/write modes;
- exact runtime binding values over logical resources;
- static buffer ranges plus optional dynamic offsets;
- structural/device binding validation;
- private bind-group and pipeline realization.

Therefore G5 does not need a permanent node-keyed execution-binding sidecar.

# Temporary renderer sidecar disposition

`RenderGpuWorkSidecar` exists because G3 predated complete G4/G5 execution contracts.
`CompiledPassExecutionPlan` mixes renderer provenance/planning, G4 pipeline meaning, generic
bindings/draw state and copy/query/present details already represented elsewhere.

Moving that enum into RunenGPU would import renderer semantics and duplicate G3/G4 truth. The
proper cutover is to make generic work complete and delete sidecar execution authority.

# Executable compute/render closure

## Compute

Generic semantics reduce to:

```text
GpuComputePipelineDescriptor
runtime binding set
GpuDispatchSize
```

Accepted G4 binding declarations distinguish storage-buffer `ReadOnly/ReadWrite` and
storage-texture `ReadOnly/WriteOnly/ReadWrite`; exact runtime values retain logical resources,
ranges and dynamic offsets. Bound-resource accesses can therefore be derived from one authority.

## Render and multi-draw

The current UI encoder proves one render pass legitimately switches several pipelines/bind groups,
instance buffers and scissors. Splitting those draws into separate render-pass graph nodes would
alter pass/load-store semantics.

Final generic shape:

```text
GpuRenderOperation
  attachments
  [GpuRenderDraw]
  timestamps

GpuRenderDraw
  pipeline descriptor
  runtime bindings
  vertex buffer bindings
  optional index buffer binding
  draw intent
  dynamic state
```

Accepted G4 already owns `GpuIndexFormat::{Uint16, Uint32}`; G5 reuses it instead of introducing a
duplicate execution-layer format type.

## Render-pass compatibility is logically knowable

Pinned WGPU 27.0.1 validates common render-attachment extent/sample count. Its render-pass versus
pipeline compatibility compares ordered color formats, optional depth/stencil format, sample count
and multiview state.

All non-multiview facts are already present in G3/G4 logical descriptors. Therefore G5A catches
them before backend encoding:

```text
one common effective render extent
one common attachment sample count
ordered color attachment formats
optional depth/stencil format
```

Every draw pipeline exactly matches the pass color-target count/formats, depth/stencil presence/
format and sample count. Existing G3 resolve validation remains authoritative for matching resolve
extent/format and single-sampled destination.

Accepted RunenGPU has no multiview or 3D depth-slice render contract. Those are separate extension
stop conditions, not raw-WGPU passthrough opportunities.

## Dynamic-state normalization

The first planning pass correctly identified viewport/scissor/blend/stencil as per-draw dynamic
state but initially overconstrained viewport/scissor to positive area. WGPU 27.0.1 permits zero-area
viewport/scissor state.

Final structural semantics:

- viewport: finite canonical f32 bits, signed-zero normalization, **nonnegative** width/height,
  checked `0 <= min_depth <= max_depth <= 1`;
- scissor: checked u32 rectangle, **zero area allowed**, checked arithmetic, inside the common render
  extent;
- blend constant: four finite canonical f64 values with signed-zero normalization and no invented
  0..1 clamp;
- stencil reference: `u32`.

G5B preparation owns backend/device-specific viewport maximum size/position checks. Default viewport
and scissor derive from the validated common render extent, not an arbitrary first attachment.

## Complete access derivation

After G5A:

- Compute derives binding accesses;
- Render derives attachment/resolve/binding/vertex/index/indirect/timestamp accesses;
- Copy/Clear/Resolve/Present retain G3 semantics;
- Upload derives one exact destination write;
- Readback derives one exact source read.

Validation considers the complete derived set so a runtime binding cannot silently conflict with an
attachment or another write-capable binding.

# Transfers remain graph-visible

A submission-prefix/suffix-only transfer model was considered and rejected: a hidden upload could
initialize/overwrite a resource without G3 seeing the write, weakening initialization/hazard proof
or forcing a second transfer access model.

Final G5 adds:

```text
GpuWorkOperation::Upload
GpuWorkOperation::Readback
```

Physical staging/mapping remains private.

## Transfer payload record identity versus semantic value

Rejected intermediate ideas:

1. repeatedly hash/sort complete upload bytes as operation metadata;
2. make `GpuTransferPayloadId` replace semantic equality/order/hash.

Final model:

```text
GpuTransferPayloadId
  opaque process-local record/correlation identity

GpuTransferPayload
  id
  immutable Arc-backed checked transfer value
```

Semantic equality compares payload value and excludes the record ID. Clones preserve ID/value; an
`is_same_record`-style predicate distinguishes record identity. Operations do not require Hash/Ord,
so graph scheduling does not hash/sort payload bytes. The ID is not content, persistence, replay,
wire, cache or cross-process identity.

## Upload

Upload is an exact graph-visible destination write and may satisfy initialization. Queue-write
lowering is allowed only when exactly equivalent to the node's graph position; private staging/copy
is required otherwise.

## Readback

Readback is an exact graph-visible source read plus process-local `GpuReadbackId`. Private
staging/mapping is hidden and normalized `GpuReadbackBytes` strips backend row padding.

CPU feedback cannot affect later nodes in the same submission.

# Progress and completion abstraction

Pinned WGPU exposes backend submission indices, callbacks and native polling; WebGPU progress is
event-loop/browser driven and mapping has its own lifecycle. Vulkan/D3D12 also support monotonic
submission-completion facts without implying one universal public fence object.

G5 therefore exposes RunenGPU-owned:

```text
GpuSubmissionId
GpuSubmission outcome
GpuReadback result
GpuContext::progress()
```

Backend submission/fence/poll/promise/mapped-range objects remain private. RunenGPU owns no
mandatory Tokio/Futures executor or immortal progress thread.

# Prepared and in-flight capacity

Final pressure domains:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

`max_prepared_submissions` counts active async prepare reservations plus published prepared values.
Prepare reserves one RAII slot before asynchronous G4 realization. Error/cancellation/drop releases
it. Successful submit atomically converts prepared capacity to in-flight/staging capacity.

Pressure rejection happens before submission-ID allocation/queue activity and preserves prepared
work. There is no hidden accepted-but-unsubmitted queue.

# G4 realization retention — no second G5 retirement owner

Accepted G4 realized handles are `Arc<Record>` values and G4 registries own bounded lookup/capacity
plus lookup-only collection.

The intermediate `max_deferred_retirement_records`/second-retirement-registry idea is rejected.
Preparing/prepared/in-flight/readback G5 state retains exact G4 realization Arcs while needed and
releases them when GPU/readback safety permits. G4 remains sole realization reclamation authority.
Private G5 staging belongs to lifecycle records and remains bounded by upload/readback/count limits.

# Submission/readback lifecycle and acceptance

No `GpuSubmissionId` exists before all synchronous admission requirements succeed. ID allocation/
publication is the irreversible semantic acceptance point. Later encoding/backend/queue/health/
device failures terminalize that accepted ID exactly once rather than rolling back admission.

```text
submission: Submitted -> Completed | Failed
readback:   Pending   -> Ready | Failed
```

Submission GPU completion and readback materialization are separate facts. Observer drop is not
cancellation.

# Terminal record retention

A context-wide terminal-history budget is rejected. After terminal result publication and safe
staging/G4-reference cleanup, the context registry reference detaches while caller observation
handles may retain immutable terminal state without consuming future execution capacity.

If observers vanish early, the context still terminalizes/cleans accepted work and then discards the
internal record. G5 does not create an unbounded global history ledger.

# Current-host surface gap and final pre-G7 integration

Accepted G4C2 keeps current acquired presentation resources usable for already-existing
attachment/copy/present roles while explicitly rejecting ordinary G4C1 realization and
sampled/storage shader binding.

G5C still needs to delete renderer raw execution, so G5 requires one narrow current-host composition
without stealing reusable G7 ownership.

## Rejected surface fixes

Do not:

- realize `SurfaceAcquired` through G4C1;
- expose raw `SurfaceTexture`/`Texture`/`TextureView` publicly;
- add sampled/storage surface exceptions;
- add a broad external-resource import API;
- retain the renderer execution bridge only for surfaces;
- implement reusable G7 identity/generation/recovery early.

## Final transaction

Reusable G5B remains surface-independent. Preparation records unresolved typed
`SurfaceAcquired` attachment/copy/present requirements only. Ordinary `submit_prepared` returns
`SurfaceBindingRequired`, allocates no ID and preserves prepared work.

G5C composes the existing current-host owner with the same submit transaction:

```text
prepared work
 -> reserve G5 submit capacity; no ID
 -> resolve logical SurfaceAcquired IDs to current-host slots
 -> acquire all required leases in deterministic logical-ID order
 -> validate affinity + identity + format/extent + allowed role
 -> commit and allocate submission ID
 -> private G5 encoding / Queue::submit
 -> current-host owner may attempt present
```

Pre-commit failure drops any acquired leases without present, releases provisional capacity and
creates no submission.

Attachment encoding receives only the acquired `TextureView`; copy encoding receives only the
acquired `Texture`, both lexically inside current-host/private-WGPU integration. They cannot become
logical work, G4 realization, renderer authority, persistence or generic callback data. Sampled/
storage use remains forbidden.

If encoding fails after ID commit but before `Queue::submit` returns, the accepted submission fails
and leases are dropped without present.

After `Queue::submit` returns, current-host may call its existing `present()` terminal. That is a
presentation **attempt**, not G5-completion or display/compositor-completion evidence. A later
asynchronous GPU/device failure may occur after present was attempted; the G5 outcome and pre-G7
presentation-attempt fact remain separate.

This terminal is explicit G7 deletion inventory, not a third generic G5 bridge.

# Shutdown

```text
Running -> ShuttingDown -> Closed
```

Shutdown rejects new prepare/submit admission. In-progress prepare cannot publish after shutdown or
stale-generation detection and releases its RAII slot on cancellation/drop. Prepared values become
non-submittable. `progress()` remains valid while accepted work/readbacks terminalize and staging/
G4 references are safely released. Product timeout/yield/recovery policy remains outside G5.

# Ordered implementation decomposition

```text
G5A executable logical work closure
 -> G5B complete surface-independent execution lifecycle
 -> G5C renderer/current-host integration and final cutover
```

### G5A

Own complete compute/render logical execution, render-pass compatibility, canonical dynamic state,
transfer value/record identity, graph-visible Upload/Readback, access derivation and sidecar deletion.
No command submission/progress implementation.

### G5B

Own finite active-prepare/prepared/in-flight/staging/readback limits, cancellation-safe preparation,
atomic submit admission, private surface-independent encoding/submission, progress, exactly-once
submission/readback outcomes, G4 Arc retention/release, terminal detachment and shutdown. Prove one
independent non-render compute/readback workload first.

### G5C

Own current-host SurfaceAcquired composition for attachment/copy/present, migrate renderer/UI/
timing/capture to accepted G5B, then delete `CurrentRenderDeviceQueue`,
`CurrentRenderExecutionBridge` and renderer raw execution ownership. Retain only separately
classified current-host/G7 raw surface mechanics as explicit G7 deletion inventory.

# Rejected alternatives summary

- permanent `GpuExecutionBindings` sidecar — duplicates G4/G5 closure;
- second command IR — duplicates G3 work/access/order authority;
- move `CompiledPassExecutionPlan` into RunenGPU — imports renderer semantics;
- hidden submission-only transfers — bypass G3 hazards/initialization;
- payload allocation ID or digest as semantic operation identity/order — confuses record/value or
  overreaches into content addressing;
- add `Ord` to G4 pipeline/program contracts — no semantic consumer;
- duplicate G5 realization-retirement registry — duplicates G4 Arc/cache authority;
- duplicate G5 index-format type — accepted G4 already owns it;
- require nonzero viewport/scissor — unnecessarily rejects valid no-area state;
- make G5B own reusable surfaces — steals G7 ownership;
- split G5B into accepted partial submit/completion APIs — creates lifecycle compatibility churn.

# Remaining uncertainty before owner review

No unresolved architecture question is currently known. Remaining work is acceptance-preparation
proof only:

- complete planning-artifact consistency/census;
- complete PR diff review for connector-write drift;
- exact-head canonical CI and Documentation Build;
- review-thread/implementation-activation census;
- stop at owner review before merge or G5 Rust implementation.