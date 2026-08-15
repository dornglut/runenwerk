---
title: RunenGPU G3R Initialization Semantics Correction
description: Canonical correction separating access/hazard envelopes from exact initialization requirements and operation-guaranteed initialization effects.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-15
related_docs:
  - ./runengpu-g3-access-work-graph-design.md
  - ./runengpu-architecture-design.md
  - ../../workspace/planning/roadmap.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
---

# RunenGPU G3R Initialization Semantics Correction

## Authority

This document is a narrow semantic correction to the accepted
[RunenGPU G3 Access and Work Graph Design](runengpu-g3-access-work-graph-design.md).

It supersedes that design where initialization simulation treats a coarse access envelope as exact
initialized-content truth. G3 access, hazard, identity, composition, dependency, deterministic
preparation, and phase ownership otherwise remain unchanged.

The corrected model has three distinct facts:

```text
GpuResourceAccess
    may read / may write
    usage + hazard envelope

initialization requirement
    exact content that must already be initialized

initialization effect
    exact content guaranteed initialized afterward
```

An access envelope can be a strict superset of an exact requirement or effect. That is intentional.

## Why the correction is required

### Shader writes

A shader may have permission to write an entire buffer range or texture subresource while
conditionals, indices, invocation count, early exits, or data-dependent control flow cause only part
of that region—or none of it—to be written.

Therefore `StorageWrite` and `StorageReadWrite` remain write hazards, but establish **no definite
initialized coverage by default**.

### Padded buffer-texture copies

A buffer-texture copy can use row and image strides. Its coarse contiguous buffer hazard envelope can
contain padding bytes that the logical copy does not read or write.

Consequently:

- buffer-to-texture must not require padding bytes to be initialized merely because they lie inside
  the hazard envelope;
- texture-to-buffer must not mark padding bytes initialized merely because they lie inside that
  envelope;
- exact logical copy coverage must remain compact instead of expanding one row at a time.

### Compute timestamp writes

The existing renderer timing path records compute-pass timestamp writes as generic query access even
though that access does not encode the command that writes the timestamps. Treating the access kind
as effect authority would recreate the same access/semantics conflation G3R is correcting.

Compute timestamp writes therefore become explicit checked `GpuComputeOperation` state, parallel to
existing `GpuRenderOperation` timestamp-write state. This preserves existing instrumentation while
removing generic access as a command/effect compatibility bridge.

## One owner per fact

### Access and hazard truth

`GpuResourceAccess` continues to own:

- may-read and may-write intent;
- normalized buffer, texture-subresource, query, and sampler hazard coverage;
- usage compatibility;
- RAW, WAR, and WAW dependency inference;
- typed overlap evidence.

G3R does not weaken or redesign hazard ordering.

### Initialization requirement truth

Graph preparation separately derives the exact content that must already be initialized.

Generic shader/caller accesses have no narrower checked operation proof, so a readable generic access
requires its declared may-read envelope. Checked operations may provide a narrower exact requirement
than their hazard envelope. The initial required case is a padded buffer-to-texture source, whose
logical row/image bytes are required while padding bytes are not.

### Definite initialization truth

Graph preparation separately derives effects guaranteed by checked operation semantics. The effect
model is internal preparation authority, not caller-authored authority.

G3R does **not** add or retain equivalents of:

```text
GpuComputeNodeBuilder::definite_storage_write
GpuWorkFragmentBuilder::add_node_with_definite_initialization
GpuWorkNode::definite_initialization as caller/compiler evidence
RenderPassNode::definite_initialization
renderer shader-name/specialization-constant postcondition inference
```

A future compiler or higher-level proof system may introduce an opaque checked shader-proof contract
only through a separately reviewed design. A non-public test seam may inject an effect only when
needed to prove structural contradiction handling; it must not be reachable from ordinary authoring
or renderer paths.

### Pass timestamp command truth

Timestamp writes are command semantics, not caller-access semantics.

`GpuComputeOperation` and `GpuRenderOperation` own their checked timestamp-write state. Each operation
derives the corresponding query write access for capability and hazard purposes, and graph
initialization derives exact query effects from that operation-owned state.

Generic caller-authored `GpuQueryAccessKind::WriteTimestamp` is never converted into a definite
effect.

## Exact buffer initialization coverage

Dense byte ranges are insufficient to represent exact padded buffer-texture copy semantics without
logical-row expansion.

G3R therefore uses one checked buffer-coverage representation capable of compact dense and repeated
strided byte sets. Exact type names are implementation-defined; the semantic shape is equivalent to:

```text
BufferCoverage = deterministic union of
    Dense(byte_range)
    Strided {
        first byte offset
        segment size
        segment stride
        segment count
        group/image stride
        group/image count
    }
```

A simpler repeated-stride representation is acceptable if it denotes the same exact byte set.

### Canonical storage

- region identity, bounds, counts, strides, and arithmetic are checked;
- dense regions are normalized/merged where exact;
- identical or provably subsumed terms are deterministically removed without expansion;
- remaining terms are kept in deterministic order rather than forcing a minimal symbolic normal form;
- construction, union storage, import/export transport, and prepared publication use memory
  proportional to retained coverage terms rather than logical row/image count;
- no public/prepared boundary expands one compact strided term into one range per logical row;
- the superseded buffer-range-only initialization authority is removed instead of preserved beside
  the new representation.

### Exact containment and equality

Initialization admission and export-contract validation remain semantically exact. Raw structural
list equality is not sufficient once compact terms exist.

- dense, dense-containing-strided, and identical/aligned strided cases use arithmetic fast paths;
- semantic equality is mutual exact containment;
- differently shaped coverage may use lazy monotonic interval traversal when arithmetic fast paths
  cannot decide it directly;
- fallback traversal must not allocate/materialize the complete logical row set and must retain
  checked monotonic progress and overflow behavior;
- no arbitrary row-count cap or conservative false-negative admission path is introduced merely to
  avoid exact comparison;
- the ordinary same-layout buffer-texture path and prepared publication remain independent of logical
  row/image count apart from constant-time checked arithmetic over the compact pattern.

This deliberately avoids both eager row materialization and an unnecessary general-purpose symbolic
set minimizer.

Graph-entry input, import/export, and `GpuPreparedResourceInitialization` preserve compact coverage.
Buffer-to-texture requirements use the same logical-row pattern and therefore do not demand initialized
padding; texture-to-buffer effects use it and therefore do not mark padding initialized.

Hazard access remains the existing coarse buffer envelope. Copy-layout detail belongs only to exact
initialization requirement/effect coverage.

## Corrected requirement and effect rules

Descriptor and graph-entry evidence remains accepted state:

```text
Zeroed descriptor        -> complete descriptor-defined initialized coverage
Prepared descriptor      -> checked prepared coverage
Uninitialized descriptor -> no initialized coverage
GpuWorkResourceInput     -> exact explicit entry coverage
validated import         -> exact producer-export coverage
```

Current operation semantics are classified as follows:

| Operation semantics | Exact initialization requirement | Definite initialization effect |
|---|---|---|
| generic shader storage access | declared readable access envelope | none by default for writes |
| buffer zero | none | exact destination bytes |
| buffer-to-buffer copy | exact source bytes | exact destination bytes |
| buffer-to-texture copy | exact logical source bytes, excluding padding | destination texture coverage only when representable exactly by current texture model |
| texture-to-buffer copy | representable source texture coverage | exact logical destination bytes, excluding padding |
| texture-to-texture copy | representable source texture coverage | destination texture coverage only when representable exactly by current texture model |
| query resolve | exact source query indices | exact destination bytes |
| explicit compute timestamp write | none | exact declared query indices |
| explicit render timestamp write | none | exact declared query indices |
| color/depth attachment `Load` | selected attachment coverage | none by itself |
| color/depth attachment `Clear` | none | selected attachment coverage |
| multisample resolve destination | render-source requirements | selected resolve coverage |
| attachment `Store` | no additional requirement | preserves post-operation coverage |
| attachment `Discard` | no additional requirement | removes later-readable attachment coverage |
| present/order-only work | existing representable present requirement where applicable | none |

A generic caller-authored `GpuResourceAccess` never becomes a definite effect merely because its
access kind sounds operational. Timestamp effects come only from checked compute/render operation
state.

## Texture precision

Accepted G3 texture initialization coverage tracks mip/layer/aspect subresources, not arbitrary texel
rectangles.

- a texture destination copy establishes initialization only when the copy covers the complete
  spatial extent of every represented selected subresource;
- a partial texture destination copy remains a normal write hazard but establishes no whole-
  subresource initialization effect;
- a partial texture source requirement conservatively requires the represented selected subresource;
- a future texel-rectangle initialization model requires its own separately reviewed extension.

## Validation order

For every prepared node:

```text
1. derive exact initialization requirements from checked operation semantics + generic caller reads
2. validate those requirements against current initialized coverage
3. derive operation-guaranteed initialization effects
4. prove every effect is contained by a compatible checked write-capable access envelope
5. apply exact definite effects
6. apply attachment Store=Discard invalidation
```

An effect outside its checked write envelope is a structural
`GpuWorkGraphCause::OperationAccessContradiction` error.

No compatibility path falls back to `writes() => initialized`, access-name-derived effects, or
renderer shader postcondition inference.

## Hazard independence

Dependency inference continues to use access truth only:

```text
write -> read   RAW
read  -> write  WAR
write -> write  WAW
```

A shader write that establishes no definite initialization remains a write for hazard ordering. A
padded copy can retain a coarse contiguous hazard envelope while its initialization requirement or
effect denotes only logical row/image bytes. Operation-owned timestamp writes derive ordinary query
write accesses so query RAW dependency into resolve remains visible to the hazard system.

## Phase boundary

G3R does not:

- implement G5 Upload/Readback operations;
- add backend execution, submission, completion, or progress semantics;
- upgrade WGPU or Naga;
- add indirect compute or zero-work execution semantics;
- broaden texture-format or capability vocabulary;
- redesign the hazard algorithm;
- add a shader definite-write assertion/proof API;
- add renderer-owned initialization authority;
- preserve generic query access as timestamp-command/effect compatibility;
- introduce an arbitrary row-count cap in place of compact exact coverage.

The explicit compute timestamp state is not new instrumentation scope. It is the semantic owner
required to preserve an already-existing timed compute-pass path after removing the invalid
access-as-command inference.

The refreshed backend phase consumes this corrected G3 authority and re-proves the accepted G1-G4
boundary before G5 planning is re-derived.

## Required proof

Acceptance requires automated evidence that:

- storage-write capability followed by a read of the same previously-uninitialized range is rejected;
- RAW/WAR/WAW hazards remain access-driven;
- buffer zero and buffer-to-buffer copy use exact source/destination bytes;
- partial texture copies do not overclaim whole-subresource destination initialization;
- full selected-subresource texture copies establish representable destination coverage;
- texture-to-buffer logical bytes become initialized while row/image padding remains uninitialized;
- buffer-to-texture accepts initialized logical row/image bytes when padding in the hazard envelope is
  uninitialized;
- a very large logical row/image count remains one compact repeated coverage term through construction
  and prepared inspection rather than one range per row;
- compact coverage round-trips through explicit input, export/import, prepared summaries, and repeated
  deterministic preparation without expansion or semantic drift;
- semantically equivalent supported compact decompositions compare by exact coverage rather than raw
  structural list equality;
- explicit compute timestamp state initializes exact query indices and retains query RAW dependency
  into resolve;
- explicit render timestamp state initializes exact query indices;
- generic `WriteTimestamp` access without corresponding operation semantics initializes nothing;
- renderer timing instrumentation still prepares compute + render timestamps, resolve, and readback
  through explicit operation semantics;
- attachment clear/load/store/discard and query-resolve semantics remain exact;
- an internal/operation-derived effect outside its compatible write envelope rejects structurally;
- descriptor, explicit input, retained/import, and export semantics remain valid;
- no renderer compatibility path reintroduces access-derived initialization.
