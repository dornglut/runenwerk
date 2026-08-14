---
title: RunenGPU G5 Execution Lifecycle Investigation
description: Exact accepted-main census and cross-backend findings for executable work, command encoding, transfers, progress, completion, realization retention, current-host surface integration, and final execution cutover.
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

That revision is the guarded squash acceptance of G4C3 PR `#242`; exact push/main CI and
Documentation Build succeeded. Issues `#214` and `#188` are accepted/closed.

Issue `#284` authorizes investigation, design and implementation specification only. No G5 Rust
implementation is authorized by this report or by the planning PR.

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
engine/Cargo.toml
```

Authority census covered the RunenGPU architecture, G3/G4 designs/specifications, G4C2
presentation-surface decision, repository-family architecture, ADR 0015 and durable roadmap.

Backend behavior was checked against pinned WGPU 27.0.1 plus primary WebGPU/Vulkan/D3D12
synchronization semantics. Those comparisons constrain the abstraction; they do not authorize a
second backend in G5.

The later [G5 Critical Review](2026-08-14-runengpu-g5-critical-review.md) rechecked the proposed
planning model against exact accepted source and corrected several intermediate conclusions. This
investigation records both the path and the final source-grounded result; the critical review and
focused design win over an explicitly marked rejected intermediate idea below.

# Accepted G4 execution boundary

## One private backend owner

`GpuContext` owns one private `WgpuContextState`. `WgpuContextState` remains the sole private owner
of WGPU `Instance`, `Adapter`, `Device`, `Queue`, shared device-health/error-attribution state and
all accepted G4 realization states.

There is no process-global GPU context and no accepted executor/thread owner in this boundary.

## Two explicitly temporary G5 seams

### `CurrentRenderDeviceQueue`

Defined inside `engine/src/plugins/gpu/backend/wgpu/state.rs` as one non-reentrant raw operation
loan containing only:

```text
&wgpu::Device
&wgpu::Queue
error-attribution guard
```

It owns no G4 realization. Accepted comments assign migration/deletion of the remaining execution
operations to G5.

The accepted structural guard proves production renderer execution has one
`current_render_device_queue()` interval and that it begins only after G4C1/G4C2/G4C3
realization.

### `CurrentRenderExecutionBridge`

The G4C3 bridge is the sole lexical bridge for already-realized private objects. It validates and
temporarily lends buffers, texture views, query sets, bind groups and compute/render pipelines
into the unchanged renderer encoder.

It creates no backend object and returns no reusable backend reference. It is deletion inventory,
not a future public API.

## Current renderer phase two

The current renderer has a clean G4/G5 split:

```text
phase 1
  G4 resource/program/layout/bind-group/pipeline realization

phase 2
  current_render_device_queue()
  create command encoder
  apply staged uploads
  encode compute/render/copy/query/timestamp/readback work
  queue.submit(...)
  drive timing/capture readback helpers
```

G5 therefore has a concentrated migration: one raw operation interval plus distributed
execution-bridge callbacks, not many independent queue owners.

# G3/G4 already contain the logical execution spine

## G3 is the graph authority

`GpuPreparedWorkGraph` already owns immutable deterministic nodes, dependencies, topological order,
graph-entry initialization, hazards, requirements, outputs and diagnostics.

`GpuWorkOperation` already covers:

```text
Compute
Render
Copy
Clear(BufferZero)
Resolve(QueryResolve)
Present
```

Those contracts already carry dispatch dimensions, render attachments/load-store/resolve
semantics, direct/indexed/indirect draw intent, copy regions/layouts, query ranges and logical
present source. Most current operations derive their resource accesses.

A second public command graph/IR would duplicate accepted G3 authority.

## Deterministic graph order does not come from operation `Ord`

The critical review rechecked a subtle type-system assumption. Current simple G3 operation payloads
happen to derive `PartialOrd/Ord/Hash`, but `GpuWorkNode` itself requires value equality only.
Graph preparation uses fragment/node/resource identities, BTree maps keyed by those identities,
explicit dependency edges and topological ordering. It does not sort nodes by comparing operation
payloads.

Accepted G4 compute/render pipeline descriptors provide complete semantic value equality/hash but
intentionally do not define a total ordering over shader/pipeline semantics.

Therefore the final G5A decision is **not** to add `Ord` to G4 descriptors. Executable operation
aggregates retain semantic `PartialEq/Eq` and drop operation-level `PartialOrd/Ord/Hash` where the
complete executable payload has no independently justified ordering/hash consumer. Lower-level IDs,
ranges and enums may retain natural ordering where useful.

## G4B/G4C3 close the execution-semantic gap G3 intentionally left

G3 was implemented before complete G4 program/pipeline contracts existed. Accepted G4 now supplies:

- complete backend-neutral `GpuComputePipelineDescriptor`;
- complete backend-neutral `GpuRenderPipelineDescriptor`;
- typed binding declarations including storage read/write modes;
- exact `GpuRuntimeBindingValue` values over logical resources;
- static buffer range plus optional per-binding dynamic offset;
- structural/device binding validation and private bind-group realization;
- opaque private pipeline realization.

Therefore G5 does not need a permanent node-keyed execution-binding sidecar. Compute/render
operations can become logically executable using accepted G4 descriptors while G5 preparation
privately realizes them.

# Temporary renderer sidecar disposition

`RenderGpuWorkSidecar` is explicitly transitional. It maps prepared node IDs to
`CompiledPassExecutionPlan` because G3 predated G4/G5 execution ownership.

`CompiledPassExecutionPlan` mixes:

- RunenRender provenance/planning (`RenderPassId`, feature IDs, view masks, authoring indices);
- G4 pipeline/program/specialization/raster meaning;
- generic bindings/vertex/index/indirect execution state;
- copy/query work already represented by G3;
- logical Present and surface mechanics that remain separately owned.

Moving this enum into RunenGPU would import renderer semantics and duplicate G3/G4 truth. The clean
cutover is to make generic work complete and delete sidecar execution authority.

# Executable compute/render closure

## Compute

Required generic semantics reduce to:

```text
GpuComputePipelineDescriptor
runtime binding set
GpuDispatchSize
```

Accepted G4 binding declarations distinguish storage-buffer `ReadOnly/ReadWrite` and
storage-texture `ReadOnly/WriteOnly/ReadWrite`; runtime values retain exact resource/range/dynamic
offset data. Bound-resource accesses and requirements can therefore be derived from one authority.

## Render

Current UI proves one render pass can legitimately contain multiple draws with different
pipelines/bindings/scissors. Splitting each draw into another render-pass node would alter
load/store/pass structure and add avoidable backend work.

The generic shape is therefore:

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

Existing `GpuDrawIntent::Indirect` already owns indirect argument buffer/range/indexed state.
Multi-draw-indirect/mesh-shader vocabulary is not required by current consumers and remains YAGNI.

## Dynamic-state normalization

The first planning pass named viewport/scissor/blend constant/stencil reference but did not bind
floating-point value semantics. The critical review corrected this:

- viewport uses finite canonical f32-bit components with signed-zero normalization, positive extent
  and checked depth range;
- scissor uses a checked positive integer rectangle within effective attachment bounds;
- blend constant uses four finite canonical f64 components with signed-zero normalization;
- stencil reference is `u32`;
- defaults are explicit semantic values, never inherited accidental backend state.

Private lowering may elide redundant setter calls without changing semantics.

## Complete operation-derived access contradiction checking

After G5A, the operation is the one executable access owner:

- Compute: runtime binding reads/writes;
- Render: attachments, runtime bindings, vertex/index/indirect resources and timestamps;
- Copy/Clear/Resolve/Present: accepted G3 semantics;
- Upload: exact destination write;
- Readback: exact source read.

Validation must consider the **complete derived set** before backend encoding. A runtime binding
cannot silently conflict with an attachment or another incompatible write merely because those
accesses originated from different substructures.

# Transfers remain graph-visible

An early possibility was to treat uploads/readbacks only as submission-prefix/suffix concerns. That
was rejected after rechecking G3 initialization/hazard authority.

A hidden upload could initialize or overwrite a resource without G3 seeing the write, forcing
weakened initialization proof or a second transfer access model. The final design therefore adds:

```text
GpuWorkOperation::Upload
GpuWorkOperation::Readback
```

Their logical accesses are graph authority; physical staging/mapping stays private G5
implementation.

## Transfer payload record identity versus semantic value

Two intermediate approaches were rejected:

1. repeatedly hash/sort complete upload bytes as operation metadata;
2. make an opaque `GpuTransferPayloadId` replace semantic operation equality/order/hash.

The first adds avoidable byte-sized metadata work. The second makes independently constructed
byte-identical uploads semantically unequal merely because their allocation identities differ.

Final model:

```text
GpuTransferPayloadId
  opaque nonzero process-local record/correlation identity

GpuTransferPayload
  id
  immutable Arc-backed checked transfer value
```

Rules:

- one ID is never rebound to another payload value;
- clones preserve ID and immutable data;
- semantic `PartialEq/Eq` compares checked payload value semantics and excludes the ID;
- independently constructed equal payload values remain semantically equal;
- an explicit `is_same_record`-style predicate distinguishes one shared payload record from equal
  independently constructed values;
- executable operations do not require operation-level Hash/Ord, so graph preparation does not
  hash/sort upload bytes merely for deterministic scheduling;
- the ID is not content, persistence, replay, wire, cache or cross-process identity;
- an optional digest may later be diagnostics/dedup evidence only.

Accepted `PreparedGpuData<TransferData>` and `GpuPreparedTextureData` remain checked immutable
transfer-data/layout building blocks.

## Upload

Upload is one exact logical destination write. A complete upload can satisfy G3 initialization
like another complete initializing write.

The logical operation does not choose queue-write versus encoded staging-copy mechanics. Queue
write is permitted only when proven ordering-equivalent to the exact graph position; otherwise
private staging/copy preserves the operation position.

## Readback

Readback is one exact logical source read plus opaque process-local `GpuReadbackId`. Private
staging/mapping is not exposed to the graph.

Result is normalized immutable `GpuReadbackBytes`; backend texture row padding is removed before
publication.

CPU feedback cannot affect later nodes inside the same submitted graph. Consumers observe the
result before constructing a later submission.

# Progress and completion abstraction

Pinned WGPU exposes backend-specific facts:

- queue submission returns a backend submission index;
- completion callbacks require runtime progress;
- native WGPU can drive callbacks/mapping through polling;
- WebGPU underlying progress is browser/event-loop driven;
- `map_async` has a separate mapping lifecycle.

WebGPU, Vulkan and D3D12 support a monotonic submission-completion abstraction but not one universal
public fence/poll object.

G5 therefore exposes RunenGPU-owned:

```text
GpuSubmissionId
GpuSubmission outcome
GpuReadback result
GpuContext::progress()
```

WGPU submission indices/poll types, Vulkan/D3D12 synchronization objects, browser promises and
mapped ranges remain private backend mechanisms.

RunenGPU owns no mandatory Tokio/Futures executor or implicit immortal progress thread. Host/event-
loop policy owns when progress is driven.

# Prepared and in-flight capacity

A later review found that bounded in-flight work alone is insufficient: asynchronous preparation
and published prepared values can pin G4 realization records before queue admission.

Final G5 execution-pressure domains are:

```text
max_prepared_submissions
max_in_flight_submissions
max_upload_bytes_in_flight
max_readback_bytes_in_flight
max_pending_readbacks
```

`max_prepared_submissions` counts both active async prepare reservations and published prepared
values.

Preparation reserves one RAII slot before asynchronous realization. Normal error, future
cancellation/drop or owned-realization abandonment releases the slot. Successful submit atomically
converts prepared capacity into in-flight/staging capacity. Pressure rejection allocates no
submission ID and preserves retryable prepared work with its slot.

There is no hidden accepted-but-unsubmitted queue.

# G4 realization retention — no second G5 retirement registry

Accepted G4 realized handles are clone-only `Arc<Record>` values. G4 resource/program/pipeline
registries remain the bounded realization lookup/cache authorities and already collect lookup-only
records while never evicting records retained by live handles.

An intermediate G5 plan introduced `max_deferred_retirement_records`. That is rejected as duplicate
realization-lifetime authority.

Final rule:

- preparing/prepared/in-flight/readback G5 state retains exact accepted G4 realization Arcs while
  they are needed;
- safe GPU/readback progress determines when G5 may release those references;
- after release, normal G4 lookup-only collection owns reclamation under G4 capacity policy;
- G5 introduces no second resource/program/pipeline retirement registry or identity;
- G5-private upload/readback staging belongs directly to the lifecycle record using it and is
  bounded by upload/readback/count pressure.

G5 owns **safe release timing**, not a parallel realization cache.

# Submission/readback lifecycle and acceptance point

No `GpuSubmissionId` exists before all synchronous admission requirements are satisfied.

Once an ID is allocated/published, the submission is accepted. Any later encoding, backend
validation, queue, health or device failure terminalizes that exact ID once instead of rolling
acceptance back into a submit rejection.

Public submission state remains intentionally small:

```text
Submitted -> Completed | Failed
```

Readback is separate:

```text
Pending -> Ready(GpuReadbackBytes) | Failed
```

A submission may be GPU-complete while dependent readback materialization remains pending.
Observer drop is not cancellation and never discards accepted work.

# Terminal record retention

A context-wide retained-terminal-history budget was considered and rejected. A slow inspector must
not backpressure unrelated future GPU work after backend cleanup is already safe.

Final rule:

- context registry owns nonterminal/backend-cleanup-pending lifecycle records;
- exactly-once terminal result is published into immutable shared state;
- after safe staging/referenced-record release, the context registry reference detaches;
- caller-held submission/readback observations may retain immutable terminal results without
  consuming future execution capacity;
- if all observers disappeared early, the context still terminalizes/cleans the accepted record and
  then discards it;
- G5 does not create an unbounded global terminal-history ledger.

# Current-host surface gap discovered by critical review

The first G5 planning pass treated physical Present as G7-owned, which is correct but incomplete for
renderer migration.

Accepted G4C2 surface authority simultaneously says:

- current acquired presentation surfaces may serve already-existing render-attachment roles;
- existing copy behavior remains separately retained;
- an acquired presentation surface is **not** an ordinary G4C1 realized resource;
- it is not a sampled/storage G4C2 shader resource;
- raw acquisition/configuration/presentation remains on the temporary current-host/G7 boundary.

G5C nevertheless intends to delete renderer raw command-encoder/resource execution reach-through.
Without another decision, the new private G5 encoder would have no legal way to encode current
surface-backed attachment/copy work before G7 exists.

## Rejected surface fixes

Do not:

- realize `SurfaceAcquired` through G4C1;
- expose raw `SurfaceTexture`/`TextureView` in public RunenGPU;
- add sampled/storage surface exceptions;
- add a broad external-resource import API;
- keep the old renderer execution bridge only for surfaces;
- implement reusable G7 surface identity/generation/recovery early.

## Final pre-G7 integration

Reusable G5B remains surface-independent. Preparation may retain typed unresolved
`SurfaceAcquired` attachment/copy/present requirements but no raw surface object. Ordinary
`submit_prepared` returns a typed `SurfaceBindingRequired` rejection, allocates no submission ID and
preserves prepared work when such requirements remain unresolved.

G5C composes the existing current-host surface owner with the **same** G5 submission transaction:

```text
prepared work
 -> reserve G5 submit capacity; no ID
 -> acquire current-host surface lease(s)
 -> validate logical SurfaceAcquired identity
    + context affinity
    + configured format/extent
    + allowed attachment/copy role
 -> commit G5 submission and allocate ID
 -> private G5 encoding / queue submit
 -> current-host owner physical present
```

If acquisition/binding fails before commit, provisional capacity is released, already-acquired
leases are dropped without presentation, prepared work is preserved/returned and no submission
exists.

Raw acquired surface values stay only inside the current-host/private-WGPU lexical integration.
They are not stored in logical work, cached as G4 realization, used for sampled/storage shader
binding, returned to renderer code or exposed through a generic callback.

Physical `present()` remains current-host/G7 migration ownership. G5 submission `Completed` means
GPU execution completion, not compositor/display completion. The current-host layer maps logical
`SurfaceAcquired` resource identities to its own surface slots; renderer-specific IDs do not enter
reusable RunenGPU contracts.

This terminal is explicit G7 deletion inventory, not a third generic G5 bridge. G7 later replaces
it with reusable typed surface capability/generation/acquisition contracts.

# Shutdown

Execution lifecycle is:

```text
Running -> ShuttingDown -> Closed
```

Shutdown rejects new prepare/submit admission. In-progress prepare cannot publish after shutdown or
stale-generation detection and releases its RAII slot when cancelled/abandoned. Prepared values
become non-submittable and release slots on drop. `progress()` remains valid while accepted work and
readbacks terminalize and private staging/G4 references are safely released.

Product timeout/block/yield/recovery policy remains Runenwerk. G7 owns device/surface recovery.

# Ordered implementation decomposition

Three slices remain sufficient and intentionally ordered:

```text
G5A executable logical work closure
 -> G5B complete surface-independent execution lifecycle
 -> G5C renderer/current-host integration and final cutover
```

## G5A

- executable compute/render logical contracts using G4 pipeline/binding types;
- semantic Eq without fabricated operation Ord/Hash;
- generic multi-draw render state with canonical dynamic values;
- transfer payload value/record-identity separation;
- graph-visible Upload/Readback operations;
- complete operation-derived accesses/requirements;
- renderer lowering into generic work;
- delete execution-semantic sidecar/manual duplicate access truth.

No command submission/progress implementation.

## G5B

- finite active-prepare/prepared/in-flight/staging/readback limits;
- cancellation-safe asynchronous preparation;
- atomic bounded submit admission and irreversible post-ID acceptance;
- private surface-independent encoding/submission;
- host-driven progress;
- exactly-once submission/readback terminal state;
- asynchronous normalized readback;
- G4 Arc retention/release without another retirement registry;
- terminal-record detachment and shutdown;
- one independent non-render compute/readback proof.

Renderer and current-host surface work remain temporarily outside reusable G5B submit.

## G5C

- owner-local current-host SurfaceAcquired binding composition with the same G5 submit transaction;
- renderer/UI/timing/capture migrate to accepted G5B execution;
- uploads/readbacks replace renderer staging/map/poll orchestration;
- delete `CurrentRenderDeviceQueue` and accessor;
- delete `CurrentRenderExecutionBridge` and accessor;
- delete renderer raw command encoder/queue submit/map/poll/acquired-surface-view execution ownership;
- retain only the separately classified current-host/G7 raw surface owner as G7 deletion inventory.

# Rejected alternatives summary

### Permanent `GpuExecutionBindings` sidecar
Rejected: G4B supplies complete logical pipeline/binding contracts; retaining a node-keyed companion
perpetuates transitional decomposition.

### Second backend-neutral command IR
Rejected: G3 already owns operation/access/hazard/order/initialization semantics.

### Move `CompiledPassExecutionPlan` into RunenGPU
Rejected: it mixes renderer planning/provenance with generic GPU execution.

### Hidden submission-only uploads/readbacks
Rejected: they bypass G3 initialization/hazard authority or force a second access model.

### Digest or payload allocation identity as semantic operation equality/order
Rejected: byte hashing is avoidable, allocation identity is not value equality, and stable digest
identity overreaches into persistence/content-addressing policy.

### Add `Ord` to G4 pipeline/program contracts
Rejected: accepted graph determinism does not depend on operation comparison and no justified total
ordering exists for those semantics.

### Second G5 realization-retirement registry
Rejected: G4 Arc-backed registries already own lookup/capacity/collection; G5 only needs safe
reference-release timing.

### Make G5B own reusable surfaces
Rejected: that steals G7 ownership and is unnecessary for proving the reusable surface-independent
execution lifecycle.

# Remaining uncertainty before owner review

No unresolved architectural unknown is currently known. Remaining work is mechanical proof:

- make every planning artifact agree with the corrected decisions;
- review the complete PR diff for connector/write drift;
- require exact-head canonical CI and Documentation Build success;
- verify no unresolved review thread or accidental implementation activation;
- stop at owner review before merge or Rust implementation.