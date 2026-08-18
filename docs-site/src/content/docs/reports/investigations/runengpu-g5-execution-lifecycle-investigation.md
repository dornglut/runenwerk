---
title: RunenGPU G5 Execution Lifecycle Replanning Investigation
description: Exact-current-main source and WGPU 30 investigation for executable logical GPU work, execution lifecycle, transfers, progress, surface sequencing, and the final renderer cutover.
status: active
owner: gpu
layer: reports
canonical: true
last_reviewed: 2026-08-18
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runengpu-g3r-definite-initialization-correction.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runengpu-g4c2-presentation-surface-binding-boundary.md
  - ../../design/active/runengpu-g5-execution-lifecycle-design.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Execution Lifecycle Replanning Investigation

## Question

What is the exact accepted RunenGPU/renderer execution boundary after G3R and the WGPU/Naga 30
refresh, and which contracts must G5 establish so RunenGPU becomes the backend-neutral logical GPU
execution layer without exposing WGPU, duplicating the G3 work graph, or creating a disposable
surface architecture before G7?

## Authority and exact baseline

Issue `#284` owns fresh G5 planning. The earlier WGPU-27 planning PR `#285` is superseded historical
evidence and is not an implementation or merge base.

The exact accepted source baseline for this investigation is:

```text
d7afaba20a27901e7a6bc4d6d75e6c761c7cbf86
```

That revision contains, in order:

```text
accepted G4C3
 -> accepted G3R semantic correction
 -> accepted G4R WGPU/Naga 30 refresh
```

The fresh planning branch is created directly from that revision. No source, Cargo, lockfile,
workflow, roadmap sequence, or external repository change is authorized by this investigation.

## Evidence inspected

### Accepted repository authority

- ADR 0015 and repository-family dependency direction;
- the canonical RunenGPU architecture and G3/G3R/G4 designs;
- the G4C2 presentation-surface boundary;
- the durable RunenGPU sequence in `workspace/planning/roadmap.md`;
- issue `#284`, including the owner-review corrections that caused the WGPU-27 G5 plan to be
  superseded;
- accepted G3R and G4R source and pull-request evidence.

### Current source

The current census covered at least:

```text
engine/src/plugins/gpu/api/context.rs
engine/src/plugins/gpu/api/context/facts.rs
engine/src/plugins/gpu/api/capability.rs
engine/src/plugins/gpu/api/work.rs
engine/src/plugins/gpu/api/program/runtime_binding/value.rs
engine/src/plugins/gpu/backend/wgpu/state.rs
engine/src/plugins/gpu/backend/wgpu/current_host.rs
engine/src/plugins/gpu/backend/wgpu/adapter_mapping.rs
engine/src/plugins/gpu/backend/wgpu/device_request.rs
engine/src/plugins/gpu/backend/wgpu/program_binding_realization/**
engine/src/plugins/render/backend/wgpu_ctx.rs
engine/src/plugins/render/renderer/render_flow/execute.rs
engine/src/plugins/render/renderer/render_flow/execute_passes.rs
engine/src/plugins/render/renderer/render_flow/gpu_timing.rs
```

The exact current `GpuContext` is non-`Clone` and owns one private `WgpuContextState`. Any future
caller-clonable execution observation/prepared handles therefore need explicit detached/shared state;
they must not accidentally extend `GpuContext` ownership merely because they are observable.

### WGPU/WebGPU 30 behavior

Primary-source review covered WGPU 30 `Queue`, `ComputePass`, `RenderPass`, `Limits`,
`DownlevelFlags`, `InstanceDescriptor`/`InstanceFlags`, dynamic-offset behavior, mapping/progress
behavior, and the current WebGPU usage-scope and dispatch rules.

External behavior is evidence only. RunenGPU owns the public contract.

## Current accepted logical boundary

G1-G4 already establish the intended semantic/physical split:

```text
GpuWorkOperation / GpuPreparedWorkGraph
    logical resource + operation + order + access + initialization truth
                         |
                         v
GpuContext + admitted device facts
                         |
                         v
private resource/program/binding/pipeline realization
                         |
                         v
WGPU 30 backend objects
```

This is the correct long-term direction. G5 must extend the logical layer through execution rather
than expose `wgpu::CommandEncoder`, `wgpu::Queue`, submission indices, mapped ranges, or backend poll
objects.

No evidence justifies a second RunenGPU command graph. `GpuPreparedWorkGraph` remains the sole
logical correctness/order authority.

## Current source gaps

### Compute work is not executable yet

`GpuComputeOperation` currently owns dispatch dimensions and timestamp writes. It does not own the
accepted G4 compute-pipeline contract or runtime bindings. Consequently the G3 node can express
compute ordering/hazards but cannot independently encode a compute dispatch.

`GpuDispatchSize` also rejects a zero dimension. Current WebGPU dispatch validation permits counts
from zero through the admitted per-dimension maximum; a zero dimension produces no shader
invocations. The RunenGPU rejection is therefore an accidental policy ceiling rather than a required
portable semantic rule.

Each compute dispatch is also one WebGPU usage scope. Effective bound resources participate in that
scope even when the invocation grid is empty. G5 therefore needs per-dispatch writable-binding alias
validation in addition to G3 inter-node hazards; zero work cannot bypass that validation.

### Render work is not executable yet

`GpuRenderOperation` owns attachments, draw intents and timestamps, but a draw does not yet own its
pipeline, runtime bindings, vertex/index bindings or complete dynamic render state. The renderer
therefore still carries execution meaning outside the generic work graph.

The current generic validation also treats overlapping write-capable render accesses too broadly.
WebGPU makes one render pass one usage scope, but permits storage usage across different draws when
the pass-wide usage list remains compatible. Writable aliasing within the effective bindings of one
draw remains invalid. G5 must preserve both facts instead of collapsing them into one pass-wide
"any write overlap is invalid" rule.

### Indirect capability naming is narrower than the admitted backend fact

`GpuCapabilityFeature::IndirectDraw` is currently populated from WGPU
`DownlevelFlags::INDIRECT_EXECUTION`. WGPU defines that flag as support for indirect drawing **and
indirect dispatching**.

If G5 accepts indirect compute, keeping the public capability name `IndirectDraw` would make one
backend-neutral fact falsely render-specific. The clean semantic correction is one normalized
`IndirectExecution` capability used by both indirect draw and indirect dispatch; no compatibility
alias should survive the cutover.

### Indirect runtime validity is currently vulnerable to backend configuration

WGPU 30's `InstanceFlags::VALIDATION_INDIRECT_CALL` validates indirect argument-buffer contents and
turns invalid indirect draw/dispatch calls into no-ops. Its own documentation states that behavior is
undefined for invalid indirect arguments if that validation is disabled.

The accepted RunenGPU context path currently creates the WGPU instance with
`InstanceDescriptor::new_without_display_handle_from_env()`. WGPU environment processing can unset
`VALIDATION_INDIRECT_CALL` through `WGPU_VALIDATION_INDIRECT_CALL=0`.

That means merely renaming the public capability to `IndirectExecution` is insufficient: without a
private invariant, the same admitted RunenGPU program could have defined no-op semantics in one
process environment and undefined backend behavior in another.

G5A must therefore make portable invalid-indirect-call behavior part of the normalized capability
contract and force/prove the required private WGPU mechanism after environment options are resolved.
Environment/debug configuration may tune diagnostics but may not weaken accepted RunenGPU semantics.
A backend unable to provide an equivalent guarantee cannot admit `IndirectExecution`.

### Color clear semantics are accidentally normalized-color-only

Current `GpuColorClearValue` validation restricts all components to finite `[0, 1]` values. WebGPU
color clear values are finite numeric values converted according to the target attachment format;
the `[0, 1]` restriction is not the generic color-attachment contract. Depth clear remains the
separate normalized `[0, 1]` case.

G5 should retain finite canonical color components and perform target-format-aware admission rather
than encode a normalized-color ceiling into the logical value type.

### Dynamic offsets are incorrectly part of physical bind-group identity

`GpuRuntimeBufferBinding` correctly carries a logical `u64` dynamic offset. However current G4C2
bind-group realization keys and records retain the complete runtime binding value, so two uses that
differ only by dynamic offset create different physical bind-group identities.

WGPU bind-group creation uses the static buffer offset/size. The dynamic offset is supplied later to
`set_bind_group` and added to the static offset for that use. Therefore:

```text
logical binding use
    = static binding identity + dynamic execution offset

physical bind-group identity
    = layout + resources + static offsets/sizes only
```

G5A is the right clean-cut owner for this correction because executable work introduces the first
proper per-use binding state. The old physical key/accessor must be replaced, not retained as a
parallel compatibility path.

### Current normalized limits are insufficient for execution admission

`GpuLimits` currently normalizes five facts:

```text
max_uniform_buffer_binding_size
max_storage_buffer_binding_size
max_color_attachments
max_vertex_buffers
max_bindings_per_group
```

The first planning draft said G5 needed "at least" compute workgroups per dimension, bind-group count,
and the combined bind-group/vertex-buffer limit. Acceptance review rejected that wording as
non-decision-complete: implementation would still have to choose public/admitted limit vocabulary.

For the G5A operations actually accepted by this plan, the exact additional normalized set is:

```text
max_texture_dimension_2d
max_bind_groups
max_bind_groups_plus_vertex_buffers
max_dynamic_uniform_buffers_per_pipeline_layout
max_dynamic_storage_buffers_per_pipeline_layout
max_compute_workgroups_per_dimension
```

These cover explicit viewport bounds, complete runtime binding/pipeline-layout use, simultaneous
render bind-group + vertex-buffer slots, dynamic buffer declarations, and compute dispatch. Existing
binding-size/count, vertex-buffer, color-attachment, and dynamic-alignment facts remain owners of
their already-normalized constraints.

Other WGPU limits are not implicitly authorized. If implementation proves one of G5A's accepted new
public operations still has a pre-encoding device-limit rejection absent from this closed set, that
is a planning defect requiring amendment, not permission to mirror WGPU opportunistically.

### The renderer still owns the final raw operation interval

Current `Renderer::render_packet` intentionally has two phases:

1. accepted G4 resource/program/binding/pipeline realization through RunenGPU;
2. one `CurrentRenderDeviceQueue` loan for uploads, command encoding, queue submission and blocking
   readback.

The remaining raw operation interval performs:

- `Device::create_command_encoder`;
- queue buffer/texture writes;
- compute/render/copy encoding through the purpose-typed `CurrentRenderExecutionBridge`;
- `Queue::submit`;
- timestamp and capture readback mapping;
- device polling;
- current-host surface presentation.

This is an intentionally narrow migration seam, not stable architecture. G5/G7A must delete it.

### Readback is renderer-owned and synchronously native-shaped

The current timing readback path calls `map_async`, then `Device::poll(Wait)` and blocks on a channel.
WGPU explicitly states that device wait/poll does not provide the same behavior on the WebGPU
backend, where callbacks are driven by the browser event loop.

A public RunenGPU `wait_for_wgpu_poll` model would therefore be native leakage. The public contract
needs backend-neutral nonblocking progress/observation, with native polling and browser event-loop
callbacks remaining private realization details.

### Queue writes are not transactionally coupled to logical submission

WGPU `Queue::write_buffer` and `write_texture` stage data immediately but begin GPU execution only on
the next `Queue::submit`, before that submission's command buffers. If G5 performs queue writes for a
logical submission and then rejects or fails before the intended queue submit, those staged writes
can be flushed by later work.

That violates G3's one-authority initialization/hazard model and G5's exactly-one logical submission
boundary. The conservative G5 baseline must therefore lower logical Upload through encoded staging
copies. Queue-write optimization is allowed only after a proof shows no rejected/failed logical
submission can strand writes for a later submit.

## WGPU 30 findings that affect G5 semantics

### Direct and indirect dispatch

WebGPU direct dispatch validates each dimension against the device's
`maxComputeWorkgroupsPerDimension`; zero is valid and means an empty invocation grid.

Indirect dispatch reads exactly three tightly packed `u32` workgroup counts (12 bytes) from an
indirect buffer. Runtime validity is evaluated on GPU/backend execution data rather than by host
planning. Invalid runtime indirect arguments are a non-executing indirect operation under the
portable contract; they are not permission for undefined native behavior.

WGPU's downlevel `INDIRECT_EXECUTION` fact covers both indirect draws and dispatches, while WGPU's
private `VALIDATION_INDIRECT_CALL` mechanism is what preserves defined no-op behavior for invalid
runtime argument contents on its native backends. Because WGPU allows that mechanism to be changed by
environment configuration, RunenGPU must restore/force it privately or reject the capability.

Therefore a backend-neutral G5 compute intent can support:

```text
Direct { x, y, z }
Indirect { buffer, offset }
```

with one normalized indirect-execution capability and one portable runtime-validity guarantee.

### Compute and render usage scopes

WebGPU defines:

- each compute dispatch as its own usage scope;
- an entire render pass as one usage scope;
- storage/storage combinations as pass-compatible;
- attachment/attachment combinations as pass-compatible where subresources themselves are valid;
- overlapping writable bindings in one effective draw/dispatch as an aliasing error.

RunenGPU therefore needs both compute-dispatch-local writable alias validation and the render split:

```text
compute dispatch usage/alias validation

render pass-wide usage compatibility
!=
draw-local writable-binding alias validation
```

Zero direct dispatch and runtime-invalid indirect dispatch still retain the conservative binding usage
scope even though they perform no shader invocations.

### Dynamic offsets

WGPU `set_bind_group` receives dynamic offsets in declaration/binding order and validates their
alignment against admitted uniform/storage alignment limits. The logical RunenGPU offset remains
`u64`; private WGPU narrowing to its `DynamicOffset` domain is a checked realization detail.

### Progress and mapping

WGPU completion/map callbacks require host/backend progress. Native code may call device/instance
polling privately. WebGPU callbacks are event-loop driven and a native-style blocking `Device::poll`
wait has no portable public meaning.

This supports one RunenGPU contract:

```text
GpuContext::progress()
    nonblocking progress/observation pump

GpuSubmission observation
    Accepted -> Completed | Failed

GpuReadback observation
    Pending -> Ready | Failed
```

`Accepted` is a RunenGPU admission fact, not a claim that the backend queue has already physically
submitted the work.

## Correct execution ownership model

The fresh G5 target should be:

```text
RunenRender / non-render consumer
        |
        v
complete GpuWorkOperation values
        |
        v
GpuPreparedWorkGraph
  semantic correctness/order/access/initialization authority
        |
        v
GpuContext execution preparation
  device-limit admission + private G4 realization + staging plan
        |
        v
single-use GpuPreparedSubmission
        |
        v
atomic RunenGPU acceptance
        |
        +--> public GpuSubmissionId / Accepted fact
        |
        v
private WGPU command encoding + queue submission + callbacks
        |
        +--> submission completion/failure
        +--> independent readback materialization
```

There is no public raw command buffer, queue, poll type, fence, submission index, mapped range or
surface object in this model.

## Prepared-work lifecycle finding

A caller-held `GpuPreparedSubmission` must not keep context execution authority alive indefinitely.
The context therefore needs an owner-local prepared-record registry; the public prepared value is a
single-use ticket/handle, not the owner of backend execution state.

Required lifecycle:

```text
prepare reservation
 -> published prepared record
 -> submit acceptance OR drop/revoke
```

Preparation reserves capacity before asynchronous realization and releases it through RAII on error,
cancellation or abandonment.

`begin_shutdown()`:

- rejects new preparation and submission acceptance;
- revokes context-owned prepared records and releases their execution capacity/G4 realization refs;
- leaves already accepted submissions observable/progressable until terminal;
- never depends on callers dropping prepared handles.

Last-`GpuContext` Drop is distinct from graceful shutdown. It is an abrupt, nonblocking owner loss:
nonterminal accepted observations terminalize with a structured context-drop failure, no public
promise is made that hardware work was synchronously cancelled, and private backend/driver lifetime
rules remain responsible for already-issued physical work. Detached terminal observation data may
outlive the backend, but it cannot retain device/queue execution authority.

## Submission acceptance finding

There must be a single irreversible semantic acceptance point.

Before acceptance, `submit_prepared` may reject because of:

- wrong context/device generation;
- revoked/consumed prepared ticket;
- shutdown state;
- finite in-flight/staging/readback pressure;
- unresolved surface requirements in the reusable surface-independent path.

Pre-acceptance rejection:

```text
allocates no GpuSubmissionId
performs no queue action
returns ownership of the prepared value to the caller
```

After capacity conversion and ID publication, the submission is **Accepted**. Any subsequent
synchronous encode/queue/backend-health failure terminalizes that accepted ID exactly once. It is not
reported as a pre-acceptance `Err`.

This keeps Rust ownership, pressure handling and observation truth aligned.

## G5A / G5B / G7A / G5C decomposition

### G5A — executable logical work closure

G5A should be the only first implementation slice activated from accepted planning. It owns:

- executable compute pipeline/binding/dispatch intent;
- compute-dispatch usage/writable-alias validation;
- executable render draws with pipeline, bindings, vertex/index state and explicit dynamic state;
- graph-visible Upload and Readback logical operations;
- exact access derivation from runtime bindings and transfer operations;
- pass-wide versus draw-local render usage validation;
- direct zero-dispatch semantics;
- indirect dispatch plus the clean `IndirectExecution` capability and private runtime-validity
  enforcement correction;
- target-format-aware color clear semantics;
- static physical bind-group identity separated from dynamic per-use offsets;
- the closed six-field execution-required normalized limit addition;
- deletion of duplicate renderer GPU execution semantics where G5A becomes authoritative.

G5A does **not** submit work.

### G5B — surface-independent execution lifecycle

After accepted G5A, G5B owns:

- finite cancellation-safe preparation;
- private G4 realization collection and prepared execution records;
- atomic acceptance and `GpuSubmissionId`;
- private WGPU command encoding and queue submission;
- staging-copy Upload baseline;
- asynchronous readback/result normalization;
- nonblocking progress and exactly-once terminalization;
- G4 realization retention while accepted work needs it;
- graceful shutdown, prepared revocation and last-context Drop semantics;
- a genuine headless non-render compute/upload/readback proof;
- native and mandatory wasm/conformance evidence, plus real browser lifecycle evidence where
  infrastructure permits.

G5B remains surface-independent.

### G7A — minimal durable surface foundation

The durable roadmap correctly places G7A after G5A/B and before G5C. This investigation finds no
reason to change that ordering.

G7A must establish only reusable generic surface facts required by final execution integration:

```text
surface identity + context/generation affinity
surface capabilities / admitted configuration
acquisition generation/lease identity
normalized acquire outcomes
present acceptance/outcome
```

It must not import Winit types or Runenwerk window/recovery policy, and it must not yet implement the
full G7B loss/reconstruction system.

### G5C — final renderer/current-host execution cutover

G5C then migrates renderer/UI/timing/capture execution onto accepted G5B + G7A and deletes, without
aliases or forwarding replacements:

```text
CurrentRenderDeviceQueue
current_render_device_queue()
CurrentRenderExecutionBridge
current_render_execution_bridge()
```

Renderer image-formation semantics remain in RunenRender/Runenwerk. Generic GPU execution does not.

## Alternatives rejected

### Expose WGPU commands publicly

Rejected. It would make WGPU the public semantic ceiling, reintroduce backend reach-through, and make
WebGPU/native progress differences consumer policy.

### Add a second command IR below `GpuPreparedWorkGraph`

Rejected. It would duplicate operation/order/resource truth. Private backend encoding may build local
command structures, but they are derived implementation state only.

### Let environment/debug flags weaken indirect semantics

Rejected. WGPU's indirect-call validation setting is a private implementation mechanism whose absence
can make invalid indirect arguments undefined. A public `IndirectExecution` capability cannot change
meaning because `WGPU_VALIDATION_INDIRECT_CALL=0` was present in the process environment.

### Keep an open-ended G5A limit list

Rejected. "At least" leaves API/admission design to implementation. The six newly required normalized
fields are closed in planning; another required execution limit is evidence that planning must be
amended.

### Keep dynamic offsets in realized bind-group identity

Rejected. The backend object does not encode the dynamic offset; retaining it in the physical key
creates duplicate physical objects and false ownership.

### Use queue-write Upload as the default fast path

Rejected as the baseline because WGPU queue writes are flushed by the next submit and can cross a
logical failure boundary. Encoded staging copy is the semantics-first baseline.

### Complete surface support inside G5B

Rejected. It would create a disposable pre-G7 surface API or smuggle current-host/Winit policy into
RunenGPU. G7A exists specifically to avoid that architecture.

### Keep `IndirectDraw` and add a second `IndirectDispatch` backend capability

Rejected. WGPU's admitted backend fact is one indirect-execution capability. Two public capability
facts would falsely imply independent hardware admission. The clean public correction is one
`IndirectExecution` fact.

### Keep three G5 RON lifecycle specs during planning

Rejected. Current workspace authority explicitly makes specs subordinate bounded handoff contracts,
not a live phase database. Fresh planning should publish the durable G5 design and only the immediate
G5A implementation handoff. G5B, G7A and G5C get their own specs only when their accepted predecessor
makes a bounded implementation handoff useful.

## Evidence gaps and later proof obligations

This planning review does not claim runtime execution proof. In particular:

- no G5 Rust exists yet;
- no fresh G5A implementation issue is active yet;
- no browser runtime G5 lifecycle has been executed yet;
- no G5 scale/cost benchmark exists yet;
- no G7A surface implementation exists yet.

Those are delivery obligations, not reasons to leave the public semantics undefined.

## Decision

Fresh G5 planning should preserve the logical-semantics architecture and activate exactly one next
implementation slice after owner review:

```text
G5A executable logical work closure
```

G5A must complete operation meaning, compute/draw usage validation, indirect runtime-validity
semantics, the closed execution-limit vocabulary, and the static/dynamic binding cutover before G5B
takes ownership of backend execution. G5B must prove reusable headless execution before G7A surface
foundation and G5C renderer cutover. The durable roadmap sequence remains unchanged.
