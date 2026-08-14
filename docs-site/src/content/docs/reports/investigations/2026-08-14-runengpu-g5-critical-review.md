---
title: RunenGPU G5 Critical Review
description: Source-grounded pre-owner-review audit of G5 executable-work, lifecycle, retention, render-pass compatibility, and current-host surface integration decisions.
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

This is the final architecture/readiness audit before **owner review** of G5 planning.
It does not authorize merge or implementation. G5A/G5B/G5C Rust work remains blocked until the
planning PR is owner-reviewed and accepted and the corresponding implementation slice is separately
activated from its accepted predecessor.

Accepted source baseline:

```text
ed2bcc2dbe4a302032c2ee53b6972facba7f940e
```

Planning issue: `#284`

Planning PR: `#285`

## Evidence inspected

The review re-read accepted source rather than relying only on planning prose, including:

- `engine/src/plugins/gpu/api/work.rs`;
- `engine/src/plugins/gpu/api/graph/{authoring,preparation}.rs`;
- G4 program/pipeline/render-state/runtime-binding contracts;
- `engine/src/plugins/gpu/api/{context,realization}.rs`;
- G4 resource and pipeline realization registries;
- private WGPU context state and current execution loan;
- current UI multi-pipeline render-pass execution;
- current-host surface ownership and accepted G4C2 presentation-surface boundary;
- pinned WGPU 27.0.1 render-pass/dynamic-state validation behavior;
- G5 investigation/design/G5A/G5B/G5C specifications;
- exact-head CI failure evidence from PR `#285`.

## Verdict before corrections

The foundational direction was sound:

```text
one G3 logical work graph
+ accepted G4 contracts/realizations
+ one G5 execution lifecycle
+ no second command IR
+ no permanent renderer execution sidecar
```

It was not yet owner-review ready. The audit found design-level ambiguities that would otherwise
force implementation to invent architecture or compatibility behavior mid-slice.

# Findings and corrections

## 1. Operation `Ord` was accidental, not semantic authority

Current simple G3 operation payloads derive `PartialOrd/Ord/Hash`, but `GpuWorkNode` itself requires
value equality only. Deterministic graph preparation uses fragment/node/resource identities,
explicit dependencies and topological order; it does not order nodes by comparing operation
payloads.

Accepted G4 compute/render pipeline descriptors deliberately provide semantic value equality/hash
without a total `Ord` over program/pipeline meaning.

**Decision:** executable operations retain semantic `PartialEq/Eq` but do not preserve or invent
operation-level `PartialOrd/Ord/Hash` where the complete G4-backed payload has no justified natural
ordering/hash consumer. Do not add `Ord` to G4 pipeline/program contracts or order through labels,
pointers or naked hashes. A discovered real operation-order/hash consumer is a stop condition for
classification.

## 2. Transfer record identity must not replace semantic equality

An intermediate plan used `GpuTransferPayloadId` as operation equality/order/hash identity to avoid
repeatedly traversing upload bytes. That makes independently constructed byte-identical uploads
semantically unequal solely due allocation identity.

**Decision:** `GpuTransferPayloadId` is opaque process-local record/correlation identity only.
Semantic `GpuTransferPayload` equality compares checked immutable payload value semantics and
excludes the ID. Clones preserve ID and value; a separate `is_same_record`-style predicate exposes
record identity when needed. Executable operations do not require Hash/Ord, so graph scheduling
does not need to hash/sort upload bytes. A future digest may be bounded diagnostic/dedup evidence
only.

## 3. Dynamic render state needed canonical value semantics without invalid narrowing

The first correction correctly identified that viewport/blend floating state needs canonical finite
value semantics, but it initially required positive viewport/scissor extents.

Pinned WGPU 27.0.1 permits zero-area viewport/scissor state. Scissor is bounded by the render
extent; viewport size/position additionally depends on device limits.

**Decision:**

- viewport: finite canonical f32 bits, signed-zero normalization, **nonnegative** width/height,
  checked `0 <= min_depth <= max_depth <= 1`;
- scissor: checked integer rectangle, **zero area allowed**, checked arithmetic, must remain inside
  common logical render extent;
- blend constant: four finite canonical f64 values with signed-zero normalization; no invented
  0..1 clamp;
- stencil reference: `u32`.

G5A owns context-free value semantics. G5B preparation validates device-specific viewport maximum
size/position rules against admitted backend/device facts. Defaults are semantic values, not
inherited backend state.

## 4. G4 runtime binding declarations are sufficient for access derivation

Accepted G4B distinguishes storage-buffer `ReadOnly/ReadWrite` and storage-texture
`ReadOnly/WriteOnly/ReadWrite`; runtime binding values retain exact logical resources/ranges and
optional dynamic offsets.

**Decision:** derive compute/render bound-resource accesses from the existing interface/runtime-
binding authority. Delete renderer duplicate caller access truth. Validate the **complete** derived
operation access set, including binding-versus-attachment and multiple-write contradictions, before
backend encoding.

## 5. Render-pass and draw-pipeline compatibility must be logical authority

Once each `GpuRenderDraw` carries a G4 render-pipeline descriptor, the operation itself has all facts
needed to prove that pipelines can legally execute inside one render pass.

Pinned WGPU 27.0.1 validates common attachment render extent/sample count and compares pass versus
pipeline color formats, depth/stencil format, sample count and multiview setting.

**Decision:** G5A derives one pass signature from logical attachments:

```text
effective render extent
common attachment sample count
ordered color formats
optional depth/stencil format
```

Every active attachment must share extent/sample count. Every draw pipeline must exactly match
ordered color-target count/formats, depth/stencil presence/format and sample count. Existing G3
resolve compatibility remains authoritative. Default viewport/scissor derive from the validated
common extent.

Accepted RunenGPU has no multiview or 3D depth-slice render contract; G5A does not invent one. A
real current need is a separately accepted extension stop condition.

## 6. Reuse accepted G4 `GpuIndexFormat`

The first G5A draft described a normalized `GpuIndexFormat::{Uint16, Uint32}` as if execution needed
a new type. Accepted G4 render-pipeline state already owns exactly that RunenGPU type.

**Decision:** G5A reuses the accepted type for runtime index-buffer binding. No duplicate index enum
or WGPU re-export is created.

## 7. G5 must compose with G4 lifetime ownership, not duplicate it

G4 realized handles are clone-only `Arc<Record>` values. Resource and pipeline registries retain
ready records and collect lookup-only records under their own bounded capacity while never evicting
records held by live handles.

**Decision:** preparing/prepared/in-flight/readback G5 state retains exact G4 realization Arcs while
needed and releases them when GPU/readback safety permits. Normal G4 lookup-only collection remains
the sole realization reclamation/capacity authority.

Reject a second G5 resource/program/pipeline retirement registry and
`max_deferred_retirement_records`. G5-owned staging remains directly owned by lifecycle records and
bounded through upload/readback/count pressure.

## 8. Prepared capacity must include in-progress async preparation

Counting only published `GpuPreparedSubmission` values permits concurrent async prepares to pin G4
work without pressure accounting.

**Decision:** `max_prepared_submissions` counts:

```text
active async prepare reservations
+ published prepared submissions
```

Preparation acquires one RAII capacity token before asynchronous realization. Error, future
cancellation/drop, shutdown/stale-generation rejection or abandoned owned realization releases the
slot transactionally.

## 9. Submission acceptance needed a precise irreversible point

**Decision:** no `GpuSubmissionId` exists before all synchronous admission checks and required
surface-independent execution inputs succeed. Once ID allocation/publication occurs, submission is
accepted. Every later encoding/backend/queue/health/device failure terminalizes that exact ID once;
it cannot roll back into a pre-submit rejection.

Pressure rejection remains pre-ID, performs no queue work and preserves retryable prepared state.

## 10. Surface-backed rendering exposed a hidden G5/G7 gap

Accepted G4C2 establishes that an acquired presentation surface may retain current
attachment/copy/present roles but is not an ordinary G4C1 realized resource and is not a
sampled/storage G4C2 shader resource. Physical acquisition/configuration/present remains the
current pre-G7 host owner's responsibility.

G5C still intends to delete renderer raw encoder/queue/resource reach-through, so the new private G5
encoder needs one legal way to consume the current acquired surface without pretending G7 exists.

### Rejected approaches

Do not:

- realize `SurfaceAcquired` through G4C1;
- expose raw `SurfaceTexture`/`Texture`/`TextureView` in public RunenGPU;
- add sampled/storage surface exceptions;
- create a broad external-resource import API;
- retain the old renderer execution bridge merely for surfaces;
- implement reusable G7 surface identity/generation/recovery early.

### Decision

Generic G5B remains surface-independent. Preparation records typed unresolved
`SurfaceAcquired` attachment/copy/present requirements but no raw surface object. Ordinary
`submit_prepared` returns `SurfaceBindingRequired`, allocates no ID and preserves prepared work.

G5C composes the existing current-host owner with the same submit transaction:

```text
prepared work
 -> reserve G5 submit capacity; no ID
 -> map logical SurfaceAcquired IDs to current-host slots
 -> acquire all required leases in deterministic logical-ID order
 -> validate affinity + logical ID + format/extent + authorized role
 -> commit and allocate GpuSubmissionId
 -> private G5 encode/Queue::submit
 -> current-host owner may attempt present
```

Partial acquisition/binding failure before commit drops already acquired leases without present,
releases provisional capacity, preserves prepared work and creates no submission ID.

Attachment encoding receives only the matching acquired `TextureView`; copy encoding receives only
the matching acquired `Texture`. Both are lexical private values and never become G4 realization,
logical work, renderer authority or generic callback data. Sampled/storage use remains forbidden.

If encoding fails after ID commit but before `Queue::submit` returns, the submission terminalizes as
Failed and the surface lease is dropped without present.

After `Queue::submit` returns, the current-host owner may call its existing `present()` terminal.
That is a **presentation attempt**, not GPU-completion or display/compositor-completion evidence. A
later asynchronous execution/device failure may be observed after a present attempt; the G5
submission outcome and pre-G7 presentation-attempt fact remain separate rather than retroactively
rewriting each other.

This owner-local terminal is explicit G7 deletion inventory, not a third generic G5 bridge.

## 11. G5B remains one complete lifecycle slice

Splitting submission from completion/readback/shutdown more aggressively was considered and remains
rejected. Accepting a public submit API while terminal outcome, readback, retention or shutdown
semantics remain undefined would knowingly create an incomplete lifecycle and compatibility churn.

Keep:

```text
G5A executable logical work
 -> G5B complete surface-independent lifecycle
 -> G5C current renderer/current-host integration and final cutover
```

G5B should decompose internally by source ownership/state-machine responsibility rather than by
accepting a partial public lifecycle.

# Additional implementation invariants

The implementation specs preserve these conclusions:

- full operation-derived access contradictions reject before backend encoding;
- pass attachments and every draw pipeline are compatible before backend encoding;
- zero-area viewport/scissor remain valid;
- device-specific viewport bounds are G5B preparation facts;
- queue-write upload lowering is permitted only when exactly equivalent to graph position;
- independent G5B proof is surface-independent and RunenRender-free;
- readback result completion is distinct from submission GPU completion;
- observer drop never cancels accepted work;
- terminal result handles detach from execution capacity after safe internal cleanup;
- callbacks/wakers execute outside internal locks;
- accepted G4 health/error-attribution authority remains single owner;
- G5 adds no global context, implicit executor, immortal progress thread or public backend
  synchronization primitive.

# Owner-review readiness criteria

Before owner review, PR `#285` must have one frozen exact head where:

1. focused design and G5A/G5B/G5C specs encode every correction above;
2. investigation/index lifecycle is consistent and discoverable;
3. no unrelated root-doc or Rust implementation change is present;
4. complete PR diff has been reviewed for accidental connector-write drift;
5. exact-head canonical CI succeeds;
6. exact-head Documentation Build succeeds;
7. no unresolved review thread or hidden implementation activation exists;
8. PR remains planning-only and no G5 implementation issue has been activated.

At that point the next action is **owner review** and work stops at that gate.