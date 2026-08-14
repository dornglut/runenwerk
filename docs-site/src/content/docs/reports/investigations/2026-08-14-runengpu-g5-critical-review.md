---
title: RunenGPU G5 Critical Review
description: Source-grounded pre-owner-review audit of G5 executable-work, lifecycle, retention, and current-host surface integration decisions.
status: active
owner: gpu
layer: reports
canonical: true
last_reviewed: 2026-08-14
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g5-execution-lifecycle-design.md
  - ../../design/active/runengpu-g4c2-presentation-surface-binding-boundary.md
  - ./runengpu-g5-execution-lifecycle-investigation.md
  - ../../workspace/specs/pt-runengpu-g5a-executable-work-contracts.ron
  - ../../workspace/specs/pt-runengpu-g5b-execution-lifecycle.ron
  - ../../workspace/specs/pt-runengpu-g5c-renderer-cutover.ron
  - ../../workspace/planning/roadmap.md
---

# RunenGPU G5 Critical Review

## Review gate

This review is the final architecture/readiness pass before **owner review** of G5 planning.
It does not authorize merge or implementation. G5A/G5B/G5C Rust work remains blocked until
the planning PR is owner-reviewed and accepted, and each implementation slice is separately
activated from its accepted predecessor.

Accepted source baseline:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

Planning issue: `#284`

Planning PR: `#285`

## Evidence inspected

The review re-read the accepted G3/G4 source rather than relying only on planning prose,
including:

- `engine/src/plugins/gpu/api/work.rs`;
- `engine/src/plugins/gpu/api/graph/{authoring,preparation}.rs`;
- G4 pipeline descriptors and runtime binding declarations;
- `engine/src/plugins/gpu/api/{context,realization}.rs`;
- G4 resource and pipeline realization registries;
- private WGPU context state and current execution loan;
- current UI multi-pipeline render-pass execution;
- current-host surface ownership and the accepted G4C2 presentation-surface boundary;
- G5 investigation, focused design, and G5A/G5B/G5C specifications;
- exact-head CI failure evidence from PR #285.

## Verdict before corrections

The initial G5 direction was fundamentally sound:

```text
one G3 logical work graph
+ G4 logical/program/resource realization contracts
+ one G5 execution lifecycle
+ no second command IR
+ no permanent render execution sidecar
```

However, it was **not yet owner-review ready**. The review found several design-level defects
that would otherwise force G5 implementation to invent architecture mid-slice.

The corrections below are binding planning decisions, not implementation changes.

# Findings and corrections

## 1. Operation `Ord` was accidental, not semantic authority

### Accepted-source fact

Current simple G3 operation payloads derive `PartialOrd/Ord/Hash`, but `GpuWorkNode` itself
requires only value equality. Deterministic graph preparation uses fragment/node/resource
identities, BTree maps keyed by those identities, explicit dependencies, and topological order.
It does not order nodes by comparing `GpuWorkOperation` payloads.

Accepted G4 compute/render pipeline descriptors deliberately provide complete value `Eq + Hash`
but do not define a total `Ord` over program/pipeline semantics.

### Rejected approach

Do **not** add `Ord` to G4 program/pipeline descriptors, and do not order operations through
labels, descriptor hashes, pointer identity, or backend objects merely to preserve an old derive.

### Decision

G5A keeps semantic `PartialEq/Eq` for executable operations but removes operation-level
`PartialOrd/Ord/Hash` where the complete executable payload no longer has a justified natural
ordering/hash consumer. Lower-level identities/ranges/enums may retain their independent natural
ordering.

If implementation discovers a real operation-order/hash consumer, that is a stop condition for
classification rather than permission to manufacture an ordering.

## 2. Transfer record identity must not replace semantic equality

### Problem

The intermediate plan used `GpuTransferPayloadId` as operation equality/order/hash identity to
avoid repeatedly traversing large upload bytes. That makes two independently constructed,
byte-identical uploads semantically unequal solely because allocation identity differs.

### Decision

`GpuTransferPayloadId` is opaque process-local **record/correlation identity** only.

`GpuTransferPayload` semantic equality compares checked immutable payload value semantics
(layout/shape/bytes as applicable) and excludes the record ID. Clones preserve both value and
record ID. A separate `is_same_record`-style predicate distinguishes record identity when needed.

Because executable operations no longer require operation-level `Hash/Ord`, graph preparation
never needs to hash/sort upload bytes as a hot-path ordering mechanism. A future digest may be
bounded diagnostic/dedup evidence only; it is not semantic identity.

## 3. Dynamic render state needed canonical value semantics

### Problem

The initial design named viewport, scissor, blend constant, and stencil reference but did not bind
how floating values participate in deterministic equality.

### Decision

G5A uses typed normalized values:

- viewport: finite canonical f32 bits, signed-zero normalization, positive extent, checked depth
  range;
- scissor: checked integer rectangle with positive extent and attachment bounds;
- blend constant: four finite canonical f64 components with signed-zero normalization;
- stencil reference: `u32`.

Defaults are semantic values, not inherited backend state. Each draw has one complete effective
state; private lowering may elide redundant setters.

## 4. G4 runtime binding declarations are sufficient for access derivation

The review verified that accepted G4B binding semantics distinguish storage-buffer
`ReadOnly/ReadWrite` and storage-texture `ReadOnly/WriteOnly/ReadWrite`, while runtime binding
values retain exact logical resources/ranges/dynamic offsets.

Therefore G5A can derive compute/render bound-resource accesses from the existing single
interface/binding authority. A second renderer-authored access list is unnecessary and remains
scheduled for deletion.

G5A must run contradiction/overlap validation across the **complete** derived operation access
set, not only attachments/indirect resources, so a runtime binding cannot silently conflict with
an attachment or another incompatible write in the same operation.

## 5. G5 must compose with G4 lifetime ownership, not duplicate it

### Accepted-source fact

G4 realized handles are clone-only `Arc<Record>` values. Resource and pipeline registries retain
ready records and collect lookup-only records under their own bounded capacity, while never
evicting records retained by live handles.

### Rejected approach

Do not add a second G5 resource/program/pipeline retirement registry or a duplicate
`max_deferred_retirement_records` pressure domain.

### Decision

Prepared/in-flight/readback G5 records retain the exact G4 realization Arcs they need. Releasing
those Arcs after safe execution/readback makes records lookup-only; normal G4 capacity collection
may reclaim them. G5-owned staging objects remain owned by the in-flight/readback records and are
bounded by upload/readback/count pressure.

G5 owns **when references may be released safely**; G4 remains the realization lookup/cache owner.

## 6. Prepared capacity must include in-progress async preparation

A limit that counts only already-published `GpuPreparedSubmission` values allows many concurrent
async prepares to reserve G4 work without entering pressure accounting.

`max_prepared_submissions` therefore counts both:

```text
in-progress prepare reservations
+ published prepared submissions
```

Preparation obtains one RAII slot before asynchronous realization. Normal failure, future
cancellation/drop, or abandoned owned realization releases the slot transactionally.

## 7. Submission acceptance needed a precise irreversible point

No `GpuSubmissionId` exists before all synchronous admission checks and required execution inputs
succeed.

After ID allocation/publication, the submission is accepted. Any later encoding, queue, backend,
health, or device failure terminalizes that exact ID once. It cannot be converted back into a
pre-submit rejection.

Pressure rejection remains retryable and preserves prepared work because it occurs before ID
allocation and queue activity.

## 8. Surface-backed rendering exposed a hidden G5/G7 gap

### Accepted-source facts

The G4C2 surface decision established that an acquired presentation surface:

- may remain a render attachment;
- may retain current copy behavior;
- is not an ordinary G4C1 realized resource;
- is not a sampled/storage G4C2 shader resource;
- remains physically acquired/presented by the current pre-G7 host owner.

G5C nevertheless intends to delete renderer raw encoder/queue/resource execution reach-through.
Without another decision, the new G5 encoder would have no legal way to bind the acquired surface
attachment/copy resource.

### Rejected approaches

Do not:

- realize `SurfaceAcquired` through G4C1;
- add raw `SurfaceTexture`/`TextureView` to public RunenGPU;
- add sampled/storage surface exceptions;
- create a broad external-resource import API;
- implement reusable G7 surface identity/generation/recovery early;
- keep the old renderer execution bridge merely for surfaces.

### Decision

Generic G5B stays surface-independent. Preparation may retain typed unresolved
`SurfaceAcquired` attachment/copy/present requirements but no raw surface object. Ordinary
`submit_prepared` returns a typed `SurfaceBindingRequired` rejection and preserves the prepared
value when such requirements remain unresolved.

G5C composes the existing current-host surface owner with the same G5 submit transaction through
one owner-local lexical terminal:

```text
prepared work
 -> reserve submit capacity, no submission ID
 -> acquire current-host surface lease(s)
 -> validate affinity + logical resource + format/extent + allowed role
 -> commit G5 submission and allocate ID
 -> private G5 encode/queue submit
 -> current-host owner physical present
```

Surface acquisition/binding failure before commit releases provisional capacity, drops any
already-acquired leases without presentation, returns/preserves prepared work, and creates no
submission ID.

Raw acquired surface objects remain only inside the current-host/private-WGPU integration call.
They may serve the already-existing attachment/copy roles but never shader sampled/storage
binding. Physical `present()` remains current-host/G7 migration ownership; G5 submission
completion does not claim compositor/display completion.

This terminal is explicit G7 deletion inventory, not a third generic G5 bridge.

## 9. G5B remains one complete lifecycle slice

The review considered splitting submission from completion/readback/shutdown more aggressively.
That remains rejected. Accepting a public submit API while terminal outcome, readback, shutdown, or
retention semantics are still undefined would create a knowingly partial lifecycle and force
compatibility churn.

The existing decomposition remains preferable:

```text
G5A executable logical work
 -> G5B complete surface-independent lifecycle
 -> G5C current renderer/current-host integration and final cutover
```

G5B is large, but its internal concerns share one state machine and one acceptance/terminality
contract. Decomposition should happen by source modules and responsibilities inside the slice,
not by accepting an incomplete lifecycle API.

# Additional implementation invariants

The implementation specs must preserve these review conclusions:

- full operation-derived access contradiction checks happen before backend encoding;
- queue-write upload lowering is permitted only when provably equivalent to the upload node's graph
  position; otherwise private staging/copy preserves exact order;
- independent G5B proof is surface-independent and does not depend on RunenRender;
- readback result completion remains separate from submission GPU-completion;
- observer drop never cancels accepted work;
- terminal result handles detach from context execution capacity after safe internal cleanup;
- callbacks/wakers run outside internal locks;
- error-attribution/health authority remains the accepted single G4 owner;
- G5 adds no process-global context, implicit executor, immortal progress thread, or public backend
  synchronization primitive.

# Owner-review readiness criteria

Before owner review is requested, PR `#285` must have one frozen exact head where:

1. the focused design and G5A/G5B/G5C specs encode every correction above;
2. investigation/index lifecycle is consistent and discoverable;
3. no unrelated root-doc or implementation change is present;
4. complete PR diff has been reviewed for accidental connector rewrite drift;
5. exact-head canonical CI succeeds;
6. exact-head Documentation Build succeeds;
7. no unresolved review thread or hidden implementation activation exists;
8. PR remains planning-only and no G5 implementation issue has been activated.

At that point the next action is **owner review**, and work stops at that gate.